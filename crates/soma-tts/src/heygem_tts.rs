//! HeyGem TTS Provider
//!
//! 通过 HTTP API 调用 HeyGem/Duix.Avatar 的 Fish-Speech 声音克隆服务。
//! 端口 18180，接口 POST /v1/invoke

use crate::provider::{SomaTtsProvider, TtsResult};
use soma_core::config::HeyGemConfig;
use soma_core::error::SomaError;
use std::path::Path;

pub struct HeyGemTts {
    config: HeyGemConfig,
    reference_audio: String,
    reference_text: String,
    speaker_id: String,
}

impl HeyGemTts {
    pub fn new(
        config: HeyGemConfig,
        reference_audio: &str,
        reference_text: &str,
        speaker_id: &str,
    ) -> Self {
        Self {
            config,
            reference_audio: reference_audio.to_string(),
            reference_text: reference_text.to_string(),
            speaker_id: speaker_id.to_string(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SomaTtsProvider for HeyGemTts {
    async fn synthesize(
        &self,
        text: &str,
        _voice_name: &str,
        _rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if text.trim().is_empty() {
            return Err(SomaError::Tts("文案不能为空".into()));
        }
        if text.chars().count() > 1000 {
            return Err(SomaError::Tts("文案长度不能超过 1000 字".into()));
        }

        let url = format!("{}/v1/invoke", self.config.get_tts_base_url());
        let body = serde_json::json!({
            "speaker": self.speaker_id,
            "text": text,
            "format": "wav",
            "topP": self.config.get_top_p(),
            "max_new_tokens": 1024,
            "chunk_length": 100,
            "repetition_penalty": self.config.get_repetition_penalty(),
            "temperature": self.config.get_temperature(),
            "need_asr": false,
            "streaming": false,
            "is_fixed_seed": 0,
            "is_norm": 0,
            "reference_audio": self.reference_audio,
            "reference_text": self.reference_text,
        });

        log::info!(
            "HeyGem TTS 请求: url={}, text_len={}",
            url,
            text.chars().count()
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.get_timeout()))
            .build()
            .map_err(|e| SomaError::Tts(format!("HTTP 客户端创建失败: {}", e)))?;

        let max_retries = self.config.get_max_retries() as usize;
        let mut last_err = None;

        for attempt in 1..=max_retries {
            let resp = client.post(&url).json(&body).send().await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    let bytes = r
                        .bytes()
                        .await
                        .map_err(|e| SomaError::Tts(format!("读取响应失败: {}", e)))?;

                    if let Some(parent) = output_path.parent() {
                        std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
                    }
                    std::fs::write(output_path, &bytes).map_err(SomaError::Io)?;

                    let audio_file = output_path.to_string_lossy().to_string();
                    log::info!("HeyGem TTS 完成: {}", audio_file);

                    return Ok(TtsResult {
                        audio_file,
                        audio_duration: 0.0,
                        subtitle_cues: vec![],
                    });
                }
                Ok(r) => {
                    let status = r.status();
                    let msg = format!("HTTP {}", status);
                    if status.is_client_error() {
                        return Err(SomaError::Tts(format!("声音合成请求参数错误: {}", msg)));
                    }
                    log::warn!("HeyGem TTS 重试 {}/{}: {}", attempt, max_retries, msg);
                    last_err = Some(SomaError::Tts(format!("服务过载: {}", msg)));
                }
                Err(e) => {
                    if e.is_connect() {
                        return Err(SomaError::Tts(
                            "HeyGem TTS 服务不可用，请检查 Fish-Speech 容器状态".into(),
                        ));
                    }
                    log::warn!("HeyGem TTS 重试 {}/{}: {}", attempt, max_retries, e);
                    last_err = Some(SomaError::Tts(format!("请求失败: {}", e)));
                }
            }

            if attempt < max_retries {
                std::thread::sleep(std::time::Duration::from_secs(1 << (attempt - 1)));
            }
        }

        Err(last_err.unwrap_or_else(|| SomaError::Tts("HeyGem TTS 请求重试耗尽".into())))
    }
}
