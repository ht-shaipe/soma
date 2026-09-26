//! soma-core：视频自动生成工具的核心库
//!
//! 提供配置管理、数据模型、错误定义和工具函数等基础能力，
//! 供上层 soma-service 等 crate 依赖使用。

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// 应用配置模块，包含 TOML 配置文件的结构定义与加载逻辑
pub mod config;
/// 错误类型模块，定义统一的 SomaError 枚举
pub mod error;
/// 数据导出模块，支持 CSV / JSON / JSONL 格式
pub mod export;
/// 敏感词过滤模块，提供敏感词库加载与文本命中检查
pub mod filter;
/// 数据模型模块，定义任务状态、视频参数、字幕等核心数据结构
pub mod models;
/// 通知推送模块，支持 Bark / 钉钉 / Telegram 多渠道通知
pub mod notify;
/// 平台签名算法模块，支持抖音 a_bogus / B站 wbi 等签名
pub mod signing;
/// 字幕高级处理模块，支持翻译/校正/合并/格式转换
pub mod subtitle;
/// 工具函数模块，提供路径处理、字符串分割、SRT 生成等通用工具
pub mod utils;

/// 重新导出应用配置结构体，方便外部直接使用
pub use config::AppConfig;
/// 重新导出核心错误类型，方便外部直接使用
pub use error::SomaError;
