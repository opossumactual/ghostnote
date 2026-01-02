use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::AppState;

// Whisper model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub id: String,
    pub name: String,
    pub size_mb: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub id: String,
    pub downloaded: bool,
    pub path: Option<String>,
    pub size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f32,
}

// Available models with their Hugging Face URLs
const MODELS: &[(&str, &str, u32, &str)] = &[
    (
        "tiny.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin",
        75,
        "Fastest, English only, lower accuracy",
    ),
    (
        "base.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
        142,
        "Fast, English only, good accuracy",
    ),
    (
        "small.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
        466,
        "Balanced speed/accuracy, English only",
    ),
    (
        "medium.en",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin",
        1500,
        "High accuracy, English only, slower",
    ),
];

fn get_models_dir() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opnotes")
        .join("models");
    fs::create_dir_all(&data_dir).ok();
    data_dir
}

fn get_model_path(model_id: &str) -> PathBuf {
    get_models_dir().join(format!("ggml-{}.bin", model_id))
}

#[tauri::command]
pub fn list_whisper_models() -> Vec<WhisperModel> {
    MODELS
        .iter()
        .map(|(id, _, size, desc)| WhisperModel {
            id: id.to_string(),
            name: id.to_string(),
            size_mb: *size,
            description: desc.to_string(),
        })
        .collect()
}

#[tauri::command]
pub fn get_model_status(model_id: String) -> ModelStatus {
    let path = get_model_path(&model_id);
    let downloaded = path.exists();
    let size = MODELS
        .iter()
        .find(|(id, _, _, _)| *id == model_id)
        .map(|(_, _, s, _)| *s)
        .unwrap_or(0);

    ModelStatus {
        id: model_id,
        downloaded,
        path: if downloaded {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        },
        size_mb: size,
    }
}

#[tauri::command]
pub fn get_selected_model(state: State<AppState>) -> String {
    state.selected_model.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_selected_model(model_id: String, state: State<AppState>) -> Result<(), String> {
    let mut selected = state.selected_model.lock().unwrap();
    *selected = model_id;
    Ok(())
}

#[tauri::command]
pub async fn download_model(model_id: String, app: AppHandle) -> Result<String, String> {
    let (_, url, _, _) = MODELS
        .iter()
        .find(|(id, _, _, _)| *id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let path = get_model_path(&model_id);

    if path.exists() {
        return Ok(path.to_string_lossy().to_string());
    }

    println!("Downloading model {} from {}", model_id, url);

    // Download with progress
    let client = reqwest::Client::new();
    let response = client
        .get(*url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // Create temp file
    let temp_path = path.with_extension("bin.tmp");
    let mut file = File::create(&temp_path).map_err(|e| e.to_string())?;

    // Stream the download
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        // Emit progress event
        let progress = DownloadProgress {
            model_id: model_id.clone(),
            downloaded_bytes: downloaded,
            total_bytes: total_size,
            percent: if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            },
        };

        app.emit("model-download-progress", &progress).ok();
    }

    // Move temp file to final location
    fs::rename(&temp_path, &path).map_err(|e| e.to_string())?;

    println!("Model downloaded to: {:?}", path);

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn transcribe(audio_path: String, state: State<'_, AppState>) -> Result<String, String> {
    let model_id = state.selected_model.lock().unwrap().clone();
    let model_path = get_model_path(&model_id);

    if !model_path.exists() {
        return Err(format!(
            "Model '{}' not downloaded. Please download it first.",
            model_id
        ));
    }

    println!("Transcribing {} with model {}", audio_path, model_id);
    println!("Model path: {:?}", model_path);

    // Clone paths for the blocking thread
    let audio_path_owned = audio_path.clone();
    let model_path_owned = model_path.clone();

    // Run transcription in a blocking thread to avoid freezing the UI
    // Wrap in catch_unwind to prevent panics from crashing the app
    let result = tokio::task::spawn_blocking(move || {
        panic::catch_unwind(panic::AssertUnwindSafe(|| {
            transcribe_blocking(audio_path_owned, model_path_owned)
        }))
        .unwrap_or_else(|e| {
            let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic in Whisper transcription".to_string()
            };
            Err(format!("Whisper crashed: {}", panic_msg))
        })
    })
    .await
    .map_err(|e| format!("Task failed: {:?}", e))??;

    Ok(result)
}

fn transcribe_blocking(audio_path: String, model_path: PathBuf) -> Result<String, String> {
    // Load the audio file
    let mut reader = hound::WavReader::open(audio_path).map_err(|e| e.to_string())?;
    let spec = reader.spec();

    if spec.sample_rate != 16000 {
        return Err(format!(
            "Audio must be 16kHz, got {}Hz",
            spec.sample_rate
        ));
    }

    // Read samples as f32
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .map(|s| s.unwrap_or(0) as f32 / i16::MAX as f32)
            .collect(),
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
    };

    // Convert to mono if stereo
    let mono_samples: Vec<f32> = if spec.channels > 1 {
        samples
            .chunks(spec.channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
            .collect()
    } else {
        samples
    };

    // Initialize Whisper
    let model_path_str = model_path
        .to_str()
        .ok_or_else(|| "Invalid model path (non-UTF8 characters)".to_string())?;

    println!("Loading Whisper model from: {}", model_path_str);

    let ctx = WhisperContext::new_with_params(
        model_path_str,
        WhisperContextParameters::default(),
    )
    .map_err(|e| format!("Failed to load Whisper model: {:?}", e))?;

    println!("Whisper model loaded successfully");

    let mut whisper_state = ctx.create_state().map_err(|e| format!("Failed to create state: {:?}", e))?;

    // Configure transcription parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);

    // Run transcription
    whisper_state
        .full(params, &mono_samples)
        .map_err(|e| format!("Transcription failed: {:?}", e))?;

    // Collect results
    let num_segments = whisper_state.full_n_segments().map_err(|e| format!("Failed to get segments: {:?}", e))?;
    let mut text = String::new();

    for i in 0..num_segments {
        if let Ok(segment) = whisper_state.full_get_segment_text(i) {
            text.push_str(&segment);
            text.push(' ');
        }
    }

    let result = text.trim().to_string();
    println!("Transcription: {}", result);

    Ok(result)
}

#[tauri::command]
pub fn delete_model(model_id: String) -> Result<(), String> {
    let path = get_model_path(&model_id);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
