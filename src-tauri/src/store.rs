use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// Vault in chiaro su `vault.json` (nessuna cifratura, scelta voluta).
pub struct Store {
    path: PathBuf,
    entries: Vec<Entry>,
}

impl Store {
    pub fn load(path: PathBuf) -> io::Result<Self> {
        let entries = match fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        Ok(Self { path, entries })
    }

    pub fn list(&self) -> Vec<Entry> {
        self.entries.clone()
    }

    pub fn save(&mut self, input: EntryInput) -> Result<Entry, String> {
        if input.service.trim().is_empty() || input.username.trim().is_empty() {
            return Err("Servizio e utente sono obbligatori".into());
        }
        let now = Utc::now();
        let tags = input
            .tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let previous = self.entries.clone();
        let entry = match input.id.and_then(|id| self.entries.iter().position(|e| e.id == id)) {
            Some(i) => {
                let e = &mut self.entries[i];
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
                self.entries.push(e.clone());
                e
            }
        };
        if let Err(e) = self.persist() {
            self.entries = previous;
            return Err(e.to_string());
        }
        Ok(entry)
    }

    pub fn delete(&mut self, id: Uuid) -> Result<(), String> {
        let previous = self.entries.clone();
        self.entries.retain(|e| e.id != id);
        if let Err(e) = self.persist() {
            self.entries = previous;
            return Err(e.to_string());
        }
        Ok(())
    }

    /// Scrittura atomica: file temporaneo + rename.
    fn persist(&self) -> io::Result<()> {
        let tmp = self.path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&tmp, json)?;
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

    #[test]
    fn load_returns_empty_store_when_file_is_missing() {
        // Arrange
        let path = tmp_path("missing");

        // Act
        let store = Store::load(path).unwrap();

        // Assert
        assert!(store.list().is_empty());
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
        let mut store = Store::load(tmp_path("create")).unwrap();

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
        let mut store = Store::load(tmp_path("tags")).unwrap();

        // Act
        let entry = store.save(input(None, "GitHub")).unwrap();

        // Assert
        assert_eq!(entry.tags, vec!["web", "lavoro"]);
    }

    #[test]
    fn save_updates_existing_entry_keeping_id_and_created_at() {
        // Arrange
        let mut store = Store::load(tmp_path("update")).unwrap();
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
        let mut store = Store::load(tmp_path("unknown-id")).unwrap();
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
        let mut store = Store::load(tmp_path("blank-service")).unwrap();

        // Act
        let result = store.save(input(None, "   "));

        // Assert
        assert!(result.is_err());
        assert!(store.list().is_empty());
    }

    #[test]
    fn save_rejects_blank_username() {
        // Arrange
        let mut store = Store::load(tmp_path("blank-user")).unwrap();
        let mut data = input(None, "GitHub");
        data.username = "".into();

        // Act
        let result = store.save(data);

        // Assert
        assert!(result.is_err());
        assert!(store.list().is_empty());
    }

    #[test]
    fn save_persists_to_disk_and_survives_reload() {
        // Arrange
        let path = tmp_path("persist");
        let mut store = Store::load(path.clone()).unwrap();

        // Act
        let entry = store.save(input(None, "GitHub")).unwrap();
        let reloaded = Store::load(path).unwrap();

        // Assert
        assert_eq!(reloaded.list().len(), 1);
        assert_eq!(reloaded.list()[0].id, entry.id);
        assert_eq!(reloaded.list()[0].password, "pw");
    }

    #[test]
    fn save_leaves_no_temporary_file_behind() {
        // Arrange
        let path = tmp_path("no-tmp");
        let mut store = Store::load(path.clone()).unwrap();

        // Act
        store.save(input(None, "GitHub")).unwrap();

        // Assert
        assert!(path.exists());
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn save_rolls_back_memory_state_when_write_fails() {
        // Arrange: cartella inesistente => la scrittura fallisce
        let path = std::env::temp_dir()
            .join(format!("forziere-absent-{}", Uuid::new_v4()))
            .join("vault.json");
        let mut store = Store::load(path).unwrap();

        // Act
        let result = store.save(input(None, "GitHub"));

        // Assert
        assert!(result.is_err());
        assert!(store.list().is_empty());
    }

    #[test]
    fn delete_removes_only_the_requested_entry() {
        // Arrange
        let mut store = Store::load(tmp_path("delete")).unwrap();
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
        let mut store = Store::load(tmp_path("delete-unknown")).unwrap();
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
        let mut store = Store::load(path.clone()).unwrap();
        let entry = store.save(input(None, "GitHub")).unwrap();

        // Act
        store.delete(entry.id).unwrap();
        let reloaded = Store::load(path).unwrap();

        // Assert
        assert!(reloaded.list().is_empty());
    }

    #[test]
    fn delete_rolls_back_memory_state_when_write_fails() {
        // Arrange: dopo il salvataggio rimuovo la cartella cosi la scrittura fallisce
        let path = tmp_path("delete-rollback");
        let mut store = Store::load(path.clone()).unwrap();
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
