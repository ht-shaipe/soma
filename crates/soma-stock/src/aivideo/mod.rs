//! AI 视频生成模块
//!
//! 提供基于 AI 模型的文本生成视频能力，支持多个国内视频生成服务：
//! - 智谱 CogVideoX（含免费模型 cogvideox-flash）
//! - 可灵 Kling（快手）
//! - MiniMax 海螺
//!
//! 所有提供商均采用异步任务模式：提交生成请求 → 轮询任务状态 → 下载视频文件。
//! 底层 HTTP 调用委托给 ai-llm-kit，本模块仅做业务层类型转换和桥接。

use async_trait::async_trait;
use soma_core::error::SomaError;

pub mod zhipu;
pub mod kling;
pub mod minimax;

pub use ai_llm_kit::multimodal::aspect_to_size;

/// AI 视频生成请求参数
#[derive(Debug, Clone)]
pub struct VideoGenParams {
    pub prompt: String,
    pub aspect_ratio: String,
    pub duration: u32,
    pub model: Option<String>,
    /// 参考图片 URL（图生视频模式，如人像口播）
    pub image_url: Option<String>,
}

/// AI 视频生成任务状态
#[derive(Debug, Clone)]
pub enum VideoGenStatus {
    Processing,
    Success { video_urls: Vec<String> },
    Failed { message: String },
}

/// AI 视频生成提供商 trait
#[async_trait(?Send)]
pub trait AiVideoProvider: Send + Sync {
    async fn create_task(&self, params: &VideoGenParams) -> Result<String, SomaError>;
    async fn query_task(&self, task_id: &str) -> Result<VideoGenStatus, SomaError>;
    async fn download_video(&self, url: &str, save_path: &str) -> Result<String, SomaError>;
}

/// 通用视频下载（委托 ai-llm-kit）
pub async fn download_video_common(url: &str, save_path: &str) -> Result<String, SomaError> {
    ai_llm_kit::multimodal::download_video_common(url, save_path).await
        .map_err(|e| SomaError::VideoGen(format!("{:?}", e)))
}

/// 轮询 AI 视频生成任务直到完成或超时
///
/// 动态轮询间隔：前30秒每3秒查询一次（快速捕获完成），
/// 之后每8秒查询一次（减少无效请求）。
pub async fn poll_until_done(
    provider: &dyn AiVideoProvider,
    task_id: &str,
    timeout: u64,
    _interval: u64,
) -> Result<VideoGenStatus, SomaError> {
    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let fast_phase = std::time::Duration::from_secs(30);

    loop {
        let status = provider.query_task(task_id).await?;
        match status {
            VideoGenStatus::Success { .. } => return Ok(status),
            VideoGenStatus::Failed { message } => return Ok(VideoGenStatus::Failed { message }),
            VideoGenStatus::Processing => {}
        }

        if start.elapsed() >= timeout_dur {
            return Err(SomaError::VideoGen(format!("AI视频生成超时（{}秒）", timeout)));
        }

        let interval = if start.elapsed() < fast_phase { 3 } else { 8 };
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}

/// 根据提供商名称创建对应的 AI 视频生成实例
pub fn create_provider(provider: &str, conf: &soma_core::config::AppConfig) -> Result<Box<dyn AiVideoProvider>, SomaError> {
    match provider {
        "cogvideox" => {
            let api_key = conf.app.zhipu_video_api_key
                .as_deref()
                .or(conf.app.zhipu_api_key.as_deref())
                .unwrap_or("");
            if api_key.is_empty() {
                return Err(SomaError::VideoGen("智谱视频生成 API Key 未配置".into()));
            }
            let model = conf.app.zhipu_video_model
                .as_deref()
                .unwrap_or("cogvideox-flash");
            Ok(Box::new(zhipu::ZhipuVideo::new(api_key, model)))
        }
        "kling" => {
            let access_key = conf.app.kling_access_key.as_deref().unwrap_or("");
            let secret_key = conf.app.kling_secret_key.as_deref().unwrap_or("");
            if access_key.is_empty() || secret_key.is_empty() {
                return Err(SomaError::VideoGen("可灵 Access Key / Secret Key 未配置".into()));
            }
            let model = conf.app.kling_video_model
                .as_deref()
                .unwrap_or("kling-v2-master");
            Ok(Box::new(kling::KlingVideo::new(access_key, secret_key, model)))
        }
        "minimax" => {
            let api_key = conf.app.minimax_video_api_key.as_deref().unwrap_or("");
            if api_key.is_empty() {
                return Err(SomaError::VideoGen("MiniMax 视频 API Key 未配置".into()));
            }
            let model = conf.app.minimax_video_model
                .as_deref()
                .unwrap_or("MiniMax-Hailuo-2.3");
            Ok(Box::new(minimax::MinimaxVideo::new(api_key, model)))
        }
        _ => Err(SomaError::VideoGen(format!("不支持的AI视频生成提供商: {}", provider))),
    }
}
