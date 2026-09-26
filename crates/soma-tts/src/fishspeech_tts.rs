//! Fish-Speech S2 TTS 引擎实现
//!
//! 基于 Fish-Speech（Fish Audio S2）的 HTTP API 进行语音合成：
//! - 自部署：`python tools/api_server.py --listen 0.0.0.0:8080` 后对接 `http://host:8080`
//! - 云端：Fish Audio 平台 API（https://api.fish.audio，需 API Key）
//!
//! 请求端点为 `POST {base_url}/v1/tts`，服务端同时接受 JSON 与 msgpack 请求体，
//! 本实现使用 JSON（参考 ServeTTSRequest schema）。
//!
//! 特色能力：
//! - 文本内嵌情感标签（如 `[whisper]`、`[excited]`、`[pause]`），由 S2 模型直接渲染
//! - 参考音色克隆（reference_id 指向服务端已保存的参考音频）
//!
//! 注意：Fish-Speech API 不支持语速参数，`rate` 将被忽略。

use async_trait::async_trait;
use soma_core::config::FishspeechSection;
use soma_core::error::SomaError;
use std::path::Path;
use std::time::Duration;

use crate::edge_tts::generate_subtitle_cues_from_text;
use crate::provider::{SomaTtsProvider, TtsResult};

/// Fish-Speech S2 语音合成器
pub struct FishspeechTts {
    /// API 基础 URL（如 "http://192.168.1.100:8080" 或 "https://api.fish.audio"）
    base_url: String,
    /// API 密钥（自部署未启用鉴权时可留空）
    api_key: String,
    /// 默认参考音色 ID（可为空，使用模型默认音色）
    default_reference_id: String,
    /// 音频格式（wav/pcm/mp3/opus）
    format: String,
    /// 是否做数字归一化
    normalize: bool,
    /// 请求超时（秒）
    timeout: u64,
}

impl FishspeechTts {
    /// 基于配置段创建实例
    pub fn from_config(config: &FishspeechSection) -> Self {
        Self {
            base_url: config.get_base_url().trim_end_matches('/').to_string(),
            api_key: config.get_api_key().to_string(),
            default_reference_id: config.get_reference_id().to_string(),
            format: config.get_format().to_string(),
            normalize: config.get_normalize(),
            timeout: config.get_timeout(),
        }
    }

    /// 创建 FishspeechTts 实例
    ///
    /// - `base_url` - API 基础 URL
    /// - `api_key` - API 密钥（可为空）
    /// - `default_reference_id` - 默认参考音色 ID（可为空）
    pub fn new(base_url: &str, api_key: &str, default_reference_id: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            default_reference_id: default_reference_id.to_string(),
            format: "mp3".to_string(),
            normalize: true,
            timeout: 120,
        }
    }

    /// 解析参考音色 ID：语音名 `fishspeech:ref-id` 优先，其次默认配置
    fn resolve_reference_id(&self, voice: &str) -> String {
        if let Some(id) = crate::voices::extract_fishspeech_voice(voice) {
            return id;
        }
        if !voice.is_empty() && !voice.starts_with("fishspeech:") {
            // tts_provider=fishspeech 时语音名直接作为参考音色 ID 传入
            return voice.to_string();
        }
        self.default_reference_id.clone()
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for FishspeechTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(SomaError::Tts("待合成文本不能为空".into()));
        }

        if self.base_url.is_empty() {
            return Err(SomaError::Config(
                "Fish-Speech 未配置 base_url，请在 config.toml 中添加 [fishspeech] 段".into(),
            ));
        }

        if (rate - 1.0).abs() > f32::EPSILON {
            log::warn!("Fish-Speech 不支持语速调节，忽略 rate={}", rate);
        }

        let reference_id = self.resolve_reference_id(voice);

        let mut payload = serde_json::json!({
            "text": trimmed,
            "format": self.format,
            "normalize": self.normalize,
            "chunk_length": 200,
            "streaming": false,
            "use_memory_cache": "on",
        });
        if !reference_id.is_empty() {
            payload["reference_id"] = serde_json::Value::String(reference_id.clone());
        }

        let url = format!("{}/v1/tts", self.base_url);
        let client = reqwest::Client::new();
        let mut req = client
            .post(&url)
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.timeout.max(10)));
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req
            .json(&payload)
            .send()
            .await
            .map_err(|e| SomaError::Http(format!("Fish-Speech TTS 请求失败: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SomaError::Tts(format!(
                "Fish-Speech TTS failed: {} - {}",
                status, body
            )));
        }

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| SomaError::Http(format!("Fish-Speech TTS 读取响应失败: {}", e)))?;

        if content_type.contains("json") {
            // 错误响应以 JSON 返回（HTTP 状态码正常但 body 为错误信息的情况）
            return Err(SomaError::Tts(format!(
                "Fish-Speech TTS 返回错误: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        if bytes.is_empty() {
            return Err(SomaError::Tts("Fish-Speech TTS 返回空音频".into()));
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        std::fs::write(output_path, &bytes).map_err(SomaError::Io)?;

        let output_str = output_path.to_string_lossy().to_string();
        let audio_duration = get_audio_duration(&output_str)?;

        // 字幕时间轴基于剥离情感标签后的纯文本生成，避免 [tag] 出现在字幕中
        let plain_text = strip_emotion_tags(trimmed);
        let cues = generate_subtitle_cues_from_text(&plain_text, audio_duration);

        Ok(TtsResult {
            audio_file: output_str,
            audio_duration,
            subtitle_cues: cues,
        })
    }
}

/// 剥离 Fish-Speech 风格的内嵌情感标签（如 `[whisper]`、`[excited]`）
///
/// 规则：移除 `[...]` 形式、内容长度 1~40 且不含句子标点（中英文逗号句号感叹问号等）
/// 的标记，并清理标签移除后残留的多余空白。
///
/// 用于：
/// - 非 Fish-Speech 引擎合成前的文本净化（其他引擎会把 `[whisper]` 当文本朗读）
/// - 字幕文本生成（标签不应出现在屏幕字幕中）
pub fn strip_emotion_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();

    while let Some((_, ch)) = chars.next() {
        if ch == '[' {
            // 尝试匹配一个标签 [content]
            let mut end = None;
            let mut len = 0usize;
            let mut has_punct = false;
            let mut inner = chars.clone();
            while let Some(&(j, c)) = inner.peek() {
                if c == ']' {
                    end = Some(j);
                    break;
                }
                if c == '[' || c == '\n' {
                    break;
                }
                if "。！？；：，、,.!?;:\n\r".contains(c) {
                    has_punct = true;
                }
                len += 1;
                inner.next();
            }
            if let Some(j) = end {
                if (1..=40).contains(&len) && !has_punct {
                    // 命中情感标签，跳到 ']' 之后，并吞掉标签后的空白
                    for (k, _) in chars.by_ref() {
                        if k == j {
                            break;
                        }
                    }
                    while let Some(&(_, c)) = chars.peek() {
                        if c == ' ' || c == '\n' || c == '\r' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        result.push(ch);
    }

    // 清理标签移除后产生的连续空格 / 行首空格
    let mut cleaned = String::with_capacity(result.len());
    let mut prev_space = false;
    for c in result.chars() {
        let is_space = c == ' ';
        if is_space && prev_space {
            continue;
        }
        cleaned.push(c);
        prev_space = is_space;
    }
    cleaned
        .replace(" ，", "，")
        .replace(" 。", "。")
        .replace(" ,", ",")
        .replace(" .", ".")
        .trim()
        .to_string()
}

/// 使用 ffprobe 获取音频文件时长（秒）
fn get_audio_duration(audio_path: &str) -> Result<f64, SomaError> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            audio_path,
        ])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|e| SomaError::Tts(format!("parse duration failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_emotion_tags_basic() {
        assert_eq!(
            strip_emotion_tags("[whisper]你好[excited]世界"),
            "你好世界"
        );
    }

    #[test]
    fn test_strip_emotion_tags_with_spaces() {
        assert_eq!(
            strip_emotion_tags("[excited] 大家好，[pause] 今天聊聊 AI。"),
            "大家好，今天聊聊 AI。"
        );
    }

    #[test]
    fn test_strip_emotion_tags_phrase() {
        assert_eq!(
            strip_emotion_tags("[whisper in small voice]安静点"),
            "安静点"
        );
    }

    #[test]
    fn test_strip_emotion_tags_keeps_bracket_sentences() {
        // 含标点的方括号内容不是情感标签，应保留
        assert_eq!(
            strip_emotion_tags("他说[你好，世界]然后离开"),
            "他说[你好，世界]然后离开"
        );
    }

    #[test]
    fn test_strip_emotion_tags_keeps_long_brackets() {
        // 超长方括号内容不是标签，应保留
        let long = format!("[{}]", "很长的内容".repeat(20));
        let text = format!("前置{}", long);
        assert_eq!(strip_emotion_tags(&text), text);
    }

    #[test]
    fn test_strip_emotion_tags_no_tags() {
        assert_eq!(strip_emotion_tags("普通文本，没有标签。"), "普通文本，没有标签。");
    }

    #[test]
    fn test_strip_emotion_tags_unmatched() {
        assert_eq!(strip_emotion_tags("未闭合[标签"), "未闭合[标签");
    }
}
