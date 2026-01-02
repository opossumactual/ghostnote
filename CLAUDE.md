# CLAUDE.md - onote

A minimalist voice notes app with local Whisper transcription.

## Tech Stack

- **Frontend**: Svelte 5 (runes: `$state`, `$derived`, `$effect`, `$props`)
- **Backend**: Tauri v2 (Rust)
- **Audio**: cpal for cross-platform audio capture
- **Transcription**: whisper-rs for local speech-to-text
- **Storage**: Markdown files in `~/Documents/opnotes/`

## Project Structure

```
src/
├── App.svelte              # Main app layout, keyboard shortcuts, theme init
├── lib/
│   ├── components/
│   │   ├── Toolbar.svelte        # Top bar with window controls
│   │   ├── Sidebar.svelte        # Folder tree, search, new folder
│   │   ├── NoteList.svelte       # Note list with delete buttons
│   │   ├── Editor.svelte         # Markdown editor textarea
│   │   ├── StatusBar.svelte      # Word count, save status
│   │   ├── RecordButton.svelte   # Recording button with spinner
│   │   ├── Settings.svelte       # Audio, model, and theme selection
│   │   └── KeyboardShortcuts.svelte  # Shortcuts overlay (Ctrl+/)
│   ├── stores/
│   │   ├── notes.svelte.ts       # Notes & folders state
│   │   ├── editor.svelte.ts      # Editor content & save state
│   │   ├── recording.svelte.ts   # Recording state & actions
│   │   ├── theme.svelte.ts       # Theme state & persistence
│   │   └── ui.svelte.ts          # UI state (panels, settings)
│   ├── themes.ts                 # Theme definitions (6 presets)
│   └── utils/
│       └── tauri-commands.ts     # Typed Tauri command wrappers

src-tauri/
├── src/
│   ├── lib.rs                    # App state, Tauri setup
│   └── commands/
│       ├── mod.rs
│       ├── notes.rs              # CRUD for notes & folders
│       ├── settings.rs           # App settings
│       ├── audio.rs              # Recording with cpal
│       └── whisper.rs            # Transcription with whisper-rs
├── Cargo.toml
└── tauri.conf.json
```

## Key Commands

```bash
# Development
npm run tauri dev        # Start dev server with hot reload

# Build
npm run tauri build      # Build production release

# Rust checks
cd src-tauri && cargo check
```

## Theme System

6 built-in themes with localStorage persistence:

| Theme | Type | Accent |
|-------|------|--------|
| Coral Terminal (default) | Dark | #ff8652 |
| Synthwave | Dark | #ff6b9d |
| Amber Terminal | Dark | #ffb347 |
| Matrix | Dark | #39ff14 |
| Light Classic | Light | #0066cc |
| Light Warm | Light | #d97706 |

Themes are defined in `src/lib/themes.ts` and applied via CSS custom properties.

## Architecture Notes

### Audio Recording (audio.rs)
- Uses atomic bools (`RECORDING_ACTIVE`, `STOP_SIGNAL`) because `cpal::Stream` is not `Send`
- Recording runs in a spawned thread, samples stored in ring buffer
- Supports F32, I16, U16, I32, U32, I8, U8 sample formats
- Resamples to 16kHz mono for Whisper compatibility
- Saves WAV to `~/Documents/opnotes/.audio/`

### Whisper Transcription (whisper.rs)
- Models downloaded from HuggingFace to `~/.local/share/onote/models/`
- Available models: tiny.en, base.en, small.en, medium.en
- Transcription runs in `tokio::task::spawn_blocking` to avoid UI freeze
- Returns plain text, auto-inserted at cursor position

### State Management (Svelte 5 Runes)
- Stores use `$state()` for reactive state
- Export object with getters for reactive reads
- Actions are async functions that update state

### Tauri Commands
- All commands defined in `src-tauri/src/commands/`
- Registered in `lib.rs` via `invoke_handler`
- Frontend wrappers in `tauri-commands.ts` with TypeScript types

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+B | Toggle sidebar |
| Ctrl+L | Toggle note list |
| Ctrl+N | New note |
| Ctrl+R | Toggle recording |
| Ctrl+S | Save note |
| Ctrl+/ | Show shortcuts |
| Delete | Delete selected note |
| Esc | Close dialogs |

## Data Storage

- Notes: `~/Documents/opnotes/<folder>/<date>-<slug>.md`
- Audio: `~/Documents/opnotes/.audio/recording_<timestamp>.wav`
- Models: `~/.local/share/onote/models/ggml-<model>.bin`
- Theme: `localStorage` key `opnotes-theme`

## CI/CD

GitHub Actions workflow in `.github/workflows/release.yml` builds for:
- Windows (x64, ARM64)
- macOS (Intel, Apple Silicon)
- Linux (x64 deb, rpm, AppImage)

Triggered by pushing a version tag: `git tag v0.1.0 && git push origin v0.1.0`

## Known Issues / TODOs

- Svelte warning about `selectedFolder` captured by value in notes store (cosmetic)
- ALSA errors on Linux with PipeWire (works via "pipewire" or "default" device)
- Live/streaming transcription not implemented (batch mode only)

## Dependencies (Cargo.toml)

```toml
whisper-rs = "0.14"      # Local Whisper inference
cpal = "0.15"            # Cross-platform audio
hound = "3.5"            # WAV file handling
ringbuf = "0.4"          # Thread-safe audio buffer
reqwest = "0.12"         # Model downloading
```
