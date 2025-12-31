---
name: tauri-v2
description: Build cross-platform desktop applications with Tauri v2 and Rust backend. Use when creating desktop apps with web frontends, implementing native features like file system access, system tray, or platform-specific functionality. Covers Tauri commands, state management, plugin system, and IPC communication.
license: MIT
---

# Tauri v2 Development

Tauri v2 is a framework for building desktop apps with web frontends and Rust backends. Apps are lightweight (typically 2-10MB), secure by default, and cross-platform.

## Project Structure

```
project/
├── src/                    # Frontend (Svelte/React/Vue)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   └── lib.rs         # Commands and state
│   ├── Cargo.toml
│   ├── tauri.conf.json    # App configuration
│   └── capabilities/      # Permission definitions
└── package.json
```

## Creating Commands

Commands are Rust functions callable from JavaScript:

```rust
// src-tauri/src/lib.rs
use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[tauri::command]
async fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// Register commands in main.rs or lib.rs
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, read_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Calling from Frontend

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<string>('greet', { name: 'World' });
const content = await invoke<string>('read_file', { path: '/tmp/test.txt' });
```

## State Management

Use `tauri::State` for shared application state:

```rust
use std::sync::Mutex;
use tauri::State;

struct AppState {
    counter: Mutex<i32>,
}

#[tauri::command]
fn increment(state: State<AppState>) -> i32 {
    let mut counter = state.counter.lock().unwrap();
    *counter += 1;
    *counter
}

// In builder
.manage(AppState { counter: Mutex::new(0) })
```

## Async Commands

For I/O operations, use async commands:

```rust
#[tauri::command]
async fn fetch_data(url: String) -> Result<String, String> {
    reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}
```

## Event System

### Emit from Rust

```rust
use tauri::Emitter;

#[tauri::command]
fn start_process(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        app.emit("progress", 50).unwrap();
        app.emit("progress", 100).unwrap();
    });
}
```

### Listen in Frontend

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<number>('progress', (event) => {
    console.log('Progress:', event.payload);
});

// Cleanup
unlisten();
```

## File System Access

```rust
use std::fs;
use std::path::PathBuf;

#[tauri::command]
async fn list_directory(path: &str) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    Ok(entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect())
}

#[tauri::command]
async fn save_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}
```

## Capabilities (Permissions)

Tauri v2 requires explicit permissions in `src-tauri/capabilities/`:

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "description": "Default permissions",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:default",
    "fs:allow-read",
    "fs:allow-write",
    "dialog:default"
  ]
}
```

## Configuration

`tauri.conf.json` key settings:

```json
{
  "productName": "My App",
  "version": "1.0.0",
  "identifier": "com.example.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "My App",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "decorations": true
      }
    ]
  }
}
```

## Common Patterns

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::ser::Serializer {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
fn risky_operation() -> Result<String, AppError> {
    // ...
}
```

### Window Management

```rust
use tauri::Manager;

#[tauri::command]
async fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    tauri::WebviewWindowBuilder::new(
        &app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into())
    )
    .title("Settings")
    .inner_size(400.0, 300.0)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}
```

## Development Workflow

```bash
# Initialize project
npm create tauri-app@latest

# Development
npm run tauri dev

# Build for production
npm run tauri build

# Add Rust dependencies
cd src-tauri && cargo add serde --features derive
```

## Common Pitfalls

1. **Blocking the main thread**: Use `async` commands for I/O
2. **Missing permissions**: Check capabilities for fs/dialog access
3. **Path handling**: Use `tauri::path` API for cross-platform paths
4. **State deadlocks**: Don't hold mutex locks across await points
