//! 素材生成功能点（关键词 → 素材文件列表）
//!
//! 覆盖原流水线第 4 步，统一三种来源：
//! - 本地素材（`video_source=local`，预处理图片转视频）
//! - AI 视频生成（`cogvideox` / `kling` / `minimax`，或人像图生视频）
//! - 在线素材下载（pexels / pixabay / coverr）

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::envelope::ArtifactKind;
use crate::feature::TypedFeature;
use crate::progress::{FeatureProgress, ProgressReporter};
use crate::runtime::block_on_async;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::config::AppConfig;
use soma_core::error::SomaError;
use soma_core::models::{AiVideoSegmentLog, MaterialInfo, VideoAspect};

/// 素材生成请求（纯函数参数，供功能点与宿主兼容层复用）
pub struct MaterialRequest<'a> {
    /// 素材搜索关键词 / 视觉提示词
    pub terms: &'a [String],
    /// 素材来源（pexels/pixabay/coverr/local/cogvideox/kling/minimax），缺省 pexels
    pub source: Option<&'a str>,
    /// 画幅比例（"16:9"/"9:16"/"1:1"），缺省 9:16
    pub aspect: Option<&'a str>,
    /// 单片段时长（秒），缺省 4
    pub clip_duration: Option<u32>,
    /// 人像图片路径或 URL（图生视频模式）
    pub portrait_image: Option<&'a str>,
    /// 本地素材列表（source=local 时使用）
    pub local_materials: &'a [MaterialInfo],
    /// 音频总时长（秒），决定在线素材的下载总量
    pub audio_duration: f64,
    /// 生成视频份数（在线素材下载总量 = audio_duration × video_count）
    pub video_count: u32,
    /// AI 视频保存目录；缺省 cache_videos
    pub ai_output_dir: Option<String>,
    /// 在线素材下载目录；缺省 cache_videos
    pub download_dir: Option<String>,
    /// FFmpeg 线程数（本地素材预处理用），缺省 2
    pub n_threads: Option<u32>,
    /// 视频编码器（本地素材预处理用），缺省用全局配置
    pub video_encoder: Option<&'a str>,
}

/// 素材生成（纯函数，供功能点与宿主兼容层复用）
///
/// 返回 (素材文件路径列表, AI 视频分段日志)。
pub fn generate_materials(
    conf: &AppConfig,
    req: &MaterialRequest,
) -> Result<(Vec<String>, Vec<AiVideoSegmentLog>), SomaError> {
    let source = req.source.unwrap_or("pexels");
    let aspect = req
        .aspect
        .and_then(VideoAspect::from_str)
        .unwrap_or(VideoAspect::Portrait);
    let clip_dur = req.clip_duration.unwrap_or(4);
    let default_dir = || {
        soma_core::utils::storage_dir("cache_videos", true)
            .to_string_lossy()
            .to_string()
    };

    // 人像图生视频（AI 视频源）
    if req.portrait_image.is_some() && source != "local" {
        let ai_source = if source == "cogvideox" || source == "kling" || source == "minimax" {
            source.to_string()
        } else {
            resolve_portrait_ai_source(conf)
        };
        let portrait_url = resolve_portrait_url(req.portrait_image.unwrap_or(""), conf);
        let save_dir = req.ai_output_dir.clone().unwrap_or_else(default_dir);
        let (paths, logs) = block_on_async(soma_stock::generate_ai_videos(
            &save_dir,
            req.terms,
            &ai_source,
            &aspect,
            clip_dur,
            conf,
            Some(portrait_url.as_str()),
        ))??;
        return Ok((paths, logs));
    }

    // 本地素材预处理
    if source == "local" {
        let codec = req.video_encoder.unwrap_or(conf.get_video_codec());
        let ffmpeg =
            soma_video::Ffmpeg::new(&conf.get_ffmpeg_binary(), req.n_threads.unwrap_or(2), codec);
        let materials =
            ffmpeg.preprocess_local_materials(req.local_materials, clip_dur, &aspect)?;
        return Ok((materials, vec![]));
    }

    // AI 视频生成（文生视频）
    if source == "cogvideox" || source == "kling" || source == "minimax" {
        let save_dir = req.ai_output_dir.clone().unwrap_or_else(default_dir);
        let (paths, logs) = block_on_async(soma_stock::generate_ai_videos(
            &save_dir, req.terms, source, &aspect, clip_dur, conf, None,
        ))??;
        return Ok((paths, logs));
    }

    // 在线素材下载
    let pexels_keys = conf.app.pexels_api_keys.clone().unwrap_or_default();
    let pixabay_keys = conf.app.pixabay_api_keys.clone().unwrap_or_default();
    let coverr_keys = conf.app.coverr_api_keys.clone().unwrap_or_default();
    let download_dir = req.download_dir.clone().unwrap_or_else(default_dir);
    let paths = block_on_async(soma_stock::download_videos(
        &download_dir,
        req.terms,
        source,
        &aspect,
        req.audio_duration * req.video_count as f64,
        clip_dur,
        &pexels_keys,
        &pixabay_keys,
        &coverr_keys,
    ))??;
    Ok((paths, vec![]))
}

/// 根据已配置的 AI 视频密钥自动选择人像视频生成源
///
/// 与原流水线 resolve_portrait_ai_source 行为一致：优先智谱（cogvideox），
/// 其次可灵（kling），再次 MiniMax，都未配置时回退 cogvideox。
fn resolve_portrait_ai_source(conf: &AppConfig) -> String {
    let zhipu_key = conf
        .app
        .zhipu_video_api_key
        .as_deref()
        .or(conf.app.zhipu_api_key.as_deref())
        .unwrap_or("");
    if !zhipu_key.is_empty() {
        return "cogvideox".to_string();
    }
    let kling_ak = conf.app.kling_access_key.as_deref().unwrap_or("");
    let kling_sk = conf.app.kling_secret_key.as_deref().unwrap_or("");
    if !kling_ak.is_empty() && !kling_sk.is_empty() {
        return "kling".to_string();
    }
    let minimax_key = conf.app.minimax_video_api_key.as_deref().unwrap_or("");
    if !minimax_key.is_empty() {
        return "minimax".to_string();
    }
    "cogvideox".to_string()
}

/// 将本地人像路径转换为可被 AI 视频服务访问的 URL
///
/// 已是 HTTP URL 直接返回；本地路径基于 endpoint 与 storage 目录推导公网地址。
fn resolve_portrait_url(path: &str, conf: &AppConfig) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    let endpoint = conf.get_endpoint();
    if endpoint.is_empty() {
        return path.to_string();
    }
    let storage_path = conf.get_storage_path();
    let storage_abs = std::path::Path::new(storage_path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(storage_path));
    let portrait_abs = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(path));
    if let Ok(rel) = portrait_abs.strip_prefix(&storage_abs) {
        let rel_str = rel.to_string_lossy();
        return format!("{}/storage/{}", endpoint.trim_end_matches('/'), rel_str);
    }
    path.to_string()
}

/// material.generate 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaterialGenerateInput {
    /// 素材搜索关键词 / 视觉提示词（与分镜顺序对应）
    pub terms: Vec<String>,
    /// 素材来源，缺省 pexels
    #[serde(default)]
    pub source: Option<String>,
    /// 画幅比例（"16:9"/"9:16"/"1:1"）
    #[serde(default)]
    pub aspect: Option<String>,
    /// 单片段时长（秒）
    #[serde(default)]
    pub clip_duration: Option<u32>,
    /// 人像图片路径或 URL（图生视频模式）
    #[serde(default)]
    pub portrait_image: Option<String>,
    /// 本地素材列表（source=local 时使用）
    #[serde(default)]
    pub local_materials: Vec<MaterialInfo>,
    /// 音频总时长（秒）
    #[serde(default)]
    pub audio_duration: f64,
    /// 视频份数
    #[serde(default)]
    pub video_count: Option<u32>,
    /// AI 视频保存目录；缺省 cache_videos
    #[serde(default)]
    pub ai_output_dir: Option<String>,
    /// 在线素材下载目录；缺省 cache_videos
    #[serde(default)]
    pub download_dir: Option<String>,
    /// FFmpeg 线程数（本地素材预处理用）
    #[serde(default)]
    pub n_threads: Option<u32>,
    /// 视频编码器（本地素材预处理用）
    #[serde(default)]
    pub video_encoder: Option<String>,
}

/// material.generate 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MaterialGenerateOutput {
    /// 素材文件路径列表
    pub materials: Vec<String>,
    /// AI 视频逐段生成日志（非 AI 来源时为空）
    pub ai_video_logs: Vec<AiVideoSegmentLog>,
}

/// 素材生成：关键词 → 素材文件（本地 / 在线下载 / AI 视频生成）
pub struct MaterialGenerateFeature;

impl TypedFeature for MaterialGenerateFeature {
    type Input = MaterialGenerateInput;
    type Output = MaterialGenerateOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "material.generate".into(),
            name: "素材生成".into(),
            description: "根据关键词获取视频素材：本地预处理、在线下载或 AI 视频生成".into(),
            kind: FeatureKind::Material,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: MaterialGenerateInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<MaterialGenerateOutput, SomaError> {
        let req = MaterialRequest {
            terms: &input.terms,
            source: input.source.as_deref(),
            aspect: input.aspect.as_deref(),
            clip_duration: input.clip_duration,
            portrait_image: input.portrait_image.as_deref(),
            local_materials: &input.local_materials,
            audio_duration: input.audio_duration,
            video_count: input.video_count.unwrap_or(1),
            ai_output_dir: input.ai_output_dir.clone(),
            download_dir: input.download_dir.clone(),
            n_threads: input.n_threads,
            video_encoder: input.video_encoder.as_deref(),
        };
        let (materials, ai_video_logs) = generate_materials(ctx.config(), &req)?;
        for path in &materials {
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            ctx.add_artifact(name, path, ArtifactKind::Video);
        }
        Ok(MaterialGenerateOutput {
            materials,
            ai_video_logs,
        })
    }
}

// ============ material.search：素材搜索（只搜不下载） ============

/// material.search 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaterialSearchInput {
    /// 搜索关键词
    pub keyword: String,
    /// 素材源（pexels/pixabay/coverr），缺省 pexels
    #[serde(default)]
    pub source: Option<String>,
    /// 画幅比例（"16:9"/"9:16"/"1:1"）
    #[serde(default)]
    pub aspect: Option<String>,
    /// 视频最小时长（秒）
    #[serde(default)]
    pub min_duration: Option<u32>,
}

/// material.search 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MaterialSearchOutput {
    /// 搜索到的素材信息列表（含来源与下载 URL，不落盘）
    pub materials: Vec<MaterialInfo>,
}

/// 素材搜索：关键词 → 素材信息列表（不下载，供预览与挑选）
pub struct MaterialSearchFeature;

impl TypedFeature for MaterialSearchFeature {
    type Input = MaterialSearchInput;
    type Output = MaterialSearchOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "material.search".into(),
            name: "素材搜索".into(),
            description: "按关键词在 Pexels/Pixabay/Coverr 搜索视频素材，返回候选列表".into(),
            kind: FeatureKind::Material,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: MaterialSearchInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<MaterialSearchOutput, SomaError> {
        let conf = ctx.config();
        let source = input.source.as_deref().unwrap_or("pexels");
        let aspect = input
            .aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait);
        let min_duration = input.min_duration.unwrap_or(0);
        let pexels_keys = conf.app.pexels_api_keys.clone().unwrap_or_default();
        let pixabay_keys = conf.app.pixabay_api_keys.clone().unwrap_or_default();
        let coverr_keys = conf.app.coverr_api_keys.clone().unwrap_or_default();

        // 无 Key（含 [""] 空串占位）时给出明确引导，而非静默返回空候选列表
        let no_key = match source {
            "pixabay" => pixabay_keys.iter().all(|k| k.trim().is_empty()),
            "coverr" => coverr_keys.iter().all(|k| k.trim().is_empty()),
            _ => pexels_keys.iter().all(|k| k.trim().is_empty()),
        };
        if no_key {
            return Err(SomaError::Config(format!(
                "素材源 {} 的 API Key 未配置，请在 系统设置 → 素材源 API Key 中填写",
                source
            )));
        }

        let materials = block_on_async(soma_stock::search_videos(
            source,
            &input.keyword,
            &aspect,
            min_duration,
            &pexels_keys,
            &pixabay_keys,
            &coverr_keys,
        ))??;
        Ok(MaterialSearchOutput { materials })
    }
}

// ============ material.download：指定 URL 下载 ============

/// material.download 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaterialDownloadInput {
    /// 素材视频 URL 列表（可来自 material.search 的结果挑选）
    pub urls: Vec<String>,
    /// 保存目录；缺省写入产物目录
    #[serde(default)]
    pub save_dir: Option<String>,
}

/// material.download 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MaterialDownloadOutput {
    /// 成功下载的本地文件路径列表（按输入顺序，失败的跳过）
    pub files: Vec<String>,
    /// 失败的 URL 列表
    pub failed: Vec<String>,
}

/// 素材下载：按 URL 列表逐个下载视频文件（与搜索功能点衔接）
pub struct MaterialDownloadFeature;

impl TypedFeature for MaterialDownloadFeature {
    type Input = MaterialDownloadInput;
    type Output = MaterialDownloadOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "material.download".into(),
            name: "素材下载".into(),
            description: "将指定 URL 的视频素材下载到本地（可与素材搜索结果衔接）".into(),
            kind: FeatureKind::Material,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: MaterialDownloadInput,
        progress: &dyn ProgressReporter,
    ) -> Result<MaterialDownloadOutput, SomaError> {
        let save_dir = match input.save_dir {
            Some(ref d) if !d.is_empty() => d.clone(),
            _ => ctx.work_dir().to_string_lossy().to_string(),
        };
        std::fs::create_dir_all(&save_dir).map_err(SomaError::Io)?;

        let total = input.urls.len();
        let mut files = Vec::new();
        let mut failed = Vec::new();
        for (i, url) in input.urls.iter().enumerate() {
            progress.report(
                FeatureProgress::new(
                    ctx.feature_id(),
                    ctx.run_id(),
                    (i as u32 * 100 / total.max(1) as u32).min(99),
                )
                .with_step("download")
                .with_message(format!("下载 {}/{}: {}", i + 1, total, url)),
            );
            let downloaded: Result<String, SomaError> =
                block_on_async(soma_stock::save_video(url, &save_dir))?;
            match downloaded {
                Ok(path) => {
                    let name = std::path::Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());
                    ctx.add_artifact(name, &path, ArtifactKind::Video);
                    files.push(path);
                }
                Err(e) => {
                    log::warn!("素材下载失败 {url}: {e}");
                    failed.push(url.clone());
                }
            }
        }
        progress.report(
            FeatureProgress::new(ctx.feature_id(), ctx.run_id(), 100).with_step("download"),
        );
        Ok(MaterialDownloadOutput { files, failed })
    }
}
