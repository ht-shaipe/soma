//! 数字人口播视频生成模块
//!
//! 提供基于第三方数字人服务的口播视频生成能力（照片 + 音频 → 口播视频）。
//! 与 `aivideo` 模块的区别：`aivideo` 仅支持图生视频（无音频输入），
//! 本模块的 `DigitalHumanProvider` 接受照片 URL + 音频 URL，生成口型同步的口播视频。
//!
//! 首期实现 HeyGen 提供商。所有提供商采用异步任务模式：
//! 提交生成请求 → 轮询任务状态 → 下载视频文件。

use async_trait::async_trait;
use soma_core::config::DigitalHumanSection;
use soma_core::error::SomaError;

pub mod heygen;
pub mod sadtalker;
pub mod echomimic_v3;
pub mod heygem;
pub mod live2d;

/// 数字人口播视频生成请求参数
#[derive(Debug, Clone)]
pub struct DhVideoGenParams {
    /// 人像照片本地文件路径
    pub portrait_path: String,
    /// 配音音频本地文件路径
    pub audio_path: String,
    /// 画面宽高比，如 "16:9"、"9:16"、"1:1"
    pub aspect_ratio: String,
    /// 数字人模型名称（可选，使用提供商默认模型）
    pub model: Option<String>,
}

/// 数字人口播视频生成任务状态
#[derive(Debug, Clone)]
pub enum DhVideoGenStatus {
    /// 任务处理中
    Processing,
    /// 任务成功，包含生成的视频 URL
    Success { video_url: String },
    /// 任务失败，包含失败原因
    Failed { message: String },
}

/// 数字人口播视频生成提供商 trait
///
/// 与 `AiVideoProvider` 区别：本 trait 的 `create_task` 接受音频 URL 参数，
/// 用于生成口型与音频同步的口播视频。
#[async_trait(?Send)]
pub trait DigitalHumanProvider: Send + Sync {
    /// 提交口播视频生成任务，返回第三方任务 ID
    async fn create_task(&self, params: &DhVideoGenParams) -> Result<String, SomaError>;
    /// 查询任务状态
    async fn query_task(&self, task_id: &str) -> Result<DhVideoGenStatus, SomaError>;
    /// 下载生成的视频到本地路径
    async fn download_video(&self, url: &str, save_path: &str) -> Result<String, SomaError>;
}

/// 通用视频下载（委托 ai-llm-kit）
pub async fn download_video_common(url: &str, save_path: &str) -> Result<String, SomaError> {
    ai_llm_kit::multi_modal::download_video_common(url, save_path)
        .await
        .map_err(|e| SomaError::VideoGen(format!("{:?}", e)))
}

/// 轮询数字人口播视频生成任务直到完成或超时
///
/// 动态轮询间隔：前30秒每3秒查询一次，之后每 `poll_interval` 秒查询一次。
pub async fn poll_until_done(
    provider: &dyn DigitalHumanProvider,
    task_id: &str,
    timeout: u64,
    poll_interval: u64,
) -> Result<DhVideoGenStatus, SomaError> {
    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let fast_phase = std::time::Duration::from_secs(30);
    let slow_interval = if poll_interval > 0 { poll_interval } else { 5 };

    loop {
        let status = provider.query_task(task_id).await?;
        match status {
            DhVideoGenStatus::Success { .. } => return Ok(status),
            DhVideoGenStatus::Failed { message } => {
                return Ok(DhVideoGenStatus::Failed { message })
            }
            DhVideoGenStatus::Processing => {}
        }

        if start.elapsed() >= timeout_dur {
            return Err(SomaError::VideoGen(format!(
                "数字人视频生成超时（{}秒）",
                timeout
            )));
        }

        let interval = if start.elapsed() < fast_phase {
            3
        } else {
            slow_interval
        };
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}

/// 根据配置创建对应的数字人口播视频生成提供商实例
///
/// 支持 "heygen"（云端 API）、"sadtalker"（本地 CPU/GPU 推理）、"echomimic_v3"（本地 GPU 推理）、
/// "heygem"（HeyGem/Duix.Avatar HTTP API）和 "live2d"（Live2D 卡通口播，纯 CPU 渲染），
/// 其他返回 `SomaError::Config`。
pub fn create_provider(conf: &DigitalHumanSection) -> Result<Box<dyn DigitalHumanProvider>, SomaError> {
    let provider = conf.get_provider();
    match provider {
        "heygen" => {
            let api_key = conf.api_key.as_deref().unwrap_or("");
            if api_key.is_empty() {
                return Err(SomaError::Config(
                    "HeyGen API Key 未配置，请在 [digital_human] 段设置 api_key".into(),
                ));
            }
            let base_url = conf
                .base_url
                .as_deref()
                .unwrap_or("https://api.heygen.com");
            let model = conf.model.as_deref().unwrap_or("");
            Ok(Box::new(heygen::HeyGenProvider::new(api_key, base_url, model)))
        }
        "sadtalker" => {
            let sadtalker_conf = &conf.sadtalker;
            if sadtalker_conf.get_env_path().is_empty() {
                return Err(SomaError::Config(
                    "SadTalker 配置不完整，请在 [digital_human.sadtalker] 段设置 env_path".into(),
                ));
            }
            if sadtalker_conf.get_model_path().is_empty() {
                return Err(SomaError::Config(
                    "SadTalker 配置不完整，请在 [digital_human.sadtalker] 段设置 model_path".into(),
                ));
            }
            Ok(Box::new(sadtalker::SadTalkerProvider::new(sadtalker_conf.clone())))
        }
        "echomimic_v3" => {
            let emv3_conf = &conf.echomimic_v3;
            if emv3_conf.get_env_path().is_empty() {
                return Err(SomaError::Config(
                    "EchoMimicV3 配置不完整，请在 [digital_human.echomimic_v3] 段设置 env_path".into(),
                ));
            }
            if emv3_conf.get_model_path().is_empty() {
                return Err(SomaError::Config(
                    "EchoMimicV3 配置不完整，请在 [digital_human.echomimic_v3] 段设置 model_path".into(),
                ));
            }
            Ok(Box::new(echomimic_v3::EchoMimicV3Provider::new(emv3_conf.clone())))
        }
        "heygem" => {
            let hg_conf = &conf.heygem;
            if hg_conf.get_tts_base_url().is_empty() {
                return Err(SomaError::Config(
                    "HeyGem 配置不完整，请在 [digital_human.heygem] 段设置 tts_base_url".into(),
                ));
            }
            if hg_conf.get_video_base_url().is_empty() {
                return Err(SomaError::Config(
                    "HeyGem 配置不完整，请在 [digital_human.heygem] 段设置 video_base_url".into(),
                ));
            }
            Ok(Box::new(heygem::HeyGemProvider::new(hg_conf.clone())))
        }
        "live2d" => {
            let l2d_conf = &conf.live2d;
            if l2d_conf.get_script_path().is_empty() {
                return Err(SomaError::Config(
                    "Live2D 配置不完整，请在 [digital_human.live2d] 段设置 script_path".into(),
                ));
            }
            if l2d_conf.get_models_dir().is_empty() {
                return Err(SomaError::Config(
                    "Live2D 配置不完整，请在 [digital_human.live2d] 段设置 models_dir".into(),
                ));
            }
            Ok(Box::new(live2d::Live2DProvider::new(l2d_conf.clone())))
        }
        _ => Err(SomaError::Config(format!(
            "不支持的数字人提供商: {}",
            provider
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soma_core::config::DigitalHumanSection;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// 用于测试 poll_until_done 的 mock provider
    struct MockProvider {
        /// 按顺序返回的状态序列
        statuses: Vec<DhVideoGenStatus>,
        /// 当前调用次数
        call_count: AtomicUsize,
    }

    impl MockProvider {
        fn new(statuses: Vec<DhVideoGenStatus>) -> Self {
            Self {
                statuses,
                call_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait(?Send)]
    impl DigitalHumanProvider for MockProvider {
        async fn create_task(&self, _params: &DhVideoGenParams) -> Result<String, SomaError> {
            Ok("mock-task-id".to_string())
        }

        async fn query_task(&self, _task_id: &str) -> Result<DhVideoGenStatus, SomaError> {
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
            if idx < self.statuses.len() {
                Ok(self.statuses[idx].clone())
            } else {
                // 超出预设状态后持续返回 Processing
                Ok(DhVideoGenStatus::Processing)
            }
        }

        async fn download_video(&self, _url: &str, _save_path: &str) -> Result<String, SomaError> {
            Ok("mock-download-path".to_string())
        }
    }


    /// 测试 create_provider heygen 提供商
    #[test]
    fn test_create_provider_heygen() {
        let conf = DigitalHumanSection {
            provider: Some("heygen".to_string()),
            api_key: Some("test-api-key".to_string()),
            base_url: Some("https://api.heygen.com".to_string()),
            model: Some("test-model".to_string()),
            ..Default::default()
        };
        let provider = create_provider(&conf);
        assert!(provider.is_ok());
    }

    /// 测试 create_provider 默认提供商（未设置 provider 时默认 heygen）
    #[test]
    fn test_create_provider_default_provider() {
        let conf = DigitalHumanSection {
            api_key: Some("key".to_string()),
            ..Default::default()
        };
        let provider = create_provider(&conf);
        assert!(provider.is_ok());
    }

    /// 测试 create_provider 未知提供商返回错误
    #[test]
    fn test_create_provider_unknown_provider() {
        let conf = DigitalHumanSection {
            provider: Some("unknown_provider".to_string()),
            api_key: Some("key".to_string()),
            ..Default::default()
        };
        let result = create_provider(&conf);
        assert!(result.is_err());
        if let Err(SomaError::Config(msg)) = result {
            assert!(msg.contains("不支持的数字人提供商"));
        } else {
            panic!("期望 SomaError::Config 错误");
        }
    }

    /// 测试 create_provider 空 API Key 返回错误
    #[test]
    fn test_create_provider_empty_api_key() {
        let conf = DigitalHumanSection {
            provider: Some("heygen".to_string()),
            api_key: Some("".to_string()),
            ..Default::default()
        };
        let result = create_provider(&conf);
        assert!(result.is_err());
        if let Err(SomaError::Config(msg)) = result {
            assert!(msg.contains("API Key 未配置"));
        } else {
            panic!("期望 SomaError::Config 错误");
        }
    }


    /// 测试 create_provider 使用默认 base_url
    #[test]
    fn test_create_provider_default_base_url() {
        let conf = DigitalHumanSection {
            provider: Some("heygen".to_string()),
            api_key: Some("key".to_string()),
            base_url: None,
            ..Default::default()
        };
        let provider = create_provider(&conf);
        assert!(provider.is_ok());
    }

    /// 测试 create_provider echomimic_v3 提供商
    #[test]
    fn test_create_provider_echomimic_v3() {
        let conf = DigitalHumanSection {
            provider: Some("echomimic_v3".to_string()),
            echomimic_v3: soma_core::config::EchoMimicV3Config {
                env_path: Some("/opt/EchoMimicV3".to_string()),
                model_path: Some("/models/echomimic".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let provider = create_provider(&conf);
        assert!(provider.is_ok());
    }

    /// 测试 create_provider echomimic_v3 缺少 env_path 返回错误
    #[test]
    fn test_create_provider_echomimic_v3_missing_env_path() {
        let conf = DigitalHumanSection {
            provider: Some("echomimic_v3".to_string()),
            echomimic_v3: soma_core::config::EchoMimicV3Config {
                model_path: Some("/models/echomimic".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = create_provider(&conf);
        assert!(result.is_err());
        if let Err(SomaError::Config(msg)) = result {
            assert!(msg.contains("env_path"));
        } else {
            panic!("期望 SomaError::Config 错误");
        }
    }

    /// 测试 create_provider echomimic_v3 缺少 model_path 返回错误
    #[test]
    fn test_create_provider_echomimic_v3_missing_model_path() {
        let conf = DigitalHumanSection {
            provider: Some("echomimic_v3".to_string()),
            echomimic_v3: soma_core::config::EchoMimicV3Config {
                env_path: Some("/opt/EchoMimicV3".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = create_provider(&conf);
        assert!(result.is_err());
        if let Err(SomaError::Config(msg)) = result {
            assert!(msg.contains("model_path"));
        } else {
            panic!("期望 SomaError::Config 错误");
        }
    }

    /// 测试 create_provider echomimic_v3 全 None 配置返回错误
    #[test]
    fn test_create_provider_echomimic_v3_all_none() {
        let conf = DigitalHumanSection {
            provider: Some("echomimic_v3".to_string()),
            ..Default::default()
        };
        let result = create_provider(&conf);
        assert!(result.is_err());
    }

    /// 测试 poll_until_done 第一次查询即返回 Success
    #[tokio::test]
    async fn test_poll_until_done_immediate_success() {
        let provider = MockProvider::new(vec![DhVideoGenStatus::Success {
            video_url: "https://cdn.example.com/v.mp4".to_string(),
        }]);
        let result = poll_until_done(&provider, "task-1", 60, 5).await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Success { video_url } => {
                assert_eq!(video_url, "https://cdn.example.com/v.mp4");
            }
            _ => panic!("期望 Success 状态"),
        }
    }

    /// 测试 poll_until_done 第一次查询即返回 Failed
    #[tokio::test]
    async fn test_poll_until_done_immediate_failed() {
        let provider = MockProvider::new(vec![DhVideoGenStatus::Failed {
            message: "生成失败".to_string(),
        }]);
        let result = poll_until_done(&provider, "task-1", 60, 5).await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Failed { message } => {
                assert_eq!(message, "生成失败");
            }
            _ => panic!("期望 Failed 状态"),
        }
    }

    /// 测试 poll_until_done 超时返回错误（timeout=0 时第一次 Processing 后立即超时）
    #[tokio::test]
    async fn test_poll_until_done_timeout() {
        let provider = MockProvider::new(vec![DhVideoGenStatus::Processing]);
        let result = poll_until_done(&provider, "task-1", 0, 5).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::VideoGen(msg) => assert!(msg.contains("超时")),
            _ => panic!("期望 SomaError::VideoGen 错误"),
        }
    }

    /// 测试 poll_until_done Processing 后 Success（经历一次 sleep）
    #[tokio::test]
    async fn test_poll_until_done_processing_then_success() {
        let provider = MockProvider::new(vec![
            DhVideoGenStatus::Processing,
            DhVideoGenStatus::Success {
                video_url: "https://cdn.example.com/v2.mp4".to_string(),
            },
        ]);
        let result = poll_until_done(&provider, "task-2", 60, 5).await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Success { video_url } => {
                assert_eq!(video_url, "https://cdn.example.com/v2.mp4");
            }
            _ => panic!("期望 Success 状态"),
        }
    }

    /// 测试 poll_until_done Processing 后 Failed（经历一次 sleep）
    #[tokio::test]
    async fn test_poll_until_done_processing_then_failed() {
        let provider = MockProvider::new(vec![
            DhVideoGenStatus::Processing,
            DhVideoGenStatus::Failed {
                message: "后续失败".to_string(),
            },
        ]);
        let result = poll_until_done(&provider, "task-3", 60, 5).await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Failed { message } => {
                assert_eq!(message, "后续失败");
            }
            _ => panic!("期望 Failed 状态"),
        }
    }

    /// 测试 poll_until_done query_task 返回错误时传播错误
    #[tokio::test]
    async fn test_poll_until_done_query_error() {
        struct ErrorProvider;
        #[async_trait(?Send)]
        impl DigitalHumanProvider for ErrorProvider {
            async fn create_task(&self, _: &DhVideoGenParams) -> Result<String, SomaError> {
                Ok("".to_string())
            }
            async fn query_task(&self, _: &str) -> Result<DhVideoGenStatus, SomaError> {
                Err(SomaError::VideoGen("查询失败".to_string()))
            }
            async fn download_video(&self, _: &str, _: &str) -> Result<String, SomaError> {
                Ok("".to_string())
            }
        }
        let provider = ErrorProvider;
        let result = poll_until_done(&provider, "task-err", 60, 5).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::VideoGen(msg) => assert!(msg.contains("查询失败")),
            _ => panic!("期望 SomaError::VideoGen 错误"),
        }
    }
}