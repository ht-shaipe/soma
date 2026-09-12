//! EchoMimicV3-Flash 本地数字人口播视频生成适配器
//!
//! 通过子进程调用 Python 封装脚本（echomimic_v3_runner.py）执行 EchoMimicV3-Flash 推理，
//! 输入人像照片 + 音频文件，输出口型同步视频。
//!
//! 与 SadTalker 的区别：GPU 强制推理、8 步 Flash 快速生成、视频扩散模型、768×768 分辨率。
//! 关键约束：create_task 必须同步执行（不可 tokio::spawn，因 block_on_async 创建临时 runtime）。

use async_trait::async_trait;
use soma_core::config::EchoMimicV3Config;
use soma_core::error::SomaError;
use soma_core::utils::validate_local_path;
use super::{DigitalHumanProvider, DhVideoGenParams, DhVideoGenStatus};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// 内部任务运行状态
#[derive(Debug, Clone)]
enum EchoMimicV3RunStatus {
    Processing,
    Success,
    Failed,
}

/// 内部任务状态
#[derive(Debug, Clone)]
struct EchoMimicV3TaskState {
    status: EchoMimicV3RunStatus,
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
    Gpu,
    Cuda,
    PythonVersion,
    GpuMemory,
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

/// EchoMimicV3-Flash 环境检查器
pub struct EchoMimicV3EnvChecker {
    config: EchoMimicV3Config,
}

impl EchoMimicV3EnvChecker {
    fn new(config: EchoMimicV3Config) -> Self {
        Self { config }
    }

    /// 执行环境健康检查，返回报告
    pub fn check(&self) -> HealthReport {
        let mut missing = Vec::new();

        if let Some(item) = self.check_env_exists() {
            missing.push(item);
        }
        if let Some(item) = self.check_python_callable() {
            missing.push(item);
        }
        if let Some(item) = self.check_script_exists() {
            missing.push(item);
        }
        if let Some(item) = self.check_model_weights() {
            missing.push(item);
        }
        if let Some(item) = self.check_gpu_available() {
            missing.push(item);
        }
        if let Some(item) = self.check_cuda_version() {
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

    fn check_python_callable(&self) -> Option<MissingItem> {
        let python = self.config.get_python_path();
        let output = std::process::Command::new(&python)
            .arg("--version")
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).to_string();
                if !version.contains("3.10") && !version.contains("3.11") {
                    return Some(MissingItem {
                        kind: MissingKind::PythonVersion,
                        description: format!("Python 版本需 3.10/3.11，当前: {}", version.trim()),
                    });
                }
                None
            }
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
        let flash_dir = PathBuf::from(model_path).join("flash");
        if !flash_dir.exists() {
            return Some(MissingItem {
                kind: MissingKind::ModelWeight,
                description: format!("模型权重目录不存在: {}/flash", model_path),
            });
        }

        let required_files = [
            ("Wan2.1-Fun-V1.1-1.3B-InP", false),
            ("chinese-wav2vec2-base", false),
            ("transformer/diffusion_pytorch_model.safetensors", false),
        ];

        let mut missing_files = Vec::new();
        for (file, is_file) in &required_files {
            let path = flash_dir.join(file);
            let exists = if *is_file { path.is_file() } else { path.exists() };
            if !exists {
                missing_files.push(file.to_string());
            }
        }

        if !missing_files.is_empty() {
            return Some(MissingItem {
                kind: MissingKind::ModelWeight,
                description: format!("模型权重文件缺失: {}", missing_files.join(", ")),
            });
        }
        None
    }

    fn check_gpu_available(&self) -> Option<MissingItem> {
        let python = self.config.get_python_path();
        let output = std::process::Command::new(&python)
            .arg("-c")
            .arg("import torch; print(torch.cuda.is_available()); print(torch.cuda.get_device_properties(0).total_memory if torch.cuda.is_available() else 0)")
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let lines: Vec<&str> = stdout.lines().collect();
                if lines.is_empty() || lines[0].trim() != "True" {
                    return Some(MissingItem {
                        kind: MissingKind::Gpu,
                        description: "GPU 不可用 (torch.cuda.is_available() 返回 False)".into(),
                    });
                }
                if lines.len() >= 2 {
                    if let Ok(total_memory) = lines[1].trim().parse::<u64>() {
                        let total_gb = total_memory / (1024 * 1024 * 1024);
                        let min_gb = self.config.get_gpu_memory_limit() as u64;
                        if total_gb < 12 {
                            return Some(MissingItem {
                                kind: MissingKind::GpuMemory,
                                description: format!("GPU 显存不足: {}GB (要求 ≥12GB)", total_gb),
                            });
                        }
                        if min_gb > 0 && total_gb < min_gb {
                            return Some(MissingItem {
                                kind: MissingKind::GpuMemory,
                                description: format!("GPU 显存不足: {}GB (配置要求 ≥{}GB)", total_gb, min_gb),
                            });
                        }
                    }
                }
                None
            }
            Ok(_) => Some(MissingItem {
                kind: MissingKind::Gpu,
                description: "GPU 检查失败 (无法执行 torch.cuda 查询)".into(),
            }),
            Err(_) => Some(MissingItem {
                kind: MissingKind::Gpu,
                description: "GPU 检查失败 (Python 子进程启动失败)".into(),
            }),
        }
    }

    fn check_cuda_version(&self) -> Option<MissingItem> {
        let python = self.config.get_python_path();
        let output = std::process::Command::new(&python)
            .arg("-c")
            .arg("import torch; print(torch.version.cuda)")
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if version == "None" || version.is_empty() {
                    return Some(MissingItem {
                        kind: MissingKind::Cuda,
                        description: "CUDA 不可用 (torch.version.cuda 为 None)".into(),
                    });
                }
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() >= 2 {
                    if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                        if major < 12 || (major == 12 && minor < 1) {
                            return Some(MissingItem {
                                kind: MissingKind::Cuda,
                                description: format!("CUDA 版本过低: {} (要求 ≥12.1)", version),
                            });
                        }
                    }
                }
                None
            }
            Ok(_) => Some(MissingItem {
                kind: MissingKind::Cuda,
                description: "CUDA 版本检查失败".into(),
            }),
            Err(_) => Some(MissingItem {
                kind: MissingKind::Cuda,
                description: "CUDA 版本检查失败 (Python 子进程启动失败)".into(),
            }),
        }
    }
}

/// EchoMimicV3-Flash 数字人提供商
pub struct EchoMimicV3Provider {
    config: EchoMimicV3Config,
    tasks: Arc<Mutex<HashMap<String, EchoMimicV3TaskState>>>,
    semaphore: Arc<Semaphore>,
    env_checker: EchoMimicV3EnvChecker,
}

impl EchoMimicV3Provider {
    /// 创建 EchoMimicV3-Flash 提供商实例
    pub fn new(config: EchoMimicV3Config) -> Self {
        let max_concurrent = config.get_max_concurrent().max(1) as usize;
        let env_checker = EchoMimicV3EnvChecker::new(config.clone());
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
                .filter(|(_, s)| !matches!(s.status, EchoMimicV3RunStatus::Processing))
                .min_by_key(|(_, s)| s.started_at)
                .map(|(k, _)| k.clone());
            if let Some(key) = oldest {
                tasks.remove(&key);
            }
        }
        tasks.insert(
            task_id.to_string(),
            EchoMimicV3TaskState {
                status: EchoMimicV3RunStatus::Processing,
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
            state.status = EchoMimicV3RunStatus::Success;
            state.video_path = Some(video_path.to_string());
            state.completed_at = Some(std::time::Instant::now());
        }
    }

    async fn update_task_failed(&self, task_id: &str, error: &str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(state) = tasks.get_mut(task_id) {
            state.status = EchoMimicV3RunStatus::Failed;
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
        let resolution = self.config.get_resolution();
        let infer_steps = self.config.get_infer_steps();
        let config_path = self.config.get_config_path();

        let mut cmd = tokio::process::Command::new(&python);
        cmd.arg(&script)
            .arg("--portrait").arg(&params.portrait_path)
            .arg("--audio").arg(&params.audio_path)
            .arg("--outfile").arg(output_path)
            .arg("--model_dir").arg(model_path)
            .arg("--device").arg(device)
            .arg("--resolution").arg(resolution.to_string())
            .arg("--infer_steps").arg(infer_steps.to_string())
            .arg("--video_length").arg("0")
            .arg("--config_path").arg(&config_path);

        if !env_path.is_empty() {
            cmd.current_dir(env_path);
            cmd.env("PYTHONPATH", env_path);
        }
        if device.starts_with("cuda:") {
            if let Some(gpu_id) = device.strip_prefix("cuda:") {
                cmd.env("CUDA_VISIBLE_DEVICES", gpu_id);
            }
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        log!("EchoMimicV3 推理启动: task_id={}", task_id);

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
                        "EchoMimicV3 推理失败 (退出码 {:?}): {}",
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
                    log!("EchoMimicV3 推理成功: task_id={}, video={}", task_id, video_path);
                    Ok(video_path.to_string())
                } else {
                    let error = json.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误");
                    Err(SomaError::VideoGen(format!("EchoMimicV3 推理失败: {}", error)))
                }
            }
            Ok(Err(e)) => Err(SomaError::VideoGen(format!("等待子进程失败: {}", e))),
            Err(_) => Err(SomaError::VideoGen(format!(
                "EchoMimicV3 推理超时（{}秒）", self.config.get_timeout()
            ))),
        }
    }
}

#[async_trait(?Send)]
impl DigitalHumanProvider for EchoMimicV3Provider {
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
                    "EchoMimicV3-Flash 运行环境未就绪: {}",
                    missing.join("; ")
                )));
            }
        }

        let device = self.config.get_device();
        if !device.starts_with("cuda") {
            return Err(SomaError::Config(
                "EchoMimicV3-Flash 仅支持 GPU 推理，请配置 device 为 cuda".into(),
            ));
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

        let max_retries = self.config.get_max_retries();
        let mut last_error = String::new();

        for attempt in 1..=max_retries {
            log!("EchoMimicV3 推理尝试 {}/{}: task_id={}", attempt, max_retries, task_id);

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
                    log!("EchoMimicV3 任务完成: task_id={}", task_id);
                    return Ok(task_id);
                }
                Err(e) => {
                    last_error = format!("{:?}", e);
                    log!("EchoMimicV3 推理失败 (尝试 {}): task_id={}, error={}", attempt, task_id, last_error);
                }
            }
        }

        self.update_task_failed(&task_id, &last_error).await;
        log!("EchoMimicV3 任务最终失败: task_id={}, error={}", task_id, last_error);

        Ok(task_id)
    }

    async fn query_task(&self, task_id: &str) -> Result<DhVideoGenStatus, SomaError> {
        let tasks = self.tasks.lock().await;
        match tasks.get(task_id) {
            Some(state) => match state.status {
                EchoMimicV3RunStatus::Processing => Ok(DhVideoGenStatus::Processing),
                EchoMimicV3RunStatus::Success => Ok(DhVideoGenStatus::Success {
                    video_url: state.video_path.clone().unwrap_or_default(),
                }),
                EchoMimicV3RunStatus::Failed => Ok(DhVideoGenStatus::Failed {
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

        log!("EchoMimicV3 视频已拷贝: {} -> {}", src.display(), save_path);
        Ok(save_path.to_string())
    }
}

/// 下载状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Idle,
    Downloading,
    Success,
    Failed,
}

/// 下载进度
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub status: DownloadStatus,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed: f64,
    pub error: Option<String>,
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            status: DownloadStatus::Idle,
            downloaded_bytes: 0,
            total_bytes: 0,
            speed: 0.0,
            error: None,
        }
    }
}

/// 下载结果
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub success: bool,
    pub total_bytes: u64,
    pub error: Option<String>,
}

/// EchoMimicV3-Flash 模型权重下载器
pub struct EchoMimicV3ModelDownloader {
    config: EchoMimicV3Config,
    progress: Arc<Mutex<DownloadProgress>>,
}

impl EchoMimicV3ModelDownloader {
    /// 创建下载器实例
    pub fn new(config: EchoMimicV3Config) -> Self {
        Self {
            config,
            progress: Arc::new(Mutex::new(DownloadProgress::default())),
        }
    }

    /// 查询当前下载进度（纯内存查询，无 I/O）
    pub async fn query_progress(&self) -> DownloadProgress {
        self.progress.lock().await.clone()
    }

    /// 检查本地模型是否已完整
    pub fn check_model_complete(&self) -> bool {
        let model_path = self.config.get_model_path();
        if model_path.is_empty() {
            return false;
        }
        let flash_dir = PathBuf::from(model_path).join("flash");
        if !flash_dir.exists() {
            return false;
        }
        let required = [
            "Wan2.1-Fun-V1.1-1.3B-InP",
            "chinese-wav2vec2-base",
            "transformer/diffusion_pytorch_model.safetensors",
        ];
        for rel in &required {
            let path = flash_dir.join(rel);
            if !path.exists() {
                return false;
            }
            if path.is_file() && path.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
                return false;
            }
        }
        true
    }

    /// 下载模型权重
    pub async fn download_model(&self) -> Result<DownloadResult, SomaError> {
        if self.check_model_complete() {
            return Ok(DownloadResult {
                success: true,
                total_bytes: 0,
                error: None,
            });
        }

        let source = self.config.get_model_source();
        if source != "modelscope" && source != "huggingface" {
            return Err(SomaError::Config(format!(
                "模型源地址不在允许范围内: {}（仅支持 modelscope / huggingface）",
                source
            )));
        }

        let model_path = self.config.get_model_path();
        if model_path.is_empty() {
            return Err(SomaError::Config(
                "模型权重目录未配置 (model_path)".into(),
            ));
        }

        {
            let mut progress = self.progress.lock().await;
            progress.status = DownloadStatus::Downloading;
            progress.downloaded_bytes = 0;
            progress.total_bytes = 0;
            progress.speed = 0.0;
            progress.error = None;
        }

        let python = self.config.get_python_path();
        let env_path = self.config.get_env_path();
        let downloader_script = if env_path.is_empty() {
            "echomimic_v3_downloader.py".to_string()
        } else {
            format!("{}/echomimic_v3_downloader.py", env_path)
        };

        let mut cmd = tokio::process::Command::new(&python);
        cmd.arg(&downloader_script)
            .arg("--model_source").arg(source)
            .arg("--target_dir").arg(model_path)
            .arg("--repo_id").arg("BadToBest/EchoMimicV3");

        if !env_path.is_empty() {
            cmd.current_dir(env_path);
            cmd.env("PYTHONPATH", env_path);
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        log!("EchoMimicV3 模型下载启动: source={}", source);

        let child = cmd
            .spawn()
            .map_err(|e| SomaError::VideoGen(format!("启动下载子进程失败: {}", e)))?;

        let timeout_dur = std::time::Duration::from_secs(300);
        let wait_result = tokio::time::timeout(timeout_dur, child.wait_with_output()).await;

        match wait_result {
            Ok(Ok(output)) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let mut progress = self.progress.lock().await;
                    progress.status = DownloadStatus::Failed;
                    progress.error = Some(stderr.clone());
                    return Ok(DownloadResult {
                        success: false,
                        total_bytes: 0,
                        error: Some(stderr),
                    });
                }

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let json: serde_json::Value = serde_json::from_str(&stdout)
                    .map_err(|e| SomaError::VideoGen(format!("解析下载输出失败: {}", e)))?;

                let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("");
                if status == "success" {
                    let total_bytes = json
                        .get("total_bytes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let mut progress = self.progress.lock().await;
                    progress.status = DownloadStatus::Success;
                    progress.total_bytes = total_bytes;
                    log!("EchoMimicV3 模型下载完成: {} bytes", total_bytes);
                    Ok(DownloadResult {
                        success: true,
                        total_bytes,
                        error: None,
                    })
                } else {
                    let error = json
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误")
                        .to_string();
                    let mut progress = self.progress.lock().await;
                    progress.status = DownloadStatus::Failed;
                    progress.error = Some(error.clone());
                    Ok(DownloadResult {
                        success: false,
                        total_bytes: 0,
                        error: Some(error),
                    })
                }
            }
            Ok(Err(e)) => {
                let mut progress = self.progress.lock().await;
                progress.status = DownloadStatus::Failed;
                progress.error = Some(format!("{}", e));
                Err(SomaError::VideoGen(format!("等待下载子进程失败: {}", e)))
            }
            Err(_) => {
                let mut progress = self.progress.lock().await;
                progress.status = DownloadStatus::Failed;
                progress.error = Some("下载超时".into());
                Err(SomaError::VideoGen("EchoMimicV3 模型下载超时（300秒）".into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_new_default_config() {
        let config = EchoMimicV3Config::default();
        let provider = EchoMimicV3Provider::new(config);
        assert_eq!(provider.config.get_device(), "cuda");
        assert_eq!(provider.config.get_resolution(), 768);
        assert_eq!(provider.config.get_infer_steps(), 8);
    }

    #[test]
    fn test_provider_new_custom_config() {
        let config = EchoMimicV3Config {
            env_path: Some("/env".to_string()),
            model_path: Some("/models".to_string()),
            device: Some("cuda:1".to_string()),
            resolution: Some(512),
            infer_steps: Some(4),
            max_concurrent: Some(3),
            ..Default::default()
        };
        let provider = EchoMimicV3Provider::new(config);
        assert_eq!(provider.config.get_device(), "cuda:1");
        assert_eq!(provider.config.get_resolution(), 512);
        assert_eq!(provider.config.get_infer_steps(), 4);
    }

    #[tokio::test]
    async fn test_query_task_nonexistent() {
        let config = EchoMimicV3Config::default();
        let provider = EchoMimicV3Provider::new(config);
        let result = provider.query_task("nonexistent-task-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::VideoGen(msg) => assert!(msg.contains("任务不存在")),
            _ => panic!("期望 SomaError::VideoGen 错误"),
        }
    }

    #[tokio::test]
    async fn test_insert_and_query_task_processing() {
        let config = EchoMimicV3Config::default();
        let provider = EchoMimicV3Provider::new(config);
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
        let config = EchoMimicV3Config::default();
        let provider = EchoMimicV3Provider::new(config);
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
        let config = EchoMimicV3Config::default();
        let provider = EchoMimicV3Provider::new(config);
        provider.insert_task("test-task-3").await;
        provider.update_task_failed("test-task-3", "推理失败").await;
        let result = provider.query_task("test-task-3").await;
        assert!(result.is_ok());
        match result.unwrap() {
            DhVideoGenStatus::Failed { message } => {
                assert_eq!(message, "推理失败");
            }
            _ => panic!("期望 Failed 状态"),
        }
    }

    #[test]
    fn test_env_checker_empty_config() {
        let config = EchoMimicV3Config::default();
        let checker = EchoMimicV3EnvChecker::new(config);
        let report = checker.check();
        assert!(!report.ready);
        assert!(!report.missing.is_empty());
        let has_env_issue = report.missing.iter().any(|m| m.description.contains("env_path"));
        assert!(has_env_issue);
    }

    #[test]
    fn test_env_checker_missing_model_weights() {
        let config = EchoMimicV3Config {
            env_path: Some("/nonexistent/env".to_string()),
            model_path: Some("/nonexistent/models".to_string()),
            ..Default::default()
        };
        let checker = EchoMimicV3EnvChecker::new(config);
        let report = checker.check();
        assert!(!report.ready);
        let has_model_issue = report.missing.iter().any(|m| m.description.contains("模型权重"));
        assert!(has_model_issue);
    }

    #[tokio::test]
    async fn test_create_task_gpu_only_rejects_cpu() {
        let config = EchoMimicV3Config {
            env_path: Some("/env".to_string()),
            model_path: Some("/models".to_string()),
            device: Some("cpu".to_string()),
            preflight_check: Some(false),
            ..Default::default()
        };
        let provider = EchoMimicV3Provider::new(config);
        let params = DhVideoGenParams {
            portrait_path: "/tmp/portrait.jpg".to_string(),
            audio_path: "/tmp/audio.wav".to_string(),
            aspect_ratio: "1:1".to_string(),
            model: None,
        };
        let result = provider.create_task(&params).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::Config(msg) => assert!(msg.contains("GPU")),
            _ => panic!("期望 SomaError::Config 错误"),
        }
    }

    #[tokio::test]
    async fn test_downloader_query_progress_initial_idle() {
        let config = EchoMimicV3Config::default();
        let downloader = EchoMimicV3ModelDownloader::new(config);
        let progress = downloader.query_progress().await;
        assert_eq!(progress.status, DownloadStatus::Idle);
    }

    #[test]
    fn test_downloader_check_model_complete_empty_path() {
        let config = EchoMimicV3Config::default();
        let downloader = EchoMimicV3ModelDownloader::new(config);
        assert!(!downloader.check_model_complete());
    }

    #[test]
    fn test_downloader_check_model_complete_nonexistent_path() {
        let config = EchoMimicV3Config {
            model_path: Some("/nonexistent/path/models".to_string()),
            ..Default::default()
        };
        let downloader = EchoMimicV3ModelDownloader::new(config);
        assert!(!downloader.check_model_complete());
    }

    #[tokio::test]
    async fn test_downloader_invalid_model_source() {
        let config = EchoMimicV3Config {
            env_path: Some("/env".to_string()),
            model_path: Some("/models".to_string()),
            model_source: Some("invalid_source".to_string()),
            ..Default::default()
        };
        let downloader = EchoMimicV3ModelDownloader::new(config);
        let result = downloader.download_model().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::Config(msg) => assert!(msg.contains("模型源地址不在允许范围内")),
            _ => panic!("期望 SomaError::Config 错误"),
        }
    }

    #[tokio::test]
    async fn test_downloader_empty_model_path() {
        let config = EchoMimicV3Config {
            model_source: Some("huggingface".to_string()),
            ..Default::default()
        };
        let downloader = EchoMimicV3ModelDownloader::new(config);
        let result = downloader.download_model().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SomaError::Config(msg) => assert!(msg.contains("model_path")),
            _ => panic!("期望 SomaError::Config 错误"),
        }
    }
}