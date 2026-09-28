//! HeyGem 商户模型训练编排
//!
//! 编排"录制视频校验 → FFmpeg 分离静音视频与音频 → 调用训练接口 → 绑定资产"流程。
//! 训练产出的 reference_audio 与 reference_text 用于后续 TTS 声音克隆。

use super::heygem_merchant::MerchantAssetStore;
use soma_core::config::HeyGemConfig;
use soma_core::error::SomaError;
use std::path::PathBuf;

/// 训练结果
#[derive(Debug, Clone)]
pub struct TrainResult {
    pub reference_audio: String,
    pub reference_text: String,
    pub silent_video: String,
    pub trained_at: chrono::DateTime<chrono::Utc>,
}

/// HeyGem 商户模型训练器
pub struct HeyGemTrainer {
    config: HeyGemConfig,
    asset_store: MerchantAssetStore,
    ffmpeg: soma_video::Ffmpeg,
}

impl HeyGemTrainer {
    /// 构造 HeyGem 声音克隆训练器（配置 / 资产库 / FFmpeg 注入）
    pub fn new(
        config: HeyGemConfig,
        asset_store: MerchantAssetStore,
        ffmpeg: soma_video::Ffmpeg,
    ) -> Self {
        Self {
            config,
            asset_store,
            ffmpeg,
        }
    }

    /// 校验训练视频时长 ∈ [60, 300] 秒
    fn validate_train_video_duration(&self, video_path: &str) -> Result<f64, SomaError> {
        let dur = self
            .ffmpeg
            .get_video_duration(video_path)
            .map_err(|e| SomaError::VideoGen(format!("{:?}", e)))?;
        if !(60.0..=300.0).contains(&dur) {
            return Err(SomaError::VideoGen(format!(
                "录制视频时长需在 1-5 分钟之间，当前 {:.1} 分钟",
                dur / 60.0
            )));
        }
        Ok(dur)
    }

    /// FFmpeg 分离静音视频与音频
    fn split_silent_and_audio(
        &self,
        video_path: &str,
        out_silent: &str,
        out_audio: &str,
    ) -> Result<(), SomaError> {
        if let Some(p) = std::path::Path::new(out_silent).parent() {
            std::fs::create_dir_all(p).map_err(SomaError::Io)?;
        }
        if let Some(p) = std::path::Path::new(out_audio).parent() {
            std::fs::create_dir_all(p).map_err(SomaError::Io)?;
        }
        let ff = &self.ffmpeg.path;
        let status = std::process::Command::new(ff)
            .args(["-y", "-i", video_path, "-an", "-c:v", "copy", out_silent])
            .status()
            .map_err(SomaError::Io)?;
        if !status.success() {
            return Err(SomaError::VideoGen("分离静音视频失败".into()));
        }
        let status = std::process::Command::new(ff)
            .args([
                "-y",
                "-i",
                video_path,
                "-vn",
                "-acodec",
                "pcm_s16le",
                "-ar",
                "24000",
                "-ac",
                "1",
                out_audio,
            ])
            .status()
            .map_err(SomaError::Io)?;
        if !status.success() {
            return Err(SomaError::VideoGen("提取音频失败".into()));
        }
        Ok(())
    }

    /// 调用 HeyGem TTS 训练接口，返回 (reference_audio_path, reference_text)
    fn call_train_api(&self, audio_path: &str) -> Result<(String, String), SomaError> {
        let url = format!("{}/v1/train", self.config.get_tts_base_url());
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.get_timeout()))
            .build()
            .map_err(|e| SomaError::Tts(format!("HTTP 客户端创建失败: {}", e)))?;

        let bytes = std::fs::read(audio_path).map_err(SomaError::Io)?;
        let max_retries = self.config.get_max_retries() as usize;
        let mut last_err: Option<String> = None;

        for attempt in 1..=max_retries {
            let part = reqwest::multipart::Part::bytes(bytes.clone())
                .file_name("train.wav")
                .mime_str("audio/wav")
                .map_err(|e| SomaError::Tts(format!("构造 multipart 失败: {}", e)))?;
            let form = reqwest::multipart::Form::new().part("audio", part);
            let url_clone = url.clone();
            let client_clone = client.clone();
            let probe = async move {
                let send_result = client_clone.post(&url_clone).multipart(form).send().await;
                let r = match send_result {
                    Ok(r) => r,
                    Err(e) => {
                        if e.is_connect() {
                            return Err("CONNECT".to_string());
                        }
                        return Err(format!("请求失败: {}", e));
                    }
                };
                if r.status().is_success() {
                    let json: serde_json::Value = match r.json().await {
                        Ok(j) => j,
                        Err(e) => return Err(format!("解析训练响应失败: {}", e)),
                    };
                    let ref_audio = match json.get("reference_audio").and_then(|v| v.as_str()) {
                        Some(s) => s.to_string(),
                        None => return Err("训练响应缺少 reference_audio".to_string()),
                    };
                    let ref_text = json
                        .get("reference_text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Ok((ref_audio, ref_text))
                } else {
                    let status = r.status();
                    if status.is_client_error() {
                        let text = r.text().await.unwrap_or_default();
                        Err(format!(
                            "CLIENT:训练请求参数错误: HTTP {} - {}",
                            status, text
                        ))
                    } else {
                        Err(format!("HTTP {}", status))
                    }
                }
            };
            match Self::block_on(probe) {
                Ok(Ok(pair)) => return Ok(pair),
                Ok(Err(msg)) => {
                    if msg == "CONNECT" {
                        return Err(SomaError::Tts(
                            "HeyGem TTS 训练服务不可用，请检查 Fish-Speech 容器状态".into(),
                        ));
                    }
                    if msg.starts_with("CLIENT:") {
                        return Err(SomaError::Tts(
                            msg.trim_start_matches("CLIENT:").to_string(),
                        ));
                    }
                    last_err = Some(msg);
                }
                Err(e) => {
                    last_err = Some(format!("{:?}", e));
                }
            }
            if attempt < max_retries {
                std::thread::sleep(std::time::Duration::from_secs(1 << (attempt - 1)));
            }
        }
        Err(SomaError::Tts(format!(
            "模型训练失败：{}，请检查 HeyGem 服务后重试",
            last_err.unwrap_or_else(|| "未知错误".into())
        )))
    }

    fn block_on<F>(fut: F) -> Result<F::Output, SomaError>
    where
        F: std::future::Future,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| SomaError::Tts(format!("tokio runtime 创建失败: {}", e)))?;
        Ok(rt.block_on(fut))
    }

    /// 训练主流程
    pub fn train(
        &self,
        merchant_id: &str,
        train_video_path: &str,
        overwrite_confirm: bool,
    ) -> Result<TrainResult, SomaError> {
        soma_core::utils::validate_merchant_id(merchant_id)?;

        let existing = self.asset_store.get_asset(merchant_id).ok();
        if existing.is_some() && !self.config.get_auto_overwrite() && !overwrite_confirm {
            return Err(SomaError::Config(
                "该商户已存在训练资产，请确认覆盖旧资产".into(),
            ));
        }

        let dir = PathBuf::from(self.config.get_assets_dir()).join(merchant_id);
        let training_dir = dir.join("training");
        std::fs::create_dir_all(&training_dir).map_err(SomaError::Io)?;

        let silent_path = training_dir
            .join("silent.mp4")
            .to_string_lossy()
            .to_string();
        let audio_path = training_dir
            .join("train_audio.wav")
            .to_string_lossy()
            .to_string();

        let cleanup = |err: SomaError| {
            let _ = std::fs::remove_dir_all(&training_dir);
            err
        };

        self.validate_train_video_duration(train_video_path)
            .map_err(&cleanup)?;

        self.split_silent_and_audio(train_video_path, &silent_path, &audio_path)
            .map_err(&cleanup)?;

        let (ref_audio, ref_text) = self.call_train_api(&audio_path).map_err(&cleanup)?;

        self.asset_store
            .bind_asset(merchant_id, &silent_path, &ref_audio, &ref_text)
            .map_err(&cleanup)?;

        let asset = self.asset_store.get_asset(merchant_id)?;
        let _ = std::fs::remove_dir_all(&training_dir);

        Ok(TrainResult {
            reference_audio: asset.reference_audio,
            reference_text: asset.reference_text,
            silent_video: asset.silent_video_path,
            trained_at: asset.trained_at,
        })
    }
}
