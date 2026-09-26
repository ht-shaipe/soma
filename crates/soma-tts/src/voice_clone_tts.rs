//! 声音克隆 TTS 引擎实现
//!
//! 通过 SSH 远程调用 GPU 服务器上的声音克隆模型（GPT-SoVITS/CosyVoice/Fish-Speech），
//! 使用参考音频克隆音色，合成目标文本的语音。
//!
//! 与 EdgeTTS 的区别：GPU 推理、SSH 远程调用、参考音频克隆音色、多模型路由。
//! 关键约束：synthesize 必须在当前 async 上下文内同步执行（不可 tokio::spawn）。

use async_trait::async_trait;
use soma_core::config::VoiceCloneConfig;
use soma_core::error::SomaError;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};

use crate::edge_tts::generate_subtitle_cues_from_text;
use crate::provider::{SomaTtsProvider, TtsResult};

// ── 数据模型 ─────────────────────────────────────────────

#[derive(Debug, Clone)]
enum VoiceCloneRunStatus {
    Processing,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
struct VoiceCloneTaskState {
    status: VoiceCloneRunStatus,
    audio_path: Option<String>,
    error: Option<String>,
    #[allow(dead_code)] // 状态时间线，供调试与后续超时清理使用
    started_at: Instant,
    completed_at: Option<Instant>,
    #[allow(dead_code)]
    retry_count: u32,
}

#[derive(Debug, Clone)]
pub enum MissingKind {
    Ssh,
    Environment,
    Dependency,
    ModelWeight,
    Script,
    Gpu,
    Cuda,
    PythonVersion,
    GpuMemory,
}

#[derive(Debug, Clone)]
pub struct MissingItem {
    pub kind: MissingKind,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct HealthReport {
    pub ready: bool,
    pub missing: Vec<MissingItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneModel {
    GptSovits,
    CosyVoice,
    FishSpeech,
}

impl CloneModel {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, SomaError> {
        match s {
            "gpt_sovits" => Ok(Self::GptSovits),
            "cosyvoice" => Ok(Self::CosyVoice),
            "fish_speech" => Ok(Self::FishSpeech),
            _ => Err(SomaError::Config(format!(
                "不支持的声音克隆模型: {}（可选 gpt_sovits/cosyvoice/fish_speech）",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GptSovits => "gpt_sovits",
            Self::CosyVoice => "cosyvoice",
            Self::FishSpeech => "fish_speech",
        }
    }
}

#[derive(Debug, Clone)]
struct SshOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SshErrorKind {
    ConnectionRefused,
    ConnectionInterrupted,
    AuthFailed,
    RemoteCommandFailed,
    Success,
}

// ── SshExecutor ──────────────────────────────────────────

#[derive(Debug, Clone)]
struct SshExecutor {
    host: String,
    port: u16,
    user: String,
    key_path: String,
}

impl SshExecutor {
    fn new(host: &str, port: u16, user: &str, key_path: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            user: user.to_string(),
            key_path: key_path.to_string(),
        }
    }

    async fn execute(&self, command: &str, timeout: Duration) -> Result<SshOutput, SomaError> {
        let mut cmd = Command::new("ssh");
        cmd.arg("-i")
            .arg(&self.key_path)
            .arg("-p")
            .arg(self.port.to_string())
            .arg("-o")
            .arg("ConnectTimeout=10")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg(format!("{}@{}", self.user, self.host))
            .arg(command);

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| SomaError::Tts(format!("启动 SSH 失败: {}", e)))?;

        let result = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| SomaError::Tts("SSH 命令执行超时".into()))?
            .map_err(|e| SomaError::Tts(format!("等待 SSH 结果失败: {}", e)))?;

        Ok(SshOutput {
            exit_code: result.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&result.stdout).to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
        })
    }

    async fn upload(&self, local: &str, remote: &str, timeout: Duration) -> Result<(), SomaError> {
        let mut cmd = Command::new("scp");
        cmd.arg("-i")
            .arg(&self.key_path)
            .arg("-P")
            .arg(self.port.to_string())
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg(local)
            .arg(format!("{}@{}:{}", self.user, self.host, remote));

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| SomaError::Tts(format!("启动 scp 失败: {}", e)))?;

        let result = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| SomaError::Tts("文件上传超时".into()))?
            .map_err(|e| SomaError::Tts(format!("等待 scp 结果失败: {}", e)))?;

        if !result.status.success() {
            return Err(SomaError::Tts(format!(
                "文件上传失败: {}",
                String::from_utf8_lossy(&result.stderr)
            )));
        }
        Ok(())
    }

    async fn download(
        &self,
        remote: &str,
        local: &str,
        timeout: Duration,
    ) -> Result<(), SomaError> {
        let mut cmd = Command::new("scp");
        cmd.arg("-i")
            .arg(&self.key_path)
            .arg("-P")
            .arg(self.port.to_string())
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg(format!("{}@{}:{}", self.user, self.host, remote))
            .arg(local);

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| SomaError::Tts(format!("启动 scp 失败: {}", e)))?;

        let result = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| SomaError::Tts("文件下载超时".into()))?
            .map_err(|e| SomaError::Tts(format!("等待 scp 结果失败: {}", e)))?;

        if !result.status.success() {
            return Err(SomaError::Tts(format!(
                "文件下载失败: {}",
                String::from_utf8_lossy(&result.stderr)
            )));
        }
        Ok(())
    }

    fn classify_ssh_error(exit_code: i32, stderr: &str) -> SshErrorKind {
        if exit_code == 0 {
            return SshErrorKind::Success;
        }
        if exit_code == 255 {
            if stderr.contains("Connection refused") || stderr.contains("Could not resolve hostname")
            {
                return SshErrorKind::ConnectionRefused;
            }
            if stderr.contains("Connection closed")
                || stderr.contains("Connection reset")
                || stderr.contains("Broken pipe")
            {
                return SshErrorKind::ConnectionInterrupted;
            }
            if stderr.contains("Permission denied") || stderr.contains("Public key") {
                return SshErrorKind::AuthFailed;
            }
        }
        SshErrorKind::RemoteCommandFailed
    }
}

// ── VoiceCloneEnvChecker ─────────────────────────────────

struct VoiceCloneEnvChecker {
    config: VoiceCloneConfig,
    ssh_executor: SshExecutor,
}

impl VoiceCloneEnvChecker {
    fn new(config: VoiceCloneConfig) -> Self {
        let ssh = SshExecutor::new(
            config.get_ssh_host(),
            config.get_ssh_port(),
            config.get_ssh_user(),
            config.get_ssh_key_path(),
        );
        Self {
            config,
            ssh_executor: ssh,
        }
    }

    pub fn check(&self) -> HealthReport {
        let mut missing = Vec::new();

        if let Some(item) = self.check_ssh_connectivity() {
            missing.push(item);
            return HealthReport {
                ready: false,
                missing,
            };
        }
        if let Some(item) = self.check_remote_env_exists() {
            missing.push(item);
        }
        if let Some(item) = self.check_remote_python_version() {
            missing.push(item);
        }
        if let Some(item) = self.check_remote_script_exists() {
            missing.push(item);
        }
        if let Some(item) = self.check_remote_model_weights() {
            missing.push(item);
        }
        if let Some(item) = self.check_remote_gpu_available() {
            missing.push(item);
        }
        if let Some(item) = self.check_remote_cuda_version() {
            missing.push(item);
        }

        HealthReport {
            ready: missing.is_empty(),
            missing,
        }
    }

    fn check_ssh_connectivity(&self) -> Option<MissingItem> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let result = rt.block_on(self.ssh_executor.execute("echo ok", Duration::from_secs(10)));
        match result {
            Ok(o) if o.exit_code == 0 && o.stdout.trim() == "ok" => None,
            Ok(o) => Some(MissingItem {
                kind: MissingKind::Ssh,
                description: format!("SSH 连接异常: exit={}, stderr={}", o.exit_code, o.stderr),
            }),
            Err(e) => Some(MissingItem {
                kind: MissingKind::Ssh,
                description: format!("SSH 连接失败: {}", e),
            }),
        }
    }

    fn check_remote_env_exists(&self) -> Option<MissingItem> {
        let path = self.config.get_remote_env_path();
        if path.is_empty() {
            return Some(MissingItem {
                kind: MissingKind::Environment,
                description: "未配置 remote_env_path".into(),
            });
        }
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let cmd = format!("test -d {}", path);
        match rt.block_on(self.ssh_executor.execute(&cmd, Duration::from_secs(10))) {
            Ok(o) if o.exit_code == 0 => None,
            _ => Some(MissingItem {
                kind: MissingKind::Environment,
                description: format!("远程环境目录不存在: {}", path),
            }),
        }
    }

    fn check_remote_python_version(&self) -> Option<MissingItem> {
        let python = self.config.get_remote_python_path();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let cmd = format!("{} --version", python);
        match rt.block_on(self.ssh_executor.execute(&cmd, Duration::from_secs(5))) {
            Ok(o) if o.exit_code == 0 => {
                let ver = o.stdout.trim();
                if ver.contains("3.10") || ver.contains("3.11") {
                    None
                } else {
                    Some(MissingItem {
                        kind: MissingKind::PythonVersion,
                        description: format!("Python 版本需 3.10/3.11，当前: {}", ver),
                    })
                }
            }
            _ => Some(MissingItem {
                kind: MissingKind::Dependency,
                description: format!("Python 不可调用: {}", python),
            }),
        }
    }

    fn check_remote_script_exists(&self) -> Option<MissingItem> {
        let script = self.config.get_remote_script_path();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let cmd = format!("test -f {}", script);
        match rt.block_on(self.ssh_executor.execute(&cmd, Duration::from_secs(10))) {
            Ok(o) if o.exit_code == 0 => None,
            _ => Some(MissingItem {
                kind: MissingKind::Script,
                description: format!("封装脚本不存在: {}", script),
            }),
        }
    }

    fn check_remote_model_weights(&self) -> Option<MissingItem> {
        let path = self.config.get_remote_model_path();
        if path.is_empty() {
            return Some(MissingItem {
                kind: MissingKind::ModelWeight,
                description: "未配置 remote_model_path".into(),
            });
        }
        let model = self.config.get_default_clone_model();
        let sub = match model {
            "gpt_sovits" => "GPT-SoVITS",
            "cosyvoice" => "CosyVoice",
            "fish_speech" => "Fish-Speech",
            _ => "unknown",
        };
        let model_dir = format!("{}/{}", path, sub);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let cmd = format!("test -d {}", model_dir);
        match rt.block_on(self.ssh_executor.execute(&cmd, Duration::from_secs(10))) {
            Ok(o) if o.exit_code == 0 => None,
            _ => Some(MissingItem {
                kind: MissingKind::ModelWeight,
                description: format!("模型权重目录不存在: {}", model_dir),
            }),
        }
    }

    fn check_remote_gpu_available(&self) -> Option<MissingItem> {
        let python = self.config.get_remote_python_path();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let cmd = format!(
            "{} -c \"import torch; print(torch.cuda.is_available(), torch.cuda.get_device_properties(0).total_memory if torch.cuda.is_available() else 0)\"",
            python
        );
        match rt.block_on(self.ssh_executor.execute(&cmd, Duration::from_secs(10))) {
            Ok(o) if o.exit_code == 0 => {
                let out = o.stdout.trim();
                let parts: Vec<&str> = out.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "True" {
                    let mem_bytes: f64 = parts[1].parse().unwrap_or(0.0);
                    let mem_gb = mem_bytes / 1024.0 / 1024.0 / 1024.0;
                    if mem_gb < 4.0 {
                        return Some(MissingItem {
                            kind: MissingKind::GpuMemory,
                            description: format!("GPU 显存不足: {:.1}GB（要求 ≥ 4GB）", mem_gb),
                        });
                    }
                    None
                } else {
                    Some(MissingItem {
                        kind: MissingKind::Gpu,
                        description: "CUDA 不可用".into(),
                    })
                }
            }
            _ => Some(MissingItem {
                kind: MissingKind::Gpu,
                description: "无法检查 GPU 状态".into(),
            }),
        }
    }

    fn check_remote_cuda_version(&self) -> Option<MissingItem> {
        let python = self.config.get_remote_python_path();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        let cmd = format!(
            "{} -c \"import torch; print(torch.version.cuda or '0')\"",
            python
        );
        match rt.block_on(self.ssh_executor.execute(&cmd, Duration::from_secs(5))) {
            Ok(o) if o.exit_code == 0 => {
                let ver = o.stdout.trim();
                if let Ok(v) = ver.parse::<f64>() {
                    if v >= 12.1 {
                        return None;
                    }
                }
                Some(MissingItem {
                    kind: MissingKind::Cuda,
                    description: format!("CUDA 版本需 ≥ 12.1，当前: {}", ver),
                })
            }
            _ => Some(MissingItem {
                kind: MissingKind::Cuda,
                description: "无法检查 CUDA 版本".into(),
            }),
        }
    }
}

// ── 参考音频校验 ─────────────────────────────────────────

pub fn validate_reference_audio(reference_audio: &str) -> Result<(), SomaError> {
    let path = soma_core::utils::validate_local_path(reference_audio, true)?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    if !matches!(ext.as_str(), "wav" | "mp3" | "m4a" | "aac" | "ogg" | "flac") {
        return Err(SomaError::Tts(
            "参考音频格式不支持，仅支持 WAV/MP3/M4A/AAC/OGG/FLAC".into(),
        ));
    }

    let metadata = std::fs::metadata(&path)
        .map_err(|e| SomaError::Tts(format!("读取参考音频文件信息失败: {}", e)))?;
    let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
    if size_mb > 10.0 {
        return Err(SomaError::Tts(
            "参考音频文件过大，要求 ≤ 10MB".into(),
        ));
    }

    Ok(())
}

// ── VoiceCloneTts ────────────────────────────────────────

/// 声音克隆 TTS 提供者
pub struct VoiceCloneTts {
    config: VoiceCloneConfig,
    reference_audio: String,
    reference_text: String,
    clone_model: String,
    tasks: Arc<Mutex<HashMap<String, VoiceCloneTaskState>>>,
    semaphore: Arc<Semaphore>,
    ssh_executor: SshExecutor,
    env_checker: VoiceCloneEnvChecker,
}

impl VoiceCloneTts {
    /// 创建 VoiceCloneTts 实例
    ///
    /// - `config` - 声音克隆配置
    /// - `reference_audio` - 参考音频本地路径
    /// - `reference_text` - 参考音频对应文本（可选，传空串）
    /// - `clone_model` - 克隆模型选择（空串则使用配置默认值）
    pub fn new(
        config: VoiceCloneConfig,
        reference_audio: &str,
        reference_text: &str,
        clone_model: &str,
    ) -> Self {
        let ssh = SshExecutor::new(
            config.get_ssh_host(),
            config.get_ssh_port(),
            config.get_ssh_user(),
            config.get_ssh_key_path(),
        );
        let env_checker = VoiceCloneEnvChecker::new(config.clone());
        let max_concurrent = config.get_max_concurrent() as usize;
        Self {
            config,
            reference_audio: reference_audio.to_string(),
            reference_text: reference_text.to_string(),
            clone_model: clone_model.to_string(),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent.max(1))),
            ssh_executor: ssh,
            env_checker,
        }
    }

    /// 环境健康检查
    pub fn check_health(&self) -> HealthReport {
        self.env_checker.check()
    }

    async fn insert_task(&self, task_id: &str) {
        let mut tasks = self.tasks.lock().await;
        tasks.insert(
            task_id.to_string(),
            VoiceCloneTaskState {
                status: VoiceCloneRunStatus::Processing,
                audio_path: None,
                error: None,
                started_at: Instant::now(),
                completed_at: None,
                retry_count: 0,
            },
        );
    }

    async fn update_task_success(&self, task_id: &str, audio_path: String) {
        let mut tasks = self.tasks.lock().await;
        if let Some(t) = tasks.get_mut(task_id) {
            t.status = VoiceCloneRunStatus::Success;
            t.audio_path = Some(audio_path);
            t.completed_at = Some(Instant::now());
        }
    }

    async fn update_task_failed(&self, task_id: &str, error: String) {
        let mut tasks = self.tasks.lock().await;
        if let Some(t) = tasks.get_mut(task_id) {
            t.status = VoiceCloneRunStatus::Failed;
            t.error = Some(error);
            t.completed_at = Some(Instant::now());
        }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for VoiceCloneTts {
    async fn synthesize(
        &self,
        text: &str,
        _voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        let task_id = soma_core::utils::get_uuid();

        if self.config.get_preflight_check() {
            let health = self.env_checker.check();
            if !health.ready {
                let missing: Vec<String> =
                    health.missing.iter().map(|m| m.description.clone()).collect();
                return Err(SomaError::Config(format!(
                    "声音克隆运行环境未就绪: {}",
                    missing.join("; ")
                )));
            }
        }

        let device = self.config.get_device();
        if !device.contains("cuda") {
            return Err(SomaError::Config(
                "声音克隆仅支持 GPU 推理，请配置 device 为 cuda".into(),
            ));
        }

        let model_str = if self.clone_model.is_empty() {
            self.config.get_default_clone_model()
        } else {
            &self.clone_model
        };
        let _clone_model = CloneModel::from_str(model_str)?;

        validate_reference_audio(&self.reference_audio)?;

        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(SomaError::Tts("文案不能为空".into()));
        }
        if trimmed.chars().count() > 1000 {
            return Err(SomaError::Tts(
                "文案长度不能超过 1000 字".into(),
            ));
        }

        self.insert_task(&task_id).await;
        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| SomaError::Tts(format!("获取并发许可失败: {}", e)))?;

        let remote_tmp = format!("/tmp/voice_clone_{}", task_id);
        let ref_ext = std::path::Path::new(&self.reference_audio)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        let remote_ref = format!("{}/ref_audio.{}", remote_tmp, ref_ext);
        let remote_out = format!("{}/output.mp3", remote_tmp);

        let timeout = Duration::from_secs(self.config.get_timeout());
        let max_retries = self.config.get_max_retries();
        let mut last_error = String::new();

        let create_dir_result = self
            .ssh_executor
            .execute(&format!("mkdir -p {}", remote_tmp), Duration::from_secs(10))
            .await;
        if let Err(e) = create_dir_result {
            self.update_task_failed(&task_id, format!("创建远程临时目录失败: {}", e))
                .await;
            return Err(SomaError::Tts(format!("创建远程临时目录失败: {}", e)));
        }

        if let Err(e) = self
            .ssh_executor
            .upload(&self.reference_audio, &remote_ref, Duration::from_secs(60))
            .await
        {
            self.update_task_failed(&task_id, format!("上传参考音频失败: {}", e))
                .await;
            let _ = self
                .ssh_executor
                .execute(&format!("rm -rf {}", remote_tmp), Duration::from_secs(5))
                .await;
            return Err(e);
        }

        for attempt in 1..=max_retries {
            log::info!(
                "声音克隆推理尝试 {}/{}: task_id={}",
                attempt,
                max_retries,
                task_id
            );

            let remote_cmd = format!(
                "cd {} && PYTHONPATH={} {} {} --reference_audio {} --reference_text '{}' --target_text '{}' --outfile {} --clone_model {} --model_dir {} --device {} --rate {}",
                self.config.get_remote_env_path(),
                self.config.get_remote_env_path(),
                self.config.get_remote_python_path(),
                self.config.get_remote_script_path(),
                remote_ref,
                self.reference_text.replace('\'', "'\\''"),
                trimmed.replace('\'', "'\\''"),
                remote_out,
                model_str,
                self.config.get_remote_model_path(),
                device,
                rate
            );

            let result = self.ssh_executor.execute(&remote_cmd, timeout).await;

            match result {
                Ok(o) if o.exit_code == 0 => {
                    let stdout = o.stdout.trim();
                    let json_str = stdout
                        .lines()
                        .rev()
                        .find(|line| line.trim_start().starts_with('{'))
                        .unwrap_or(stdout);
                    let json: serde_json::Value = match serde_json::from_str(json_str.trim()) {
                        Ok(v) => v,
                        Err(e) => {
                            last_error = format!("解析推理输出失败: {} (stdout: {})", e, stdout);
                            continue;
                        }
                    };

                    if json.get("status").and_then(|v| v.as_str()) == Some("success") {
                        if let Some(parent) = output_path.parent() {
                            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                                SomaError::Tts(format!("创建输出目录失败: {}", e))
                            })?;
                        }

                        if let Err(e) = self
                            .ssh_executor
                            .download(
                                &remote_out,
                                &output_path.to_string_lossy(),
                                Duration::from_secs(60),
                            )
                            .await
                        {
                            last_error = format!("下载音频失败: {}", e);
                            continue;
                        }

                        if !output_path.exists() {
                            last_error = "输出音频文件不存在".into();
                            continue;
                        }

                        let audio_file = output_path.to_string_lossy().to_string();
                        let audio_duration = json
                            .get("duration")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        let subtitle_cues =
                            generate_subtitle_cues_from_text(trimmed, audio_duration);

                        self.update_task_success(&task_id, audio_file.clone()).await;
                        let _ = self
                            .ssh_executor
                            .execute(&format!("rm -rf {}", remote_tmp), Duration::from_secs(5))
                            .await;

                        log::info!("声音克隆成功: task_id={}, audio={}", task_id, audio_file);
                        return Ok(TtsResult {
                            audio_file,
                            audio_duration,
                            subtitle_cues,
                        });
                    } else {
                        let error = json
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未知错误");
                        last_error = error.to_string();
                        if error.contains("CUDA out of memory") {
                            break;
                        }
                    }
                }
                Ok(o) => {
                    let kind = SshExecutor::classify_ssh_error(o.exit_code, &o.stderr);
                    last_error = format!(
                        "SSH 推理失败 (exit={}): {}",
                        o.exit_code,
                        o.stderr
                    );
                    match kind {
                        SshErrorKind::ConnectionRefused | SshErrorKind::AuthFailed => break,
                        _ => {}
                    }
                }
                Err(e) => {
                    last_error = format!("SSH 执行错误: {}", e);
                }
            }
        }

        self.update_task_failed(&task_id, last_error.clone()).await;
        let _ = self
            .ssh_executor
            .execute(&format!("rm -rf {}", remote_tmp), Duration::from_secs(5))
            .await;

        Err(SomaError::Tts(format!(
            "声音克隆推理失败，请重试或更换克隆模型: {}",
            last_error
        )))
    }
}

// ── 单元测试 ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_model_from_str() {
        assert_eq!(CloneModel::from_str("gpt_sovits").unwrap(), CloneModel::GptSovits);
        assert_eq!(CloneModel::from_str("cosyvoice").unwrap(), CloneModel::CosyVoice);
        assert_eq!(CloneModel::from_str("fish_speech").unwrap(), CloneModel::FishSpeech);
        assert!(CloneModel::from_str("unknown").is_err());
        assert!(CloneModel::from_str("").is_err());
    }

    #[test]
    fn test_clone_model_as_str() {
        assert_eq!(CloneModel::GptSovits.as_str(), "gpt_sovits");
        assert_eq!(CloneModel::CosyVoice.as_str(), "cosyvoice");
        assert_eq!(CloneModel::FishSpeech.as_str(), "fish_speech");
    }

    #[test]
    fn test_classify_ssh_error() {
        assert_eq!(SshExecutor::classify_ssh_error(0, ""), SshErrorKind::Success);
        assert_eq!(
            SshExecutor::classify_ssh_error(255, "Connection refused"),
            SshErrorKind::ConnectionRefused
        );
        assert_eq!(
            SshExecutor::classify_ssh_error(255, "Connection closed by remote"),
            SshErrorKind::ConnectionInterrupted
        );
        assert_eq!(
            SshExecutor::classify_ssh_error(255, "Permission denied (publickey)"),
            SshErrorKind::AuthFailed
        );
        assert_eq!(
            SshExecutor::classify_ssh_error(1, "some error"),
            SshErrorKind::RemoteCommandFailed
        );
    }

    #[test]
    fn test_voice_clone_config_defaults() {
        let config = VoiceCloneConfig::default();
        assert_eq!(config.get_ssh_host(), "");
        assert_eq!(config.get_ssh_port(), 22);
        assert_eq!(config.get_ssh_user(), "root");
        assert_eq!(config.get_device(), "cuda");
        assert_eq!(config.get_default_clone_model(), "gpt_sovits");
        assert_eq!(config.get_timeout(), 300);
        assert_eq!(config.get_max_concurrent(), 1);
        assert_eq!(config.get_max_retries(), 3);
        assert!(config.get_preflight_check());
        assert_eq!(config.get_gpu_memory_limit(), 24);
    }

    #[test]
    fn test_voice_clone_config_remote_script_path_default() {
        let config = VoiceCloneConfig {
            remote_env_path: Some("/root/voice_clone".into()),
            ..Default::default()
        };
        assert_eq!(
            config.get_remote_script_path(),
            "/root/voice_clone/voice_clone_runner.py"
        );
    }

    #[test]
    fn test_voice_clone_config_remote_script_path_explicit() {
        let config = VoiceCloneConfig {
            remote_script_path: Some("/custom/path/runner.py".into()),
            ..Default::default()
        };
        assert_eq!(config.get_remote_script_path(), "/custom/path/runner.py");
    }
}