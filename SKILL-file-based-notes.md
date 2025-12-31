---
name: file-based-notes
description: Design and implement file-based note storage systems using plain text or markdown files. Use when building note-taking applications that store notes as portable files rather than databases, implementing folder organization, full-text search, and metadata handling without proprietary formats.
license: MIT
---

# File-Based Notes Architecture

Store notes as plain text/markdown files for portability, version control compatibility, and user ownership. No proprietary databases or formats.

## Directory Structure

```
notes/
├── .notesapp/              # App metadata (hidden)
│   ├── config.json
│   └── index.json          # Search index cache
├── inbox/                  # Quick capture
│   └── 2024-01-15-untitled.md
├── work/
│   ├── projects/
│   │   └── website-redesign.md
│   └── meetings/
│       └── 2024-01-15-standup.md
└── personal/
    └── ideas.md
```

## Note File Format

### Simple Markdown

```markdown
# Meeting Notes - January 15

Attendees: Alice, Bob, Carol

## Discussion Points

- Budget review
- Q1 planning
- New hire onboarding

## Action Items

- [ ] Alice: Send budget proposal
- [ ] Bob: Schedule interviews
```

### With YAML Frontmatter (Optional)

```markdown
---
title: Meeting Notes
created: 2024-01-15T10:30:00Z
modified: 2024-01-15T11:45:00Z
tags: [work, meetings]
---

# Meeting Notes

Content here...
```

## Rust Implementation

### Note Data Structures

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteMeta {
    pub path: PathBuf,
    pub title: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub preview: String,      // First ~100 chars
    pub word_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteContent {
    pub path: PathBuf,
    pub content: String,
    pub frontmatter: Option<Frontmatter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created: Option<DateTime<Utc>>,
}
```

### File Operations

```rust
use std::fs;
use walkdir::WalkDir;

pub fn list_notes(folder: &Path) -> Result<Vec<NoteMeta>, String> {
    let mut notes = Vec::new();
    
    for entry in WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "md" || ext == "txt")
                .unwrap_or(false)
        })
    {
        let path = entry.path();
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        
        let title = extract_title(&content, path);
        let preview = extract_preview(&content);
        let word_count = content.split_whitespace().count();
        
        notes.push(NoteMeta {
            path: path.to_path_buf(),
            title,
            created: metadata.created()
                .map(DateTime::<Utc>::from)
                .unwrap_or_else(|_| Utc::now()),
            modified: metadata.modified()
                .map(DateTime::<Utc>::from)
                .unwrap_or_else(|_| Utc::now()),
            preview,
            word_count,
        });
    }
    
    // Sort by modified date, newest first
    notes.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(notes)
}

fn extract_title(content: &str, path: &Path) -> String {
    // Try to extract from first # heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return trimmed[2..].to_string();
        }
    }
    
    // Fall back to filename
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

fn extract_preview(content: &str) -> String {
    // Skip frontmatter and headings
    let text: String = content.lines()
        .skip_while(|l| l.starts_with("---") || l.starts_with("#") || l.trim().is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    
    if text.len() > 100 {
        format!("{}...", &text[..100])
    } else {
        text
    }
}
```

### Create and Save Notes

```rust
pub fn create_note(folder: &Path, title: Option<&str>) -> Result<PathBuf, String> {
    let timestamp = Utc::now().format("%Y-%m-%d");
    let slug = title
        .map(|t| slugify(t))
        .unwrap_or_else(|| "untitled".to_string());
    
    let filename = format!("{}-{}.md", timestamp, slug);
    let path = folder.join(&filename);
    
    // Handle duplicates
    let path = if path.exists() {
        let mut counter = 1;
        loop {
            let new_filename = format!("{}-{}-{}.md", timestamp, slug, counter);
            let new_path = folder.join(&new_filename);
            if !new_path.exists() {
                break new_path;
            }
            counter += 1;
        }
    } else {
        path
    };
    
    // Create with minimal content
    let initial_content = title
        .map(|t| format!("# {}\n\n", t))
        .unwrap_or_default();
    
    fs::write(&path, initial_content).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn save_note(path: &Path, content: &str) -> Result<(), String> {
    // Create backup before overwriting (optional)
    if path.exists() {
        let backup = path.with_extension("md.bak");
        fs::copy(path, backup).ok();
    }
    
    fs::write(path, content).map_err(|e| e.to_string())
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
```

### Full-Text Search

```rust
use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
use grep_searcher::sinks::UTF8;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub title: String,
    pub matches: Vec<SearchMatch>,
}

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line_content: String,
}

pub fn search_notes(folder: &Path, query: &str) -> Result<Vec<SearchResult>, String> {
    let matcher = RegexMatcher::new_line_matcher(&regex::escape(query))
        .map_err(|e| e.to_string())?;
    
    let mut results = Vec::new();
    
    for entry in WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
    {
        let path = entry.path();
        let mut matches = Vec::new();
        
        Searcher::new().search_path(
            &matcher,
            path,
            UTF8(|line_num, line| {
                matches.push(SearchMatch {
                    line_number: line_num as usize,
                    line_content: line.trim().to_string(),
                });
                Ok(true)
            }),
        ).ok();
        
        if !matches.is_empty() {
            let content = fs::read_to_string(path).unwrap_or_default();
            results.push(SearchResult {
                path: path.to_path_buf(),
                title: extract_title(&content, path),
                matches,
            });
        }
    }
    
    Ok(results)
}
```

### Folder Operations

```rust
pub fn list_folders(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut folders = Vec::new();
    
    for entry in WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| !e.path().to_string_lossy().contains(".notesapp"))
    {
        folders.push(entry.path().to_path_buf());
    }
    
    folders.sort();
    Ok(folders)
}

pub fn create_folder(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub fn move_note(from: &Path, to_folder: &Path) -> Result<PathBuf, String> {
    let filename = from.file_name()
        .ok_or("Invalid source path")?;
    let dest = to_folder.join(filename);
    
    fs::rename(from, &dest).map_err(|e| e.to_string())?;
    Ok(dest)
}
```

## Auto-Save Pattern (Frontend)

```typescript
// Svelte store with debounced save
import { invoke } from '@tauri-apps/api/core';

export function createNoteEditor() {
    let content = $state('');
    let path = $state<string | null>(null);
    let isDirty = $state(false);
    let isSaving = $state(false);
    let saveTimeout: ReturnType<typeof setTimeout>;
    
    function setContent(newContent: string) {
        content = newContent;
        isDirty = true;
        
        // Debounce save
        clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => save(), 1000);
    }
    
    async function save() {
        if (!path || !isDirty) return;
        
        isSaving = true;
        try {
            await invoke('save_note', { path, content });
            isDirty = false;
        } catch (e) {
            console.error('Save failed:', e);
        } finally {
            isSaving = false;
        }
    }
    
    return {
        get content() { return content; },
        get isDirty() { return isDirty; },
        get isSaving() { return isSaving; },
        setContent,
        save,
        load: async (notePath: string) => {
            const result = await invoke<{ content: string }>('read_note', { path: notePath });
            path = notePath;
            content = result.content;
            isDirty = false;
        }
    };
}
```

## Benefits of File-Based Storage

1. **Portability** - Notes work in any text editor
2. **Version control** - Git-friendly
3. **No lock-in** - User owns their data
4. **Simplicity** - No database migrations
5. **Backup** - Standard file backup tools work
6. **Sync** - Dropbox/iCloud/Syncthing compatible
