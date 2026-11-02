//! Cifratura del vault: Argon2id (KDF) + AES-256-GCM (AEAD).
//!
//! Solo funzioni pure, senza I/O. Il formato su disco (`VaultFile`) è versionato
//! e l'header (versione + parametri KDF + salt) è autenticato come AAD.

use aes_gcm::{
    Aes256Gcm, KeyInit,
    aead::{Aead, Payload},
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const FORMAT_VERSION: u32 = 2;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// Messaggio unico per password errata e file manomesso (GCM non li distingue).
pub const ERR_BAD_PASSWORD: &str = "Password errata o vault danneggiato";

pub type Key = Zeroizing<[u8; KEY_LEN]>;

/// Parametri Argon2id salvati nel file, così si possono alzare in futuro.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KdfParams {
    pub alg: String,
    /// Memoria in KiB.
    pub m: u32,
    /// Iterazioni.
    pub t: u32,
    /// Parallelismo.
    pub p: u32,
    /// Salt in base64.
    pub salt: String,
}

impl KdfParams {
    /// Parametri con salt nuovo. Produzione: minimo OWASP (19 MiB, t=2, p=1);
    /// nei test costi minimi per non rallentare la suite.
    pub fn new_random() -> Self {
        #[cfg(not(test))]
        let (m, t, p) = (19 * 1024, 2, 1);
        #[cfg(test)]
        let (m, t, p) = (8, 1, 1);
        let mut salt = [0u8; SALT_LEN];
        rand::rng().fill_bytes(&mut salt);
        Self {
            alg: "argon2id".into(),
            m,
            t,
            p,
            salt: B64.encode(salt),
        }
    }
}

/// Contenuto di `vault.json` nel formato v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFile {
    pub version: u32,
    pub kdf: KdfParams,
    pub nonce: String,
    pub ciphertext: String,
}

impl VaultFile {
    /// AAD: versione + parametri KDF (salt incluso), non alterabili senza far fallire il GCM.
    fn aad(version: u32, kdf: &KdfParams) -> Vec<u8> {
        format!(
            "forziere:v{version}:{}:{}:{}:{}:{}",
            kdf.alg, kdf.m, kdf.t, kdf.p, kdf.salt
        )
        .into_bytes()
    }
}

/// Deriva la chiave a 32 byte dalla master password.
pub fn derive_key(password: &str, kdf: &KdfParams) -> Result<Key, String> {
    if kdf.alg != "argon2id" {
        return Err(format!("Algoritmo KDF non supportato: {}", kdf.alg));
    }
    let salt = B64
        .decode(&kdf.salt)
        .map_err(|_| "Salt non valido".to_string())?;
    let params = Params::new(kdf.m, kdf.t, kdf.p, Some(KEY_LEN))
        .map_err(|e| format!("Parametri KDF non validi: {e}"))?;
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), &salt, key.as_mut())
        .map_err(|e| format!("Derivazione chiave fallita: {e}"))?;
    Ok(key)
}

/// Cifra `plaintext` con nonce casuale nuovo; restituisce il file completo.
pub fn encrypt(key: &Key, kdf: &KdfParams, plaintext: &[u8]) -> Result<VaultFile, String> {
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|e| e.to_string())?;
    let mut nonce = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce);
    let aad = VaultFile::aad(FORMAT_VERSION, kdf);
    let ciphertext = cipher
        .encrypt(
            (&nonce).into(),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| "Cifratura fallita".to_string())?;
    Ok(VaultFile {
        version: FORMAT_VERSION,
        kdf: kdf.clone(),
        nonce: B64.encode(nonce),
        ciphertext: B64.encode(ciphertext),
    })
}

/// Decifra un `VaultFile`. Password errata e file alterato danno lo stesso errore.
pub fn decrypt(key: &Key, file: &VaultFile) -> Result<Zeroizing<Vec<u8>>, String> {
    let bad = || ERR_BAD_PASSWORD.to_string();
    if file.version != FORMAT_VERSION {
        return Err(format!("Versione del vault non supportata: {}", file.version));
    }
    let nonce: [u8; NONCE_LEN] = B64
        .decode(&file.nonce)
        .map_err(|_| bad())?
        .try_into()
        .map_err(|_| bad())?;
    let ciphertext = B64.decode(&file.ciphertext).map_err(|_| bad())?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|e| e.to_string())?;
    let aad = VaultFile::aad(file.version, &file.kdf);
    cipher
        .decrypt(
            (&nonce).into(),
            Payload {
                msg: &ciphertext,
                aad: &aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| bad())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sealed(password: &str, data: &[u8]) -> (Key, VaultFile) {
        let kdf = KdfParams::new_random();
        let key = derive_key(password, &kdf).unwrap();
        let file = encrypt(&key, &kdf, data).unwrap();
        (key, file)
    }

    #[test]
    fn encrypt_then_decrypt_round_trips() {
        // Arrange
        let (key, file) = sealed("master-password-1", b"segreto");

        // Act
        let plain = decrypt(&key, &file).unwrap();

        // Assert
        assert_eq!(plain.as_slice(), b"segreto");
    }

    #[test]
    fn decrypt_with_wrong_password_fails() {
        // Arrange
        let (_, file) = sealed("master-password-1", b"segreto");
        let wrong = derive_key("altra-password-xx", &file.kdf).unwrap();

        // Act
        let result = decrypt(&wrong, &file);

        // Assert
        assert_eq!(result.unwrap_err(), ERR_BAD_PASSWORD);
    }

    #[test]
    fn decrypt_fails_when_ciphertext_is_tampered() {
        // Arrange
        let (key, mut file) = sealed("master-password-1", b"segreto");
        let mut raw = B64.decode(&file.ciphertext).unwrap();
        raw[0] ^= 0x01;
        file.ciphertext = B64.encode(raw);

        // Act
        let result = decrypt(&key, &file);

        // Assert
        assert_eq!(result.unwrap_err(), ERR_BAD_PASSWORD);
    }

    #[test]
    fn decrypt_fails_when_kdf_header_is_tampered() {
        // Arrange
        let (key, mut file) = sealed("master-password-1", b"segreto");
        file.kdf.t += 1;

        // Act
        let result = decrypt(&key, &file);

        // Assert
        assert_eq!(result.unwrap_err(), ERR_BAD_PASSWORD);
    }

    #[test]
    fn decrypt_fails_when_nonce_is_malformed() {
        // Arrange
        let (key, mut file) = sealed("master-password-1", b"segreto");
        file.nonce = B64.encode([1u8, 2, 3]);

        // Act
        let result = decrypt(&key, &file);

        // Assert
        assert_eq!(result.unwrap_err(), ERR_BAD_PASSWORD);
    }

    #[test]
    fn decrypt_rejects_unknown_format_version() {
        // Arrange
        let (key, mut file) = sealed("master-password-1", b"segreto");
        file.version = 99;

        // Act
        let result = decrypt(&key, &file);

        // Assert
        assert!(result.unwrap_err().contains("non supportata"));
    }

    #[test]
    fn encrypt_uses_a_fresh_nonce_each_time() {
        // Arrange
        let kdf = KdfParams::new_random();
        let key = derive_key("master-password-1", &kdf).unwrap();

        // Act
        let a = encrypt(&key, &kdf, b"x").unwrap();
        let b = encrypt(&key, &kdf, b"x").unwrap();

        // Assert
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[test]
    fn derive_key_differs_with_different_salt() {
        // Arrange
        let a = KdfParams::new_random();
        let b = KdfParams::new_random();

        // Act
        let ka = derive_key("master-password-1", &a).unwrap();
        let kb = derive_key("master-password-1", &b).unwrap();

        // Assert
        assert_ne!(ka.as_ref(), kb.as_ref());
    }

    #[test]
    fn derive_key_is_deterministic_for_same_password_and_salt() {
        // Arrange
        let kdf = KdfParams::new_random();

        // Act
        let k1 = derive_key("master-password-1", &kdf).unwrap();
        let k2 = derive_key("master-password-1", &kdf).unwrap();

        // Assert
        assert_eq!(k1.as_ref(), k2.as_ref());
    }

    #[test]
    fn derive_key_rejects_unsupported_algorithm() {
        // Arrange
        let mut kdf = KdfParams::new_random();
        kdf.alg = "pbkdf2".into();

        // Act
        let result = derive_key("master-password-1", &kdf);

        // Assert
        assert!(result.unwrap_err().contains("non supportato"));
    }

    #[test]
    fn derive_key_rejects_invalid_salt() {
        // Arrange
        let mut kdf = KdfParams::new_random();
        kdf.salt = "!!!non-base64!!!".into();

        // Act
        let result = derive_key("master-password-1", &kdf);

        // Assert
        assert!(result.is_err());
    }
}
