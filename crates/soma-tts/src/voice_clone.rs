//! 语音克隆模块
//!
//! 支持 ElevenLabs 和 SiliconFlow (CosyVoice2) 两种引擎的语音克隆。
//! 克隆声音的元数据持久化存储在 `storage/voices/` 目录下。

use serde::{Deserialize, Serialize};
use soma_core::error::SomaError;

/// 克隆声音的引擎类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CloneEngine {
    Elevenlabs,
    Siliconflow,
    Volcengine,
    Xfyun,
}

impl std::fmt::Display for CloneEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloneEngine::Elevenlabs => write!(f, "elevenlabs"),
            CloneEngine::Siliconflow => write!(f, "siliconflow"),
            CloneEngine::Volcengine => write!(f, "volcengine"),
            CloneEngine::Xfyun => write!(f, "xfyun"),
        }
    }
}

impl std::str::FromStr for CloneEngine {
    type Err = SomaError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "elevenlabs" => Ok(CloneEngine::Elevenlabs),
            "siliconflow" => Ok(CloneEngine::Siliconflow),
            "volcengine" => Ok(CloneEngine::Volcengine),
            "xfyun" => Ok(CloneEngine::Xfyun),
            _ => Err(SomaError::Tts(format!("unsupported clone engine: {}", s))),
        }
    }
}

/// 克隆认证配置，统一封装各引擎的认证信息
#[derive(Debug, Clone, Default)]
pub struct CloneAuth {
    /// API Key（ElevenLabs / SiliconFlow / 讯飞）
    pub api_key: String,
    /// App ID（火山引擎 / 讯飞）
    pub app_id: String,
    /// API Secret（讯飞）
    pub api_secret: String,
    /// Access Token（火山引擎）
    pub access_token: String,
}

/// 克隆声音的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonedVoice {
    /// 唯一标识（UUID）
    pub id: String,
    /// 用户给声音起的名字
    pub name: String,
    /// 克隆引擎
    pub engine: CloneEngine,
    /// 引擎端的声音 ID（ElevenLabs 的 voice_id 或 SiliconFlow 的 audio uri）
    pub remote_id: String,
    /// 原始音频文件名
    pub source_file: String,
    /// 本地保存的音频样本路径
    pub sample_path: String,
    /// 创建时间（ISO 8601）
    pub created_at: String,
}

/// ElevenLabs 语音克隆结果
#[derive(Debug, Deserialize)]
struct ElevenlabsAddVoiceResponse {
    voice_id: String,
}

/// SiliconFlow 音频上传结果
#[derive(Debug, Deserialize)]
struct SiliconflowUploadResponse {
    uri: String,
}

/// 克隆声音：上传音频样本到指定引擎并创建自定义声音
///
/// - `engine` - 克隆引擎
/// - `auth` - 引擎认证配置
/// - `voice_name` - 用户给声音起的名字
/// - `audio_data` - 音频文件的二进制数据
/// - `audio_filename` - 音频文件名（用于推断 MIME 类型）
///
/// 返回引擎端的声音 ID
pub async fn clone_voice(
    engine: &CloneEngine,
    auth: &CloneAuth,
    voice_name: &str,
    audio_data: &[u8],
    audio_filename: &str,
) -> Result<String, SomaError> {
    match engine {
        CloneEngine::Elevenlabs => {
            if auth.api_key.is_empty() {
                return Err(SomaError::Tts("ElevenLabs API key not set".into()));
            }
            clone_elevenlabs(&auth.api_key, voice_name, audio_data, audio_filename).await
        }
        CloneEngine::Siliconflow => {
            if auth.api_key.is_empty() {
                return Err(SomaError::Tts("SiliconFlow API key not set".into()));
            }
            clone_siliconflow(&auth.api_key, audio_data, audio_filename).await
        }
        CloneEngine::Volcengine => {
            if auth.app_id.is_empty() || auth.access_token.is_empty() {
                return Err(SomaError::Tts("火山引擎 App ID 或 Access Token 未设置".into()));
            }
            clone_volcengine(&auth.app_id, &auth.access_token, voice_name, audio_data, audio_filename).await
        }
        CloneEngine::Xfyun => {
            if auth.app_id.is_empty() || auth.api_key.is_empty() || auth.api_secret.is_empty() {
                return Err(SomaError::Tts("讯飞 App ID / API Key / API Secret 未设置".into()));
            }
            clone_xfyun(&auth.app_id, &auth.api_key, &auth.api_secret, voice_name, audio_data, audio_filename).await
        }
    }
}

/// 删除克隆声音
///
/// - `engine` - 克隆引擎
/// - `auth` - 引擎认证配置
/// - `remote_id` - 引擎端的声音 ID
pub async fn delete_voice(
    engine: &CloneEngine,
    auth: &CloneAuth,
    remote_id: &str,
) -> Result<(), SomaError> {
    match engine {
        CloneEngine::Elevenlabs => {
            if auth.api_key.is_empty() {
                return Err(SomaError::Tts("ElevenLabs API key not set".into()));
            }
            let client = reqwest::Client::new();
            let url = format!("https://api.elevenlabs.io/v1/voices/{}", remote_id);
            let resp = client
                .delete(&url)
                .header("xi-api-key", &auth.api_key)
                .send()
                .await
                .map_err(|e| SomaError::Http(e.to_string()))?;

            if !resp.status().is_success() {
                return Err(SomaError::Tts(format!(
                    "ElevenLabs delete voice failed: {}",
                    resp.status()
                )));
            }
            Ok(())
        }
        CloneEngine::Siliconflow | CloneEngine::Volcengine | CloneEngine::Xfyun => {
            // 这些引擎没有显式的删除接口或为异步训练型，跳过远端删除
            Ok(())
        }
    }
}

/// ElevenLabs 语音克隆：调用 /v1/voices/add 接口
async fn clone_elevenlabs(
    api_key: &str,
    voice_name: &str,
    audio_data: &[u8],
    audio_filename: &str,
) -> Result<String, SomaError> {
    let client = reqwest::Client::new();
    let mime = guess_mime_type(audio_filename);

    let part = reqwest::multipart::Part::bytes(audio_data.to_vec())
        .file_name(audio_filename.to_string())
        .mime_str(&mime)
        .map_err(|e| SomaError::Tts(format!("invalid mime type: {}", e)))?;

    let form = reqwest::multipart::Form::new()
        .text("name", voice_name.to_string())
        .text("labels", "{}")
        .part("files", part);

    let resp = client
        .post("https://api.elevenlabs.io/v1/voices/add")
        .header("xi-api-key", api_key)
        .multipart(form)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SomaError::Tts(format!(
            "ElevenLabs clone voice failed: {} - {}",
            status, body
        )));
    }

    let result: ElevenlabsAddVoiceResponse = resp
        .json()
        .await
        .map_err(|e| SomaError::Tts(format!("parse ElevenLabs response failed: {}", e)))?;

    Ok(result.voice_id)
}

/// SiliconFlow 语音克隆：上传音频到 /v1/audio/upload 接口，获取 audio uri
async fn clone_siliconflow(
    api_key: &str,
    audio_data: &[u8],
    audio_filename: &str,
) -> Result<String, SomaError> {
    let client = reqwest::Client::new();
    let mime = guess_mime_type(audio_filename);

    let part = reqwest::multipart::Part::bytes(audio_data.to_vec())
        .file_name(audio_filename.to_string())
        .mime_str(&mime)
        .map_err(|e| SomaError::Tts(format!("invalid mime type: {}", e)))?;

    let form = reqwest::multipart::Form::new().part("file", part);

    let resp = client
        .post("https://api.siliconflow.cn/v1/audio/upload")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SomaError::Tts(format!(
            "SiliconFlow upload audio failed: {} - {}",
            status, body
        )));
    }

    let result: SiliconflowUploadResponse = resp
        .json()
        .await
        .map_err(|e| SomaError::Tts(format!("parse SiliconFlow response failed: {}", e)))?;

    Ok(result.uri)
}

/// 火山引擎语音克隆：上传音频 → 训练 → 轮询状态 → 返回 voice_id
async fn clone_volcengine(
    app_id: &str,
    access_token: &str,
    voice_name: &str,
    audio_data: &[u8],
    audio_filename: &str,
) -> Result<String, SomaError> {
    let client = reqwest::Client::new();
    let mime = guess_mime_type(audio_filename);
    let base_url = "https://openspeech.bytedance.com/api/v1/mega_tts";

    // 1. 上传音频
    let part = reqwest::multipart::Part::bytes(audio_data.to_vec())
        .file_name(audio_filename.to_string())
        .mime_str(&mime)
        .map_err(|e| SomaError::Tts(format!("invalid mime type: {}", e)))?;

    let form = reqwest::multipart::Form::new()
        .text("appid", app_id.to_string())
        .text("token", access_token.to_string())
        .part("audio_file", part);

    let resp = client
        .post(format!("{}/audio/upload", base_url))
        .header("Authorization", format!("Bearer;{}", access_token))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    let resp_json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| SomaError::Tts(format!("parse volcengine upload response failed: {}", e)))?;

    let upload_id = resp_json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SomaError::Tts(format!("火山引擎上传失败: {:?}", resp_json)))?
        .to_string();

    // 2. 发起训练
    let train_payload = serde_json::json!({
        "appid": app_id,
        "token": access_token,
        "id": upload_id,
        "audio_name": voice_name,
    });

    let _ = client
        .post(format!("{}/audio/train", base_url))
        .header("Authorization", format!("Bearer;{}", access_token))
        .header("Content-Type", "application/json")
        .json(&train_payload)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    // 3. 轮询训练状态（最多等待 5 分钟）
    let query_payload = serde_json::json!({
        "appid": app_id,
        "token": access_token,
        "id": upload_id,
    });

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let resp = client
            .post(format!("{}/audio/query", base_url))
            .header("Authorization", format!("Bearer;{}", access_token))
            .header("Content-Type", "application/json")
            .json(&query_payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        let q_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Tts(format!("parse volcengine query response failed: {}", e)))?;

        let status = q_json.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status == "success" || status == "Success" {
            let voice_id = q_json
                .get("voice_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| SomaError::Tts("火山引擎训练完成但未返回 voice_id".into()))?
                .to_string();
            return Ok(voice_id);
        }
        if status == "failed" || status == "Failed" {
            return Err(SomaError::Tts(format!("火山引擎训练失败: {:?}", q_json)));
        }
    }

    Err(SomaError::Tts("火山引擎训练超时（5分钟）".into()))
}

/// 科大讯飞声音复刻：上传音频 → 训练 → 轮询状态 → 返回 vcn
async fn clone_xfyun(
    app_id: &str,
    api_key: &str,
    api_secret: &str,
    voice_name: &str,
    audio_data: &[u8],
    audio_filename: &str,
) -> Result<String, SomaError> {
    let client = reqwest::Client::new();
    let mime = guess_mime_type(audio_filename);
    let base_url = "https://aibrain.xfyun.cn/v1/private/sr3";

    // 1. 上传音频
    let part = reqwest::multipart::Part::bytes(audio_data.to_vec())
        .file_name(audio_filename.to_string())
        .mime_str(&mime)
        .map_err(|e| SomaError::Tts(format!("invalid mime type: {}", e)))?;

    let form = reqwest::multipart::Form::new()
        .text("app_id", app_id.to_string())
        .part("file", part);

    let resp = client
        .post(format!("{}/upload", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    let resp_json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| SomaError::Tts(format!("parse xfyun upload response failed: {}", e)))?;

    let audio_id = resp_json
        .get("data")
        .and_then(|d| d.get("audio_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| SomaError::Tts(format!("讯飞上传失败: {:?}", resp_json)))?
        .to_string();

    // 2. 发起训练
    let train_payload = serde_json::json!({
        "app_id": app_id,
        "api_key": api_key,
        "api_secret": api_secret,
        "audio_id": audio_id,
        "voice_name": voice_name,
    });

    let train_resp = client
        .post(format!("{}/train", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&train_payload)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    let train_json: serde_json::Value = train_resp
        .json()
        .await
        .map_err(|e| SomaError::Tts(format!("parse xfyun train response failed: {}", e)))?;

    let train_id = train_json
        .get("data")
        .and_then(|d| d.get("train_id"))
        .and_then(|v| v.as_str())
        .unwrap_or(&audio_id)
        .to_string();

    // 3. 轮询训练状态（最多等待 5 分钟）
    let query_payload = serde_json::json!({
        "app_id": app_id,
        "api_key": api_key,
        "api_secret": api_secret,
        "train_id": train_id,
    });

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let resp = client
            .post(format!("{}/query", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&query_payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        let q_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Tts(format!("parse xfyun query response failed: {}", e)))?;

        let status = q_json
            .get("data")
            .and_then(|d| d.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if status == "success" || status == "Success" {
            let vcn = q_json
                .get("data")
                .and_then(|d| d.get("vcn"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| SomaError::Tts("讯飞训练完成但未返回 vcn".into()))?
                .to_string();
            return Ok(vcn);
        }
        if status == "failed" || status == "Failed" {
            return Err(SomaError::Tts(format!("讯飞训练失败: {:?}", q_json)));
        }
    }

    Err(SomaError::Tts("讯飞训练超时（5分钟）".into()))
}

/// 根据文件扩展名推断 MIME 类型
fn guess_mime_type(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "mp3" => "audio/mpeg".to_string(),
        "wav" => "audio/wav".to_string(),
        "m4a" => "audio/mp4".to_string(),
        "aac" => "audio/aac".to_string(),
        "flac" => "audio/flac".to_string(),
        "ogg" => "audio/ogg".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

// ==================== 本地持久化存储 ====================

/// 获取克隆声音的存储目录（storage/voices/）
fn voices_store_dir() -> std::path::PathBuf {
    soma_core::utils::storage_dir("voices", true)
}

/// 获取单个克隆声音的元数据文件路径
fn voice_meta_path(id: &str) -> std::path::PathBuf {
    voices_store_dir().join(format!("{}.json", id))
}

/// 保存克隆声音元数据到本地
pub fn save_cloned_voice(voice: &ClonedVoice) -> Result<(), SomaError> {
    let path = voice_meta_path(&voice.id);
    let json = serde_json::to_string_pretty(voice)
        .map_err(|e| SomaError::Tts(format!("serialize voice failed: {}", e)))?;
    std::fs::write(&path, json).map_err(SomaError::Io)?;
    Ok(())
}

/// 加载所有已克隆的声音
pub fn list_cloned_voices() -> Result<Vec<ClonedVoice>, SomaError> {
    let dir = voices_store_dir();
    let mut voices = Vec::new();

    if !dir.exists() {
        return Ok(voices);
    }

    let entries = std::fs::read_dir(&dir).map_err(SomaError::Io)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(voice) = serde_json::from_str::<ClonedVoice>(&content) {
                    voices.push(voice);
                }
            }
        }
    }

    voices.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(voices)
}

/// 根据 ID 加载单个克隆声音
pub fn get_cloned_voice(id: &str) -> Result<Option<ClonedVoice>, SomaError> {
    let path = voice_meta_path(id);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path).map_err(SomaError::Io)?;
    let voice = serde_json::from_str::<ClonedVoice>(&content)
        .map_err(|e| SomaError::Tts(format!("deserialize voice failed: {}", e)))?;
    Ok(Some(voice))
}

/// 删除克隆声音的本地元数据和音频样本
pub fn delete_cloned_voice_local(id: &str) -> Result<ClonedVoice, SomaError> {
    let voice = get_cloned_voice(id)?
        .ok_or_else(|| SomaError::Tts(format!("voice not found: {}", id)))?;

    // 删除元数据文件
    let meta_path = voice_meta_path(id);
    if meta_path.exists() {
        std::fs::remove_file(&meta_path).map_err(SomaError::Io)?;
    }

    // 删除本地音频样本
    let sample_path = std::path::Path::new(&voice.sample_path);
    if sample_path.exists() {
        std::fs::remove_file(sample_path).map_err(SomaError::Io)?;
    }

    Ok(voice)
}

/// 生成克隆声音在语音列表中使用的 name 字段
///
/// ElevenLabs 格式: `elevenlabs:{voice_id}:{voice_name}`
/// SiliconFlow 格式: `siliconflow:FunAudioLLM/CosyVoice2-0.5B:{uri}`
/// 火山引擎格式: `volcengine:{voice_id}`
/// 讯飞格式: `xfyun:{vcn}`
pub fn cloned_voice_name(voice: &ClonedVoice) -> String {
    match voice.engine {
        CloneEngine::Elevenlabs => format!("elevenlabs:{}:{}", voice.remote_id, voice.name),
        CloneEngine::Siliconflow => format!(
            "siliconflow:FunAudioLLM/CosyVoice2-0.5B:{}",
            voice.remote_id
        ),
        CloneEngine::Volcengine => format!("volcengine:{}", voice.remote_id),
        CloneEngine::Xfyun => format!("xfyun:{}", voice.remote_id),
    }
}
