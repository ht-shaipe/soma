//! soma-tts crate：文本转语音（TTS）核心模块
//!
//! 提供统一的 TTS 服务抽象（[`SomaTtsProvider`]）以及多个 TTS 引擎的实现，
//! 包括 EdgeTTS、SiliconFlow、ElevenLabs 和 MiMo。同时包含语音名称解析、
//! 字幕生成与修正等辅助功能。

pub mod provider;
pub mod edge_tts;
pub mod azure_tts;
pub mod siliconflow_tts;
pub mod mimo_tts;
pub mod elevenlabs_tts;
pub mod gemini_tts;
pub mod voices;
pub mod subtitle;

// 统一导出核心 trait 和常用工具函数，方便外部 crate 直接使用
pub use provider::SomaTtsProvider;
pub use voices::parse_voice_name;
