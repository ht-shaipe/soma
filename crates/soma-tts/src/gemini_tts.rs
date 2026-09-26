//! Gemini TTS 引擎实现
//!
//! 基于 Google Gemini 2.5 Flash TTS API 进行语音合成。
//! 使用 generateContent 接口，response_modalities=["AUDIO"]，
//! 返回 Linear PCM（24kHz, 16bit, mono）音频数据。

use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use async_trait::async_trait;
use soma_core::error::SomaError;
use std::path::Path;

pub struct GeminiTts {
    api_key: String,
    base_url: String,
    model_name: String,
}

impl GeminiTts {
    pub fn new(api_key: &str, base_url: &str, model_name: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: if base_url.is_empty() {
                "https://generativelanguage.googleapis.com".into()
            } else {
                base_url.into()
            },
            model_name: if model_name.is_empty() {
                "gemini-2.5-flash-preview-tts".into()
            } else {
                model_name.into()
            },
        }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for GeminiTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        _rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if self.api_key.is_empty() {
            return Err(SomaError::Tts("Gemini API key not set".into()));
        }

        let voice_name =
            crate::voices::extract_gemini_voice(voice).unwrap_or_else(|| voice.to_string());

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url, self.model_name, self.api_key
        );

        let payload = serde_json::json!({
            "contents": [{ "parts": [{ "text": text }] }],
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": {
                            "voiceName": voice_name
                        }
                    }
                }
            }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SomaError::Tts(format!(
                "Gemini TTS failed: {} - {}",
                status, body
            )));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Tts(format!("Gemini TTS parse response failed: {}", e)))?;

        let audio_b64 = resp_json
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("inlineData"))
            .and_then(|d| d.get("data"))
            .and_then(|d| d.as_str())
            .ok_or_else(|| SomaError::Tts("Gemini TTS: 无音频数据返回".into()))?;

        let audio_bytes = base64_decode(audio_b64)
            .map_err(|e| SomaError::Tts(format!("Gemini TTS base64 decode failed: {}", e)))?;

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        let raw_path = output_path.with_extension("raw");
        std::fs::write(&raw_path, &audio_bytes).map_err(SomaError::Io)?;

        let output_str = output_path.to_string_lossy().to_string();
        let raw_str = raw_path.to_string_lossy().to_string();
        convert_pcm_to_mp3(&raw_str, &output_str, 24000)?;
        let _ = std::fs::remove_file(&raw_path);

        let audio_duration = get_audio_duration(&output_str)?;
        let cues = generate_subtitle_cues_from_text(text, audio_duration);

        Ok(TtsResult {
            audio_file: output_str,
            audio_duration,
            subtitle_cues: cues,
        })
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let decoder = base64::engine::general_purpose::STANDARD;
    let mut buf = Vec::new();
    let mut cursor = base64::read::DecoderReader::new(input.as_bytes(), &decoder);
    cursor.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn convert_pcm_to_mp3(pcm_path: &str, mp3_path: &str, sample_rate: u32) -> Result<(), SomaError> {
    let result = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "s16le",
            "-ar",
            &sample_rate.to_string(),
            "-ac",
            "1",
            "-i",
            pcm_path,
            "-c:a",
            "libmp3lame",
            "-q:a",
            "4",
            mp3_path,
        ])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg pcm to mp3 failed: {}", e)))?;
    if !result.status.success() {
        return Err(SomaError::Ffmpeg("ffmpeg pcm to mp3 failed".into()));
    }
    Ok(())
}
