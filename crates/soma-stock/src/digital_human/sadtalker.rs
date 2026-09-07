//! SadTalker 本地数字人口播视频生成适配器
//!
//! 通过子进程调用 Python 封装脚本（sadtalker_runner.py）执行 SadTalker 推理，
//! 输入人像照片 + 音频文件，输出口型同步视频。
//!
//! 与 HeyGen 的区别：本地推理（非 HTTP API）、内存任务状态表、Semaphore 串行控制。

use async_trait::async_trait;
use soma_core::config::SadTalkerConfig;
use soma_core::error::SomaError;
use soma_core::utils::validate_local_path;
use super::{DigitalHumanProvider, DhVideoGenParams, DhVideoGenStatus};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// 内部任务运行状态
#[derive(Debug, Clone)]
enum SadTalkerRunStatus {
    Processing,
    Success,
    Failed,
}

/// 内部任务状态
#[derive(Debug, Clone)]
struct SadTalkerTaskState {
    status: SadTalkerRunStatus,
    video_path: Option<String>,
    error: Option<String>,
    started_at: std::time::Instant,
    completed_at: Option<std::time::Instant>,
}

/// 环境检查缺失项类型
#[derive(Debug, Clone)]
enum MissingKind {
    Environment,
    Dependency,
    ModelWeight,
    Script,
}

/// 环境检查缺失项
#[derive(Debug, Clone)]
struct MissingItem {
    kind: MissingKind,
    description: String,
}

/// 环境健康检查报告
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub ready: bool,
    pub missing: Vec<MissingItem>,
}

/// SadTalker 环境检查器
pub struct SadTalkerEnvChecker {
    config: SadTalkerConfig,
}

impl SadTalkerEnvChecker {
    fn new(config: SadTalkerConfig) -> Self {
        Self { config }
    }

    /// 执行环境健康检查，返回报告
    pub fn check(&self) -> HealthReport {
        let mut missing = Vec::new();

        if let Some(item) = self.check_env_exists() {
            missing.push(item);
        }
        if let Some(item) = self.check_script_exists() {
            missing.push(item);
        }
        if let Some(item) = self.check_model_weights() {
            missing.push(item);
        }
        if let Some(item) = self.check_python_callable() {
            missing.push(item);
        }

        HealthReport {
            ready: missing.is_empty(),
            missing,
        }
    }

    fn check_env_exists(&self) -> Option<MissingItem> {
        let env_path = self.config.get_env_path();
        if env_path.is_empty() {
            return Some(MissingItem {
                kind: MissingKind::Environment,
                description: "运行环境目录未配置 (env_path)".into(),
            });
        }
        if !PathBuf::from(env_path).exists() {
            return Some(MissingItem {
                kind: MissingKind::Environment,
                description: format!("运行环境目录不存在: {}", env_path),
            });
        }
        None
    }

    fn check_script_exists(&self) -> Option<MissingItem> {
        let script_path = self.config.get_script_path();
        if !PathBuf::from(&script_path).exists() {
            return Some(MissingItem {
                kind: MissingKind::Script,
                description: format!("封装脚本不存在: {}", script_path),
            });
        }
        None
    }

    fn check_model_weights(&self) -> Option<MissingItem> {
        let model_path = self.config.get_model_path();
        if model_path.is_empty() {
            return Some(MissingItem {
                kind: MissingKind::ModelWeight,
                description: "模型权重目录未配置 (model_path)".into(),
            });
        }
        let path = PathBuf::from(model_path);
        if !path.exists() {
            return Some(MissingItem {
                kind: MissingKind::ModelWeight,
                description: format!("模型权重目录不存在: {}", model_path),
            });
        }
        if path.read_dir().ok()?.next().is_none() {
            return Some(MissingItem {
                kind: MissingKind::ModelWeight,
                description: format!("模型权重目录为空: {}", model_path),
            });
        }
        None
    }

    fn check_python_callable(&self) -> Option<MissingItem> {
        let python = self.config.get_python_path();
        let output = std::process::Command::new(&python)
            .arg("--version")
            .output();
        match output {
            Ok(o) if o.status.success() => None,
            Ok(_) => Some(MissingItem {
                kind: MissingKind::Dependency,
                description: format!("Python 解释器异常: {}", python),
            }),
            Err(_) => Some(MissingItem {
                kind: MissingKind::Dependency,
                description: format!("Python 解释器不可调用: {}", python),
            }),
        }
    }
}

/// SadTalker 数字人提供商
pub struct SadTalkerProvider {
    config: SadTalkerConfig,
    tasks: Arc<Mutex<HashMap<String, SadTalkerTaskState>>>,
    semaphore: Arc<Semaphore>,
    env_checker: SadTalkerEnvChecker,
}

impl SadTalkerProvider {
    /// 创建 SadTalker 提供商实例
    pub fn new(config: SadTalkerConfig) -> Self {
        let max_concurrent = config.get_max_concurrent().max(1) as usize;
        let env_checker = SadTalkerEnvChecker::new(config.clone());
        Self {
            config,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            env_checker,
        }
    }

    /// 环境健康检查
    pub fn check_health(&self) -> HealthReport {
        self.env_checker.check()
    }

    async fn insert_task(&self, task_id: &str) {
        let mut tasks = self.tasks.lock().await;
        if tasks.len() > 1000 {
            let oldest = tasks
                .iter()
                .filter(|(_, s)| !matches!(s.status, SadTalkerRunStatus::Processing))
                .min_by_key(|(_, s)| s.started_at)
                .map(|(k, _)| k.clone());
            if let Some(key) = oldest {
                tasks.remove(&key);
            }
        }
        tasks.insert(
            task_id.to_string(),
            SadTalkerTaskState {
                status: SadTalkerRunStatus::Processing,
                video_path: None,
                error: None,
                started_at: std::time::Instant::now(),
                completed_at: None,
            },
        );
    }

    async fn update_task_success(&self, task_id: &str, video_path: &str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(state) = tasks.get_mut(task_id) {
            state.status = SadTalkerRunStatus::Success;
            state.video_path = Some(video_path.to_string());
            state.completed_at = Some(std::time::Instant::now());
        }
    }

    async fn update_task_failed(&self, task_id: &str, error: &str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(state) = tasks.get_mut(task_id) {
            state.status = SadTalkerRunStatus::Failed;
            state.error = Some(error.to_string());
            state.completed_at = Some(std::time::Instant::now());
        }
    }

    /// 执行子进程推理
    async fn run_inference(
        &self,
        task_id: &str,
        params: &DhVideoGenParams,
        output_path: &str,
    ) -> Result<String, SomaError> {
        let python = self.config.get_python_path();
        let script = self.config.get_script_path();
        let model_path = self.config.get_model_path();
        let device = self.config.get_device();
        let env_path = self.config.get_env_path();

        let mut cmd = tokio::process::Command::new(&python);
        cmd.arg(&script)
            .arg("--portrait").arg(&params.portrait_path)
            .arg("--audio").arg(&params.audio_path)
            .arg("--outfile").arg(output_path)
            .arg("--checkpoint_dir").arg(model_path)
            .arg("--device").arg(device)
            .arg("--still").arg(if self.config.get_still_mode() { "true" } else { "false" })
            .arg("--full").arg(if self.config.get_full_enhancer() { "true" } else { "false" })
            .arg("--size").arg(self.config.get_size().to_string())
            .arg("--pose_style").arg(self.config.get_pose_style().to_string())
            .arg("--exp_scale").arg(self.config.get_exp_scale().to_string())
            .arg("--batch_size").arg(self.config.get_batch_size().to_string());

        if !env_path.is_empty() {
            cmd.current_dir(env_path);
            cmd.env("PYTHONPATH", env_path);
        }
        if device.contains("cuda") {
            cmd.env("CUDA_VISIBLE_DEVICES", "0");
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        log!("SadTalker 推理启动: task_id={}", task_id);

        let child = cmd
            .spawn()
            .map_err(|e| SomaError::VideoGen(format!("启动子进程失败: {}", e)))?;

        let timeout_dur = std::time::Duration::from_secs(self.config.get_timeout());
        let wait_result = tokio::time::timeout(timeout_dur, child.wait_with_output()).await;

        match wait_result {
            Ok(Ok(output)) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    return Err(SomaError::VideoGen(format!(
                        "SadTalker 推理失败 (退出码 {:?}): {}",
                        output.status.code(),
                        stderr
                    )));
                }
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let json: serde_json::Value = serde_json::from_str(&stdout)
                    .map_err(|e| SomaError::VideoGen(format!("解析推理输出失败: {} (stdout: {})", e, stdout)))?;

                let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("");
                if status == "success" {
                    let video_path = json
                        .get("video_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(output_path);
                    if !PathBuf::from(video_path).exists() {
                        return Err(SomaError::VideoGen(format!(
                            "推理输出文件不存在: {}", video_path
                        )));
                    }
                    log!("SadTalker 推理成功: task_id={}, video={}", task_id, video_path);
                    Ok(video_path.to_string())
                } else {
                    let error = json.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误");
                    Err(SomaError::VideoGen(format!("SadTalker 推理失败: {}", error)))
                }
            }
            Ok(Err(e)) => Err(SomaError::VideoGen(format!("等待子进程失败: {}", e))),
            Err(_) => Err(SomaError::VideoGen(format!(
                "SadTalker 推理超时（{}秒）", self.config.get_timeout()
            ))),
        }
    }
}

#[async_trait(?Send)]
impl DigitalHumanProvider for SadTalkerProvider {
    async fn create_task(&self, params: &DhVideoGenParams) -> Result<String, SomaError> {
        if self.config.get_preflight_check() {
            let health = self.env_checker.check();
            if !health.ready {
                let missing: Vec<String> = health
                    .missing
                    .iter()
                    .map(|m| m.description.clone())
                    .collect();
                return Err(SomaError::Config(format!(
                    "SadTalker 运行环境未就绪: {}",
                    missing.join("; ")
                )));
            }
        }

        validate_local_path(&params.portrait_path, true)
            .map_err(|e| SomaError::VideoGen(format!("人像照片校验失败: {:?}", e)))?;
        validate_local_path(&params.audio_path, true)
            .map_err(|e| SomaError::VideoGen(format!("配音音频校验失败: {:?}", e)))?;

        let task_id = soma_core::utils::get_uuid();
        self.insert_task(&task_id).await;

        let temp_dir = soma_core::utils::storage_dir("tasks", true)
            .join(&task_id);
        let _ = tokio::fs::create_dir_all(&temp_dir).await;
        let output_path = temp_dir.join("portrait_video.mp4");
        let output_str = output_path.to_string_lossy().to_string();

        let max_retries = 3;
        let mut last_error = String::new();

        for attempt in 1..=max_retries {
            log!("SadTalker 推理尝试 {}/{}: task_id={}", attempt, max_retries, task_id);

            let result = self
                .run_inference(&task_id, &DhVideoGenParams {
                    portrait_path: params.portrait_path.clone(),
                    audio_path: params.audio_path.clone(),
                    aspect_ratio: "auto".to_string(),
                    model: None,
                }, &output_str)
                .await;

            match result {
                Ok(video_path) => {
                    self.update_task_success(&task_id, &video_path).await;
                    log!("SadTalker 任务完成: task_id={}", task_id);
                    return Ok(task_id);
                }
                Err(e) => {
                    last_error = format!("{:?}", e);
                    log!("SadTalker 推理失败 (尝试 {}): task_id={}, error={}", attempt, task_id, last_error);
                }
            }
        }

        self.update_task_failed(&task_id, &last_error).await;
        log!("SadTalker 任务最终失败: task_id={}, error={}", task_id, last_error);

        Ok(task_id)
    }

    async fn query_task(&self, task_id: &str) -> Result<DhVideoGenStatus, SomaError> {
        let tasks = self.tasks.lock().await;
        match tasks.get(task_id) {
            Some(state) => match state.status {
                SadTalkerRunStatus::Processing => Ok(DhVideoGenStatus::Processing),
                SadTalkerRunStatus::Success => Ok(DhVideoGenStatus::Success {
                    video_url: state.video_path.clone().unwrap_or_default(),
                }),
                SadTalkerRunStatus::Failed => Ok(DhVideoGenStatus::Failed {
                    message: state.error.clone().unwrap_or_else(|| "未知错误".to_string()),
                }),
            },
            None => Err(SomaError::VideoGen(format!("任务不存在: {}", task_id))),
        }
    }

    async fn download_video(&self, url: &str, save_path: &str) -> Result<String, SomaError> {
        let src = validate_local_path(url, true)
            .map_err(|e| SomaError::VideoGen(format!("视频源文件校验失败: {:?}", e)))?;

        let dest = PathBuf::from(save_path);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| SomaError::VideoGen(format!("创建目录失败: {}", e)))?;
        }

        tokio::fs::copy(&src, &dest)
            .await
            .map_err(|e| SomaError::VideoGen(format!("拷贝视频文件失败: {}", e)))?;

        log!("SadTalker 视频已拷贝: {} -> {}", src.display(), save_path);
        Ok(save_path.to_string())
    }
}