//! Live2D 卡通口播视频生成适配器
//!
//! 通过子进程调用 Python 封装脚本（live2d_runner.py）执行 Live2D 模型渲染，
//! 输入音频文件 + Live2D 模型目录，输出口型同步视频。
//!
//!
//! 关键约束：create_task 必须同步执行（不可 tokio::spawn，因 block_on_async 创建临时 runtime）。
//! Live2D 路径下 DhVideoGenParams.portrait_path 复用为模型目录路径。

use async_trait::async_trait;
use soma_core::config::Live2DConfig;
use soma_core::error::SomaError;
use soma_core::utils::validate_local_path;
use super::{DigitalHumanProvider, DhVideoGenParams, DhVideoGenStatus};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// 内部任务运行状态
#[derive(Debug, Clone)]
enum Live2DRunStatus {
    Processing,
    Success,
    Failed,
}

/// 内部任务状态
#[derive(Debug, Clone)]
struct Live2DTaskState {
    status: Live2DRunStatus,
    video_path: Option<String>,
    error: Option<String>,
    started_at: std::time::Instant,
    completed_at: Option<std::time::Instant>,
}

/// 环境检查缺失项类型
#[derive(Debug, Clone)]
pub enum Live2DMissingKind {
    Environment,
    Dependency,
    Script,
    PythonVersion,
    Ffmpeg,
    Live2DModule,
    PhonemizerModule,
}

/// 环境检查缺失项
#[derive(Debug, Clone)]
pub struct Live2DMissingItem {
    pub kind: Live2DMissingKind,
    pub description: String,
}

/// 环境健康检查报告
#[derive(Debug, Clone)]
pub struct Live2DHealthReport {
    pub ready: bool,
    pub missing: Vec<Live2DMissingItem>,
}

/// Live2D 环境检查器
pub struct Live2DEnvChecker {
    config: Live2DConfig,
}

impl Live2DEnvChecker {
    pub fn new(config: Live2DConfig) -> Self {
        Self { config }
    }

    /// 执行环境健康检查，返回报告
    pub fn check(&self) -> Live2DHealthReport {
        let mut missing = Vec::new();

        if let Some(item) = self.check_python_callable() {
            missing.push(item);
        }
        if let Some(item) = self.check_live2d_module() {
            missing.push(item);
        }
        if let Some(item) = self.check_phonemizer_module() {
            missing.push(item);
        }
        if let Some(item) = self.check_script_exists() {
            missing.push(item);
        }
        if let Some(item) = self.check_ffmpeg_available() {
            missing.push(item);
        }

        Live2DHealthReport {
            ready: missing.is_empty(),
            missing,
        }
    }

    fn check_python_callable(&self) -> Option<Live2DMissingItem> {
        let python = self.config.get_python_path();
        let output = std::process::Command::new(python)
            .arg("--version")
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).to_string();
                let parts: Vec<&str> = version.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let ver = parts[1];
                    let nums: Vec<&str> = ver.split('.').collect();
                    if nums.len() >= 2 {
                        if let (Ok(major), Ok(minor)) =
                            (nums[0].parse::<u32>(), nums[1].parse::<u32>())
                        {
                            if major < 3 || (major == 3 && minor < 8) {
                                return Some(Live2DMissingItem {
                                    kind: Live2DMissingKind::PythonVersion,
                                    description: format!(
                                        "Python 版本过低，需要 3.8+，当前: {}",
                                        version.trim()
                                    ),
                                });
                            }
                        }
                    }
                }
                None
            }
            Ok(_) => Some(Live2DMissingItem {
                kind: Live2DMissingKind::Environment,
                description: format!("Python 解释器异常: {}", python),
            }),
            Err(_) => Some(Live2DMissingItem {
                kind: Live2DMissingKind::Environment,
                description: format!("Python 解释器不可调用: {}", python),
            }),
        }
    }

    fn check_live2d_module(&self) -> Option<Live2DMissingItem> {
        let python = self.config.get_python_path();
        let output = std::process::Command::new(python)
            .arg("-c")
            .arg("import live2d")
            .output();
        match output {
            Ok(o) if o.status.success() => None,
            _ => Some(Live2DMissingItem {
                kind: Live2DMissingKind::Live2DModule,
                description: "live2d-py 未安装，请执行 pip install live2d-py".into(),
            }),
        }
    }

    fn check_phonemizer_module(&self) -> Option<Live2DMissingItem> {
        let python = self.config.get_python_path();
        let output = std::process::Command::new(python)
            .arg("-c")
            .arg("try:\n    import phonemizer\nexcept Exception:\n    import aeneas")
            .output();
        match output {
            Ok(o) if o.status.success() => None,
            _ => Some(Live2DMissingItem {
                kind: Live2DMissingKind::PhonemizerModule,
                description: "音素提取库未安装，请执行 pip install phonemizer 或 aeneas".into(),
            }),
        }
    }

    fn check_script_exists(&self) -> Option<Live2DMissingItem> {
        let script_path = self.config.get_script_path();
        if !PathBuf::from(script_path).exists() {
            return Some(Live2DMissingItem {
                kind: Live2DMissingKind::Script,
                description: format!("渲染脚本不存在: {}", script_path),
            });
        }
        None
    }

    fn check_ffmpeg_available(&self) -> Option<Live2DMissingItem> {
        let output = std::process::Command::new("ffmpeg")
            .arg("-version")
            .output();
        match output {
            Ok(o) if o.status.success() => None,
            _ => Some(Live2DMissingItem {
                kind: Live2DMissingKind::Ffmpeg,
                description: "FFmpeg 未安装或不在 PATH 中".into(),
            }),
        }
    }
}

/// Live2D 数字人提供商
pub struct Live2DProvider {
    config: Live2DConfig,
    tasks: Arc<Mutex<HashMap<String, Live2DTaskState>>>,
    semaphore: Arc<Semaphore>,
    env_checker: Live2DEnvChecker,
}

impl Live2DProvider {
    /// 创建 Live2D 提供商实例
    pub fn new(config: Live2DConfig) -> Self {
        let max_concurrent = 1usize;
        let env_checker = Live2DEnvChecker::new(config.clone());
        Self {
            config,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            env_checker,
        }
    }

    /// 环境健康检查
    pub fn check_health(&self) -> Live2DHealthReport {
        self.env_checker.check()
    }

    async fn insert_task(&self, task_id: &str) {
        let mut tasks = self.tasks.lock().await;
        if tasks.len() > 1000 {
            let oldest = tasks
                .iter()
                .filter(|(_, s)| !matches!(s.status, Live2DRunStatus::Processing))
                .min_by_key(|(_, s)| s.started_at)
                .map(|(k, _)| k.clone());
            if let Some(key) = oldest {
                tasks.remove(&key);
            }
        }
        tasks.insert(
            task_id.to_string(),
            Live2DTaskState {
                status: Live2DRunStatus::Processing,
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
            state.status = Live2DRunStatus::Success;
            state.video_path = Some(video_path.to_string());
            state.completed_at = Some(std::time::Instant::now());
        }
    }

    async fn update_task_failed(&self, task_id: &str, error: &str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(state) = tasks.get_mut(task_id) {
            state.status = Live2DRunStatus::Failed;
            state.error = Some(error.to_string());
            state.completed_at = Some(std::time::Instant::now());
        }
    }

    /// 执行子进程渲染
    async fn run_render(
        &self,
        task_id: &str,
        params: &DhVideoGenParams,
        output_path: &str,
    ) -> Result<String, SomaError> {
        let python = self.config.get_python_path();
        let script = self.config.get_script_path();
        let fps = self.config.get_fps();
        let width = self.config.get_width();
        let height = self.config.get_height();
        let render_threads = self.config.get_render_threads();

        let mut cmd = tokio::process::Command::new(python);
        cmd.arg(script)
            .arg("--audio").arg(&params.audio_path)
            .arg("--model_dir").arg(&params.portrait_path)
            .arg("--outfile").arg(output_path)
            .arg("--fps").arg(fps.to_string())
            .arg("--width").arg(width.to_string())
            .arg("--height").arg(height.to_string())
            .arg("--render_threads").arg(render_threads.to_string());

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        log::info!("Live2D 渲染启动: task_id={}", task_id);

        let child = cmd
            .spawn()
            .map_err(|e| SomaError::VideoGen(format!("渲染脚本启动失败: {}", e)))?;

        let timeout_dur = std::time::Duration::from_secs(self.config.get_timeout());
        let wait_result = tokio::time::timeout(timeout_dur, child.wait_with_output()).await;

        match wait_result {
            Ok(Ok(output)) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    return Err(SomaError::VideoGen(format!(
                        "Live2D 渲染失败 (退出码 {:?}): {}",
                        output.status.code(),
                        stderr
                    )));
                }
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let json: serde_json::Value = serde_json::from_str(&stdout)
                    .map_err(|e| SomaError::VideoGen(format!(
                        "解析渲染输出失败: {} (stdout: {})", e, stdout
                    )))?;

                let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("");
                if status == "success" {
                    let video_path = json
                        .get("video_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(output_path);
                    if !PathBuf::from(video_path).exists() {
                        return Err(SomaError::VideoGen(format!(
                            "渲染输出文件不存在: {}", video_path
                        )));
                    }
                    log::info!("Live2D 渲染成功: task_id={}, video={}", task_id, video_path);
                    Ok(video_path.to_string())
                } else {
                    let error = json
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误");
                    Err(SomaError::VideoGen(format!("Live2D 渲染失败: {}", error)))
                }
            }
            Ok(Err(e)) => Err(SomaError::VideoGen(format!("等待子进程失败: {}", e))),
            Err(_) => Err(SomaError::VideoGen(format!(
                "Live2D 渲染超时（{}秒）", self.config.get_timeout()
            ))),
        }
    }
}

#[async_trait(?Send)]
impl DigitalHumanProvider for Live2DProvider {
    async fn create_task(&self, params: &DhVideoGenParams) -> Result<String, SomaError> {
        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| SomaError::VideoGen(format!("获取并发许可失败: {}", e)))?;

        if self.config.get_preflight_check() {
            let health = self.env_checker.check();
            if !health.ready {
                let missing: Vec<String> = health
                    .missing
                    .iter()
                    .map(|m| m.description.clone())
                    .collect();
                return Err(SomaError::Config(format!(
                    "Live2D 渲染环境未就绪: {}",
                    missing.join("; ")
                )));
            }
        }

        validate_local_path(&params.portrait_path, true)
            .map_err(|e| SomaError::VideoGen(format!("模型目录校验失败: {:?}", e)))?;
        validate_local_path(&params.audio_path, true)
            .map_err(|e| SomaError::VideoGen(format!("配音音频校验失败: {:?}", e)))?;

        let task_id = soma_core::utils::get_uuid();
        self.insert_task(&task_id).await;

        let temp_dir = soma_core::utils::storage_dir("tasks", true)
            .join(&task_id);
        let _ = tokio::fs::create_dir_all(&temp_dir).await;
        let output_path = temp_dir.join("portrait_video.mp4");
        let output_str = output_path.to_string_lossy().to_string();

        let max_retries = self.config.get_max_retries();
        let mut last_error = String::new();

        for attempt in 1..=max_retries {
            log::info!(
                "Live2D 渲染尝试 {}/{}: task_id={}",
                attempt, max_retries, task_id
            );

            let result = self
                .run_render(&task_id, params, &output_str)
                .await;

            match result {
                Ok(video_path) => {
                    self.update_task_success(&task_id, &video_path).await;
                    log::info!("Live2D 任务完成: task_id={}", task_id);
                    return Ok(task_id);
                }
                Err(e) => {
                    last_error = format!("{:?}", e);
                    log::warn!(
                        "Live2D 渲染失败 (尝试 {}): task_id={}, error={}",
                        attempt, task_id, last_error
                    );
                    if attempt < max_retries {
                        let backoff = 1u64 << (attempt - 1);
                        tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                    }
                }
            }
        }

        self.update_task_failed(&task_id, &last_error).await;
        log::error!(
            "Live2D 任务最终失败: task_id={}, error={}",
            task_id, last_error
        );

        Ok(task_id)
    }

    async fn query_task(&self, task_id: &str) -> Result<DhVideoGenStatus, SomaError> {
        let tasks = self.tasks.lock().await;
        match tasks.get(task_id) {
            Some(state) => match state.status {
                Live2DRunStatus::Processing => Ok(DhVideoGenStatus::Processing),
                Live2DRunStatus::Success => Ok(DhVideoGenStatus::Success {
                    video_url: state.video_path.clone().unwrap_or_default(),
                }),
                Live2DRunStatus::Failed => Ok(DhVideoGenStatus::Failed {
                    message: state.error.clone().unwrap_or_else(|| "未知错误".to_string()),
                }),
            },
            None => Ok(DhVideoGenStatus::Failed {
                message: "任务不存在".to_string(),
            }),
        }
    }

    async fn download_video(&self, url: &str, save_path: &str) -> Result<String, SomaError> {
        let src = validate_local_path(url, true)
            .map_err(|e| SomaError::VideoGen(format!("视频源文件校验失败: {:?}", e)))?;

        let dest = PathBuf::from(save_path);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| SomaError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("创建目录失败: {}", e),
                )))?;
        }

        tokio::fs::copy(&src, &dest)
            .await
            .map_err(|e| SomaError::Io(e))?;

        log::info!("Live2D 视频已拷贝: {} -> {}", src.display(), save_path);
        Ok(save_path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_new_default_config() {
        let config = Live2DConfig::default();
        let provider = Live2DProvider::new(config);
        assert_eq!(provider.config.get_fps(), 30);
        assert_eq!(provider.config.get_width(), 1080);
        assert_eq!(provider.config.get_height(), 1920);
    }

    #[test]
    fn test_env_checker_default_config() {
        let config = Live2DConfig::default();
        let checker = Live2DEnvChecker::new(config);
        let report = checker.check();
        assert!(!report.ready);
    }

    #[tokio::test]
    async fn test_query_task_nonexistent() {
        let config = Live2DConfig::default();
        let provider = Live2DProvider::new(config);
        let result = provider.query_task("nonexistent-task-id").await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Failed { message } => {
                assert!(message.contains("任务不存在"));
            }
            _ => panic!("期望 Failed 状态"),
        }
    }

    #[tokio::test]
    async fn test_insert_and_query_task_processing() {
        let config = Live2DConfig::default();
        let provider = Live2DProvider::new(config);
        provider.insert_task("test-task-1").await;
        let result = provider.query_task("test-task-1").await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Processing => {}
            _ => panic!("期望 Processing 状态"),
        }
    }

    #[tokio::test]
    async fn test_insert_update_and_query_task_success() {
        let config = Live2DConfig::default();
        let provider = Live2DProvider::new(config);
        provider.insert_task("test-task-2").await;
        provider.update_task_success("test-task-2", "/tmp/video.mp4").await;
        let result = provider.query_task("test-task-2").await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Success { video_url } => {
                assert_eq!(video_url, "/tmp/video.mp4");
            }
            _ => panic!("期望 Success 状态"),
        }
    }

    #[tokio::test]
    async fn test_insert_update_and_query_task_failed() {
        let config = Live2DConfig::default();
        let provider = Live2DProvider::new(config);
        provider.insert_task("test-task-3").await;
        provider.update_task_failed("test-task-3", "渲染失败").await;
        let result = provider.query_task("test-task-3").await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Failed { message } => {
                assert_eq!(message, "渲染失败");
            }
            _ => panic!("期望 Failed 状态"),
        }
    }
}