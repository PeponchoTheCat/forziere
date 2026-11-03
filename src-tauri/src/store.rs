use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;
use crate::crypto::{self, ERR_BAD_PASSWORD, Key, KdfParams, VaultFile};
use std::{fs, io, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub service: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dati inviati dal frontend; `id` assente = nuova voce.
#[derive(Debug, Deserialize)]
pub struct EntryInput {
    pub id: Option<Uuid>,
    pub service: String,
    #[serde(default)]
    pub url: String,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Identificativo dell'app prima della rinomina in Forziere.
pub const LEGACY_APP_ID: &str = "com.giorgio-marini.fcs-password-mamanger";

/// Copia `vault.json` dalla vecchia cartella dati (affiancata a `dir`) se la
/// nuova non ne ha ancora uno. Restituisce `true` se ha migrato. Il vecchio
/// file non viene toccato.
pub fn migrate_legacy_vault(dir: &std::path::Path) -> io::Result<bool> {
    let new_file = dir.join("vault.json");
    let Some(parent) = dir.parent() else {
        return Ok(false);
    };
    let old_file = parent.join(LEGACY_APP_ID).join("vault.json");
    if new_file.exists() || !old_file.is_file() {
        return Ok(false);
    }
    fs::create_dir_all(dir)?;
    fs::copy(&old_file, &new_file)?;
    Ok(true)
}

/// Lunghezza minima della master password.
pub const MIN_PASSWORD_LEN: usize = 12;
pub const ERR_LOCKED: &str = "vault_locked";
pub const ERR_VAULT_EXISTS: &str = "vault_exists";
pub const ERR_ALREADY_UNLOCKED: &str = "already_unlocked";
pub const ERR_NO_VAULT_TO_UNLOCK: &str = "no_vault_to_unlock";
pub const ERR_WRONG_CURRENT_PASSWORD: &str = "wrong_current_password";
pub const ERR_REQUIRED_FIELDS: &str = "entry_required_fields";
pub const ERR_UNKNOWN_FORMAT: &str = "vault_format_unknown";

/// Stato del vault visto dal frontend.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VaultStatus {
    /// Nessun file: primo avvio, serve creare la master password.
    NoVault,
    /// File v1 in chiaro: va cifrato impostando una master password.
    LegacyPlaintext,
    /// File cifrato, chiave non in memoria.
    Locked,
    /// Chiave in memoria, voci leggibili.
    Unlocked,
}

enum State {
    NoVault,
    Legacy,
    Locked,
    Unlocked {
        kdf: KdfParams,
        key: Key,
        entries: Vec<Entry>,
    },
}

/// Vault cifrato (Argon2id + AES-256-GCM) su `vault.json`.
pub struct Store {
    path: PathBuf,
    state: State,
}

fn invalid(e: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
}

fn check_new_password(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(format!("password_too_short:{MIN_PASSWORD_LEN}"));
    }
    Ok(())
}

impl Store {
    /// Rileva lo stato dal file senza leggere le voci: assente = primo avvio,
    /// array = vault v1 in chiaro, oggetto = vault cifrato v2.
    pub fn load(path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            state: Self::detect(&path)?,
            path,
        })
    }

    fn detect(path: &std::path::Path) -> io::Result<State> {
        let text = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(State::NoVault),
            Err(e) => return Err(e),
        };
        match serde_json::from_str::<serde_json::Value>(&text).map_err(invalid)? {
            serde_json::Value::Array(_) => Ok(State::Legacy),
            serde_json::Value::Object(_) => {
                serde_json::from_str::<VaultFile>(&text).map_err(invalid)?;
                Ok(State::Locked)
            }
            _ => Err(invalid(ERR_UNKNOWN_FORMAT)),
        }
    }

    pub fn status(&self) -> VaultStatus {
        match self.state {
            State::NoVault => VaultStatus::NoVault,
            State::Legacy => VaultStatus::LegacyPlaintext,
            State::Locked => VaultStatus::Locked,
            State::Unlocked { .. } => VaultStatus::Unlocked,
        }
    }

    /// Crea il vault (primo avvio) o cifra sul posto quello in chiaro v1.
    /// Al termine il vault è sbloccato; il file in chiaro viene sostituito.
    pub fn setup(&mut self, password: &str) -> Result<(), String> {
        let entries = match self.state {
            State::NoVault => Vec::new(),
            State::Legacy => {
                let text = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
                serde_json::from_str(&text).map_err(|e| e.to_string())?
            }
            _ => return Err(ERR_VAULT_EXISTS.into()),
        };
        check_new_password(password)?;
        let kdf = KdfParams::new_random();
        let key = crypto::derive_key(password, &kdf)?;
        self.write(&kdf, &key, &entries)
            .map_err(|e| e.to_string())?;
        self.state = State::Unlocked { kdf, key, entries };
        Ok(())
    }

    pub fn unlock(&mut self, password: &str) -> Result<(), String> {
        if !matches!(self.state, State::Locked) {
            return Err(match self.state {
                State::Unlocked { .. } => ERR_ALREADY_UNLOCKED.into(),
                _ => ERR_NO_VAULT_TO_UNLOCK.to_string(),
            });
        }
        let text = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        let file: VaultFile = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let key = crypto::derive_key(password, &file.kdf)?;
        let plain = crypto::decrypt(&key, &file)?;
        let entries = serde_json::from_slice(&plain).map_err(|_| ERR_BAD_PASSWORD.to_string())?;
        self.state = State::Unlocked {
            kdf: file.kdf,
            key,
            entries,
        };
        Ok(())
    }

    /// Azzera la chiave e le voci in memoria.
    pub fn lock(&mut self) {
        if matches!(self.state, State::Unlocked { .. }) {
            self.state = State::Locked;
        }
    }

    /// Verifica la vecchia password, poi ricifra con salt e chiave nuovi.
    pub fn change_password(&mut self, old: &str, new: &str) -> Result<(), String> {
        let State::Unlocked { kdf, key, entries } = &self.state else {
            return Err(ERR_LOCKED.into());
        };
        let candidate = crypto::derive_key(old, kdf)?;
        if candidate.as_ref() != key.as_ref() {
            return Err(ERR_WRONG_CURRENT_PASSWORD.into());
        }
        check_new_password(new)?;
        let new_kdf = KdfParams::new_random();
        let new_key = crypto::derive_key(new, &new_kdf)?;
        self.write(&new_kdf, &new_key, entries)
            .map_err(|e| e.to_string())?;
        if let State::Unlocked { kdf, key, .. } = &mut self.state {
            *kdf = new_kdf;
            *key = new_key;
        }
        Ok(())
    }

    /// Voci in memoria; vuote se il vault è bloccato.
    pub fn list(&self) -> Vec<Entry> {
        match &self.state {
            State::Unlocked { entries, .. } => entries.clone(),
            _ => Vec::new(),
        }
    }

    pub fn save(&mut self, input: EntryInput) -> Result<Entry, String> {
        if !matches!(self.state, State::Unlocked { .. }) {
            return Err(ERR_LOCKED.into());
        }
        if input.service.trim().is_empty() || input.username.trim().is_empty() {
            return Err(ERR_REQUIRED_FIELDS.into());
        }
        let now = Utc::now();
        let tags = input
            .tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let State::Unlocked { entries, .. } = &mut self.state else {
            unreachable!()
        };
        let previous = entries.clone();
        let entry = match input.id.and_then(|id| entries.iter().position(|e| e.id == id)) {
            Some(i) => {
                let e = &mut entries[i];
                e.service = input.service;
                e.url = input.url;
                e.username = input.username;
                e.password = input.password;
                e.notes = input.notes;
                e.tags = tags;
                e.updated_at = now;
                e.clone()
            }
            None => {
                let e = Entry {
                    id: input.id.unwrap_or_else(Uuid::new_v4),
                    service: input.service,
                    url: input.url,
                    username: input.username,
                    password: input.password,
                    notes: input.notes,
                    tags,
                    created_at: now,
                    updated_at: now,
                };
                entries.push(e.clone());
                e
            }
        };
        if let Err(e) = self.persist() {
            self.set_entries(previous);
            return Err(e.to_string());
        }
        Ok(entry)
    }

    pub fn delete(&mut self, id: Uuid) -> Result<(), String> {
        let State::Unlocked { entries, .. } = &mut self.state else {
            return Err(ERR_LOCKED.into());
        };
        let previous = entries.clone();
        entries.retain(|e| e.id != id);
        if let Err(e) = self.persist() {
            self.set_entries(previous);
            return Err(e.to_string());
        }
        Ok(())
    }

    fn set_entries(&mut self, previous: Vec<Entry>) {
        if let State::Unlocked { entries, .. } = &mut self.state {
            *entries = previous;
        }
    }

    fn persist(&self) -> io::Result<()> {
        match &self.state {
            State::Unlocked { kdf, key, entries } => self.write(kdf, key, entries),
            _ => Err(io::Error::other(ERR_LOCKED)),
        }
    }

    /// Cifra le voci (nonce nuovo) e scrive in modo atomico: file temporaneo + rename.
    fn write(&self, kdf: &KdfParams, key: &Key, entries: &[Entry]) -> io::Result<()> {
        let plain = Zeroizing::new(serde_json::to_vec(entries)?);
        let file = crypto::encrypt(key, kdf, &plain).map_err(io::Error::other)?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(&file)?)?;
        fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(id: Option<Uuid>, service: &str) -> EntryInput {
        EntryInput {
            id,
            service: service.into(),
            url: "https://example.com".into(),
            username: "mario".into(),
            password: "pw".into(),
            notes: "nota".into(),
            tags: vec![" web ".into(), "".into(), "lavoro".into()],
        }
    }

    fn tmp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("forziere-{name}-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir.join("vault.json")
    }

    const PW: &str = "master-password-1";

    /// Store con vault creato e sbloccato.
    fn unlocked(path: PathBuf) -> Store {
        let mut store = Store::load(path).unwrap();
        store.setup(PW).unwrap();
        store
    }

    /// Riapre il file da zero e lo sblocca, come dopo un riavvio.
    fn reopen(path: PathBuf) -> Store {
        let mut store = Store::load(path).unwrap();
        store.unlock(PW).unwrap();
        store
    }

    #[test]
    fn load_reports_no_vault_when_file_is_missing() {
        // Arrange
        let path = tmp_path("missing");

        // Act
        let store = Store::load(path).unwrap();

        // Assert
        assert_eq!(store.status(), VaultStatus::NoVault);
        assert!(store.list().is_empty());
    }

    #[test]
    fn load_reports_legacy_plaintext_for_v1_array_file() {
        // Arrange
        let path = tmp_path("legacy");
        fs::write(&path, "[]").unwrap();

        // Act
        let store = Store::load(path).unwrap();

        // Assert
        assert_eq!(store.status(), VaultStatus::LegacyPlaintext);
    }

    #[test]
    fn load_reports_locked_for_encrypted_file() {
        // Arrange
        let path = tmp_path("locked");
        unlocked(path.clone());

        // Act
        let store = Store::load(path).unwrap();

        // Assert
        assert_eq!(store.status(), VaultStatus::Locked);
    }

    #[test]
    fn load_fails_with_invalid_data_for_unrecognised_json() {
        // Arrange
        let path = tmp_path("scalar");
        fs::write(&path, "42").unwrap();

        // Act
        let result = Store::load(path);

        // Assert
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn setup_creates_encrypted_file_without_plaintext_fields() {
        // Arrange
        let path = tmp_path("setup");
        let mut store = unlocked(path.clone());
        store.save(input(None, "ServizioSegreto")).unwrap();

        // Act
        let raw = fs::read_to_string(&path).unwrap();

        // Assert
        assert_eq!(store.status(), VaultStatus::Unlocked);
        assert!(raw.contains("argon2id"));
        assert!(!raw.contains("ServizioSegreto"));
        assert!(!raw.contains("mario"));
    }

    #[test]
    fn setup_rejects_short_password() {
        // Arrange
        let mut store = Store::load(tmp_path("short")).unwrap();

        // Act
        let result = store.setup("corta");

        // Assert
        assert_eq!(result.unwrap_err(), "password_too_short:12");
        assert_eq!(store.status(), VaultStatus::NoVault);
    }

    #[test]
    fn setup_rejects_existing_encrypted_vault() {
        // Arrange
        let path = tmp_path("setup-twice");
        unlocked(path.clone());
        let mut store = Store::load(path).unwrap();

        // Act
        let result = store.setup("un-altra-password-1");

        // Assert
        assert_eq!(result.unwrap_err(), ERR_VAULT_EXISTS);
        assert_eq!(store.status(), VaultStatus::Locked);
    }

    #[test]
    fn setup_encrypts_legacy_plaintext_vault_keeping_entries() {
        // Arrange
        let path = tmp_path("migrate-v1");
        let mut seed = unlocked(tmp_path("seed"));
        let entry = seed.save(input(None, "GitHub")).unwrap();
        fs::write(&path, serde_json::to_string(&seed.list()).unwrap()).unwrap();
        let mut store = Store::load(path.clone()).unwrap();

        // Act
        store.setup(PW).unwrap();
        let raw = fs::read_to_string(&path).unwrap();

        // Assert
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, entry.id);
        assert!(!raw.contains("GitHub"));
        assert_eq!(reopen(path).list()[0].service, "GitHub");
    }

    #[test]
    fn unlock_with_correct_password_loads_entries() {
        // Arrange
        let path = tmp_path("unlock-ok");
        let mut first = unlocked(path.clone());
        first.save(input(None, "GitHub")).unwrap();
        let mut store = Store::load(path).unwrap();

        // Act
        store.unlock(PW).unwrap();

        // Assert
        assert_eq!(store.status(), VaultStatus::Unlocked);
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn unlock_with_wrong_password_fails_and_stays_locked() {
        // Arrange
        let path = tmp_path("unlock-bad");
        unlocked(path.clone());
        let mut store = Store::load(path).unwrap();

        // Act
        let result = store.unlock("password-sbagliata-1");

        // Assert
        assert_eq!(result.unwrap_err(), ERR_BAD_PASSWORD);
        assert_eq!(store.status(), VaultStatus::Locked);
    }

    #[test]
    fn unlock_fails_when_ciphertext_was_modified_on_disk() {
        // Arrange
        let path = tmp_path("unlock-tamper");
        unlocked(path.clone());
        let mut file: VaultFile =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        file.ciphertext.push_str("AAAA");
        fs::write(&path, serde_json::to_string(&file).unwrap()).unwrap();
        let mut store = Store::load(path).unwrap();

        // Act
        let result = store.unlock(PW);

        // Assert
        assert_eq!(result.unwrap_err(), ERR_BAD_PASSWORD);
    }

    #[test]
    fn lock_clears_entries_and_blocks_operations() {
        // Arrange
        let mut store = unlocked(tmp_path("lock"));
        store.save(input(None, "GitHub")).unwrap();

        // Act
        store.lock();

        // Assert
        assert_eq!(store.status(), VaultStatus::Locked);
        assert!(store.list().is_empty());
        assert_eq!(store.save(input(None, "X")).unwrap_err(), ERR_LOCKED);
        assert_eq!(store.delete(Uuid::new_v4()).unwrap_err(), ERR_LOCKED);
    }

    #[test]
    fn lock_then_unlock_restores_entries() {
        // Arrange
        let mut store = unlocked(tmp_path("lock-unlock"));
        store.save(input(None, "GitHub")).unwrap();
        store.lock();

        // Act
        store.unlock(PW).unwrap();

        // Assert
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn change_password_reencrypts_with_new_password_only() {
        // Arrange
        let path = tmp_path("change");
        let mut store = unlocked(path.clone());
        store.save(input(None, "GitHub")).unwrap();

        // Act
        store.change_password(PW, "nuova-password-12").unwrap();
        let mut old = Store::load(path.clone()).unwrap();
        let mut new = Store::load(path).unwrap();

        // Assert
        assert!(old.unlock(PW).is_err());
        new.unlock("nuova-password-12").unwrap();
        assert_eq!(new.list().len(), 1);
    }

    #[test]
    fn change_password_rejects_wrong_current_password() {
        // Arrange
        let mut store = unlocked(tmp_path("change-wrong"));

        // Act
        let result = store.change_password("sbagliata-sbagliata", "nuova-password-12");

        // Assert
        assert_eq!(result.unwrap_err(), ERR_WRONG_CURRENT_PASSWORD);
    }

    #[test]
    fn change_password_rejects_short_new_password() {
        // Arrange
        let mut store = unlocked(tmp_path("change-short"));

        // Act
        let result = store.change_password(PW, "corta");

        // Assert
        assert_eq!(result.unwrap_err(), "password_too_short:12");
    }

    #[test]
    fn change_password_requires_unlocked_vault() {
        // Arrange
        let mut store = Store::load(tmp_path("change-locked")).unwrap();

        // Act
        let result = store.change_password(PW, "nuova-password-12");

        // Assert
        assert_eq!(result.unwrap_err(), ERR_LOCKED);
    }

    #[test]
    fn each_write_uses_a_fresh_nonce() {
        // Arrange
        let path = tmp_path("nonce");
        let mut store = unlocked(path.clone());
        let before: VaultFile = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

        // Act
        store.save(input(None, "GitHub")).unwrap();
        let after: VaultFile = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

        // Assert
        assert_ne!(before.nonce, after.nonce);
    }

    #[test]
    fn load_fails_with_invalid_data_when_file_is_corrupt() {
        // Arrange
        let path = tmp_path("corrupt");
        fs::write(&path, "non e json").unwrap();

        // Act
        let result = Store::load(path);

        // Assert
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn load_fails_when_path_is_not_a_readable_file() {
        // Arrange: una directory non e leggibile come file
        let dir = std::env::temp_dir();

        // Act
        let result = Store::load(dir);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn save_creates_new_entry_with_generated_id_and_equal_timestamps() {
        // Arrange
        let mut store = unlocked(tmp_path("create"));

        // Act
        let entry = store.save(input(None, "GitHub")).unwrap();

        // Assert
        assert_eq!(entry.service, "GitHub");
        assert_eq!(entry.created_at, entry.updated_at);
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, entry.id);
    }

    #[test]
    fn save_trims_tags_and_drops_empty_ones() {
        // Arrange
        let mut store = unlocked(tmp_path("tags"));

        // Act
        let entry = store.save(input(None, "GitHub")).unwrap();

        // Assert
        assert_eq!(entry.tags, vec!["web", "lavoro"]);
    }

    #[test]
    fn save_updates_existing_entry_keeping_id_and_created_at() {
        // Arrange
        let mut store = unlocked(tmp_path("update"));
        let original = store.save(input(None, "GitHub")).unwrap();

        // Act
        let updated = store.save(input(Some(original.id), "GitLab")).unwrap();

        // Assert
        assert_eq!(updated.id, original.id);
        assert_eq!(updated.service, "GitLab");
        assert_eq!(updated.created_at, original.created_at);
        assert!(updated.updated_at >= original.updated_at);
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn save_with_unknown_id_creates_entry_using_that_id() {
        // Arrange
        let mut store = unlocked(tmp_path("unknown-id"));
        let id = Uuid::new_v4();

        // Act
        let entry = store.save(input(Some(id), "GitHub")).unwrap();

        // Assert
        assert_eq!(entry.id, id);
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn save_rejects_blank_service() {
        // Arrange
        let mut store = unlocked(tmp_path("blank-service"));

        // Act
        let result = store.save(input(None, "   "));

        // Assert
        assert_eq!(result.unwrap_err(), ERR_REQUIRED_FIELDS);
        assert!(store.list().is_empty());
    }

    #[test]
    fn save_rejects_blank_username() {
        // Arrange
        let mut store = unlocked(tmp_path("blank-user"));
        let mut data = input(None, "GitHub");
        data.username = "".into();

        // Act
        let result = store.save(data);

        // Assert
        assert_eq!(result.unwrap_err(), ERR_REQUIRED_FIELDS);
        assert!(store.list().is_empty());
    }

    #[test]
    fn save_persists_to_disk_and_survives_reload() {
        // Arrange
        let path = tmp_path("persist");
        let mut store = unlocked(path.clone());

        // Act
        let entry = store.save(input(None, "GitHub")).unwrap();
        let reloaded = reopen(path);

        // Assert
        assert_eq!(reloaded.list().len(), 1);
        assert_eq!(reloaded.list()[0].id, entry.id);
        assert_eq!(reloaded.list()[0].password, "pw");
    }

    #[test]
    fn save_leaves_no_temporary_file_behind() {
        // Arrange
        let path = tmp_path("no-tmp");
        let mut store = unlocked(path.clone());

        // Act
        store.save(input(None, "GitHub")).unwrap();

        // Assert
        assert!(path.exists());
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn save_rolls_back_memory_state_when_write_fails() {
        // Arrange: dopo il setup rimuovo la cartella cosi la scrittura fallisce
        let path = tmp_path("save-rollback");
        let mut store = unlocked(path.clone());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();

        // Act
        let result = store.save(input(None, "GitHub"));

        // Assert
        assert!(result.is_err());
        assert!(store.list().is_empty());
    }

    #[test]
    fn delete_removes_only_the_requested_entry() {
        // Arrange
        let mut store = unlocked(tmp_path("delete"));
        let keep = store.save(input(None, "Keep")).unwrap();
        let drop = store.save(input(None, "Drop")).unwrap();

        // Act
        store.delete(drop.id).unwrap();

        // Assert
        let left = store.list();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].id, keep.id);
    }

    #[test]
    fn delete_unknown_id_is_a_noop() {
        // Arrange
        let mut store = unlocked(tmp_path("delete-unknown"));
        store.save(input(None, "Keep")).unwrap();

        // Act
        let result = store.delete(Uuid::new_v4());

        // Assert
        assert!(result.is_ok());
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn delete_persists_to_disk() {
        // Arrange
        let path = tmp_path("delete-persist");
        let mut store = unlocked(path.clone());
        let entry = store.save(input(None, "GitHub")).unwrap();

        // Act
        store.delete(entry.id).unwrap();
        let reloaded = reopen(path);

        // Assert
        assert!(reloaded.list().is_empty());
    }

    #[test]
    fn delete_rolls_back_memory_state_when_write_fails() {
        // Arrange: dopo il salvataggio rimuovo la cartella cosi la scrittura fallisce
        let path = tmp_path("delete-rollback");
        let mut store = unlocked(path.clone());
        let entry = store.save(input(None, "GitHub")).unwrap();
        fs::remove_dir_all(path.parent().unwrap()).unwrap();

        // Act
        let result = store.delete(entry.id);

        // Assert
        assert!(result.is_err());
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn entry_input_applies_defaults_for_optional_fields() {
        // Arrange
        let json = r#"{"service":"S","username":"u","password":"p"}"#;

        // Act
        let data: EntryInput = serde_json::from_str(json).unwrap();

        // Assert
        assert!(data.id.is_none());
        assert_eq!(data.url, "");
        assert_eq!(data.notes, "");
        assert!(data.tags.is_empty());
    }

    #[test]
    fn migrate_legacy_vault_copies_old_file_when_new_is_missing() {
        // Arrange
        let base = tmp_path("migrate-copy").parent().unwrap().to_path_buf();
        let old = base.join(LEGACY_APP_ID);
        let new = base.join("com.giorgio-marini.forziere");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("vault.json"), "[]").unwrap();
        // Act
        let migrated = migrate_legacy_vault(&new).unwrap();
        // Assert
        assert!(migrated);
        assert_eq!(fs::read_to_string(new.join("vault.json")).unwrap(), "[]");
        assert!(old.join("vault.json").exists());
    }

    #[test]
    fn migrate_legacy_vault_does_not_overwrite_existing_new_file() {
        // Arrange
        let base = tmp_path("migrate-keep").parent().unwrap().to_path_buf();
        let old = base.join(LEGACY_APP_ID);
        let new = base.join("com.giorgio-marini.forziere");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&new).unwrap();
        fs::write(old.join("vault.json"), "[1]").unwrap();
        fs::write(new.join("vault.json"), "[]").unwrap();
        // Act
        let migrated = migrate_legacy_vault(&new).unwrap();
        // Assert
        assert!(!migrated);
        assert_eq!(fs::read_to_string(new.join("vault.json")).unwrap(), "[]");
    }

    #[test]
    fn migrate_legacy_vault_is_noop_without_old_file() {
        // Arrange
        let base = tmp_path("migrate-none").parent().unwrap().to_path_buf();
        let new = base.join("com.giorgio-marini.forziere");
        // Act
        let migrated = migrate_legacy_vault(&new).unwrap();
        // Assert
        assert!(!migrated);
        assert!(!new.join("vault.json").exists());
    }
}
