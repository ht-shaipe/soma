/// 跨平台视频发布模块
///
/// 通过 Upload-Post API 将生成的视频发布到 TikTok、Instagram、YouTube Shorts 等平台。
/// API 文档：https://docs.upload-post.com

use soma_core::error::SomaError;
use soma_core::config::UiSection;
use std::path::Path;

const API_BASE: &str = "https://api.upload-post.com";

/// 跨平台发布结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadResult {
    pub success: bool,
    pub request_id: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
    pub platform_results: Option<serde_json::Value>,
}

/// 检查 Upload-Post 是否已配置
pub fn is_configured(ui: &UiSection) -> bool {
    let enabled = ui.upload_post_enabled.unwrap_or(false);
    let has_key = ui.upload_post_api_key.as_deref().unwrap_or("") != "";
    let has_user = ui.upload_post_username.as_deref().unwrap_or("") != "";
    enabled && has_key && has_user
}

/// 将视频跨平台发布到 TikTok/Instagram/YouTube 等
///
/// - `video_path`: 视频文件路径
/// - `title`: 视频标题
/// - `platforms`: 目标平台列表（如 ["tiktok", "instagram", "youtube"]）
/// - `ui`: UI/发布配置
/// - `youtube_extra`: YouTube 额外参数（description, tags, privacyStatus 等）
pub async fn upload_video(
    video_path: &str,
    title: &str,
    platforms: &[String],
    ui: &UiSection,
    youtube_extra: Option<&serde_json::Value>,
) -> Result<UploadResult, SomaError> {
    if !is_configured(ui) {
        return Err(SomaError::Upload("Upload-Post 未配置".into()));
    }

    if !Path::new(video_path).exists() {
        return Err(SomaError::Upload(format!("视频文件不存在: {}", video_path)));
    }

    let api_key = ui.upload_post_api_key.as_deref().unwrap_or("");
    let username = ui.upload_post_username.as_deref().unwrap_or("");
    let privacy = ui.upload_post_youtube_privacy_status.as_deref().unwrap_or("public");

    let file_bytes = std::fs::read(video_path).map_err(SomaError::Io)?;
    let file_name = Path::new(video_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut form = reqwest::multipart::Form::new()
        .text("user", username.to_string())
        .text("title", title.chars().take(2200).collect::<String>())
        .text("privacy_level", "PUBLIC_TO_EVERYONE".to_string());

    for platform in platforms {
        form = form.text("platform[]", platform.clone());
    }

    if let Some(ye) = youtube_extra {
        if platforms.iter().any(|p| p == "youtube") {
            if let Some(desc) = ye.get("youtube_description").and_then(|d| d.as_str()) {
                form = form.text("youtube_description", desc.to_string());
            }
            if let Some(tags) = ye.get("tags").and_then(|t| t.as_array()) {
                for tag in tags {
                    if let Some(tag_str) = tag.as_str() {
                        form = form.text("tags[]", tag_str.to_string());
                    }
                }
            }
            let yt_privacy = ye.get("privacyStatus").and_then(|p| p.as_str()).unwrap_or(privacy);
            form = form.text("privacyStatus", yt_privacy.to_string());
            form = form.text("containsSyntheticMedia", "true".to_string());
        }
    }

    let part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(file_name)
        .mime_str("video/mp4")
        .unwrap_or_else(|_| reqwest::multipart::Part::bytes(vec![]).file_name("video.mp4"));
    form = form.part("video", part);

    let client = reqwest::Client::new();
    let resp = client
        .post(&format!("{}/api/upload", API_BASE))
        .header("Authorization", format!("Apikey {}", api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SomaError::Upload(format!("Upload-Post API 错误: {} - {}", status, body)));
    }

    let resp_json: serde_json::Value = resp.json().await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    Ok(UploadResult {
        success: resp_json.get("success").and_then(|s| s.as_bool()).unwrap_or(false),
        request_id: resp_json.get("request_id").and_then(|r| r.as_str()).map(String::from),
        message: resp_json.get("message").and_then(|m| m.as_str()).map(String::from),
        error: None,
        platform_results: resp_json.get("platform_results").cloned(),
    })
}

/// 查询发布状态
pub async fn check_status(request_id: &str, api_key: &str) -> Result<serde_json::Value, SomaError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/api/uploadposts/status", API_BASE))
        .query(&[("request_id", request_id)])
        .header("Authorization", format!("Apikey {}", api_key))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| SomaError::Http(e.to_string()))?;

    resp.json().await.map_err(|e| SomaError::Http(e.to_string()))
}
