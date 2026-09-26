//! 平台签名算法模块
//!
//! 实现各社交媒体平台 API 的签名算法，用于 API 直连（不依赖浏览器）。
//! 当前支持：
//! - 抖音：a_bogus / msToken 签名
//! - B站：wbi 签名
//!
//! 这些算法均为纯计算逻辑，无外部依赖。

pub mod abogus;
pub mod mstoken;
pub mod rc4;
pub mod sm3;
pub mod wbi;

/// 抖音签名工具集
pub mod douyin {
    pub use super::abogus::generate_abogus;
    pub use super::mstoken::generate_mstoken;
}

/// B站签名工具集
pub mod bilibili {
    pub use super::wbi::wbi_sign;
}
