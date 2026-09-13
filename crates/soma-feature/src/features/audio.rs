//! 音频处理功能点（原子能力）
//!
//! 独立暴露音频处理原语，与 tts.synthesize / subtitle.generate 组合使用。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::envelope::ArtifactKind;
use crate::feature::TypedFeature;
use crate::progress::ProgressReporter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::error::SomaError;

// ============ audio.concat：多段音频合并 ============

/// audio.concat 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioConcatInput {
    /// 音频文件路径列表（按顺序拼接）
    pub audio_files: Vec<String>,
    /// 输出文件路径；缺省写入产物目录 merged.mp3
    #[serde(default)]
    pub output_path: Option<String>,
}

/// audio.concat 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AudioConcatOutput {
    /// 合并后的音频文件路径
    pub audio_file: String,
    /// 合并后总时长（秒）
    pub duration: f64,
}

/// 音频拼接：多段音频按顺序合并为单个文件
pub struct AudioConcatFeature;

impl TypedFeature for AudioConcatFeature {
    type Input = AudioConcatInput;
    type Output = AudioConcatOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "audio.concat".into(),
            name: "音频拼接".into(),
            description: "将多段音频文件按顺序合并为一个音频文件".into(),
            kind: FeatureKind::Audio,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: AudioConcatInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<AudioConcatOutput, SomaError> {
        if input.audio_files.is_empty() {
            return Err(SomaError::Config("音频文件列表为空".into()));
        }
        let conf = ctx.config();
        let ffmpeg = soma_video::Ffmpeg::new(
            &conf.get_ffmpeg_binary(),
            2,
            conf.get_video_codec(),
        );
        let output_path = match input.output_path {
            Some(ref p) if !p.is_empty() => p.clone(),
            _ => ctx.artifact_path("merged.mp3").to_string_lossy().to_string(),
        };
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        ffmpeg.concat_audios(&input.audio_files, &output_path)?;
        let duration = ffmpeg.get_audio_duration(&output_path)?;
        ctx.add_artifact("merged.mp3", &output_path, ArtifactKind::Audio);
        Ok(AudioConcatOutput {
            audio_file: output_path,
            duration,
        })
    }
}
