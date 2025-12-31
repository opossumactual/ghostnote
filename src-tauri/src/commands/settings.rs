use crate::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub notes_dir: String,
    pub model: String,
    pub font_size: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_dir = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opnotes");

        Self {
            notes_dir: default_dir.to_string_lossy().to_string(),
            model: "small.en".to_string(),
            font_size: 16,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opnotes")
        .join("config.json")
}

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    let path = config_path();

    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(AppSettings::default())
    }
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, state: State<AppState>) -> Result<(), String> {
    let path = config_path();

    // Ensure config directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Update app state with new notes directory
    {
        let mut notes_dir = state.notes_dir.lock().unwrap();
        *notes_dir = PathBuf::from(&settings.notes_dir);
    }

    // Ensure notes directory exists
    fs::create_dir_all(&settings.notes_dir).map_err(|e| e.to_string())?;

    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}
