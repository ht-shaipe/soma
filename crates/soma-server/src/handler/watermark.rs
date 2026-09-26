//! 视频去水印 API 处理器
//!
//! 基于多帧梯度分析的静态水印检测与去除
//! （soma-video::watermark，算法源自 johnson7788/remove_watermark 的 Rust 原生实现）。
//!
//! - `detect`：检测水印蒙版，返回蒙版路径 + 帧/蒙版预览图（data URL，
//!   HTTP 与桌面模式均可直接展示），供用户确认检测区域后再执行去除
//! - `remove`：执行去除；未提供蒙版路径时自动检测

use base64::Engine;
use soma_video::watermark;
use std::path::PathBuf;
use tube::{Result, Value};

use crate::Config;

/// 由全局配置构造 FFmpeg 二进制路径（跟随设置页 ffmpeg_path，M3.4 sidecar 前置）
fn ffmpeg_binaries() -> watermark::FfmpegBinaries {
    let conf = Config::get();
    watermark::FfmpegBinaries::from_ffmpeg(&conf.app.get_ffmpeg_binary())
}
use tube_web::RequestParameter;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "detect" => detect(param).await,
        "remove" => remove(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 从请求参数解析检测选项（全部可缺省）
fn detect_options(param: &RequestParameter) -> watermark::WatermarkDetectOptions {
    let v = &param.value;
    watermark::WatermarkDetectOptions {
        keyframes: v
            .get("keyframes")
            .and_then(|x| x.as_u64())
            .map(|x| x.clamp(2, 120) as u32)
            .unwrap_or(30),
        grad_threshold: v
            .get("gradThreshold")
            .and_then(|x| x.as_f32())
            .map(|x| x.clamp(0.5, 120.0))
            .unwrap_or(8.0),
        mask_threshold: v
            .get("maskThreshold")
            .and_then(|x| x.as_f32())
            .map(|x| x.clamp(0.01, 0.99))
            .unwrap_or(0.2),
        blur_sigma: v
            .get("blurSigma")
            .and_then(|x| x.as_f32())
            .map(|x| x.clamp(0.5, 10.0))
            .unwrap_or(2.0),
    }
}

/// 读取小图并编码为 data URL（PNG）
fn png_data_url(path: &PathBuf) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|e| error!("读取预览图失败: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}

/// 检测静态水印：产出蒙版与预览
///
/// 请求体字段：videoPath（必填）、keyframes / gradThreshold / maskThreshold / blurSigma（可选）。
async fn detect(param: &RequestParameter) -> Result<Value> {
    let video_path = param.value.get_def_string("videoPath", "");
    if video_path.is_empty() {
        return Err(error!("缺少 videoPath 参数"));
    }
    let opts = detect_options(param);
    let work_dir =
        soma_core::utils::storage_dir(&format!("watermark/{}", soma_core::utils::get_uuid()), true);

    // 检测为同步 ffmpeg 子进程调用，放入阻塞线程池
    let bins = ffmpeg_binaries();
    actix_web::web::block(move || {
        let video = PathBuf::from(&video_path);
        let mask = watermark::detect_from_video(&video, &opts, &work_dir, &bins)
            .map_err(|e| error!("水印检测失败: {}", e))?;

        let mask_path = work_dir.join("mask.bmp");
        watermark::save_mask_bmp(&mask, &mask_path).map_err(|e| error!("蒙版保存失败: {}", e))?;

        let (frame_png, mask_png) =
            watermark::write_previews(&video, &mask_path, &work_dir, 640, &bins)
                .map_err(|e| error!("预览生成失败: {}", e))?;
        let frame_preview = png_data_url(&frame_png)?;
        let mask_preview = png_data_url(&mask_png)?;

        Ok(value!({
            "maskPath": mask_path.to_string_lossy().to_string(),
            "framePreview": frame_preview,
            "maskPreview": mask_preview,
            "width": mask.width,
            "height": mask.height,
            "coverage": mask.coverage,
            "keyframesUsed": mask.frames_used,
        }))
    })
    .await
    .map_err(|e| error!("水印检测线程异常: {}", e))?
}

/// 执行水印去除
///
/// 请求体字段：videoPath（必填）、maskPath（可选，缺省自动检测）、
/// outputDir / codec / crf（可选）+ 检测选项（自动检测时生效）。
async fn remove(param: &RequestParameter) -> Result<Value> {
    let video_path = param.value.get_def_string("videoPath", "");
    if video_path.is_empty() {
        return Err(error!("缺少 videoPath 参数"));
    }
    let mask_path = param.value.get_def_string("maskPath", "");
    let output_dir = {
        let d = param.value.get_def_string("outputDir", "");
        if d.is_empty() {
            soma_core::utils::storage_dir("watermark", true)
        } else {
            PathBuf::from(d)
        }
    };
    let codec = param.value.get_def_string("codec", "");
    let crf = param
        .value
        .get("crf")
        .and_then(|x| x.as_u64())
        .map(|x| x.clamp(0, 51) as u32)
        .unwrap_or(18);
    let opts = detect_options(param);
    let started = std::time::Instant::now();

    // 去除为同步长任务（重编码可能耗时数分钟），放入阻塞线程池
    let bins = ffmpeg_binaries();
    actix_web::web::block(move || {
        let video = PathBuf::from(&video_path);
        let work_dir = output_dir.join(soma_core::utils::get_uuid());
        std::fs::create_dir_all(&work_dir).map_err(|e| error!("创建工作目录失败: {}", e))?;

        // 蒙版：优先复用 detect 产物，否则自动检测
        let mask_file = if mask_path.is_empty() {
            let mask = watermark::detect_from_video(&video, &opts, &work_dir, &bins)
                .map_err(|e| error!("自动检测水印失败: {}", e))?;
            let p = work_dir.join("mask.bmp");
            watermark::save_mask_bmp(&mask, &p).map_err(|e| error!("蒙版保存失败: {}", e))?;
            p
        } else {
            let p = PathBuf::from(&mask_path);
            if !p.exists() {
                return Err(error!("蒙版文件不存在: {}", mask_path));
            }
            p
        };

        let stem = video
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "video".to_string());
        let output_path = work_dir.join(format!("{}_cleaned.mp4", stem));

        watermark::remove_watermark(
            &video,
            &mask_file,
            &output_path,
            &work_dir,
            &codec,
            crf,
            &bins,
        )
        .map_err(|e| error!("{}", e))?;

        Ok(value!({
            "success": true,
            "outputPath": output_path.to_string_lossy().to_string(),
            "elapsedMs": started.elapsed().as_millis() as u64,
        }))
    })
    .await
    .map_err(|e| error!("水印去除线程异常: {}", e))?
}
