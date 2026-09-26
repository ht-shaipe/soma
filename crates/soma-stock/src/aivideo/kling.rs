//! 可灵 Kling 视频生成适配器（桥接层）
//!
//! 委托 ai-llm-kit 的 KlingMultiModal 实现，Soma 侧仅做类型转换。

use super::{AiVideoProvider, VideoGenParams, VideoGenStatus};
use ai_llm_kit::kling::Kling;
use ai_llm_kit::multi_modal::{
    download_video_common, LlmMultiModalService, MultiModalGenerationExt, VideoGenerationParams,
};
use async_trait::async_trait;
use soma_core::error::SomaError;

pub struct KlingVideo {
    multi_modal: Box<dyn LlmMultiModalService>,
    model: String,
}

impl KlingVideo {
    pub fn new(access_key: &str, secret_key: &str, model: &str) -> Self {
        Self {
            multi_modal: Kling::new(access_key, secret_key).multi_modal(),
            model: model.to_string(),
        }
    }
}

#[async_trait(?Send)]
impl AiVideoProvider for KlingVideo {
    async fn create_task(&self, params: &VideoGenParams) -> Result<String, SomaError> {
        let mm_params = VideoGenerationParams {
            model: self.model.clone(),
            prompt: params.prompt.clone(),
            image_url: params.image_url.clone(),
            size: None,
            quality: None,
            with_audio: None,
            fps: None,
            duration: Some(params.duration as i32),
            aspect_ratio: Some(params.aspect_ratio.clone()),
            prompt_optimizer: None,
        };
        let result = self
            .multi_modal
            .generate_video(&mm_params)
            .await
            .map_err(|e| SomaError::VideoGen(format!("{:?}", e)))?;
        if result.task_id.is_empty() {
            return Err(SomaError::VideoGen(
                "可灵返回空任务ID，可能被限流或参数错误".into(),
            ));
        }
        Ok(result.task_id)
    }

    async fn query_task(&self, task_id: &str) -> Result<VideoGenStatus, SomaError> {
        let result = self
            .multi_modal
            .query_video_task(task_id)
            .await
            .map_err(|e| SomaError::VideoGen(format!("{:?}", e)))?;
        if result.is_success() {
            Ok(VideoGenStatus::Success {
                video_urls: result.video_urls,
            })
        } else if result.is_failed() {
            Ok(VideoGenStatus::Failed {
                message: result.task_status,
            })
        } else {
            Ok(VideoGenStatus::Processing)
        }
    }

    async fn download_video(&self, url: &str, save_path: &str) -> Result<String, SomaError> {
        download_video_common(url, save_path)
            .await
            .map_err(|e| SomaError::VideoGen(format!("{:?}", e)))
    }
}
