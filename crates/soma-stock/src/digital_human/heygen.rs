//! HeyGen 数字人口播视频生成适配器（v3 API）
//!
//! 通过 HeyGen v3 API 实现照片 + 音频 → 口播视频的生成。
//! API 流程：上传素材 → 创建 image 类型视频 → 轮询状态 → 下载视频。
//!
//! HeyGen API 文档：https://developers.heygen.com/audio-to-video

use super::{DhVideoGenParams, DhVideoGenStatus, DigitalHumanProvider};
use async_trait::async_trait;
use soma_core::error::SomaError;

/// 从 HeyGen 查询响应文本解析任务状态
///
/// 将 HeyGen API 响应 JSON 解析为 `DhVideoGenStatus`。
/// 提取为独立函数以便单元测试覆盖各种状态映射（completed/failed/processing）。
fn parse_status_from_response(resp_text: &str) -> Result<DhVideoGenStatus, SomaError> {
    let resp_json: serde_json::Value = serde_json::from_str(resp_text)
        .map_err(|e| SomaError::VideoGen(format!("HeyGen 响应解析失败: {}", e)))?;

    let data = resp_json
        .get("data")
        .ok_or_else(|| SomaError::VideoGen(format!("HeyGen 响应缺少 data: {}", resp_text)))?;

    let status_str = data.get("status").and_then(|v| v.as_str()).unwrap_or("");

    match status_str {
        "completed" | "success" => {
            let video_url = data
                .get("video_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    SomaError::VideoGen(format!("HeyGen 完成但缺少 video_url: {}", resp_text))
                })?;
            Ok(DhVideoGenStatus::Success {
                video_url: video_url.to_string(),
            })
        }
        "failed" | "error" => {
            let message = data
                .get("failure_message")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("error").and_then(|v| v.as_str()))
                .unwrap_or("HeyGen 任务失败")
                .to_string();
            Ok(DhVideoGenStatus::Failed { message })
        }
        _ => Ok(DhVideoGenStatus::Processing),
    }
}

/// HeyGen 数字人提供商
pub struct HeyGenProvider {
    /// HeyGen API 密钥
    api_key: String,
    /// HeyGen API 基础 URL，默认 "https://api.heygen.com"
    base_url: String,
    /// 数字人模型名称（可选）
    model: String,
}

impl HeyGenProvider {
    /// 创建 HeyGen 提供商实例
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    /// 构建带认证头的 HTTP 客户端
    fn client(&self) -> Result<reqwest::Client, SomaError> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| SomaError::Http(format!("构建 HTTP 客户端失败: {}", e)))
    }

    /// 上传本地文件到 HeyGen 作为素材，返回 asset_id
    async fn upload_asset(&self, file_path: &str) -> Result<String, SomaError> {
        let client = self.client()?;
        let url = format!("{}/v3/assets", self.base_url);

        let file_bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| SomaError::VideoGen(format!("读取文件失败 {}: {}", file_path, e)))?;

        let filename = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(filename)
            .mime_str("application/octet-stream")
            .map_err(|e| SomaError::Http(format!("构建 multipart 失败: {}", e)))?;

        let form = reqwest::multipart::Form::new().part("file", part);

        log!("HeyGen 上传素材: {} -> {}", file_path, url);

        let resp = client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 上传请求失败: {}", e)))?;

        let status = resp.status();
        let resp_text = resp
            .text()
            .await
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 读取响应失败: {}", e)))?;

        if !status.is_success() {
            log!("HeyGen 上传素材失败: status={}, body={}", status, resp_text);
            return Err(SomaError::VideoGen(format!(
                "HeyGen 上传素材失败 (HTTP {}): {}",
                status, resp_text
            )));
        }

        let resp_json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 响应解析失败: {}", e)))?;

        let asset_id = resp_json
            .get("data")
            .and_then(|d| d.get("asset_id").or_else(|| d.get("id")))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SomaError::VideoGen(format!("HeyGen 上传响应缺少 asset id: {}", resp_text))
            })?;

        log!("HeyGen 素材已上传: asset_id={}", asset_id);
        Ok(asset_id.to_string())
    }
}

#[async_trait(?Send)]
impl DigitalHumanProvider for HeyGenProvider {
    /// 提交 HeyGen 口播视频生成任务
    ///
    /// 先上传照片和音频素材，再调用 POST /v3/videos 创建 image 类型视频。
    async fn create_task(&self, params: &DhVideoGenParams) -> Result<String, SomaError> {
        let image_asset_id = self.upload_asset(&params.portrait_path).await?;
        let audio_asset_id = self.upload_asset(&params.audio_path).await?;

        let client = self.client()?;
        let url = format!("{}/v3/videos", self.base_url);

        let mut body = serde_json::json!({
            "type": "image",
            "image": {
                "type": "asset_id",
                "asset_id": image_asset_id
            },
            "audio_asset_id": audio_asset_id,
            "aspect_ratio": params.aspect_ratio,
        });
        if !self.model.is_empty() {
            body["engine"] = serde_json::json!({ "type": self.model });
        }
        if let Some(ref m) = params.model {
            if !m.is_empty() {
                body["engine"] = serde_json::json!({ "type": m });
            }
        }

        log!("HeyGen 创建口播任务: POST {}", url);

        let resp = client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 请求失败: {}", e)))?;

        let status = resp.status();
        let resp_text = resp
            .text()
            .await
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 读取响应失败: {}", e)))?;

        if !status.is_success() {
            log!("HeyGen 创建任务失败: status={}, body={}", status, resp_text);
            return Err(SomaError::VideoGen(format!(
                "HeyGen 创建任务失败 (HTTP {}): {}",
                status, resp_text
            )));
        }

        let resp_json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 响应解析失败: {}", e)))?;

        let video_id = resp_json
            .get("data")
            .and_then(|d| d.get("video_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SomaError::VideoGen(format!("HeyGen 响应缺少 video_id: {}", resp_text))
            })?;

        log!("HeyGen 任务已提交: video_id={}", video_id);
        Ok(video_id.to_string())
    }

    /// 查询 HeyGen 任务状态
    ///
    /// 调用 GET /v3/videos/{video_id}，将状态映射为 `DhVideoGenStatus`。
    async fn query_task(&self, task_id: &str) -> Result<DhVideoGenStatus, SomaError> {
        let client = self.client()?;
        let url = format!("{}/v3/videos/{}", self.base_url, task_id);

        let resp = client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 查询失败: {}", e)))?;

        let status = resp.status();
        let resp_text = resp
            .text()
            .await
            .map_err(|e| SomaError::VideoGen(format!("HeyGen 读取响应失败: {}", e)))?;

        if !status.is_success() {
            return Err(SomaError::VideoGen(format!(
                "HeyGen 查询失败 (HTTP {}): {}",
                status, resp_text
            )));
        }

        parse_status_from_response(&resp_text)
    }

    /// 下载生成的视频到本地路径
    async fn download_video(&self, url: &str, save_path: &str) -> Result<String, SomaError> {
        super::download_video_common(url, save_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试构造函数去除 base_url 末尾斜杠
    #[test]
    fn test_new_trims_trailing_slash() {
        let provider = HeyGenProvider::new("key", "https://api.heygen.com/", "model");
        assert_eq!(provider.base_url, "https://api.heygen.com");

        let provider2 = HeyGenProvider::new("key", "https://api.heygen.com///", "model");
        assert_eq!(provider2.base_url, "https://api.heygen.com");
    }

    /// 测试 parse_status_from_response 解析 completed 状态
    #[test]
    fn test_parse_status_completed() {
        let resp = r#"{"data":{"status":"completed","video_url":"https://cdn.example.com/v.mp4"}}"#;
        let result = parse_status_from_response(resp).unwrap();
        match result {
            DhVideoGenStatus::Success { video_url } => {
                assert_eq!(video_url, "https://cdn.example.com/v.mp4");
            }
            _ => panic!("期望 Success 状态"),
        }
    }

    /// 测试 parse_status_from_response 解析 failed 状态（带 failure_message）
    #[test]
    fn test_parse_status_failed_with_failure_message() {
        let resp = r#"{"data":{"status":"failed","failure_message":"生成失败"}}"#;
        let result = parse_status_from_response(resp).unwrap();
        match result {
            DhVideoGenStatus::Failed { message } => {
                assert_eq!(message, "生成失败");
            }
            _ => panic!("期望 Failed 状态"),
        }
    }

    /// 测试 parse_status_from_response 解析 error 状态（带 error 字段）
    #[test]
    fn test_parse_status_error_with_error_field() {
        let resp = r#"{"data":{"status":"error","error":"内部错误"}}"#;
        let result = parse_status_from_response(resp).unwrap();
        match result {
            DhVideoGenStatus::Failed { message } => {
                assert_eq!(message, "内部错误");
            }
            _ => panic!("期望 Failed 状态"),
        }
    }

    /// 测试 parse_status_from_response 解析 failed 状态（无失败原因，使用默认消息）
    #[test]
    fn test_parse_status_failed_default_message() {
        let resp = r#"{"data":{"status":"failed"}}"#;
        let result = parse_status_from_response(resp).unwrap();
        match result {
            DhVideoGenStatus::Failed { message } => {
                assert_eq!(message, "HeyGen 任务失败");
            }
            _ => panic!("期望 Failed 状态"),
        }
    }

    /// 测试 parse_status_from_response 解析 processing 状态
    #[test]
    fn test_parse_status_processing() {
        let resp = r#"{"data":{"status":"processing"}}"#;
        let result = parse_status_from_response(resp).unwrap();
        assert!(matches!(result, DhVideoGenStatus::Processing));
    }

    /// 测试 parse_status_from_response 未知状态映射为 Processing
    #[test]
    fn test_parse_status_unknown_status() {
        let resp = r#"{"data":{"status":"unknown_state"}}"#;
        let result = parse_status_from_response(resp).unwrap();
        assert!(matches!(result, DhVideoGenStatus::Processing));
    }

    /// 测试 parse_status_from_response completed 状态但缺少 video_url 时返回错误
    #[test]
    fn test_parse_status_completed_missing_video_url() {
        let resp = r#"{"data":{"status":"completed"}}"#;
        let result = parse_status_from_response(resp);
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::VideoGen(msg) => assert!(msg.contains("缺少 video_url")),
            _ => panic!("期望 SomaError::VideoGen 错误"),
        }
    }

    /// 测试 parse_status_from_response 缺少 data 字段时返回错误
    #[test]
    fn test_parse_status_missing_data() {
        let resp = r#"{"code":200}"#;
        let result = parse_status_from_response(resp);
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::VideoGen(msg) => assert!(msg.contains("缺少 data")),
            _ => panic!("期望 SomaError::VideoGen 错误"),
        }
    }

    /// 测试 parse_status_from_response 无效 JSON 时返回错误
    #[test]
    fn test_parse_status_invalid_json() {
        let resp = "not a json";
        let result = parse_status_from_response(resp);
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::VideoGen(msg) => assert!(msg.contains("响应解析失败")),
            _ => panic!("期望 SomaError::VideoGen 错误"),
        }
    }

    /// 测试 create_task 在文件不存在时返回错误（避免真实网络调用）
    #[tokio::test]
    async fn test_create_task_file_not_found() {
        let provider = HeyGenProvider::new("key", "https://api.heygen.com", "model");
        let params = DhVideoGenParams {
            portrait_path: "/nonexistent/path/portrait_12345.jpg".to_string(),
            audio_path: "/nonexistent/path/audio_12345.mp3".to_string(),
            aspect_ratio: "16:9".to_string(),
            model: None,
        };
        let result = provider.create_task(&params).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::VideoGen(msg) => assert!(msg.contains("读取文件失败")),
            _ => panic!("期望 SomaError::VideoGen 错误"),
        }
    }

    /// 测试 query_task 在连接失败时返回错误（使用 .invalid TLD 确保 DNS 解析失败）
    #[tokio::test]
    async fn test_query_task_connection_error() {
        let provider = HeyGenProvider::new("key", "http://heygen-test.invalid", "model");
        let result = provider.query_task("test-task-id").await;
        assert!(result.is_err());
    }

    /// 测试 download_video 在无效 URL 时返回错误
    #[tokio::test]
    async fn test_download_video_invalid_url() {
        let provider = HeyGenProvider::new("key", "https://api.heygen.com", "model");
        let save_path = format!("/tmp/soma_test_dl_{}.mp4", soma_core::utils::get_uuid());
        let result = provider
            .download_video("http://download-test.invalid/video.mp4", &save_path)
            .await;
        assert!(result.is_err());
    }
}
