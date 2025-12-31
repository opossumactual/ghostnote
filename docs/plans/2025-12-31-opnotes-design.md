# opnotes Design Document

**Date:** 2025-12-31
**Status:** Approved

## Overview

opnotes is a minimalist desktop note-taking application with integrated speech-to-text transcription. Built with Svelte 5, Tauri v2, and whisper-rs for fully offline voice notes.

## Key Decisions

| Decision | Choice |
|----------|--------|
| Layout | Three-panel, collapsible (focus mode) |
| Transcription insertion | At cursor position |
| Recording trigger | Toggle button + push-to-talk hotkey |
| Whisper model | small.en (466MB), auto-download on first launch |
| Note format | Plain markdown (no frontmatter) |
| Storage location | User-configurable, default `~/Documents/opnotes` |
| Keyboard shortcuts | Fixed for v1 |
| Settings scope | Notes folder, model selection, font size |

---

## Architecture

**Tech Stack:**
- **Frontend:** Svelte 5 with runes
- **Desktop:** Tauri v2 (Rust backend)
- **Speech-to-Text:** whisper-rs with small.en model
- **Storage:** Plain markdown files

**Data Flow:**

```
┌─────────────────────────────────────────────────────────┐
│                    Svelte Frontend                       │
│  ┌──────────┐  ┌───────────┐  ┌──────────────────────┐  │
│  │ Sidebar  │  │ Note List │  │       Editor         │  │
│  │ (folders)│  │ (previews)│  │  (markdown + cursor) │  │
│  └──────────┘  └───────────┘  └──────────────────────┘  │
│                         │                                │
│              invoke() / events                           │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────┴───────────────────────────────┐
│                    Tauri Rust Backend                    │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ File Ops    │  │ Audio Record │  │ Transcription  │  │
│  │ (notes CRUD)│  │ (cpal)       │  │ (whisper-rs)   │  │
│  └─────────────┘  └──────────────┘  └────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                    ~/Documents/opnotes/
```

---

## UI Layout

```
┌────────────────────────────────────────────────────────────────┐
│  opnotes                                    [─] [□] [×]        │
├────────────────────────────────────────────────────────────────┤
│  [+ New]  [🎤]  ──────────────────────────────  [⚙ Settings]  │
├────────────┬─────────────────┬─────────────────────────────────┤
│            │                 │                                 │
│  FOLDERS   │  NOTES          │  EDITOR                         │
│            │                 │                                 │
│  > Inbox   │  Meeting Notes  │  # Meeting Notes                │
│  > Work    │  Jan 15 · 234w  │                                 │
│    > Proj  │  ─────────────  │  Attendees: Alice, Bob          │
│  > Personal│  Quick Ideas    │                                 │
│            │  Jan 14 · 89w   │  ## Discussion                  │
│            │                 │  - Budget review                │
│ ────────── │                 │                                 │
│ [Search...│                 │                                 │
├────────────┴─────────────────┴─────────────────────────────────┤
│  words: 234  ·  saved                                          │
└────────────────────────────────────────────────────────────────┘
```

**Toolbar:**
- `[+ New]` - Create new note
- `[🎤]` - Record button (glows/pulses red when recording)
- `[⚙]` - Settings modal

**Collapse Behavior:**
- `Ctrl+B` toggles sidebar
- `Ctrl+L` toggles note list
- Both collapsed = focus mode

---

## Components

### Svelte Components

| Component | Purpose |
|-----------|---------|
| `App.svelte` | Main layout, panel visibility state |
| `Toolbar.svelte` | New, Record, Settings buttons |
| `Sidebar.svelte` | Folder tree + search input |
| `NoteList.svelte` | Note cards with title, date, word count |
| `Editor.svelte` | Textarea for markdown editing |
| `RecordButton.svelte` | Toggle button with pulse animation |
| `StatusBar.svelte` | Word count, save status |
| `Settings.svelte` | Modal for preferences |

### Record Button States

| State | Appearance |
|-------|------------|
| Idle | Subtle icon, muted color |
| Hover | Slight highlight |
| Recording | Red glow, pulsing animation |
| Processing | Spinner, "Transcribing..." |

---

## Rust Backend

**Module Structure:**

```
src-tauri/src/
├── main.rs
├── lib.rs
├── commands/
│   ├── mod.rs
│   ├── notes.rs
│   ├── search.rs
│   ├── audio.rs
│   ├── transcribe.rs
│   └── settings.rs
├── audio/
│   ├── mod.rs
│   ├── recorder.rs
│   └── processor.rs
└── state.rs
```

**Tauri Commands:**

| Command | Purpose |
|---------|---------|
| `list_folders(root)` | Get folder tree |
| `list_notes(folder)` | Get notes with metadata |
| `read_note(path)` | Load note content |
| `save_note(path, content)` | Write note |
| `create_note(folder, title?)` | Create new .md file |
| `delete_note(path)` | Remove note |
| `search_notes(query)` | Full-text search |
| `start_recording()` | Begin audio capture |
| `stop_recording()` | Stop + return transcription |
| `get_settings()` | Load config |
| `save_settings(config)` | Persist config |
| `download_model(name)` | Fetch whisper model |

**Shared State:**

```rust
pub struct AppState {
    pub notes_dir: Mutex<PathBuf>,
    pub recorder: Mutex<AudioRecorder>,
    pub transcriber: Mutex<Option<Transcriber>>,
}
```

---

## Frontend State

**Store Files:**

```
src/lib/stores/
├── notes.svelte.ts
├── editor.svelte.ts
├── recording.svelte.ts
├── settings.svelte.ts
└── ui.svelte.ts
```

**State Shape:**

```typescript
// notes.svelte.ts
folders: Folder[]
notes: NoteMeta[]
selectedNoteId: string | null

// editor.svelte.ts
content: string
path: string | null
isDirty: boolean
isSaving: boolean
cursorPosition: number

// recording.svelte.ts
status: 'idle' | 'recording' | 'processing'
duration: number

// ui.svelte.ts
sidebarVisible: boolean
noteListVisible: boolean
settingsOpen: boolean
```

---

## First-Run Experience

1. Check for model at `~/.opnotes/models/ggml-small.en.bin`
2. If missing, show download screen with progress bar
3. Download from HuggingFace (~466MB)
4. Initialize transcriber, proceed to main app

**Config Storage:**

```
~/.opnotes/
├── config.json
└── models/
    └── ggml-small.en.bin
```

**config.json:**

```json
{
  "notes_dir": "/home/user/Documents/opnotes",
  "model": "small.en",
  "font_size": 16
}
```

---

## Dark Theme

```css
:root {
  /* Surfaces */
  --surface-0: #0f0f0f;
  --surface-1: #1a1a1a;
  --surface-2: #242424;
  --surface-3: #2a2a2a;

  /* Text */
  --text-primary: #e0e0e0;
  --text-secondary: #888888;
  --text-disabled: #555555;

  /* Accent */
  --accent: #5c7cfa;
  --accent-hover: #748ffc;

  /* Recording */
  --recording: #ff6b6b;
  --recording-glow: rgba(255, 107, 107, 0.4);

  /* Borders */
  --border: rgba(255, 255, 255, 0.08);

  /* Typography */
  --font-ui: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
  --font-size-base: 16px;

  /* Transitions */
  --transition-fast: 150ms ease;
}
```

**Record Button Animation:**

```css
.record-btn.recording {
  background: var(--recording);
  box-shadow: 0 0 20px var(--recording-glow);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { box-shadow: 0 0 10px var(--recording-glow); }
  50% { box-shadow: 0 0 25px var(--recording-glow); }
}
```

---

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New note | `Ctrl+N` |
| Save (force) | `Ctrl+S` |
| Search notes | `Ctrl+F` |
| Toggle recording | `Ctrl+R` |
| Push-to-talk (hold) | `Ctrl+Shift+R` |
| Toggle sidebar | `Ctrl+B` |
| Toggle note list | `Ctrl+L` |
| Focus editor | `Escape` |

---

## Implementation Phases

### Phase 1: Project Scaffolding & Core Notes
- Initialize Tauri v2 + Svelte 5 project
- Set up directory structure
- Implement file-based notes CRUD (Rust)
- Build three-panel layout (collapsible)
- Wire up note listing, reading, saving
- Auto-save with debounce
- Dark theme CSS

### Phase 2: Search & Folders
- Folder tree navigation
- Create/rename/delete folders
- Full-text search
- Search results UI

### Phase 3: Voice Transcription
- First-run model download flow
- Audio recording with cpal
- Whisper transcription integration
- Recording button with glow/pulse
- Insert transcription at cursor
- Push-to-talk hotkey

### Phase 4: Settings & Polish
- Settings modal (notes dir, model, font size)
- Keyboard shortcuts implementation
- Status bar (word count, save state)
- Error handling & user feedback
- Performance optimization

---

## Dependencies

### Rust (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
whisper-rs = "0.11"
cpal = "0.15"
hound = "3.5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
walkdir = "2"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["stream"] }
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

---

## Future Considerations (Out of Scope for v1)

- Mobile version with bundled tiny model
- Customizable keyboard shortcuts
- Light theme option
- YAML frontmatter / tagging
- Markdown preview
- Export to PDF
