# onote

A minimalist voice notes app with local AI transcription. Your notes stay on your device.

## Features

- **Voice Recording** - Record audio and transcribe to text with one click
- **Local Whisper AI** - Speech-to-text runs entirely on your machine, no cloud required
- **Markdown Notes** - Plain text files you own, organized in folders
- **6 Themes** - Dark and light themes with customizable aesthetics
- **Cross-Platform** - Windows, macOS, and Linux

## Screenshots

*Coming soon*

## Download

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| Windows | [onote_x64-setup.exe](https://github.com/opossumactual/onote/releases/latest) |
| macOS (Apple Silicon) | [onote_aarch64.dmg](https://github.com/opossumactual/onote/releases/latest) |
| macOS (Intel) | [onote_x64.dmg](https://github.com/opossumactual/onote/releases/latest) |
| Linux (Debian/Ubuntu) | [onote_amd64.deb](https://github.com/opossumactual/onote/releases/latest) |
| Linux (Fedora/RHEL) | [onote_x86_64.rpm](https://github.com/opossumactual/onote/releases/latest) |
| Linux (Universal) | [onote_amd64.AppImage](https://github.com/opossumactual/onote/releases/latest) |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New note |
| `Ctrl+R` | Start/stop recording |
| `Ctrl+S` | Save note |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+L` | Toggle note list |
| `Ctrl+/` | Show all shortcuts |
| `Delete` | Delete selected note |
| `Esc` | Close dialogs |

## How It Works

1. **Record** - Click the record button or press `Ctrl+R`
2. **Transcribe** - Audio is processed locally using Whisper AI
3. **Edit** - Transcription appears in your note, ready to edit
4. **Save** - Notes are saved as markdown files in `~/Documents/opnotes/`

### Whisper Models

On first use, you'll need to download a Whisper model in Settings:

| Model | Size | Speed | Accuracy |
|-------|------|-------|----------|
| tiny.en | 75 MB | Fastest | Good |
| base.en | 142 MB | Fast | Better |
| small.en | 466 MB | Medium | Great |
| medium.en | 1.5 GB | Slow | Best |

Models are stored locally in `~/.local/share/onote/models/`

## Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/) with runes
- **Backend**: [Tauri v2](https://tauri.app/) (Rust)
- **Audio**: [cpal](https://github.com/RustAudio/cpal) for cross-platform capture
- **Transcription**: [whisper-rs](https://github.com/tazz4843/whisper-rs) (OpenAI Whisper)

## Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- Platform-specific dependencies (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

Outputs will be in `src-tauri/target/release/bundle/`

## Privacy

- All transcription happens locally on your device
- No data is sent to any server
- Notes are plain markdown files you can read with any text editor
- No account required, no telemetry

## License

MIT

## Contributing

Contributions welcome! Please open an issue to discuss changes before submitting a PR.
