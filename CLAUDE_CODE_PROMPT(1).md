# Voice Notes App - Claude Code Project Prompt

## Project Overview

Build a minimalist, utilitarian desktop note-taking application inspired by Apple Notes with integrated speech-to-text transcription. The app should feel refined and professional while remaining simple to use.

**Stack**: Svelte 5 (with runes) + Tauri v2 + whisper-rs for local STT

## Core Requirements

### 1. Note Management
- Create, edit, delete notes stored as plain text/markdown files
- Notes organized in folders (flat or nested)
- Full-text search across all notes
- Auto-save with debouncing
- Note metadata: created date, modified date, word count

### 2. Voice Transcription
- Push-to-talk or toggle recording button
- Real-time audio capture via Tauri Rust backend
- Local transcription using whisper.cpp/whisper-rs (no cloud)
- Transcribed text appends to current note or creates new note
- Visual recording indicator with waveform or pulse animation
- Support for whisper models: tiny, base, small (user-configurable)

### 3. User Interface

**Layout**: Three-panel design
- Left sidebar: Folder tree + search
- Middle panel: Note list with previews
- Right panel: Note editor

**Dark Theme Requirements**:
- Base background: `#1a1a1a` to `#0f0f0f` (NOT pure black)
- Elevated surfaces: `#242424`, `#2a2a2a` for depth
- Text: `#e0e0e0` primary, `#888888` secondary
- Single accent color (user preference or default blue-gray `#5c7cfa`)
- Subtle borders: `#333333`
- No harsh contrasts - everything should feel cohesive

**Typography**:
- Monospace option for note content (JetBrains Mono, Fira Code)
- System UI font for interface elements
- Comfortable line height (1.6-1.7)

**Interactions**:
- Keyboard shortcuts for all common actions
- Smooth transitions (150-200ms)
- Subtle hover states
- Focus indicators for accessibility

### 4. File Storage
- Notes stored as `.md` or `.txt` files in a user-configurable directory
- Folder structure mirrors filesystem
- No proprietary database - plain files for portability
- Optional: frontmatter for metadata

## Technical Architecture

### Svelte 5 Frontend (`src/`)
```
src/
├── lib/
│   ├── stores/
│   │   ├── notes.svelte.ts      # Note state management with $state
│   │   ├── folders.svelte.ts    # Folder tree state
│   │   ├── settings.svelte.ts   # App preferences
│   │   └── recording.svelte.ts  # Recording state
│   ├── components/
│   │   ├── Sidebar.svelte
│   │   ├── NoteList.svelte
│   │   ├── Editor.svelte
│   │   ├── RecordButton.svelte
│   │   └── SearchBar.svelte
│   └── utils/
│       ├── tauri-commands.ts    # Invoke wrappers
│       └── debounce.ts
├── routes/
│   └── +page.svelte
└── app.css                      # Global styles + CSS variables
```

### Tauri Rust Backend (`src-tauri/`)
```
src-tauri/
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── notes.rs         # CRUD for notes/folders
│   │   ├── transcribe.rs    # Whisper integration
│   │   └── audio.rs         # Mic recording
│   └── audio/
│       ├── mod.rs
│       ├── recorder.rs      # cpal-based recording
│       └── processor.rs     # Audio preprocessing
├── Cargo.toml
└── tauri.conf.json
```

### Key Tauri Commands
```rust
#[tauri::command]
async fn list_notes(folder: &str) -> Result<Vec<NoteMeta>, String>

#[tauri::command]
async fn read_note(path: &str) -> Result<NoteContent, String>

#[tauri::command]
async fn save_note(path: &str, content: &str) -> Result<(), String>

#[tauri::command]
async fn start_recording() -> Result<(), String>

#[tauri::command]
async fn stop_recording() -> Result<String, String> // Returns transcription

#[tauri::command]
async fn search_notes(query: &str, folder: &str) -> Result<Vec<SearchResult>, String>
```

## Implementation Phases

### Phase 1: Core Note App
1. Set up Tauri v2 + Svelte 5 project
2. Implement file-based note storage (read/write/list)
3. Build three-panel UI layout
4. Add search functionality
5. Implement dark theme with CSS variables

### Phase 2: Voice Transcription
1. Add whisper-rs dependency and model downloading
2. Implement audio recording with cpal
3. Create transcription pipeline
4. Add recording UI with visual feedback
5. Connect transcription output to editor

### Phase 3: Polish
1. Keyboard shortcuts
2. Settings panel (theme, model size, notes directory)
3. Performance optimization
4. Error handling and user feedback

## Design References

Look at these apps for inspiration:
- **Bear** - Clean dark mode, excellent typography
- **iA Writer** - Focus mode, distraction-free
- **Obsidian** - File-based notes, three-panel layout
- **Apple Notes** - Adaptive toolbar, folder organization

## Key Dependencies

### Rust (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
whisper-rs = "0.11"
cpal = "0.15"
hound = "3.5"  # WAV file handling
serde = { version = "1", features = ["derive"] }
serde_json = "1"
walkdir = "2"  # Directory traversal
grep-regex = "0.1"  # Search
tokio = { version = "1", features = ["full"] }
```

### JavaScript (package.json)
```json
{
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^4",
    "@tauri-apps/cli": "^2",
    "svelte": "^5",
    "vite": "^5"
  },
  "dependencies": {
    "@tauri-apps/api": "^2"
  }
}
```

## Success Criteria

- App launches in under 2 seconds
- Notes save within 100ms of typing pause
- Transcription begins within 500ms of recording stop
- UI remains responsive during transcription
- All data stored as portable plain text files
- Works fully offline
