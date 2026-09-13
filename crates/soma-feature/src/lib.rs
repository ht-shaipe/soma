//! soma-feature：功能点（Feature）抽象层
//!
//! 把视频生成流水线中的每个能力（需求理解、脚本、分镜、TTS、字幕、素材、
//! AI 视频、合成、数字人…）拆分为可独立注册、独立调用、独立产出产物的"功能点"，
//! 供多个宿主复用，业务代码零复制：
//!
//! - soma-server：HTTP API 宿主（`features/*` 端点、任务编排器）
//! - soma-app：Tauri 桌面宿主（`#[tauri::command]` + event）
//!
//! 核心概念：
//! - [`Feature`]：功能点抽象，输入输出为 JSON 信封
//! - [`TypedFeature`]：强类型功能点（serde 结构体定义即 JSON Schema），自动桥接为 [`Feature`]
//! - [`FeatureRegistry`]：功能点注册表与统一运行器（产物落盘 + 运行记录持久化）
//! - [`ProgressReporter`]：进度上报抽象，宿主各自实现推送通道（SSE / Tauri event）
//!
//! 产物目录约定：`storage/features/{feature_id}/{run_id}/`，
//! 每次运行写入 `input.json`（原始入参）与 `run_record.json`（运行记录），支撑历史查询。

// 引入 tube 框架宏（log! 等），供 LLM 服务模块使用
#[macro_use]
extern crate tube;

#[macro_use]
extern crate lazy_static;

/// 功能点运行上下文（配置、产物目录、产物登记）
pub mod context;
/// 功能点描述与分类定义
pub mod descriptor;
/// 统一输入/输出信封结构（请求、产物、运行结果）
pub mod envelope;
/// Feature trait 与 TypedFeature 强类型桥接
pub mod feature;
/// 内置功能点实现（流水线各步骤的独立化形态）
pub mod features;
/// LLM 服务（需求理解/脚本/分镜/旁白/关键词/社交元数据）
pub mod llm;
/// 进度上报抽象与内置实现
pub mod progress;
/// 功能点注册表与统一运行器
pub mod registry;
/// 运行时辅助（阻塞桥接 async、重试）
pub mod runtime;

pub use context::FeatureContext;
pub use descriptor::{FeatureDescriptor, FeatureKind, FeatureMeta};
pub use envelope::{
    Artifact, ArtifactKind, FeatureInput, FeatureOutput, FeatureRequest, FeatureStatus,
};
pub use feature::{Feature, TypedFeature};
pub use progress::{FeatureProgress, FnReporter, NoopProgress, ProgressReporter};
pub use registry::FeatureRegistry;
