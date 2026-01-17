# ghostnote

A security-focused voice notes app with local AI transcription and AES-256 encryption. Your notes are encrypted at rest and never leave your device.

## Features

- **End-to-End Encryption** - AES-256-GCM encryption for all notes at rest
- **Master Password** - Vault protected by Argon2id key derivation
- **Recovery Key** - 24-character recovery key for password reset
- **Memory-Only Audio** - Recordings are transcribed in RAM, never written to disk
- **Auto-Lock** - Configurable timeout locks vault after inactivity
- **Voice Recording** - Record audio and transcribe to text with one click
- **Local Whisper AI** - Speech-to-text runs entirely on your machine
- **Markdown Notes** - Encrypted files you own, organized in folders
- **10 Themes** - Dark stealth themes (Covert, Obsidian, Stealth, Midnight) plus classics

## Download

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| Windows | [ghostnote_x64-setup.exe](https://github.com/opossumactual/ghostnote/releases/latest) |
| macOS (Apple Silicon) | [ghostnote_aarch64.dmg](https://github.com/opossumactual/ghostnote/releases/latest) |
| macOS (Intel) | [ghostnote_x64.dmg](https://github.com/opossumactual/ghostnote/releases/latest) |
| Linux (Debian/Ubuntu) | [ghostnote_amd64.deb](https://github.com/opossumactual/ghostnote/releases/latest) |
| Linux (Fedora/RHEL) | [ghostnote_x86_64.rpm](https://github.com/opossumactual/ghostnote/releases/latest) |
| Linux (Universal) | [ghostnote_amd64.AppImage](https://github.com/opossumactual/ghostnote/releases/latest) |

## Linux Installation

### Debian/Ubuntu
```bash
sudo dpkg -i ghostnote_0.1.0_amd64.deb
```

### Fedora/RHEL
```bash
sudo rpm -i ghostnote-0.1.0-1.x86_64.rpm
```

### Arch Linux
Extract the deb and copy the binary:
```bash
cd /tmp
ar x ghostnote_0.1.0_amd64.deb
tar xf data.tar.gz
cp usr/bin/opnotes ~/.local/bin/gnote
chmod +x ~/.local/bin/gnote
```

Add `~/.local/bin` to your PATH if not already (add to `~/.bashrc`):
```bash
export PATH="$HOME/.local/bin:$PATH"
```

Optional alias for background launch (add to `~/.bashrc`):
```bash
alias gnote='~/.local/bin/gnote &>/dev/null & disown'
```

### AppImage (Universal)
```bash
chmod +x ghostnote_0.1.0_amd64.AppImage
./ghostnote_0.1.0_amd64.AppImage
```

## macOS Installation

1. Download the `.dmg` for your Mac (Apple Silicon or Intel)
2. Open the `.dmg` and drag ghostnote to Applications
3. Install whisper-cpp for transcription:
   ```bash
   brew install whisper-cpp
   ```
4. On first launch, you may need to bypass Gatekeeper:
   ```bash
   xattr -cr /Applications/ghostnote.app
   ```

## First Run Setup

1. **Create Master Password** - Choose a strong password (min 8 characters)
2. **Save Recovery Key** - Copy and store your 24-character recovery key safely
3. **Confirm** - Acknowledge that without password or recovery key, notes cannot be recovered

Your vault is now encrypted and ready to use.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New note |
| `Ctrl+R` | Start/stop recording |
| `Ctrl+S` | Save note |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+L` | Toggle note list |
| `Ctrl+/` | Show all shortcuts |
| `Ctrl+D` | Delete selected note/folder |
| Arrow keys | Navigate folders/notes/editor |
| `Esc` | Close dialogs / exit editor |

## How It Works

1. **Unlock** - Enter your master password to unlock the vault
2. **Record** - Click the record button or press `Ctrl+R`
3. **Transcribe** - Audio is processed in memory using Whisper AI (never saved to disk)
4. **Edit** - Transcription appears in your note, encrypted on save
5. **Lock** - Vault auto-locks after inactivity or manually via Settings

### Security Architecture

- **KEK (Key Encryption Key)** - Derived from your password using Argon2id
- **DEK (Data Encryption Key)** - Unique per note, wrapped by KEK
- **AES-256-GCM** - Industry-standard authenticated encryption
- **Zeroize** - Keys are securely cleared from memory when locked

### Whisper Models

On first use, download a Whisper model in Settings:

| Model | Size | Speed | Accuracy |
|-------|------|-------|----------|
| tiny.en | 75 MB | Fastest | Good |
| base.en | 142 MB | Fast | Better |
| small.en | 466 MB | Medium | Great |
| medium.en | 1.5 GB | Slow | Best |

Models are stored in `~/.local/share/ghostnote/models/`

## Data Storage

| Data | Location |
|------|----------|
| Encrypted notes | `~/Documents/ghostnote/` |
| Vault files | `~/Documents/ghostnote/.vault/` |
| Whisper models | `~/.local/share/ghostnote/models/` |

## Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/) with runes
- **Backend**: [Tauri v2](https://tauri.app/) (Rust)
- **Encryption**: [aes-gcm](https://crates.io/crates/aes-gcm), [argon2](https://crates.io/crates/argon2)
- **Audio**: [cpal](https://github.com/RustAudio/cpal) for cross-platform capture
- **Transcription**: [whisper-rs](https://github.com/tazz4843/whisper-rs) on Linux/Windows, [whisper-cli](https://github.com/ggerganov/whisper.cpp) on macOS

## Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- Platform-specific dependencies (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

### Development

```bash
git clone https://github.com/opossumactual/ghostnote.git
cd ghostnote

npm install
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

Outputs will be in `src-tauri/target/release/bundle/`

## Privacy & Security

- All transcription happens locally - no cloud APIs
- Audio exists only in memory during transcription
- Notes encrypted with AES-256-GCM before writing to disk
- No account required, no telemetry, no network access
- Recovery key is the only way to reset a forgotten password

## License

MIT

## Contributing

Contributions welcome! Please open an issue to discuss changes before submitting a PR.
