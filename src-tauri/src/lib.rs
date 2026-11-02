mod generator;
mod store;

use std::sync::Mutex;
use store::{Entry, EntryInput, Store};
use tauri::{Manager, State};
use uuid::Uuid;

type VaultState<'a> = State<'a, Mutex<Store>>;

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

    fn app_with_vault() -> tauri::App<MockRuntime> {
        let dir = std::env::temp_dir().join(format!("forziere-cmd-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let app = mock_app();
        app.manage(Mutex::new(Store::load(dir.join("vault.json")).unwrap()));
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
}
