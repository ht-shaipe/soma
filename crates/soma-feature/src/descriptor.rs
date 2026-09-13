//! 功能点描述与分类定义
//!
//! [`FeatureDescriptor`] 是 `features/list` 接口的返回单元，
//! 前端据此渲染功能点工作台（分组、表单、参数说明）。

use serde::{Deserialize, Serialize};

/// 功能点分类，用于工作台分组展示与权限管理
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureKind {
    /// LLM 文本生成类（需求理解/脚本/分镜/旁白/关键词/元数据）
    Llm,
    /// 语音合成类（TTS 完整合成/声音克隆合成）
    Tts,
    /// 音频处理类（音频拼接等）
    Audio,
    /// 字幕类（SRT 字幕生成）
    Subtitle,
    /// 素材类（素材搜索下载/AI 视频生成）
    Material,
    /// 视频合成类（一键完整合成）
    VideoCompose,
    /// 视频处理类（拼接/渲染/水印/转场/裁切/探测等原子能力）
    VideoTool,
    /// 数字人类（口播视频生成等）
    DigitalHuman,
    /// 工具类（不属于以上分类的辅助能力）
    Utility,
}

/// 功能点描述信息（id/名称/说明/分类 + 输入输出 Schema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDescriptor {
    /// 功能点唯一标识，如 `tts.synthesize`
    pub id: String,
    /// 显示名（中文）
    pub name: String,
    /// 功能说明
    pub description: String,
    /// 功能分类
    pub kind: FeatureKind,
    /// 输入参数 JSON Schema（TypedFeature 自动生成；直接实现 Feature 可为 None）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// 输出数据 JSON Schema
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// 功能点元信息（不含 Schema）
///
/// [`TypedFeature`](crate::TypedFeature) 实现者只需填写元信息，
/// 输入输出 Schema 由泛型桥接层根据类型定义自动生成。
#[derive(Debug, Clone)]
pub struct FeatureMeta {
    /// 功能点唯一标识
    pub id: String,
    /// 显示名（中文）
    pub name: String,
    /// 功能说明
    pub description: String,
    /// 功能分类
    pub kind: FeatureKind,
}
