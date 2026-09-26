//! 智谱 CogVideoX 视频生成适配器（桥接层）
//!
//! 委托 ai-llm-kit 的 ZhipuMultiModal 实现，Soma 侧仅做类型转换。

use super::{AiVideoProvider, VideoGenParams, VideoGenStatus};
use ai_llm_kit::multi_modal::{
    download_video_common, LlmMultiModalService, MultiModalGenerationExt, VideoGenerationParams,
};
use ai_llm_kit::zpu::ZhipuAI;
use async_trait::async_trait;
use soma_core::error::SomaError;

pub struct ZhipuVideo {
    multi_modal: Box<dyn LlmMultiModalService>,
    model: String,
}

impl ZhipuVideo {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            multi_modal: ZhipuAI::new(api_key).multi_modal(),
            model: model.to_string(),
        }
    }
}

#[async_trait(?Send)]
impl AiVideoProvider for ZhipuVideo {
    async fn create_task(&self, params: &VideoGenParams) -> Result<String, SomaError> {
        let mm_params = VideoGenerationParams {
            model: self.model.clone(),
            prompt: params.prompt.clone(),
            image_url: params.image_url.clone(),
            size: Some(super::aspect_to_size(&params.aspect_ratio).to_string()),
            quality: Some("speed".to_string()),
            with_audio: None,
            fps: Some(30),
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
                "智谱返回空任务ID，可能被限流或参数错误".into(),
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
