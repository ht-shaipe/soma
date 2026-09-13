//! 功能点统一输入/输出信封结构
//!
//! 所有功能点共用一套信封：宿主（HTTP / Tauri command）无需了解具体功能点
//! 即可完成分发调用（`features/run`）与结果回传。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 功能点运行请求（`features/run` 的入参）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRequest {
    /// 目标功能点 ID
    #[serde(alias = "featureId")]
    pub feature_id: String,
    /// 运行实例 ID；缺省时由运行器生成 UUID
    #[serde(default, alias = "runId")]
    pub run_id: Option<String>,
    /// 功能点入参（结构由该功能点的 input schema 约定）
    #[serde(default)]
    pub input: serde_json::Value,
}

/// 功能点入参信封
///
/// `payload` 为功能点实际入参；`meta` 预留给宿主附加上下文（如来源任务 ID），
/// 使功能点未来可以在"裸调用"与"任务内调用"之间透明切换。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureInput {
    /// 功能点实际入参
    #[serde(default)]
    pub payload: serde_json::Value,
    /// 宿主附加元信息（预留字段）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// 功能点运行状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStatus {
    /// 执行成功
    Success,
    /// 执行失败（见 `error` 字段）
    Failed,
}

/// 产物文件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// 音频（mp3/wav…）
    Audio,
    /// 视频（mp4…）
    Video,
    /// 字幕（srt…）
    Subtitle,
    /// 图片（jpg/png…）
    Image,
    /// 文本（script/storyboard json…）
    Text,
    /// 结构化数据文件（json…）
    Data,
    /// 其他类型
    Other(String),
}

/// 功能点产物文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// 产物名称（通常为文件名，如 `audio.mp3`）
    pub name: String,
    /// 产物文件绝对路径
    pub path: String,
    /// 产物类型
    pub kind: ArtifactKind,
    /// 文件大小（字节；文件不存在时为 0）
    pub size: u64,
}

/// 功能点运行结果信封
///
/// 同时作为运行记录持久化到产物目录（`run_record.json`），
/// 是 `features/history` 的数据单元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureOutput {
    /// 功能点 ID
    pub feature_id: String,
    /// 运行实例 ID
    pub run_id: String,
    /// 运行状态
    pub status: FeatureStatus,
    /// 功能点输出数据（结构由该功能点的 output schema 约定）
    #[serde(default)]
    pub data: serde_json::Value,
    /// 本次运行登记的产物文件
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    /// 失败时的错误信息
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 原始入参（回显，用于历史记录与断点重跑）
    #[serde(default)]
    pub input: serde_json::Value,
    /// 开始时间（UTC）
    pub started_at: DateTime<Utc>,
    /// 结束时间（UTC）
    pub finished_at: DateTime<Utc>,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
}
