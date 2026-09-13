//! 功能点进度上报抽象
//!
//! 功能点在执行过程中通过 [`ProgressReporter`] 上报进度；
//! 宿主自行实现推送通道：
//! - soma-server：HTTP 轮询/SSE（转发到任务状态存储）
//! - soma-app：Tauri event（`progress://{run_id}`）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureProgress {
    /// 功能点 ID
    pub feature_id: String,
    /// 运行实例 ID
    pub run_id: String,
    /// 进度百分比（0~100，超出范围会被截断）
    pub percent: u32,
    /// 当前步骤标识（如 `synthesize`、`download`），用于前端展示阶段
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// 人类可读的进度说明
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 事件时间（UTC）
    pub at: DateTime<Utc>,
}

impl FeatureProgress {
    /// 构造一条进度事件
    pub fn new(feature_id: impl Into<String>, run_id: impl Into<String>, percent: u32) -> Self {
        Self {
            feature_id: feature_id.into(),
            run_id: run_id.into(),
            percent: percent.clamp(0, 100),
            step: None,
            message: None,
            at: Utc::now(),
        }
    }

    /// 附加步骤标识
    pub fn with_step(mut self, step: impl Into<String>) -> Self {
        self.step = Some(step.into());
        self
    }

    /// 附加进度说明
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// 进度上报 trait
///
/// 实现需满足 Send + Sync：功能点可能在独立线程中运行，
/// 同一 reporter 可能被多个运行共享。
pub trait ProgressReporter: Send + Sync {
    /// 上报一条进度事件
    fn report(&self, progress: FeatureProgress);
}

/// 空实现：丢弃所有进度事件（用于测试或不关心进度的调用方）
pub struct NoopProgress;

impl ProgressReporter for NoopProgress {
    fn report(&self, _progress: FeatureProgress) {}
}

/// 闭包实现：把进度事件交给调用方提供的闭包处理
///
/// # 示例
/// ```
/// use soma_feature::{FnReporter, FeatureProgress, NoopProgress, ProgressReporter};
///
/// let reporter = FnReporter(|p: FeatureProgress| {
///     println!("{}: {}%", p.feature_id, p.percent);
/// });
/// reporter.report(FeatureProgress::new("tts.synthesize", "run-1", 30));
/// ```
pub struct FnReporter<F>(pub F)
where
    F: Fn(FeatureProgress) + Send + Sync;

impl<F> ProgressReporter for FnReporter<F>
where
    F: Fn(FeatureProgress) + Send + Sync,
{
    fn report(&self, progress: FeatureProgress) {
        (self.0)(progress)
    }
}
