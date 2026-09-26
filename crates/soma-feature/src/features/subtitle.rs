//! 字幕生成功能点（音频 + 文本 → SRT）
//!
//! 覆盖原流水线第 5 步的字幕部分，同时可作为独立入口：
//! 按 `subtitle_provider` 配置选择 Whisper 识别校正或 Edge TTS 时间轴推算。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::envelope::ArtifactKind;
use crate::feature::TypedFeature;
use crate::progress::ProgressReporter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::config::AppConfig;
use soma_core::error::SomaError;

/// 字幕生成（纯函数，供功能点与宿主兼容层复用）
///
/// `subtitle_enabled=false` 时返回空字符串（与原流水线行为一致）。
#[allow(clippy::too_many_arguments)]
pub fn generate_subtitle_to(
    conf: &AppConfig,
    audio_file: &str,
    text: &str,
    subtitle_enabled: bool,
    n_threads: u32,
    video_encoder: Option<&str>,
    output_path: &str,
) -> Result<String, SomaError> {
    if !subtitle_enabled {
        return Ok(String::new());
    }

    let subtitle_provider = conf.get_subtitle_provider();
    if subtitle_provider == "whisper" {
        let ws = &conf.whisper;
        soma_tts::subtitle::generate_whisper_subtitle(
            audio_file,
            output_path,
            ws.model_size.as_deref().unwrap_or("base"),
            ws.device.as_deref().unwrap_or("cpu"),
            ws.compute_type.as_deref().unwrap_or("int8"),
        )?;
        let mut cues = soma_tts::subtitle::file_to_subtitles(output_path);
        let plain_text = soma_tts::fishspeech_tts::strip_emotion_tags(text);
        soma_tts::subtitle::correct_subtitle(&mut cues, &plain_text);
        soma_tts::subtitle::create_subtitle_file(&cues, output_path)?;
    } else {
        let codec = video_encoder.unwrap_or(conf.get_video_codec());
        let ffmpeg = soma_video::Ffmpeg::new(&conf.get_ffmpeg_binary(), n_threads, codec);
        let audio_dur = ffmpeg.get_audio_duration(audio_file)?;
        // 文本可能包含 Fish-Speech 情感标签，字幕中不应显示
        let plain_text = soma_tts::fishspeech_tts::strip_emotion_tags(text);
        let cues = soma_tts::edge_tts::generate_subtitle_cues_from_text(&plain_text, audio_dur);
        soma_tts::subtitle::create_subtitle_file(&cues, output_path)?;
    }

    Ok(output_path.to_string())
}

/// subtitle.generate 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubtitleGenerateInput {
    /// 音频文件路径（时长基准）
    pub audio_file: String,
    /// 对应文本（用于字幕校正 / 时间轴按文本推算）
    pub text: String,
    /// 是否生成字幕；false 时返回空字符串
    #[serde(default)]
    pub subtitle_enabled: bool,
    /// FFmpeg 线程数（Edge 模式探测时长用）
    #[serde(default)]
    pub n_threads: Option<u32>,
    /// 视频编码器（缺省用全局配置）
    #[serde(default)]
    pub video_encoder: Option<String>,
    /// 输出 SRT 路径；缺省写入产物目录 subtitle.srt
    #[serde(default)]
    pub output_path: Option<String>,
}

/// subtitle.generate 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubtitleGenerateOutput {
    /// 字幕文件路径（禁用字幕时为空字符串）
    pub subtitle_path: String,
}

/// 字幕生成：音频 → SRT 字幕文件（可脱离任务独立调用）
pub struct SubtitleGenerateFeature;

impl TypedFeature for SubtitleGenerateFeature {
    type Input = SubtitleGenerateInput;
    type Output = SubtitleGenerateOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "subtitle.generate".into(),
            name: "字幕生成".into(),
            description: "根据音频与文本生成 SRT 字幕（Whisper 或时间轴推算）".into(),
            kind: FeatureKind::Subtitle,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: SubtitleGenerateInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<SubtitleGenerateOutput, SomaError> {
        let output_path = match input.output_path {
            Some(ref p) if !p.is_empty() => p.clone(),
            _ => ctx
                .artifact_path("subtitle.srt")
                .to_string_lossy()
                .to_string(),
        };
        let subtitle_path = generate_subtitle_to(
            ctx.config(),
            &input.audio_file,
            &input.text,
            input.subtitle_enabled,
            input.n_threads.unwrap_or(2),
            input.video_encoder.as_deref(),
            &output_path,
        )?;
        if !subtitle_path.is_empty() {
            ctx.add_artifact("subtitle.srt", &subtitle_path, ArtifactKind::Subtitle);
        }
        Ok(SubtitleGenerateOutput { subtitle_path })
    }
}
