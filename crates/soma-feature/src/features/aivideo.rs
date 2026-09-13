//! AI 视频生成功能点（文生视频 / 图生视频）
//!
//! 独立暴露原流水线第 4 步中的 AI 视频能力：
//! 支持智谱 CogVideoX、可灵 Kling、MiniMax 三家提供者，
//! 以及人像图生视频（口播数字人的图生视频模式）。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::envelope::ArtifactKind;
use crate::feature::TypedFeature;
use crate::progress::{FeatureProgress, ProgressReporter};
use crate::runtime::block_on_async;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::error::SomaError;
use soma_core::models::{AiVideoSegmentLog, VideoAspect};

/// AI 视频生成入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AiVideoGenerateInput {
    /// 视觉提示词列表（每段一条英文提示词）
    pub terms: Vec<String>,
    /// AI 视频提供者：cogvideox / kling / minimax
    pub source: String,
    /// 画幅比例（"16:9"/"9:16"/"1:1"），缺省 9:16
    #[serde(default)]
    pub aspect: Option<String>,
    /// 单段时长（秒），缺省 4
    #[serde(default)]
    pub clip_duration: Option<u32>,
    /// 图生视频模式的人像图片 URL（可选）
    #[serde(default)]
    pub portrait_image: Option<String>,
    /// 保存目录；缺省写入产物目录
    #[serde(default)]
    pub output_dir: Option<String>,
}

/// AI 视频生成出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AiVideoGenerateOutput {
    /// 生成的视频文件路径列表（部分失败时仅含成功项）
    pub videos: Vec<String>,
    /// 逐段生成日志（每段提示词与状态）
    pub logs: Vec<AiVideoSegmentLog>,
}

/// AI 视频生成：提示词 → 视频文件（文生视频 / 图生视频）
pub struct AiVideoGenerateFeature;

impl TypedFeature for AiVideoGenerateFeature {
    type Input = AiVideoGenerateInput;
    type Output = AiVideoGenerateOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "aivideo.generate".into(),
            name: "AI 视频生成".into(),
            description: "调用 CogVideoX/可灵/MiniMax 生成 AI 视频，支持文生视频与图生视频".into(),
            kind: FeatureKind::Material,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: AiVideoGenerateInput,
        progress: &dyn ProgressReporter,
    ) -> Result<AiVideoGenerateOutput, SomaError> {
        if !matches!(input.source.as_str(), "cogvideox" | "kling" | "minimax") {
            return Err(SomaError::VideoGen(format!(
                "不支持的 AI 视频提供者: {}（可选 cogvideox/kling/minimax）",
                input.source
            )));
        }
        let aspect = input
            .aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait);
        let save_dir = match input.output_dir {
            Some(ref d) if !d.is_empty() => d.clone(),
            _ => ctx.work_dir().to_string_lossy().to_string(),
        };

        progress.report(
            FeatureProgress::new(ctx.feature_id(), ctx.run_id(), 5)
                .with_step("submit")
                .with_message("提交 AI 视频生成任务"),
        );

        let (videos, logs) = block_on_async(soma_stock::generate_ai_videos(
            &save_dir,
            &input.terms,
            &input.source,
            &aspect,
            input.clip_duration.unwrap_or(4),
            ctx.config(),
            input.portrait_image.as_deref(),
        ))??;

        for path in &videos {
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            ctx.add_artifact(name, path, ArtifactKind::Video);
        }
        Ok(AiVideoGenerateOutput { videos, logs })
    }
}
