use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use soma_core::utils;
use crate::provider::{SomaTtsProvider, TtsResult};
use crate::voices;
use std::path::Path;

pub struct EdgeTts {
    timeout: Option<f64>,
}

impl EdgeTts {
    pub fn new(timeout: Option<f64>) -> Self {
        Self { timeout }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for EdgeTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if voices::is_no_voice(voice) {
            return self.synthesize_silent(text, output_path).await;
        }

        let voice_name = voices::parse_voice_name(voice);
        let rate_str = voices::convert_rate_to_percent(rate);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        let timeout_args: Vec<String> = self.timeout
            .map(|t| vec![format!("--timeout={}", t as u64)])
            .unwrap_or_default();

        let output_str = output_path.to_string_lossy().to_string();
        let result = tokio::process::Command::new("edge-tts")
            .args(&[
                "--voice", &voice_name,
                "--rate", &rate_str,
                "--text", text,
                "--write-media", &output_str,
            ])
            .args(&timeout_args)
            .output()
            .await
            .map_err(|e| SomaError::Tts(format!("edge-tts command failed: {}", e)))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Tts(format!("edge-tts failed: {}", stderr)));
        }

        let audio_duration = get_audio_duration(&output_str)?;
        let cues = generate_subtitle_cues_from_text(text, audio_duration);

        Ok(TtsResult {
            audio_file: output_str,
            audio_duration,
            subtitle_cues: cues,
        })
    }
}

impl EdgeTts {
    async fn synthesize_silent(&self, text: &str, output_path: &Path) -> Result<TtsResult, SomaError> {
        let duration = voices::estimate_no_voice_duration(text);
        let output_str = output_path.to_string_lossy().to_string();

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        let ffmpeg = "ffmpeg";
        let result = tokio::process::Command::new(ffmpeg)
            .args(&[
                "-y", "-f", "lavfi",
                "-i", &format!("anullsrc=r=44100:cl=mono"),
                "-t", &format!("{:.3}", duration),
                "-codec:a", "libmp3lame",
                "-q:a", "4",
                &output_str,
            ])
            .output()
            .await
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg silent audio failed: {}", e)))?;

        if !result.status.success() {
            return Err(SomaError::Ffmpeg("failed to generate silent audio".into()));
        }

        let cues = generate_subtitle_cues_from_text(text, duration);
        Ok(TtsResult {
            audio_file: output_str,
            audio_duration: duration,
            subtitle_cues: cues,
        })
    }
}

pub fn get_audio_duration(audio_path: &str) -> Result<f64, SomaError> {
    let output = std::process::Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            audio_path,
        ])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;

    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    duration_str.parse::<f64>()
        .map_err(|e| SomaError::Tts(format!("failed to parse audio duration: {}", e)))
}

pub fn generate_subtitle_cues_from_text(text: &str, audio_duration: f64) -> Vec<SubtitleCue> {
    let sentences = utils::split_string_by_punctuations(text);
    if sentences.is_empty() {
        return vec![SubtitleCue {
            index: 1,
            start_ms: 0,
            end_ms: (audio_duration * 10000.0) as u64,
            text: text.to_string(),
        }];
    }

    let total_chars: usize = sentences.iter().map(|s| s.chars().count()).sum();
    if total_chars == 0 {
        return vec![];
    }

    let audio_duration_100ns = (audio_duration * 10_000_000.0) as u64;
    let mut cues = Vec::new();
    let mut current_offset: u64 = 0;

    for (i, sentence) in sentences.iter().enumerate() {
        if sentence.trim().is_empty() {
            continue;
        }
        let sentence_chars = sentence.chars().count();
        let sentence_duration = if i == sentences.len() - 1 {
            audio_duration_100ns.saturating_sub(current_offset)
        } else {
            ((audio_duration_100ns as f64) * (sentence_chars as f64 / total_chars as f64)).max(1.0) as u64
        };
        let sentence_end = (current_offset + sentence_duration).min(audio_duration_100ns);

        cues.push(SubtitleCue {
            index: (cues.len() + 1) as u32,
            start_ms: current_offset / 10_000,
            end_ms: sentence_end / 10_000,
            text: sentence.clone(),
        });

        current_offset = sentence_end;
    }

    cues
}

pub fn create_subtitle_file(cues: &[SubtitleCue], output_path: &str) -> Result<(), SomaError> {
    let mut content = String::new();
    for cue in cues {
        let start = utils::time_convert_seconds_to_hmsm(cue.start_ms as f64 / 1000.0);
        let end = utils::time_convert_seconds_to_hmsm(cue.end_ms as f64 / 1000.0);
        content.push_str(&format!("{}\n{} --> {}\n{}\n\n", cue.index, start, end, cue.text));
    }
    std::fs::write(output_path, content).map_err(SomaError::Io)
}
