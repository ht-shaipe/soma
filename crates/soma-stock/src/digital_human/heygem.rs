//! HeyGem/Duix.Avatar 视频合成 Provider
//!
//! 通过 HTTP API 调用 HeyGem Docker 服务生成口播视频。
//! 端口 8383，接口 POST /easy/submit + GET /easy/query

use super::{DhVideoGenParams, DhVideoGenStatus, DigitalHumanProvider};
use async_trait::async_trait;
use soma_core::config::HeyGemConfig;
use soma_core::error::SomaError;

pub struct HeyGemProvider {
    config: HeyGemConfig,
}

impl HeyGemProvider {
    pub fn new(config: HeyGemConfig) -> Self {
        Self { config }
    }
}

#[async_trait(?Send)]
impl DigitalHumanProvider for HeyGemProvider {
    async fn create_task(&self, params: &DhVideoGenParams) -> Result<String, SomaError> {
        let url = format!("{}/easy/submit", self.config.get_video_base_url());
        let task_code = uuid::Uuid::new_v4().to_string();

        let body = serde_json::json!({
            "audio_url": params.audio_path,
            "video_url": params.portrait_path,
            "code": task_code,
            "chaofen": 0,
            "watermark_switch": 0,
            "pn": 1,
        });

        log::info!("HeyGem 视频合成请求: url={}, code={}", url, task_code);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.get_timeout()))
            .build()
            .map_err(|e| SomaError::VideoGen(format!("HTTP 客户端创建失败: {}", e)))?;

        let max_retries = self.config.get_max_retries() as usize;
        for attempt in 1..=max_retries {
            let resp = client.post(&url).json(&body).send().await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    log::info!("HeyGem 视频合成提交成功: code={}", task_code);
                    return Ok(task_code);
                }
                Ok(r) => {
                    let status = r.status();
                    if status.is_client_error() {
                        let text = r.text().await.unwrap_or_default();
                        return Err(SomaError::VideoGen(format!(
                            "视频合成请求参数错误: HTTP {} - {}",
                            status, text
                        )));
                    }
                    log::warn!(
                        "HeyGem 视频合成重试 {}/{}: HTTP {}",
                        attempt,
                        max_retries,
                        status
                    );
                }
                Err(e) => {
                    if e.is_connect() {
                        return Err(SomaError::VideoGen(
                            "HeyGem 视频合成服务不可达，请确认 Docker 容器已启动".into(),
                        ));
                    }
                    log::warn!("HeyGem 视频合成重试 {}/{}: {}", attempt, max_retries, e);
                }
            }
            if attempt < max_retries {
                std::thread::sleep(std::time::Duration::from_secs(1 << (attempt - 1)));
            }
        }

        Err(SomaError::VideoGen("视频合成服务过载或超时".into()))
    }

    async fn query_task(&self, task_id: &str) -> Result<DhVideoGenStatus, SomaError> {
        let url = format!(
            "{}/easy/query?code={}",
            self.config.get_video_base_url(),
            task_id
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| SomaError::VideoGen(format!("HTTP 客户端创建失败: {}", e)))?;

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| SomaError::VideoGen(format!("查询失败: {}", e)))?;

        if !resp.status().is_success() {
            return Err(SomaError::VideoGen(format!(
                "查询失败: HTTP {}",
                resp.status()
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::VideoGen(format!("解析响应失败: {}", e)))?;

        let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let progress = json.get("progress").and_then(|v| v.as_f64()).unwrap_or(0.0);

        match status {
            "success" | "done" | "completed" => {
                let video_url = json
                    .get("video_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(DhVideoGenStatus::Success { video_url })
            }
            "failed" | "error" => {
                let msg = json
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误")
                    .to_string();
                Ok(DhVideoGenStatus::Failed { message: msg })
            }
            _ => {
                log::info!(
                    "HeyGem 视频合成进度: code={}, progress={:.0}%",
                    task_id,
                    progress * 100.0
                );
                Ok(DhVideoGenStatus::Processing)
            }
        }
    }

    async fn download_video(&self, url: &str, save_path: &str) -> Result<String, SomaError> {
        super::download_video_common(url, save_path).await
    }
}

/// HeyGem 健康检查缺失类型
#[derive(Debug, Clone)]
pub enum HeyGemMissingKind {
    Environment,
    Dependency,
}

/// HeyGem 健康检查缺失项
#[derive(Debug, Clone)]
pub struct HeyGemMissingItem {
    pub kind: HeyGemMissingKind,
    pub description: String,
}

/// HeyGem 健康检查报告
#[derive(Debug, Clone)]
pub struct HeyGemHealthReport {
    pub ready: bool,
    pub missing: Vec<HeyGemMissingItem>,
}

/// HeyGem 连通性检查器
pub struct HeyGemHealthChecker {
    config: HeyGemConfig,
}

impl HeyGemHealthChecker {
    pub fn new(config: HeyGemConfig) -> Self {
        Self { config }
    }

    /// 对 TTS 与视频合成端点发起轻量 HTTP 探测（超时 3 秒）
    pub fn check(&self) -> HeyGemHealthReport {
        let mut missing = Vec::new();

        if let Some(item) = self.check_endpoint(self.config.get_tts_base_url(), "TTS") {
            missing.push(item);
        }
        if let Some(item) = self.check_endpoint(self.config.get_video_base_url(), "视频合成") {
            missing.push(item);
        }

        HeyGemHealthReport {
            ready: missing.is_empty(),
            missing,
        }
    }

    fn check_endpoint(&self, url: &str, label: &str) -> Option<HeyGemMissingItem> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let probe = async {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()
                .map_err(|e| format!("{}", e))?;
            client.get(url).send().await.map_err(|e| format!("{}", e))
        };
        match rt.block_on(probe) {
            Ok(r) if r.status().is_success() || r.status().is_client_error() => None,
            Ok(r) => Some(HeyGemMissingItem {
                kind: HeyGemMissingKind::Dependency,
                description: format!("{} 服务过载: HTTP {}", label, r.status()),
            }),
            Err(e) => {
                let s = e.to_string();
                if s.contains("connect") || s.contains("dns") || s.contains("resolve") {
                    Some(HeyGemMissingItem {
                        kind: HeyGemMissingKind::Environment,
                        description: format!("{} 服务未部署: {}", label, s),
                    })
                } else {
                    Some(HeyGemMissingItem {
                        kind: HeyGemMissingKind::Dependency,
                        description: format!("{} 服务不可达: {}", label, s),
                    })
                }
            }
        }
    }
}

impl HeyGemProvider {
    /// 委托 HeyGemHealthChecker 执行健康检查
    pub fn check_health(&self) -> HeyGemHealthReport {
        HeyGemHealthChecker::new(self.config.clone()).check()
    }
}
