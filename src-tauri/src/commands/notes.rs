use crate::commands::vault::{decrypt, encrypt, unwrap_dek, wrap_dek, Dek, VaultState};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub name: String,
    pub path: String,
    pub children: Vec<FolderInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteMeta {
    pub id: String,
    pub path: String,
    pub title: String,
    pub preview: String,
    pub modified: String,
    pub word_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteContent {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub matches: Vec<SearchMatch>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line_content: String,
}

fn extract_title(content: &str, path: &Path) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return trimmed[2..].to_string();
        }
    }
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

fn extract_preview(content: &str) -> String {
    let text: String = content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");

    if text.len() > 100 {
        format!("{}...", &text[..100])
    } else {
        text
    }
}

fn format_date(time: std::time::SystemTime) -> String {
    use chrono::{DateTime, Local};
    let datetime: DateTime<Local> = time.into();
    datetime.format("%b %d").to_string()
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Get the encrypted file path (.enc) from a base path
fn enc_path(path: &Path) -> PathBuf {
    path.with_extension("enc")
}

/// Get the key file path (.key) from a base path
fn key_path(path: &Path) -> PathBuf {
    path.with_extension("key")
}

/// Check if a note is encrypted (has .enc file)
fn is_encrypted(notes_dir: &Path, rel_path: &str) -> bool {
    let base_path = notes_dir.join(rel_path);
    enc_path(&base_path).exists()
}

/// Read and decrypt a note's content
fn read_encrypted_note(
    notes_dir: &Path,
    rel_path: &str,
    vault: &VaultState,
) -> Result<String, String> {
    let base_path = notes_dir.join(rel_path);
    let enc_file = enc_path(&base_path);
    let key_file = key_path(&base_path);

    // Read wrapped DEK
    let wrapped_dek = fs::read(&key_file)
        .map_err(|e| format!("Failed to read key file: {}", e))?;

    // Unwrap DEK with KEK
    let dek = vault.with_kek(|kek| unwrap_dek(kek, &wrapped_dek))?;

    // Read and decrypt content
    let encrypted_content = fs::read(&enc_file)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let decrypted = decrypt(dek.as_bytes(), &encrypted_content)?;

    String::from_utf8(decrypted)
        .map_err(|e| format!("Invalid UTF-8 in decrypted content: {}", e))
}

/// Encrypt and save a note's content
fn write_encrypted_note(
    notes_dir: &Path,
    rel_path: &str,
    content: &str,
    vault: &VaultState,
    existing_dek: Option<Dek>,
) -> Result<(), String> {
    let base_path = notes_dir.join(rel_path);
    let enc_file = enc_path(&base_path);
    let key_file = key_path(&base_path);

    // Ensure parent directory exists
    if let Some(parent) = base_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Use existing DEK or generate new one
    let dek = existing_dek.unwrap_or_else(Dek::generate);

    // Encrypt content with DEK
    let encrypted_content = encrypt(dek.as_bytes(), content.as_bytes())?;

    // Wrap DEK with KEK
    let wrapped_dek = vault.with_kek(|kek| wrap_dek(kek, &dek))?;

    // Write both files
    fs::write(&enc_file, &encrypted_content)
        .map_err(|e| format!("Failed to write encrypted file: {}", e))?;
    fs::write(&key_file, &wrapped_dek)
        .map_err(|e| format!("Failed to write key file: {}", e))?;

    Ok(())
}

/// Delete encrypted note files
fn delete_encrypted_note(notes_dir: &Path, rel_path: &str) -> Result<(), String> {
    let base_path = notes_dir.join(rel_path);
    let enc_file = enc_path(&base_path);
    let key_file = key_path(&base_path);

    // Remove both files (ignore errors if they don't exist)
    let _ = fs::remove_file(&enc_file);
    let _ = fs::remove_file(&key_file);

    Ok(())
}

#[tauri::command]
pub fn list_folders(state: State<AppState>) -> Result<Vec<FolderInfo>, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();

    fn build_tree(dir: &Path, base: &Path) -> Vec<FolderInfo> {
        let mut folders = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                    let rel_path = path.strip_prefix(base).unwrap_or(&path);
                    folders.push(FolderInfo {
                        name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                        path: rel_path.to_string_lossy().to_string(),
                        children: build_tree(&path, base),
                    });
                }
            }
        }

        folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        folders
    }

    Ok(build_tree(&notes_dir, &notes_dir))
}

#[tauri::command]
pub fn list_notes(
    folder: String,
    state: State<AppState>,
    vault: State<VaultState>,
) -> Result<Vec<NoteMeta>, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let folder_path = notes_dir.join(&folder);

    if !folder_path.exists() {
        return Ok(Vec::new());
    }

    let mut notes = Vec::new();

    if let Ok(entries) = fs::read_dir(&folder_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                // Handle encrypted files (.enc)
                if ext == "enc" {
                    // Get the base path (without .enc extension)
                    let base_path = path.with_extension("");
                    let rel_base = base_path
                        .strip_prefix(&notes_dir)
                        .unwrap_or(&base_path)
                        .to_string_lossy()
                        .to_string();

                    // Try to decrypt and read content
                    if let Ok(content) = read_encrypted_note(&notes_dir, &rel_base, &vault) {
                        let metadata = fs::metadata(&path).ok();
                        let modified = metadata
                            .and_then(|m| m.modified().ok())
                            .map(format_date)
                            .unwrap_or_else(|| "Unknown".to_string());

                        notes.push(NoteMeta {
                            id: rel_base.clone(),
                            path: rel_base,
                            title: extract_title(&content, &path),
                            preview: extract_preview(&content),
                            modified,
                            word_count: content.split_whitespace().count(),
                        });
                    }
                }
                // Also handle legacy unencrypted files (.md, .txt)
                else if ext == "md" || ext == "txt" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let metadata = fs::metadata(&path).ok();
                        let modified = metadata
                            .and_then(|m| m.modified().ok())
                            .map(format_date)
                            .unwrap_or_else(|| "Unknown".to_string());

                        let rel_path = path.strip_prefix(&notes_dir).unwrap_or(&path);

                        notes.push(NoteMeta {
                            id: rel_path.to_string_lossy().to_string(),
                            path: rel_path.to_string_lossy().to_string(),
                            title: extract_title(&content, &path),
                            preview: extract_preview(&content),
                            modified,
                            word_count: content.split_whitespace().count(),
                        });
                    }
                }
            }
        }
    }

    // Sort by modified date (newest first)
    notes.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(notes)
}

#[tauri::command]
pub fn read_note(
    path: String,
    state: State<AppState>,
    vault: State<VaultState>,
) -> Result<NoteContent, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();

    // Check if encrypted version exists
    if is_encrypted(&notes_dir, &path) {
        let content = read_encrypted_note(&notes_dir, &path, &vault)?;
        Ok(NoteContent { path, content })
    } else {
        // Fall back to legacy unencrypted read
        let full_path = notes_dir.join(&path);
        let content = fs::read_to_string(&full_path).map_err(|e| e.to_string())?;
        Ok(NoteContent { path, content })
    }
}

#[tauri::command]
pub fn save_note(
    path: String,
    content: String,
    state: State<AppState>,
    vault: State<VaultState>,
) -> Result<(), String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();

    // Always save as encrypted
    write_encrypted_note(&notes_dir, &path, &content, &vault, None)
}

#[tauri::command]
pub fn create_note(
    folder: String,
    title: Option<String>,
    state: State<AppState>,
    vault: State<VaultState>,
) -> Result<String, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let folder_path = notes_dir.join(&folder);

    // Ensure folder exists
    fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;

    let now = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let slug = title
        .as_ref()
        .map(|t| slugify(t))
        .unwrap_or_else(|| "untitled".to_string());

    // Use base name without extension (we'll add .enc and .key)
    let base_name = format!("{}-{}", date_str, slug);
    let mut base_path = folder_path.join(&base_name);

    // Handle duplicates (check for .enc file)
    let mut counter = 1;
    while enc_path(&base_path).exists() {
        let new_base_name = format!("{}-{}-{}", date_str, slug, counter);
        base_path = folder_path.join(&new_base_name);
        counter += 1;
    }

    // Create with initial content (encrypted)
    let initial_content = title
        .as_ref()
        .map(|t| format!("# {}\n\n", t))
        .unwrap_or_else(|| "# Untitled\n\n".to_string());

    let rel_path = base_path
        .strip_prefix(&notes_dir)
        .unwrap_or(&base_path)
        .to_string_lossy()
        .to_string();

    write_encrypted_note(&notes_dir, &rel_path, &initial_content, &vault, None)?;

    Ok(rel_path)
}

#[tauri::command]
pub fn delete_note(path: String, state: State<AppState>) -> Result<(), String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();

    // Check if encrypted
    if is_encrypted(&notes_dir, &path) {
        delete_encrypted_note(&notes_dir, &path)
    } else {
        // Legacy unencrypted delete
        let full_path = notes_dir.join(&path);
        fs::remove_file(full_path).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn create_folder(name: String, parent: Option<String>, state: State<AppState>) -> Result<String, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();

    let folder_path = if let Some(parent_path) = parent {
        notes_dir.join(&parent_path).join(&name)
    } else {
        notes_dir.join(&name)
    };

    fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;

    let rel_path = folder_path.strip_prefix(&notes_dir).unwrap_or(&folder_path);
    Ok(rel_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn delete_folder(path: String, state: State<AppState>) -> Result<(), String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let full_path = notes_dir.join(&path);

    // Recursively delete folder and all contents
    fs::remove_dir_all(&full_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_folder(old_path: String, new_name: String, state: State<AppState>) -> Result<String, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let old_full_path = notes_dir.join(&old_path);

    // Get parent directory
    let parent = old_full_path.parent()
        .ok_or_else(|| "Invalid folder path".to_string())?;

    let new_full_path = parent.join(&new_name);

    // Check if target already exists
    if new_full_path.exists() {
        return Err(format!("A folder named '{}' already exists", new_name));
    }

    fs::rename(&old_full_path, &new_full_path).map_err(|e| e.to_string())?;

    let rel_path = new_full_path.strip_prefix(&notes_dir).unwrap_or(&new_full_path);
    Ok(rel_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn search_notes(
    query: String,
    state: State<AppState>,
    vault: State<VaultState>,
) -> Result<Vec<SearchResult>, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let query_lower = query.to_lowercase();

    let mut results = Vec::new();

    for entry in WalkDir::new(&notes_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "enc" || ext == "md" || ext == "txt")
                .unwrap_or(false)
        })
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Get content (encrypted or plain)
        let content = if ext == "enc" {
            let base_path = path.with_extension("");
            let rel_base = base_path
                .strip_prefix(&notes_dir)
                .unwrap_or(&base_path)
                .to_string_lossy()
                .to_string();
            read_encrypted_note(&notes_dir, &rel_base, &vault).ok()
        } else {
            fs::read_to_string(path).ok()
        };

        if let Some(content) = content {
            let mut matches = Vec::new();

            for (line_num, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    matches.push(SearchMatch {
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                    });
                }
            }

            if !matches.is_empty() {
                // Use base path for encrypted files
                let rel_path = if ext == "enc" {
                    let base_path = path.with_extension("");
                    base_path
                        .strip_prefix(&notes_dir)
                        .unwrap_or(&base_path)
                        .to_string_lossy()
                        .to_string()
                } else {
                    path.strip_prefix(&notes_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string()
                };

                results.push(SearchResult {
                    path: rel_path,
                    title: extract_title(&content, path),
                    matches,
                });
            }
        }
    }

    Ok(results)
}
