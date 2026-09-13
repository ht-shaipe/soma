//! 内置功能点实现
//!
//! 流水线的每个步骤对应一个可独立调用的功能点（强类型输入输出，
//! 数据传递不再依赖 TaskInfo 字段的隐式共享）：
//!
//! | 功能点 ID | 职责 | 原流水线步骤 |
//! |-----------|------|--------------|
//! | `llm.intent` | 需求理解 | 第1步 |
//! | `llm.script` | 文案/剧情生成 | 第2步 |
//! | `llm.storyboard` | 分镜脚本 + 提示词 + 关键词 | 第3步 |
//! | `llm.narration` | 旁白文案改写 | 第3.5步 |
//! | `material.generate` | 素材生成（本地/在线/AI 视频来源选择） | 第4步 |
//! | `material.search` | 素材搜索（只搜不下载） | 独立 |
//! | `material.download` | 按 URL 列表下载素材 | 独立 |
//! | `aivideo.generate` | AI 视频生成（文生/图生视频） | 第4步拆出 |
//! | `audio.concat` | 多段音频合并 | 原子能力 |
//! | `tts.synthesize` | TTS 完整合成（文本→音频） | 第5步（音频） |
//! | `subtitle.generate` | 字幕生成（音频→SRT） | 第5步（字幕） |
//! | `video.compose` | 视频合成（一键完整合成） | 第6步 |
//! | `video.concat` / `video.render` / `video.watermark` / `video.join` / `video.transition` / `video.clip_resize` / `video.info` | 视频处理原子能力（可自由组合后期链） | 第6步拆分 |

pub mod aivideo;
pub mod audio;
pub mod compose;
pub mod llm;
pub mod materials;
pub mod subtitle;
pub mod tts;
pub mod video_tools;

use crate::registry::FeatureRegistry;
use soma_core::error::SomaError;
use std::sync::Arc;

/// 注册全部内置功能点
///
/// 同一注册表重复调用会因功能点 ID 冲突报错。
pub fn register_builtin(registry: &FeatureRegistry) -> Result<(), SomaError> {
    registry.register(Arc::new(llm::LlmIntentFeature))?;
    registry.register(Arc::new(llm::LlmScriptFeature))?;
    registry.register(Arc::new(llm::LlmStoryboardFeature))?;
    registry.register(Arc::new(llm::LlmNarrationFeature))?;
    registry.register(Arc::new(llm::LlmTermsFeature))?;
    registry.register(Arc::new(llm::LlmSocialFeature))?;
    registry.register(Arc::new(tts::TtsSynthesizeFeature))?;
    registry.register(Arc::new(subtitle::SubtitleGenerateFeature))?;
    registry.register(Arc::new(materials::MaterialGenerateFeature))?;
    registry.register(Arc::new(materials::MaterialSearchFeature))?;
    registry.register(Arc::new(materials::MaterialDownloadFeature))?;
    registry.register(Arc::new(aivideo::AiVideoGenerateFeature))?;
    registry.register(Arc::new(compose::VideoComposeFeature))?;
    registry.register(Arc::new(video_tools::VideoConcatFeature))?;
    registry.register(Arc::new(video_tools::VideoRenderFeature))?;
    registry.register(Arc::new(video_tools::VideoWatermarkFeature))?;
    registry.register(Arc::new(video_tools::VideoJoinFeature))?;
    registry.register(Arc::new(video_tools::VideoTransitionFeature))?;
    registry.register(Arc::new(video_tools::VideoClipResizeFeature))?;
    registry.register(Arc::new(video_tools::VideoInfoFeature))?;
    registry.register(Arc::new(audio::AudioConcatFeature))?;
    Ok(())
}
