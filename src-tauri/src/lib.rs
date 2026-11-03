mod crypto;
mod generator;
mod store;

use std::sync::Mutex;
use store::{Entry, EntryInput, Store, VaultStatus};
use tauri::{Manager, State};
use uuid::Uuid;

type VaultState<'a> = State<'a, Mutex<Store>>;

#[tauri::command]
fn vault_status(vault: VaultState) -> Result<VaultStatus, String> {
    Ok(vault.lock().map_err(|e| e.to_string())?.status())
}

#[tauri::command]
fn setup_vault(vault: VaultState, password: String) -> Result<(), String> {
    vault.lock().map_err(|e| e.to_string())?.setup(&password)
}

#[tauri::command]
fn unlock(vault: VaultState, password: String) -> Result<(), String> {
    vault.lock().map_err(|e| e.to_string())?.unlock(&password)
}

#[tauri::command]
fn lock(vault: VaultState) -> Result<(), String> {
    vault.lock().map_err(|e| e.to_string())?.lock();
    Ok(())
}

#[tauri::command]
fn change_password(vault: VaultState, old: String, new: String) -> Result<(), String> {
    vault
        .lock()
        .map_err(|e| e.to_string())?
        .change_password(&old, &new)
}

#[tauri::command]
fn list_entries(vault: VaultState) -> Result<Vec<Entry>, String> {
    Ok(vault.lock().map_err(|e| e.to_string())?.list())
}

#[tauri::command]
fn save_entry(vault: VaultState, entry: EntryInput) -> Result<Entry, String> {
    vault.lock().map_err(|e| e.to_string())?.save(entry)
}

#[tauri::command]
fn delete_entry(vault: VaultState, id: Uuid) -> Result<(), String> {
    vault.lock().map_err(|e| e.to_string())?.delete(id)
}

#[tauri::command]
fn generate_password(options: generator::GenOptions) -> Result<String, String> {
    generator::generate(&options)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            store::migrate_legacy_vault(&dir)?;
            app.manage(Mutex::new(Store::load(dir.join("vault.json"))?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            vault_status,
            setup_vault,
            unlock,
            lock,
            change_password,
            list_entries,
            save_entry,
            delete_entry,
            generate_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{MockRuntime, mock_app};

    const PW: &str = "master-password-1";

    /// App con vault ancora da creare (primo avvio).
    fn app_without_vault() -> tauri::App<MockRuntime> {
        let dir = std::env::temp_dir().join(format!("forziere-cmd-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let app = mock_app();
        app.manage(Mutex::new(Store::load(dir.join("vault.json")).unwrap()));
        app
    }

    /// App con vault creato e sbloccato.
    fn app_with_vault() -> tauri::App<MockRuntime> {
        let app = app_without_vault();
        setup_vault(app.state(), PW.into()).unwrap();
        app
    }

    fn input(service: &str) -> EntryInput {
        EntryInput {
            id: None,
            service: service.into(),
            url: String::new(),
            username: "mario".into(),
            password: "pw".into(),
            notes: String::new(),
            tags: vec![],
        }
    }

    #[test]
    fn list_entries_returns_empty_list_on_new_vault() {
        // Arrange
        let app = app_with_vault();

        // Act
        let entries = list_entries(app.state()).unwrap();

        // Assert
        assert!(entries.is_empty());
    }

    #[test]
    fn save_entry_then_list_entries_returns_saved_entry() {
        // Arrange
        let app = app_with_vault();

        // Act
        let saved = save_entry(app.state(), input("GitHub")).unwrap();
        let entries = list_entries(app.state()).unwrap();

        // Assert
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, saved.id);
    }

    #[test]
    fn save_entry_propagates_validation_error() {
        // Arrange
        let app = app_with_vault();

        // Act
        let result = save_entry(app.state(), input(""));

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn delete_entry_removes_saved_entry() {
        // Arrange
        let app = app_with_vault();
        let saved = save_entry(app.state(), input("GitHub")).unwrap();

        // Act
        delete_entry(app.state(), saved.id).unwrap();

        // Assert
        assert!(list_entries(app.state()).unwrap().is_empty());
    }

    #[test]
    fn generate_password_command_returns_password_of_requested_length() {
        // Arrange
        let options = generator::GenOptions {
            length: 24,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: false,
        };

        // Act
        let password = generate_password(options).unwrap();

        // Assert
        assert_eq!(password.chars().count(), 24);
    }

    #[test]
    fn generate_password_command_propagates_error_when_no_class_selected() {
        // Arrange
        let options = generator::GenOptions {
            length: 24,
            uppercase: false,
            lowercase: false,
            digits: false,
            symbols: false,
        };

        // Act
        let result = generate_password(options);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn vault_status_is_no_vault_on_first_launch() {
        // Arrange
        let app = app_without_vault();

        // Act
        let status = vault_status(app.state()).unwrap();

        // Assert
        assert_eq!(status, VaultStatus::NoVault);
    }

    #[test]
    fn setup_vault_unlocks_the_new_vault() {
        // Arrange
        let app = app_without_vault();

        // Act
        setup_vault(app.state(), PW.into()).unwrap();

        // Assert
        assert_eq!(vault_status(app.state()).unwrap(), VaultStatus::Unlocked);
    }

    #[test]
    fn setup_vault_propagates_short_password_error() {
        // Arrange
        let app = app_without_vault();

        // Act
        let result = setup_vault(app.state(), "corta".into());

        // Assert
        assert_eq!(result.unwrap_err(), "password_too_short:12");
        assert_eq!(vault_status(app.state()).unwrap(), VaultStatus::NoVault);
    }

    #[test]
    fn lock_command_locks_and_hides_entries() {
        // Arrange
        let app = app_with_vault();
        save_entry(app.state(), input("GitHub")).unwrap();

        // Act
        lock(app.state()).unwrap();

        // Assert
        assert_eq!(vault_status(app.state()).unwrap(), VaultStatus::Locked);
        assert!(list_entries(app.state()).unwrap().is_empty());
        assert!(save_entry(app.state(), input("X")).is_err());
    }

    #[test]
    fn unlock_command_with_correct_password_restores_entries() {
        // Arrange
        let app = app_with_vault();
        save_entry(app.state(), input("GitHub")).unwrap();
        lock(app.state()).unwrap();

        // Act
        unlock(app.state(), PW.into()).unwrap();

        // Assert
        assert_eq!(list_entries(app.state()).unwrap().len(), 1);
    }

    #[test]
    fn unlock_command_with_wrong_password_fails() {
        // Arrange
        let app = app_with_vault();
        lock(app.state()).unwrap();

        // Act
        let result = unlock(app.state(), "password-sbagliata-1".into());

        // Assert
        assert!(result.is_err());
        assert_eq!(vault_status(app.state()).unwrap(), VaultStatus::Locked);
    }

    #[test]
    fn change_password_command_switches_to_new_password() {
        // Arrange
        let app = app_with_vault();

        // Act
        change_password(app.state(), PW.into(), "nuova-password-12".into()).unwrap();
        lock(app.state()).unwrap();

        // Assert
        assert!(unlock(app.state(), PW.into()).is_err());
        assert!(unlock(app.state(), "nuova-password-12".into()).is_ok());
    }

    #[test]
    fn change_password_command_rejects_wrong_current_password() {
        // Arrange
        let app = app_with_vault();

        // Act
        let result = change_password(app.state(), "sbagliata-sbagliata".into(), "nuova-password-12".into());

        // Assert
        assert!(result.is_err());
    }
}
