---
name: whisper-stt
description: Integrate local speech-to-text transcription using whisper-rs (Rust bindings for whisper.cpp). Use when building applications requiring offline audio transcription, voice notes, dictation features, or real-time speech recognition in Tauri or native Rust applications.
license: MIT
---

# Speech-to-Text with whisper-rs

whisper-rs provides Rust bindings to whisper.cpp for local, offline speech recognition using OpenAI's Whisper models.

## Setup

### Cargo Dependencies

```toml
[dependencies]
whisper-rs = "0.11"
hound = "3.5"        # WAV file handling
cpal = "0.15"        # Audio capture
```

### Model Downloads

Models must be downloaded separately. Store in app data directory:

```rust
// Model sizes and accuracy/speed tradeoffs
// tiny.en   - 75MB,  fastest, English only
// base.en   - 142MB, fast, English only  
// small.en  - 466MB, balanced, English only
// medium.en - 1.5GB, accurate, English only
// large-v3  - 3GB,   most accurate, multilingual

const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";

async fn download_model(model_name: &str, dest: &Path) -> Result<(), Box<dyn Error>> {
    let url = format!("{}ggml-{}.bin", MODEL_URL, model_name);
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;
    std::fs::write(dest, bytes)?;
    Ok(())
}
```

## Audio Recording

Use cpal for cross-platform audio capture:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        }
    }
    
    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device")?;
        
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000), // Whisper expects 16kHz
            buffer_size: cpal::BufferSize::Default,
        };
        
        let samples = self.samples.clone();
        
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                samples.lock().unwrap().extend_from_slice(data);
            },
            |err| eprintln!("Audio error: {}", err),
            None
        ).map_err(|e| e.to_string())?;
        
        stream.play().map_err(|e| e.to_string())?;
        self.stream = Some(stream);
        Ok(())
    }
    
    pub fn stop(&mut self) -> Vec<f32> {
        self.stream = None;
        let mut samples = self.samples.lock().unwrap();
        std::mem::take(&mut *samples)
    }
}
```

## Transcription

```rust
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn new(model_path: &str) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default()
        ).map_err(|e| e.to_string())?;
        
        Ok(Self { ctx })
    }
    
    pub fn transcribe(&self, audio: &[f32]) -> Result<String, String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // Configuration
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_single_segment(false);
        
        // Create state and run
        let mut state = self.ctx.create_state()
            .map_err(|e| e.to_string())?;
        
        state.full(params, audio)
            .map_err(|e| e.to_string())?;
        
        // Collect segments
        let num_segments = state.full_n_segments()
            .map_err(|e| e.to_string())?;
        
        let mut result = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                result.push_str(&segment);
                result.push(' ');
            }
        }
        
        Ok(result.trim().to_string())
    }
}
```

## Tauri Integration

```rust
// src-tauri/src/commands/transcribe.rs
use std::sync::Mutex;
use tauri::State;

pub struct TranscriptionState {
    recorder: Mutex<AudioRecorder>,
    transcriber: Mutex<Option<Transcriber>>,
}

#[tauri::command]
pub async fn init_transcriber(
    state: State<'_, TranscriptionState>,
    model_path: String
) -> Result<(), String> {
    let transcriber = Transcriber::new(&model_path)?;
    *state.transcriber.lock().unwrap() = Some(transcriber);
    Ok(())
}

#[tauri::command]
pub fn start_recording(state: State<'_, TranscriptionState>) -> Result<(), String> {
    state.recorder.lock().unwrap().start()
}

#[tauri::command]
pub async fn stop_and_transcribe(
    state: State<'_, TranscriptionState>
) -> Result<String, String> {
    let audio = state.recorder.lock().unwrap().stop();
    
    let transcriber = state.transcriber.lock().unwrap();
    let transcriber = transcriber.as_ref()
        .ok_or("Transcriber not initialized")?;
    
    transcriber.transcribe(&audio)
}
```

## Audio Preprocessing

For best results, preprocess audio before transcription:

```rust
pub fn preprocess_audio(samples: &[f32]) -> Vec<f32> {
    // Ensure mono, 16kHz
    // Normalize amplitude
    let max_amp = samples.iter()
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);
    
    if max_amp > 0.0 {
        samples.iter().map(|s| s / max_amp * 0.95).collect()
    } else {
        samples.to_vec()
    }
}

pub fn remove_silence(samples: &[f32], threshold: f32) -> Vec<f32> {
    // Simple voice activity detection
    let window_size = 1600; // 100ms at 16kHz
    let mut result = Vec::new();
    
    for chunk in samples.chunks(window_size) {
        let energy: f32 = chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32;
        if energy > threshold {
            result.extend_from_slice(chunk);
        }
    }
    
    result
}
```

## WAV File Support

For saving/loading recordings:

```rust
use hound::{WavReader, WavWriter, WavSpec};

pub fn save_wav(samples: &[f32], path: &str) -> Result<(), String> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(path, spec)
        .map_err(|e| e.to_string())?;
    
    for &sample in samples {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer.write_sample(amplitude).map_err(|e| e.to_string())?;
    }
    
    writer.finalize().map_err(|e| e.to_string())
}

pub fn load_wav(path: &str) -> Result<Vec<f32>, String> {
    let reader = WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    
    // Convert to f32 mono 16kHz
    let samples: Vec<f32> = reader.into_samples::<i16>()
        .filter_map(|s| s.ok())
        .map(|s| s as f32 / i16::MAX as f32)
        .collect();
    
    // Resample if needed (simplified - use rubato crate for production)
    if spec.sample_rate != 16000 {
        // Resample here
    }
    
    Ok(samples)
}
```

## Performance Tips

1. **Use smaller models** for real-time applications (tiny, base)
2. **Process in chunks** for streaming transcription
3. **Run transcription on separate thread** to avoid blocking UI
4. **Cache model in memory** rather than reloading
5. **Use GPU acceleration** when available (whisper-rs-cuda feature)

## Common Issues

| Issue | Solution |
|-------|----------|
| Silent audio | Check microphone permissions, verify device selection |
| Gibberish output | Ensure 16kHz sample rate, check audio normalization |
| Slow transcription | Use smaller model, enable GPU if available |
| Missing words | Increase audio quality, reduce background noise |
