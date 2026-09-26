//! TTS 语音合成功能点（文本 → 音频文件）
//!
//! 覆盖原流水线第 5 步的音频部分，同时可作为独立完整合成入口：
//! 按 provider 选择 TTS 引擎（voice_clone / heygem / fishspeech /
//! siliconflow / elevenlabs / mimo / gemini / azure / volcengine / xfyun / edge）。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::envelope::ArtifactKind;
use crate::feature::TypedFeature;
use crate::progress::ProgressReporter;
use crate::runtime::{block_on_async, retry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::config::AppConfig;
use soma_core::error::SomaError;

/// TTS 完整合成（纯函数，供功能点与宿主兼容层复用）
///
/// 按 `tts_provider` 与语音名称路由到具体 TTS 引擎，将文本合成为音频文件。
/// 返回 (音频文件路径, 时长秒)。
#[allow(clippy::too_many_arguments)]
pub fn synthesize_to(
    conf: &AppConfig,
    text: &str,
    voice_name: &str,
    voice_rate: f32,
    tts_provider: &str,
    clone_reference_audio: &str,
    clone_reference_text: &str,
    clone_model: &str,
    merchant_id: Option<&str>,
    output_path: &str,
) -> Result<(String, f64), SomaError> {
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
    }

    // Fish-Speech 引擎支持文本内嵌情感标签（如 [whisper] [excited]），保留原文；
    // 其他引擎会把标签当普通文本朗读，合成前统一剥离。
    let use_fishspeech = tts_provider == "fishspeech"
        || soma_tts::voices::is_fishspeech_voice(voice_name);
    let effective_text = if use_fishspeech {
        text.to_string()
    } else {
        soma_tts::fishspeech_tts::strip_emotion_tags(text)
    };
    let text = effective_text.as_str();

    let result = if tts_provider == "voice_clone" {
        let vc_conf = &conf.digital_human.voice_clone;
        if vc_conf.get_ssh_host().is_empty() {
            return Err(SomaError::Config(
                "声音克隆未配置，请在 config.toml 中添加 [digital_human.voice_clone] 段".into(),
            ));
        }
        if vc_conf.get_ssh_key_path().is_empty() {
            return Err(SomaError::Config("声音克隆未配置 ssh_key_path".into()));
        }
        if vc_conf.get_remote_env_path().is_empty() {
            return Err(SomaError::Config("声音克隆未配置 remote_env_path".into()));
        }
        if vc_conf.get_remote_model_path().is_empty() {
            return Err(SomaError::Config("声音克隆未配置 remote_model_path".into()));
        }
        if clone_reference_audio.is_empty() {
            return Err(SomaError::Config(
                "使用声音克隆时必须提供参考音频（clone_reference_audio 字段）".into(),
            ));
        }
        let ref_audio = if std::path::Path::new(clone_reference_audio).is_absolute()
            || std::path::Path::new(clone_reference_audio).exists()
        {
            clone_reference_audio.to_string()
        } else {
            soma_core::utils::storage_dir("voice_clone_refs", false)
                .join(clone_reference_audio)
                .to_string_lossy()
                .to_string()
        };
        let tts = soma_tts::voice_clone_tts::VoiceCloneTts::new(
            vc_conf.clone(),
            &ref_audio,
            clone_reference_text,
            clone_model,
        );
        let max_retries = vc_conf.get_max_retries() as usize;
        retry(max_retries, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if tts_provider == "heygem" {
        let hg_conf = &conf.digital_human.heygem;
        let (reference_audio, reference_text) = get_heygem_asset(hg_conf.get_assets_dir(), merchant_id)?;
        let tts = soma_tts::heygem_tts::HeyGemTts::new(
            hg_conf.clone(),
            &reference_audio,
            &reference_text,
            merchant_id.unwrap_or(""),
        );
        let max_retries = hg_conf.get_max_retries() as usize;
        retry(max_retries, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if use_fishspeech {
        let fs_conf = &conf.fishspeech;
        if fs_conf.get_base_url().is_empty() {
            return Err(SomaError::Config(
                "Fish-Speech 未配置，请在 config.toml 中添加 [fishspeech] 段".into(),
            ));
        }
        let tts = soma_tts::fishspeech_tts::FishspeechTts::from_config(fs_conf);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if soma_tts::voices::is_siliconflow_voice(voice_name) {
        let sf_key = conf.siliconflow.api_key.as_deref().unwrap_or("");
        let tts = soma_tts::siliconflow_tts::SiliconflowTts::new(sf_key);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if soma_tts::voices::is_elevenlabs_voice(voice_name) {
        let el_key = conf.elevenlabs.api_key.as_deref().unwrap_or("");
        let el_model = conf.elevenlabs.model_id.as_deref().unwrap_or("eleven_multilingual_v2");
        let tts = soma_tts::elevenlabs_tts::ElevenlabsTts::new(el_key, el_model);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if soma_tts::voices::is_mimo_voice(voice_name) {
        let mimo_key = conf.app.mimo_api_key.as_deref().unwrap_or("");
        let mimo_base = conf.app.mimo_base_url.as_deref().unwrap_or("");
        let mimo_model = conf.app.mimo_tts_model_name.as_deref().unwrap_or("");
        let mimo_style = conf.app.mimo_tts_style_prompt.as_deref().unwrap_or("");
        let tts = soma_tts::mimo_tts::MimoTts::new(mimo_key, mimo_base, mimo_model, mimo_style);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if soma_tts::voices::is_gemini_voice(voice_name) {
        let gemini_key = conf.app.gemini_api_key.as_deref().unwrap_or("");
        let gemini_model = conf.app.gemini_model_name.as_deref().unwrap_or("gemini-2.5-flash-preview-tts");
        let tts = soma_tts::gemini_tts::GeminiTts::new(gemini_key, "", gemini_model);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if soma_tts::voices::is_azure_voice(voice_name) {
        let azure_key = conf.azure.speech_key.as_deref().unwrap_or("");
        let azure_region = conf.azure.speech_region.as_deref().unwrap_or("eastasia");
        let tts = soma_tts::azure_tts::AzureTts::new(azure_key, azure_region);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if soma_tts::voices::is_volcengine_voice(voice_name) {
        let volc_appid = conf.volcengine.app_id.as_deref().unwrap_or("");
        let volc_token = conf.volcengine.access_token.as_deref().unwrap_or("");
        let volc_cluster = conf.volcengine.cluster.as_deref().unwrap_or("volcano_tts");
        let tts = soma_tts::volcengine_tts::VolcengineTts::new(volc_appid, volc_token, volc_cluster);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else if soma_tts::voices::is_xfyun_voice(voice_name) {
        let xfyun_appid = conf.xfyun.app_id.as_deref().unwrap_or("");
        let xfyun_key = conf.xfyun.api_key.as_deref().unwrap_or("");
        let xfyun_secret = conf.xfyun.api_secret.as_deref().unwrap_or("");
        let tts = soma_tts::xfyun_tts::XfyunTts::new(xfyun_appid, xfyun_key, xfyun_secret);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    } else {
        let tts = soma_tts::edge_tts::EdgeTts::new(conf.get_edge_tts_timeout());
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(
                &tts, text, voice_name, voice_rate, std::path::Path::new(output_path),
            );
            block_on_async(fut)?
        })
    }?;

    Ok((result.audio_file, result.audio_duration))
}

/// 读取 HeyGem 商户参考音频资产（只读）
///
/// 与 soma-server `MerchantAssetStore::get_asset` 行为一致（错误信息相同），
/// 但不耦合任务存储，供功能点层独立调用。
fn get_heygem_asset(
    assets_dir: &str,
    merchant_id: Option<&str>,
) -> Result<(String, String), SomaError> {
    let merchant_id = merchant_id.ok_or_else(|| {
        SomaError::Config("HeyGem TTS 需要商户标识（merchant_id）".into())
    })?;
    soma_core::utils::validate_merchant_id(merchant_id)?;
    let path = std::path::Path::new(assets_dir).join(merchant_id).join("asset.json");
    if !path.exists() {
        return Err(SomaError::Config(format!("商户 {} 不存在", merchant_id)));
    }
    let content = std::fs::read_to_string(&path).map_err(SomaError::Io)?;
    let asset: soma_core::models::MerchantAsset = serde_json::from_str(&content)
        .map_err(|e| SomaError::Config(format!("解析 asset.json 失败: {}", e)))?;
    if asset.asset_status != soma_core::models::AssetStatus::Ready {
        return Err(SomaError::Config(format!(
            "商户 {} 模型未就绪，请先完成模型训练",
            merchant_id
        )));
    }
    Ok((asset.reference_audio, asset.reference_text))
}

/// tts.synthesize 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TtsSynthesizeInput {
    /// 待合成文本
    pub text: String,
    /// TTS 语音名称（如 "zh-CN-XiaoxiaoNeural"）
    #[serde(default)]
    pub voice_name: Option<String>,
    /// 语速倍率
    #[serde(default)]
    pub voice_rate: Option<f32>,
    /// TTS 提供者（voice_clone/heygem/fishspeech/siliconflow/elevenlabs/mimo/gemini/azure/volcengine/xfyun/edge），
    /// 也可由语音名称前缀自动识别
    #[serde(default)]
    pub tts_provider: Option<String>,
    /// 声音克隆参考音频路径（tts_provider=voice_clone 时必填）
    #[serde(default)]
    pub clone_reference_audio: Option<String>,
    /// 参考音频对应文本
    #[serde(default)]
    pub clone_reference_text: Option<String>,
    /// 克隆模型选择（gpt_sovits/cosyvoice/fish_speech）
    #[serde(default)]
    pub clone_model: Option<String>,
    /// HeyGem 商户标识（tts_provider=heygem 时必填）
    #[serde(default)]
    pub merchant_id: Option<String>,
    /// 输出音频文件路径；缺省写入产物目录 audio.mp3
    #[serde(default)]
    pub output_path: Option<String>,
}

/// tts.synthesize 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TtsSynthesizeOutput {
    /// 音频文件路径
    pub audio_file: String,
    /// 音频时长（秒）
    pub audio_duration: f64,
}

/// TTS 完整合成：文本 → 音频文件（可脱离任务独立调用）
pub struct TtsSynthesizeFeature;

impl TypedFeature for TtsSynthesizeFeature {
    type Input = TtsSynthesizeInput;
    type Output = TtsSynthesizeOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "tts.synthesize".into(),
            name: "语音合成".into(),
            description: "将文本通过 TTS 合成为音频文件，支持多提供者与声音克隆".into(),
            kind: FeatureKind::Tts,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: TtsSynthesizeInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<TtsSynthesizeOutput, SomaError> {
        let output_path = match input.output_path {
            Some(ref p) if !p.is_empty() => p.clone(),
            _ => ctx.artifact_path("audio.mp3").to_string_lossy().to_string(),
        };
        let (audio_file, audio_duration) = synthesize_to(
            ctx.config(),
            &input.text,
            input.voice_name.as_deref().unwrap_or(""),
            input.voice_rate.unwrap_or(1.0),
            input.tts_provider.as_deref().unwrap_or("edge"),
            input.clone_reference_audio.as_deref().unwrap_or(""),
            input.clone_reference_text.as_deref().unwrap_or(""),
            input.clone_model.as_deref().unwrap_or(""),
            input.merchant_id.as_deref(),
            &output_path,
        )?;
        let name = std::path::Path::new(&audio_file)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "audio.mp3".into());
        ctx.add_artifact(name, &audio_file, ArtifactKind::Audio);
        Ok(TtsSynthesizeOutput {
            audio_file,
            audio_duration,
        })
    }
}
