/// 素材库存模块（soma-stock）
///
/// 提供视频素材的搜索、下载和管理功能，支持多个素材源：
/// - Pexels：免费视频素材平台
/// - Pixabay：免费视频素材平台
/// - Coverr：免费视频素材平台
///
/// 本模块是 soma 视频自动剪辑系统的素材采集层，
/// 负责从第三方 API 搜索视频素材并下载到本地。
#[macro_use]
extern crate tube;

use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};

pub mod pexels;
pub mod pixabay;
pub mod coverr;

/// 素材供应商特征（Trait）
///
/// 定义了所有视频素材供应商必须实现的接口，
/// 包括搜索视频功能。所有具体的供应商（Pexels、Pixabay、Coverr）
/// 都需要实现此特征。
#[async_trait(?Send)]
pub trait SomaStockProvider: Send + Sync {
    /// 根据关键词搜索视频素材
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    /// - `video_aspect`: 视频宽高比要求
    /// - `min_duration`: 视频最小时长（秒）
    ///
    /// # 返回
    /// 匹配搜索条件的素材信息列表，失败时返回 SomaError
    async fn search(&self, keyword: &str, video_aspect: &VideoAspect, min_duration: u32) -> Result<Vec<MaterialInfo>, SomaError>;
}

/// 从 API 密钥列表中获取第一个可用密钥
///
/// 大部分素材 API 需要密钥认证，此函数从密钥列表中
/// 选取第一个密钥用于请求。
///
/// # 参数
/// - `keys`: API 密钥列表
///
/// # 返回
/// 第一个可用的 API 密钥，列表为空时返回错误
pub fn get_api_key(keys: &[String]) -> Result<String, SomaError> {
    if keys.is_empty() {
        return Err(SomaError::Stock("API key is not set".into()));
    }
    Ok(keys[0].clone())
}

/// 根据素材源搜索视频
///
/// 根据指定的素材来源（source）创建对应的供应商实例，
/// 并调用其搜索接口查询视频素材。
///
/// # 参数
/// - `source`: 素材源名称，支持 "pixabay"、"coverr"，默认使用 pexels
/// - `keyword`: 搜索关键词
/// - `video_aspect`: 视频宽高比要求
/// - `min_duration`: 视频最小时长（秒）
/// - `pexels_keys`: Pexels API 密钥列表
/// - `pixabay_keys`: Pixabay API 密钥列表
/// - `coverr_keys`: Coverr API 密钥列表
///
/// # 返回
/// 搜索到的素材信息列表，失败时返回 SomaError
pub async fn search_videos(
    source: &str,
    keyword: &str,
    video_aspect: &VideoAspect,
    min_duration: u32,
    pexels_keys: &[String],
    pixabay_keys: &[String],
    coverr_keys: &[String],
) -> Result<Vec<MaterialInfo>, SomaError> {
    match source {
        "pixabay" => {
            let provider = pixabay::Pixabay::new(pixabay_keys);
            provider.search(keyword, video_aspect, min_duration).await
        }
        "coverr" => {
            let provider = coverr::Coverr::new(coverr_keys);
            provider.search(keyword, video_aspect, min_duration).await
        }
        // 默认使用 Pexels 作为素材源
        _ => {
            let provider = pexels::Pexels::new(pexels_keys);
            provider.search(keyword, video_aspect, min_duration).await
        }
    }
}

/// 保存视频到本地目录
///
/// 下载指定 URL 的视频文件并保存到本地目录。
/// 使用 URL 的 MD5 哈希作为文件名，避免重复下载。
/// 如果文件已存在且非空，则直接返回路径，不再重复下载。
///
/// # 参数
/// - `video_url`: 视频下载地址
/// - `save_dir`: 本地保存目录路径
///
/// # 返回
/// 保存后的视频文件绝对路径，失败时返回 SomaError
pub async fn save_video(video_url: &str, save_dir: &str) -> Result<String, SomaError> {
    let dir = std::path::Path::new(save_dir);
    // 确保保存目录存在，不存在则递归创建
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(SomaError::Io)?;
    }
    // 去除 URL 查询参数，用纯 URL 计算 MD5 作为文件标识
    let url_without_query = video_url.split('?').next().unwrap_or(video_url);
    let hash = soma_core::utils::md5(url_without_query);
    let video_id = format!("vid-{}", hash);
    let video_path = dir.join(format!("{}.mp4", video_id));
    let video_path_str = video_path.to_string_lossy().to_string();

    // 文件已存在且非空，跳过下载（缓存命中）
    if video_path.exists() && video_path.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
        return Ok(video_path_str);
    }

    // 发起 HTTP 请求下载视频
    let resp = reqwest::get(video_url).await.map_err(|e| SomaError::Http(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(SomaError::Stock(format!("download failed: status {}", resp.status())));
    }
    let bytes = resp.bytes().await.map_err(|e| SomaError::Http(e.to_string()))?;
    std::fs::write(&video_path, &bytes).map_err(SomaError::Io)?;

    // 验证下载后的文件是否有效（非空）
    if video_path.exists() && video_path.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
        Ok(video_path_str)
    } else {
        Err(SomaError::Stock("downloaded file is empty".into()))
    }
}

/// 批量搜索并下载视频素材
///
/// 根据多个搜索关键词依次搜索视频素材，去重后按顺序下载，
/// 直到下载的视频总时长达到音频时长要求为止。
/// 此函数是视频素材采集的入口，供上层剪辑流程调用。
///
/// # 参数
/// - `task_id`: 任务 ID，用于确定素材保存路径
/// - `search_terms`: 搜索关键词列表，每个关键词独立搜索
/// - `source`: 素材源名称（"pixabay"、"coverr" 或默认 pexels）
/// - `video_aspect`: 视频宽高比要求
/// - `audio_duration`: 音频总时长（秒），视频素材总时长需达到此值
/// - `max_clip_duration`: 单个视频片段最大时长（秒）
/// - `pexels_keys`: Pexels API 密钥列表
/// - `pixabay_keys`: Pixabay API 密钥列表
/// - `coverr_keys`: Coverr API 密钥列表
/// - `material_directory`: 素材保存目录，空字符串使用默认缓存目录，"task" 使用任务专属目录
///
/// # 返回
/// 下载成功的视频文件路径列表，按下载顺序排列
pub async fn download_videos(
    task_id: &str,
    search_terms: &[String],
    source: &str,
    video_aspect: &VideoAspect,
    audio_duration: f64,
    max_clip_duration: u32,
    pexels_keys: &[String],
    pixabay_keys: &[String],
    coverr_keys: &[String],
    material_directory: &str,
) -> Result<Vec<String>, SomaError> {
    // 确定素材保存目录：空字符串使用默认缓存目录，"task" 使用任务目录，否则使用指定路径
    let save_dir = if material_directory.is_empty() {
        soma_core::utils::storage_dir("cache_videos", true).to_string_lossy().to_string()
    } else if material_directory == "task" {
        soma_core::utils::task_dir(task_id).to_string_lossy().to_string()
    } else {
        material_directory.to_string()
    };

    let mut valid_items: Vec<MaterialInfo> = Vec::new();
    // 用于 URL 去重，避免同一视频被重复收录
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 逐个关键词搜索，合并结果并按 URL 去重
    for term in search_terms {
        let items = search_videos(source, term, video_aspect, max_clip_duration, pexels_keys, pixabay_keys, coverr_keys).await?;
        for item in items {
            if seen_urls.insert(item.url.clone()) {
                valid_items.push(item);
            }
        }
    }

    let mut video_paths: Vec<String> = Vec::new();
    let mut total_duration: f64 = 0.0;

    // 按顺序下载视频，直到总时长达到音频时长要求
    for item in &valid_items {
        if total_duration >= audio_duration {
            break;
        }
        match save_video(&item.url, &save_dir).await {
            Ok(path) => {
                // 单个片段时长不超过最大限制
                let seconds = max_clip_duration.min(item.duration as u32);
                total_duration += seconds as f64;
                video_paths.push(path);
            }
            Err(e) => {
                // 下载失败时记录日志，继续下载下一个
                log!("failed to download video: {:?}", e);
            }
        }
    }

    Ok(video_paths)
}
