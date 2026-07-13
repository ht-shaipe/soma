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
use soma_core::models::{MaterialInfo, VideoAspect, AiVideoSegmentLog};

pub mod pexels;
pub mod pixabay;
pub mod coverr;
pub mod aivideo;

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

/// 从 API 密钥列表中轮换获取密钥
///
/// 使用原子计数器实现线程安全的轮询（Round-Robin）策略，
/// 每次调用返回下一个密钥，循环使用所有密钥。
/// 支持多 Key 轮换以避免单一 Key 被限流。
///
/// # 参数
/// - `keys`: API 密钥列表
///
/// # 返回
/// 当前轮次的 API 密钥，列表为空时返回错误
pub fn get_api_key(keys: &[String]) -> Result<String, SomaError> {
    if keys.is_empty() {
        return Err(SomaError::Stock("API key is not set".into()));
    }
    if keys.len() == 1 {
        return Ok(keys[0].clone());
    }
    // 线程安全的原子计数器轮换
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let idx = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % keys.len();
    Ok(keys[idx].clone())
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
/// 支持通过环境变量 HTTP_PROXY/HTTPS_PROXY 配置代理。
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

    // 构建支持系统代理的 HTTP 客户端（自动读取 HTTP_PROXY/HTTPS_PROXY/NO_PROXY 环境变量）
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let resp = client.get(video_url).send().await.map_err(|e| SomaError::Http(e.to_string()))?;
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
    let save_dir = if material_directory.is_empty() {
        soma_core::utils::storage_dir("cache_videos", true).to_string_lossy().to_string()
    } else if material_directory == "task" {
        soma_core::utils::task_dir(task_id).to_string_lossy().to_string()
    } else {
        material_directory.to_string()
    };

    // 按关键词分组搜索，保持脚本顺序
    let mut candidate_groups: Vec<Vec<MaterialInfo>> = Vec::new();
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    for term in search_terms {
        let items = search_videos(source, term, video_aspect, max_clip_duration, pexels_keys, pixabay_keys, coverr_keys).await?;
        let mut group = Vec::new();
        for item in items {
            if seen_urls.insert(item.url.clone()) {
                group.push(item);
            }
        }
        if !group.is_empty() {
            candidate_groups.push(group);
        }
    }

    // 轮询下载：第1轮取每个关键词的第1个候选，第2轮取第2个...
    // 确保素材按脚本关键词顺序排列
    let mut video_paths: Vec<String> = Vec::new();
    let mut total_duration: f64 = 0.0;
    let mut candidate_index: usize = 0;

    while !candidate_groups.is_empty() && total_duration < audio_duration {
        let mut has_candidate = false;
        for group in &candidate_groups {
            if candidate_index >= group.len() {
                continue;
            }
            has_candidate = true;
            let item = &group[candidate_index];
            match save_video(&item.url, &save_dir).await {
                Ok(path) => {
                    if validate_video_file(&path) {
                        let seconds = max_clip_duration.min(item.duration as u32);
                        total_duration += seconds as f64;
                        video_paths.push(path);
                    } else {
                        log!("invalid video file, removing: {}", item.url);
                        let _ = std::fs::remove_file(&path);
                    }
                }
                Err(e) => {
                    log!("failed to download video: {:?}", e);
                }
            }
        }
        if !has_candidate {
            break;
        }
        candidate_index += 1;
    }

    Ok(video_paths)
}

/// AI 视频生成：根据脚本段落提示词调用 AI 视频生成 API
///
/// 优化策略：
/// 1. 串行提交 + 间隔2秒（避免被 API 限流）
/// 2. 提交后立即开始并发轮询（流水线化，不等全部提交完）
/// 3. 动态轮询间隔（前30秒每3秒，之后每8秒）
/// 4. 提交失败重试3次
/// 5. 并发下载完成的视频
/// 6. 部分失败容忍（尽可能返回已成功的视频）
pub async fn generate_ai_videos(
    task_id: &str,
    search_terms: &[String],
    source: &str,
    video_aspect: &VideoAspect,
    clip_duration: u32,
    conf: &soma_core::config::AppConfig,
    portrait_image: Option<&str>,
) -> Result<(Vec<String>, Vec<AiVideoSegmentLog>), SomaError> {
    let provider = aivideo::create_provider(source, conf)?;
    let provider = std::sync::Arc::new(provider);

    let aspect_str = match video_aspect {
        VideoAspect::Landscape => "16:9",
        VideoAspect::Portrait => "9:16",
        VideoAspect::Square => "1:1",
    };
    let timeout = conf.app.video_gen_timeout.unwrap_or(300);

    let save_dir = soma_core::utils::task_dir(task_id).to_string_lossy().to_string();
    let dir = std::path::Path::new(&save_dir);
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(SomaError::Io)?;
    }

    let total = search_terms.len();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout);
    let start = std::time::Instant::now();

    let mut seg_logs: Vec<AiVideoSegmentLog> = search_terms.iter().enumerate().map(|(i, term)| {
        AiVideoSegmentLog {
            scene_id: (i + 1) as u32,
            prompt: term.clone(),
            status: "pending".to_string(),
            message: None,
        }
    }).collect();

    // 阶段1：串行提交任务（间隔2秒避免限流），带重试
    let mut pending: Vec<(usize, String)> = Vec::new(); // (idx, ai_task_id)
    let mut failed_indices: Vec<usize> = Vec::new();

    for (idx, term) in search_terms.iter().enumerate() {
        if std::time::Instant::now() >= deadline {
            log!("AI视频生成: 提交阶段超时，已提交 {}/{}", idx, total);
            failed_indices.extend(idx..total);
            break;
        }

        let params = aivideo::VideoGenParams {
            prompt: term.clone(),
            aspect_ratio: aspect_str.to_string(),
            duration: clip_duration,
            model: None,
            image_url: portrait_image.map(|s| s.to_string()),
        };

        log!("AI视频生成 提交任务 [{}/{}]: {} ...", idx + 1, total, term);

        let mut submitted = false;
        for attempt in 1..=3 {
            match provider.create_task(&params).await {
                Ok(ai_task_id) => {
                    log!("AI视频生成 任务已提交 [{}/{}], task_id={}", idx + 1, total, ai_task_id);
                    seg_logs[idx].status = "submitted".to_string();
                    seg_logs[idx].message = Some(format!("task_id={}", ai_task_id));
                    pending.push((idx, ai_task_id));
                    submitted = true;
                    break;
                }
                Err(e) => {
                    log!("AI视频生成 提交失败 [{}/{}] 第{}次: {:?}", idx + 1, total, attempt, e);
                    if attempt < 3 {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }

        if !submitted {
            log!("AI视频生成 提交彻底失败 [{}/{}]", idx + 1, total);
            seg_logs[idx].status = "failed".to_string();
            seg_logs[idx].message = Some("提交3次均失败".to_string());
            failed_indices.push(idx);
        }

        // 提交间隔2秒，避免限流（最后一个不等）
        if idx + 1 < total && submitted {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    if pending.is_empty() {
        return Err(SomaError::VideoGen("所有AI视频任务提交失败".into()));
    }

    log!("AI视频生成: 提交完成 {}/{}, 开始并发轮询...", pending.len(), total);

    // 阶段2：并发轮询所有任务状态
    let mut completed: Vec<(usize, aivideo::VideoGenStatus)> = Vec::new();

    while !pending.is_empty() && std::time::Instant::now() < deadline {
        let elapsed_secs = start.elapsed().as_secs();
        let interval = if elapsed_secs < 30 { 3 } else { 8 };

        let poll_futs: Vec<_> = pending.iter()
            .map(|(idx, ai_task_id)| {
                let p = provider.clone();
                let tid = ai_task_id.clone();
                let idx_val = *idx;
                async move {
                    let status = p.query_task(&tid).await;
                    (idx_val, tid, status)
                }
            })
            .collect();

        let poll_results = futures::future::join_all(poll_futs).await;

        let mut still_pending = Vec::new();
        let mut done_count = completed.len();
        for (idx, ai_task_id, result) in poll_results {
            match result {
                Ok(status) => match &status {
                    aivideo::VideoGenStatus::Processing => {
                        still_pending.push((idx, ai_task_id));
                        if seg_logs[idx].status != "processing" {
                            seg_logs[idx].status = "processing".to_string();
                        }
                    }
                    aivideo::VideoGenStatus::Success { .. } => {
                        done_count += 1;
                        log!("AI视频生成 完成 [{}/{}]", idx + 1, total);
                        seg_logs[idx].status = "success".to_string();
                        seg_logs[idx].message = None;
                        completed.push((idx, status));
                    }
                    aivideo::VideoGenStatus::Failed { message } => {
                        done_count += 1;
                        log!("AI视频生成 失败 [{}/{}]: {}", idx + 1, total, message);
                        seg_logs[idx].status = "failed".to_string();
                        seg_logs[idx].message = Some(message.clone());
                        completed.push((idx, status));
                    }
                },
                Err(e) => {
                    done_count += 1;
                    log!("AI视频 轮询失败 [{}/{}]: {:?}", idx + 1, total, e);
                    seg_logs[idx].status = "failed".to_string();
                    seg_logs[idx].message = Some(format!("{:?}", e));
                    completed.push((idx, aivideo::VideoGenStatus::Failed { message: format!("{:?}", e) }));
                }
            }
        }

        pending = still_pending;
        if !pending.is_empty() {
            log!("AI视频生成 轮询中: {}/{} 完成, {} 等待中...", done_count, total - failed_indices.len(), pending.len());
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    }

    // 超时的任务标记
    for (idx, _) in &pending {
        log!("AI视频生成 超时 [{}/{}]", idx + 1, total);
        seg_logs[*idx].status = "timeout".to_string();
        seg_logs[*idx].message = Some("生成超时".to_string());
        completed.push((*idx, aivideo::VideoGenStatus::Failed { message: "生成超时".into() }));
    }

    // 按 idx 排序
    completed.sort_by_key(|(idx, _)| *idx);

    // 阶段3：并发下载视频
    let mut download_futs = Vec::new();
    for (idx, status) in &completed {
        if let aivideo::VideoGenStatus::Success { video_urls } = status {
            if let Some(url) = video_urls.first() {
                let filename = format!("ai-{}-{}.mp4", idx + 1, soma_core::utils::md5(url));
                let save_path = dir.join(&filename).to_string_lossy().to_string();
                let url_clone = url.clone();
                let idx_val = *idx;
                download_futs.push(async move {
                    let result = aivideo::download_video_common(&url_clone, &save_path).await;
                    (idx_val, result)
                });
            }
        }
    }

    let download_results: Vec<(usize, Result<String, SomaError>)> =
        futures::future::join_all(download_futs).await;

    let mut indexed_paths: Vec<(usize, String)> = Vec::new();
    let mut download_failures = Vec::new();
    for (idx, result) in download_results {
        match result {
            Ok(path) => {
                log!("AI视频下载成功: {}", path);
                indexed_paths.push((idx, path));
            }
            Err(e) => {
                log!("AI视频下载失败 [{}]: {:?}", idx + 1, e);
                download_failures.push(idx);
            }
        }
    }

    // 部分失败容忍：如果有成功的视频就返回，而不是一个失败全部放弃
    if indexed_paths.is_empty() {
        let first_err = completed.iter()
            .filter_map(|(_, s)| if let aivideo::VideoGenStatus::Failed { message } = s { Some(message.clone()) } else { None })
            .next()
            .unwrap_or_else(|| "所有AI视频生成失败".into());
        return Err(SomaError::VideoGen(first_err));
    }

    if !failed_indices.is_empty() || !download_failures.is_empty() || pending.len() > 0 {
        log!("AI视频生成: 部分失败，成功 {}/{} 个视频", indexed_paths.len(), total);
    }

    indexed_paths.sort_by_key(|(idx, _)| *idx);
    Ok((indexed_paths.into_iter().map(|(_, path)| path).collect(), seg_logs))
}

/// 验证视频文件是否有效可播放
///
/// 通过 ffprobe 检查视频的时长和帧率，确认文件可正常解码。
/// 无效文件（时长为0或无法读取）返回 false。
fn validate_video_file(path: &str) -> bool {
    let output = std::process::Command::new("ffprobe")
        .args(&["-v", "error", "-show_entries", "format=duration:stream=r_frame_rate", "-of", "default=noprint_wrappers=1", path])
        .output();
    match output {
        Ok(out) => {
            let info = String::from_utf8_lossy(&out.stdout);
            // 检查有时长信息且非零
            info.contains("duration=") && !info.contains("duration=N/A") && !info.contains("duration=0")
        }
        Err(_) => false,
    }
}
