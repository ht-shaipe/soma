#[macro_use]
extern crate tube;

use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};

pub mod pexels;
pub mod pixabay;
pub mod coverr;

#[async_trait(?Send)]
pub trait SomaStockProvider: Send + Sync {
    async fn search(&self, keyword: &str, video_aspect: &VideoAspect, min_duration: u32) -> Result<Vec<MaterialInfo>, SomaError>;
}

pub fn get_api_key(keys: &[String]) -> Result<String, SomaError> {
    if keys.is_empty() {
        return Err(SomaError::Stock("API key is not set".into()));
    }
    Ok(keys[0].clone())
}

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
        _ => {
            let provider = pexels::Pexels::new(pexels_keys);
            provider.search(keyword, video_aspect, min_duration).await
        }
    }
}

pub async fn save_video(video_url: &str, save_dir: &str) -> Result<String, SomaError> {
    let dir = std::path::Path::new(save_dir);
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(SomaError::Io)?;
    }
    let url_without_query = video_url.split('?').next().unwrap_or(video_url);
    let hash = soma_core::utils::md5(url_without_query);
    let video_id = format!("vid-{}", hash);
    let video_path = dir.join(format!("{}.mp4", video_id));
    let video_path_str = video_path.to_string_lossy().to_string();

    if video_path.exists() && video_path.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
        return Ok(video_path_str);
    }

    let resp = reqwest::get(video_url).await.map_err(|e| SomaError::Http(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(SomaError::Stock(format!("download failed: status {}", resp.status())));
    }
    let bytes = resp.bytes().await.map_err(|e| SomaError::Http(e.to_string()))?;
    std::fs::write(&video_path, &bytes).map_err(SomaError::Io)?;

    if video_path.exists() && video_path.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
        Ok(video_path_str)
    } else {
        Err(SomaError::Stock("downloaded file is empty".into()))
    }
}

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
    let save_dir = if material_directory.is_empty() {
        soma_core::utils::storage_dir("cache_videos", true).to_string_lossy().to_string()
    } else if material_directory == "task" {
        soma_core::utils::task_dir(task_id).to_string_lossy().to_string()
    } else {
        material_directory.to_string()
    };

    let mut valid_items: Vec<MaterialInfo> = Vec::new();
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

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

    for item in &valid_items {
        if total_duration >= audio_duration {
            break;
        }
        match save_video(&item.url, &save_dir).await {
            Ok(path) => {
                let seconds = max_clip_duration.min(item.duration as u32);
                total_duration += seconds as f64;
                video_paths.push(path);
            }
            Err(e) => {
                log!("failed to download video: {:?}", e);
            }
        }
    }

    Ok(video_paths)
}
