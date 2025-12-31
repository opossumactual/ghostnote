use crate::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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
pub fn list_notes(folder: String, state: State<AppState>) -> Result<Vec<NoteMeta>, String> {
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
                if ext == "md" || ext == "txt" {
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

    // Sort by modified date (newest first) - for now by name
    notes.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(notes)
}

#[tauri::command]
pub fn read_note(path: String, state: State<AppState>) -> Result<NoteContent, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let full_path = notes_dir.join(&path);

    let content = fs::read_to_string(&full_path).map_err(|e| e.to_string())?;

    Ok(NoteContent { path, content })
}

#[tauri::command]
pub fn save_note(path: String, content: String, state: State<AppState>) -> Result<(), String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let full_path = notes_dir.join(&path);

    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&full_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_note(folder: String, title: Option<String>, state: State<AppState>) -> Result<String, String> {
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

    let filename = format!("{}-{}.md", date_str, slug);
    let mut full_path = folder_path.join(&filename);

    // Handle duplicates
    let mut counter = 1;
    while full_path.exists() {
        let new_filename = format!("{}-{}-{}.md", date_str, slug, counter);
        full_path = folder_path.join(&new_filename);
        counter += 1;
    }

    // Create with initial content
    let initial_content = title
        .map(|t| format!("# {}\n\n", t))
        .unwrap_or_else(|| "# Untitled\n\n".to_string());

    fs::write(&full_path, initial_content).map_err(|e| e.to_string())?;

    let rel_path = full_path.strip_prefix(&notes_dir).unwrap_or(&full_path);
    Ok(rel_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn delete_note(path: String, state: State<AppState>) -> Result<(), String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let full_path = notes_dir.join(&path);

    fs::remove_file(full_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_notes(query: String, state: State<AppState>) -> Result<Vec<SearchResult>, String> {
    let notes_dir = state.notes_dir.lock().unwrap().clone();
    let query_lower = query.to_lowercase();

    let mut results = Vec::new();

    for entry in WalkDir::new(&notes_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "md" || ext == "txt")
                .unwrap_or(false)
        })
    {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
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
                let rel_path = path.strip_prefix(&notes_dir).unwrap_or(path);
                results.push(SearchResult {
                    path: rel_path.to_string_lossy().to_string(),
                    title: extract_title(&content, path),
                    matches,
                });
            }
        }
    }

    Ok(results)
}
