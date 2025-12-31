mod commands;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub notes_dir: Mutex<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        let default_dir = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opnotes");

        Self {
            notes_dir: Mutex::new(default_dir),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .setup(|app| {
            // Ensure notes directory exists
            let state = app.state::<AppState>();
            let notes_dir = state.notes_dir.lock().unwrap().clone();
            if !notes_dir.exists() {
                std::fs::create_dir_all(&notes_dir).ok();
                // Create default inbox folder
                std::fs::create_dir_all(notes_dir.join("inbox")).ok();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::notes::list_folders,
            commands::notes::list_notes,
            commands::notes::read_note,
            commands::notes::save_note,
            commands::notes::create_note,
            commands::notes::delete_note,
            commands::notes::search_notes,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
