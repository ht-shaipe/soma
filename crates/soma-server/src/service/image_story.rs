/// 图片故事视频生成流水线
///
/// 处理图片故事任务的执行逻辑：
/// 1. 验证图片文件存在
/// 2. 为每个场景调用AI视频生成
/// 3. 合并所有视频片段

use soma_core::error::SomaError;
use soma_core::models::{TaskStatus, ImageStoryParams};
use crate::state;
use crate::Config;

/// 执行图片故事任务（同步入口，由 task.rs 在独立线程中调用）
pub fn run_task(task_id: &str, params: &ImageStoryParams) -> Result<(), SomaError> {
    let conf = Config::get();

    log::info!("图片故事任务 {} 开始执行，共 {} 个场景", task_id, params.scenes.len());

    state::update_image_story_task_data(task_id, Some(TaskStatus::Processing.as_i32()), Some(0), None, None);

    let task_dir = soma_core::utils::task_dir(task_id);
    std::fs::create_dir_all(&task_dir).map_err(SomaError::Io)?;

    let total_scenes = params.scenes.len();

    let mut video_paths: Vec<String> = Vec::new();

    for (index, scene) in params.scenes.iter().enumerate() {
        log::info!("图片故事任务 {} 处理场景 {}/{}", task_id, index + 1, total_scenes);

        let progress = ((index as u32 * 80) / total_scenes as u32) + 10;
        state::update_image_story_task_data(task_id, None, Some(progress), None, None);

        if !std::path::Path::new(&scene.image_path).exists() {
            return Err(SomaError::VideoGen(format!("图片文件不存在: {}", scene.image_path)));
        }

        let video_path = generate_scene_video_sync(task_id, scene, params, &conf.app)?;
        video_paths.push(video_path);
    }

    state::update_image_story_task_data(task_id, None, Some(80), None, None);

    log::info!("图片故事任务 {} 开始合并 {} 个视频片段", task_id, video_paths.len());
    let output_path = task_dir.join("final_video.mp4").to_string_lossy().to_string();

    merge_videos(&video_paths, &output_path, params)?;

    state::update_image_story_task_data(
        task_id,
        Some(TaskStatus::Completed.as_i32()),
        Some(100),
        Some(output_path),
        None,
    );

    log::info!("图片故事任务 {} 执行完成", task_id);
    Ok(())
}

/// 为单个场景生成视频（同步版本，内部构建 tokio runtime）
fn generate_scene_video_sync(
    task_id: &str,
    scene: &soma_core::models::ImageStoryScene,
    params: &ImageStoryParams,
    conf: &soma_core::config::AppConfig,
) -> Result<String, SomaError> {
    let task_dir = soma_core::utils::task_dir(task_id);
    let output_path = task_dir.join(format!("scene_{}.mp4", scene.scene_id)).to_string_lossy().to_string();

    let provider_name = params.get_ai_provider().to_string();
    let image_path = scene.image_path.clone();
    let description = scene.description.clone();
    let aspect_ratio = params.get_video_aspect().as_str().to_string();
    let duration = scene.duration.unwrap_or(params.get_scene_duration());
    let output_path_clone = output_path.clone();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| SomaError::VideoGen(format!("创建 tokio runtime 失败: {}", e)))?;

    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, async {
        let provider = soma_stock::aivideo::create_provider(&provider_name, conf)?;

        let video_params = soma_stock::aivideo::VideoGenParams {
            prompt: description,
            aspect_ratio,
            duration,
            model: None,
            image_url: Some(image_path),
        };

        let ai_task_id = provider.create_task(&video_params).await?;

        let status = soma_stock::aivideo::poll_until_done(
            provider.as_ref(),
            &ai_task_id,
            300,
            3,
        ).await?;

        match status {
            soma_stock::aivideo::VideoGenStatus::Success { video_urls } => {
                if let Some(url) = video_urls.first() {
                    provider.download_video(url, &output_path_clone).await?;
                    Ok(output_path_clone)
                } else {
                    Err(SomaError::VideoGen("AI视频生成成功但未返回视频URL".into()))
                }
            }
            soma_stock::aivideo::VideoGenStatus::Failed { message } => {
                Err(SomaError::VideoGen(format!("AI视频生成失败: {}", message)))
            }
            _ => Err(SomaError::VideoGen("AI视频生成超时".into())),
        }
    })
}

/// 合并多个视频片段
fn merge_videos(
    video_paths: &[String],
    output_path: &str,
    _params: &ImageStoryParams,
) -> Result<(), SomaError> {
    if video_paths.is_empty() {
        return Err(SomaError::VideoGen("没有视频片段可合并".into()));
    }

    if video_paths.len() == 1 {
        std::fs::copy(&video_paths[0], output_path).map_err(SomaError::Io)?;
        return Ok(());
    }

    let ffmpeg = soma_video::Ffmpeg::new(output_path, 2, "libx264");
    ffmpeg.concat_clips(video_paths, output_path)?;

    Ok(())
}