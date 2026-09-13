//! 功能点运行上下文
//!
//! 单次功能点运行的运行时环境：应用配置、产物工作目录、产物登记表。
//! 功能点实现通过 [`FeatureContext`] 读取配置、写产物文件并登记产物，
//! 与"宿主是谁"（HTTP 服务器 / Tauri 桌面）彻底解耦。

use crate::envelope::{Artifact, ArtifactKind};
use soma_core::config::AppConfig;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// 单次功能点运行的上下文
pub struct FeatureContext {
    config: AppConfig,
    feature_id: String,
    run_id: String,
    work_dir: PathBuf,
    artifacts: Mutex<Vec<Artifact>>,
}

impl FeatureContext {
    /// 创建运行上下文（由注册表运行器调用，一般不需要手动构造）
    pub fn new(
        config: AppConfig,
        feature_id: impl Into<String>,
        run_id: impl Into<String>,
        work_dir: PathBuf,
    ) -> Self {
        Self {
            config,
            feature_id: feature_id.into(),
            run_id: run_id.into(),
            work_dir,
            artifacts: Mutex::new(Vec::new()),
        }
    }

    /// 应用配置（只读）
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 功能点 ID
    pub fn feature_id(&self) -> &str {
        &self.feature_id
    }

    /// 运行实例 ID
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// 本次运行的产物工作目录（`storage/features/{feature_id}/{run_id}/`）
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// 为产物文件生成路径（`work_dir/{filename}`）
    ///
    /// 功能点应把产物文件写到该目录下，保证产物与运行记录同处归档。
    pub fn artifact_path(&self, filename: &str) -> PathBuf {
        self.work_dir.join(filename)
    }

    /// 登记一个产物文件
    ///
    /// 自动统计文件大小；文件不存在时记 0 并输出告警。
    pub fn add_artifact(
        &self,
        name: impl Into<String>,
        path: impl Into<PathBuf>,
        kind: ArtifactKind,
    ) -> Artifact {
        let path = path.into();
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if size == 0 {
            log::warn!("产物文件不存在或为空: {}", path.display());
        }
        let artifact = Artifact {
            name: name.into(),
            path: path.to_string_lossy().to_string(),
            kind,
            size,
        };
        if let Ok(mut list) = self.artifacts.lock() {
            list.push(artifact.clone());
        }
        artifact
    }

    /// 获取本次运行已登记的全部产物
    pub fn artifacts(&self) -> Vec<Artifact> {
        self.artifacts
            .lock()
            .map(|list| list.clone())
            .unwrap_or_default()
    }
}
