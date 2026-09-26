//! 视频静态水印检测与去除
//!
//! 算法源自开源项目 johnson7788/remove_watermark（MIT）的 Rust 原生实现：
//! 等间隔采样多帧灰度图 → 对带符号梯度做多帧平均（运动内容正负抵消，
//! 静态水印边缘保留）→ 高斯平滑 + 归一化阈值生成二值蒙版 →
//! FFmpeg `removelogo` 滤镜修复。
//!
//! 相比原 Python 实现的增强：
//! - 采样确定性化：等间隔时间点替代随机关键帧（结果可复现）
//! - 阈值/采样数/平滑参数全部可调（原实现硬编码）
//! - 输出蒙版覆盖率，供调用方判断检测结果是否可信
//! - 蒙版与采样帧预览产物，供 UI 叠加展示检测区域
//!
//! 局限（与原实现一致）：仅支持位置固定的静态水印；`removelogo` 为
//! 插值修复，复杂纹理区域会有涂抹痕迹；字幕/台标等静态元素可能被误判，
//! 建议结合蒙版预览确认后再执行去除。

use soma_core::error::SomaError;
use std::path::{Path, PathBuf};

/// 水印检测参数
#[derive(Debug, Clone)]
pub struct WatermarkDetectOptions {
    /// 采样帧数（等间隔分布于全片）
    pub keyframes: u32,
    /// 梯度均值阈值（0-255 灰度梯度尺度）
    pub grad_threshold: f32,
    /// 归一化后的蒙版二值化阈值（0.0-1.0）
    pub mask_threshold: f32,
    /// 蒙版高斯平滑 sigma（像素）
    pub blur_sigma: f32,
}

impl Default for WatermarkDetectOptions {
    fn default() -> Self {
        Self {
            keyframes: 30,
            grad_threshold: 8.0,
            mask_threshold: 0.2,
            blur_sigma: 2.0,
        }
    }
}

/// 检测产出的二值蒙版（data 每像素 0 或 255）
#[derive(Debug, Clone)]
pub struct WatermarkMask {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    /// 蒙版像素占整帧比例（0.0-1.0），异常小/大提示检测可疑
    pub coverage: f32,
    /// 实际参与计算的采样帧数
    pub frames_used: u32,
}

/// 从多帧灰度图计算水印蒙版（纯函数，无 ffmpeg 依赖，可单测）
///
/// 算法：逐帧计算带符号梯度（x/y 方向中央差分），对全部帧取平均后取绝对值——
/// 运动画面的梯度符号随运动翻转、多帧平均后抵消，静态水印边缘符号一致得以保留；
/// 随后阈值化 → 高斯平滑 → min-max 归一化 → 二值化。
pub fn compute_mask(
    frames: &[Vec<u8>],
    width: usize,
    height: usize,
    opts: &WatermarkDetectOptions,
) -> Result<WatermarkMask, SomaError> {
    let n = frames.len();
    if n < 2 {
        return Err(SomaError::Ffmpeg(format!(
            "水印检测至少需要 2 帧采样，实际 {} 帧",
            n
        )));
    }
    let expected = width * height;
    if frames.iter().any(|f| f.len() != expected) {
        return Err(SomaError::Ffmpeg("采样帧尺寸与视频分辨率不一致".into()));
    }

    // 带符号梯度的多帧累加（先平均后取绝对值，运动内容得以抵消）
    let mut sum_dx = vec![0f32; expected];
    let mut sum_dy = vec![0f32; expected];
    for frame in frames {
        for y in 0..height {
            let row = y * width;
            for x in 0..width {
                let i = row + x;
                // x 方向：内部中央差分，边界单侧差分（对齐 numpy.gradient 语义）
                let dx = if x == 0 {
                    (frame[i + 1] as f32) - (frame[i] as f32)
                } else if x + 1 == width {
                    (frame[i] as f32) - (frame[i - 1] as f32)
                } else {
                    ((frame[i + 1] as f32) - (frame[i - 1] as f32)) * 0.5
                };
                // y 方向：同上
                let dy = if y == 0 {
                    (frame[i + width] as f32) - (frame[i] as f32)
                } else if y + 1 == height {
                    (frame[i] as f32) - (frame[i - width] as f32)
                } else {
                    ((frame[i + width] as f32) - (frame[i - width] as f32)) * 0.5
                };
                sum_dx[i] += dx;
                sum_dy[i] += dy;
            }
        }
    }
    let inv_n = 1.0 / n as f32;

    // 阈值化显著图
    let mut salient = vec![0f32; expected];
    for i in 0..expected {
        let mean_dx = (sum_dx[i] * inv_n).abs();
        let mean_dy = (sum_dy[i] * inv_n).abs();
        if mean_dx > opts.grad_threshold || mean_dy > opts.grad_threshold {
            salient[i] = 1.0;
        }
    }

    // 高斯平滑 + min-max 归一化 + 二值化
    let blurred = gaussian_blur(&salient, width, height, opts.blur_sigma);
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for v in &blurred {
        if *v < min {
            min = *v;
        }
        if *v > max {
            max = *v;
        }
    }
    let range = max - min;
    let mut data = vec![0u8; expected];
    let mut masked = 0usize;
    for i in 0..expected {
        let norm = if range > 0.0 {
            (blurred[i] - min) / range
        } else {
            0.0
        };
        if norm > opts.mask_threshold {
            data[i] = 255;
            masked += 1;
        }
    }

    Ok(WatermarkMask {
        width: width as u32,
        height: height as u32,
        data,
        coverage: masked as f32 / expected as f32,
        frames_used: n as u32,
    })
}

/// 一维高斯核
fn gaussian_kernel(sigma: f32) -> Vec<f32> {
    let radius = (3.0 * sigma).ceil().max(1.0) as usize;
    let denom = 2.0 * sigma * sigma;
    let mut kernel: Vec<f32> = (0..=radius)
        .map(|x| (-((x * x) as f32) / denom).exp())
        .collect();
    // 对称展开为完整核
    let mut full: Vec<f32> = kernel.iter().skip(1).rev().cloned().collect();
    full.append(&mut kernel);
    let sum: f32 = full.iter().sum();
    if sum > 0.0 {
        for v in &mut full {
            *v /= sum;
        }
    }
    full
}

/// 可分离高斯模糊（先水平后垂直）
fn gaussian_blur(src: &[f32], width: usize, height: usize, sigma: f32) -> Vec<f32> {
    if sigma <= 0.0 || width == 0 || height == 0 {
        return src.to_vec();
    }
    let kernel = gaussian_kernel(sigma);
    let radius = kernel.len() / 2;

    // 水平卷积（边界镜像）
    let mut tmp = vec![0f32; src.len()];
    for y in 0..height {
        let row = y * width;
        for x in 0..width {
            let mut acc = 0f32;
            for (k, w) in kernel.iter().enumerate() {
                let mut sx = x as isize + k as isize - radius as isize;
                sx = sx.clamp(0, width as isize - 1);
                acc += src[row + sx as usize] * w;
            }
            tmp[row + x] = acc;
        }
    }

    // 垂直卷积（边界镜像）
    let mut out = vec![0f32; src.len()];
    for y in 0..height {
        for x in 0..width {
            let mut acc = 0f32;
            for (k, w) in kernel.iter().enumerate() {
                let mut sy = y as isize + k as isize - radius as isize;
                sy = sy.clamp(0, height as isize - 1);
                acc += tmp[sy as usize * width + x] * w;
            }
            out[y * width + x] = acc;
        }
    }
    out
}

/// ffprobe 查询视频时长（秒）与分辨率（宽, 高）
fn probe_video(video_path: &Path) -> Result<(f64, u32, u32), SomaError> {
    let out = std::process::Command::new("ffprobe")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height:format=duration",
            "-of",
            "default=noprint_wrappers=1",
        ])
        .arg(video_path)
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe 启动失败: {}", e)))?;
    if !out.status.success() {
        return Err(SomaError::Ffmpeg(format!(
            "ffprobe 查询失败: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut width = 0u32;
    let mut height = 0u32;
    let mut duration = 0f64;
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("width=") {
            width = v.parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("height=") {
            height = v.parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("duration=") {
            duration = v.parse().unwrap_or(0.0);
        }
    }
    if width == 0 || height == 0 {
        return Err(SomaError::Ffmpeg(
            "无法解析视频分辨率（可能是音频或损坏文件）".into(),
        ));
    }
    Ok((duration, width, height))
}

/// 从视频中检测静态水印
///
/// 等间隔采样 `opts.keyframes` 帧灰度图（-ss 快速定位），
/// 经 [`compute_mask`] 生成蒙版。`work_dir` 为采样帧临时目录。
pub fn detect_from_video(
    video_path: &Path,
    opts: &WatermarkDetectOptions,
    work_dir: &Path,
) -> Result<WatermarkMask, SomaError> {
    if !video_path.exists() {
        return Err(SomaError::Ffmpeg(format!(
            "视频文件不存在: {}",
            video_path.display()
        )));
    }
    std::fs::create_dir_all(work_dir)
        .map_err(|e| SomaError::Ffmpeg(format!("创建工作目录失败: {}", e)))?;

    let (duration, width, height) = probe_video(video_path)?;
    let keyframes = opts.keyframes.max(2);

    // 等间隔采样：取片内 (i+0.5)/n 时间点，避开首尾黑帧
    let timestamps: Vec<f64> = if duration > 0.0 {
        (0..keyframes)
            .map(|i| duration * (i as f64 + 0.5) / keyframes as f64)
            .collect()
    } else {
        // 时长未知（如部分 GIF/LIVP），退化为中心单点多次采样
        vec![0.0; keyframes as usize]
    };

    let expected = (width as usize) * (height as usize);
    let mut frames: Vec<Vec<u8>> = Vec::with_capacity(timestamps.len());
    for (idx, t) in timestamps.iter().enumerate() {
        let frame_path = work_dir.join(format!("frame_{:03}.gray", idx));
        let status = std::process::Command::new("ffmpeg")
            .args(["-y", "-hide_banner", "-loglevel", "error", "-ss"])
            .arg(format!("{:.3}", t))
            .args(["-i"])
            .arg(video_path)
            .args(["-vframes", "1", "-pix_fmt", "gray", "-f", "rawvideo"])
            .arg(&frame_path)
            .status()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg 启动失败: {}", e)))?;
        if !status.success() {
            continue;
        }
        match std::fs::read(&frame_path) {
            Ok(bytes) if bytes.len() == expected => frames.push(bytes),
            _ => {}
        }
        let _ = std::fs::remove_file(&frame_path);
    }

    compute_mask(&frames, width as usize, height as usize, opts)
}

/// 将蒙版写为 24 位 BGR BMP（FFmpeg image2 可读，浏览器可直接显示）
pub fn save_mask_bmp(mask: &WatermarkMask, path: &Path) -> Result<(), SomaError> {
    let width = mask.width as usize;
    let height = mask.height as usize;
    let row_size = (width * 3 + 3) & !3; // 4 字节对齐
    let pixel_bytes = row_size * height;
    let file_size = 54 + pixel_bytes;

    let mut buf = Vec::with_capacity(file_size);
    // BITMAPFILEHEADER（14 字节）
    buf.extend_from_slice(b"BM");
    buf.extend_from_slice(&(file_size as u32).to_le_bytes());
    buf.extend_from_slice(&[0; 4]); // 保留位
    buf.extend_from_slice(&54u32.to_le_bytes()); // 像素数据偏移
                                                 // BITMAPINFOHEADER（40 字节）
    buf.extend_from_slice(&40u32.to_le_bytes());
    buf.extend_from_slice(&(mask.width as i32).to_le_bytes());
    buf.extend_from_slice(&(mask.height as i32).to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // 平面数
    buf.extend_from_slice(&24u16.to_le_bytes()); // 位深
    buf.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB 无压缩
    buf.extend_from_slice(&(pixel_bytes as u32).to_le_bytes());
    buf.extend_from_slice(&[0x13, 0x0B, 0, 0]); // 2835 px/m
    buf.extend_from_slice(&[0x13, 0x0B, 0, 0]);
    buf.extend_from_slice(&0u32.to_le_bytes()); // 调色板
    buf.extend_from_slice(&0u32.to_le_bytes()); // 重要色

    // 像素数据自底向上、BGR 顺序
    let padding = row_size - width * 3;
    for y in (0..height).rev() {
        for x in 0..width {
            let v = mask.data[y * width + x];
            buf.extend_from_slice(&[v, v, v]);
        }
        buf.extend(std::iter::repeat_n(0u8, padding));
    }

    std::fs::write(path, buf).map_err(|e| SomaError::Ffmpeg(format!("蒙版写入失败: {}", e)))
}

/// 执行水印去除：FFmpeg `removelogo` 滤镜 + 重编码
///
/// 为规避 FFmpeg 滤镜参数的路径转义问题，将蒙版复制到 `work_dir`
/// 后以相对文件名引用，并以 `work_dir` 为工作目录执行。
pub fn remove_watermark(
    video_path: &Path,
    mask_path: &Path,
    output_path: &Path,
    work_dir: &Path,
    codec: &str,
    crf: u32,
) -> Result<(), SomaError> {
    if !video_path.exists() {
        return Err(SomaError::Ffmpeg(format!(
            "视频文件不存在: {}",
            video_path.display()
        )));
    }
    if !mask_path.exists() {
        return Err(SomaError::Ffmpeg(format!(
            "蒙版文件不存在: {}",
            mask_path.display()
        )));
    }
    let codec = if codec.is_empty() { "libx264" } else { codec };
    std::fs::create_dir_all(work_dir)
        .map_err(|e| SomaError::Ffmpeg(format!("创建工作目录失败: {}", e)))?;
    std::fs::create_dir_all(
        output_path
            .parent()
            .ok_or_else(|| SomaError::Ffmpeg("输出路径无父目录".into()))?,
    )
    .map_err(|e| SomaError::Ffmpeg(format!("创建输出目录失败: {}", e)))?;

    // 蒙版统一落到工作目录，以相对名引用避开转义
    let local_mask = if mask_path.parent() == Some(work_dir) {
        mask_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "mask.bmp".to_string())
    } else {
        std::fs::copy(mask_path, work_dir.join("mask.bmp"))
            .map_err(|e| SomaError::Ffmpeg(format!("蒙版复制失败: {}", e)))?;
        "mask.bmp".to_string()
    };

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(video_path)
        .args(["-vf", &format!("removelogo={}", local_mask), "-c:v"])
        .arg(codec)
        .args(["-crf"])
        .arg(crf.to_string())
        .args(["-preset", "medium", "-c:a", "copy"])
        .arg(output_path)
        .current_dir(work_dir);

    let output = cmd
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg 启动失败: {}", e)))?;
    if !output.status.success() {
        return Err(SomaError::Ffmpeg(format!(
            "水印去除失败: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

/// 生成检测预览图（PNG）：采样帧缩略图 + 蒙版缩略图（同宽，便于 UI 叠加）
///
/// 返回 (frame_preview_path, mask_preview_path)。
pub fn write_previews(
    video_path: &Path,
    mask_path: &Path,
    work_dir: &Path,
    preview_width: u32,
) -> Result<(PathBuf, PathBuf), SomaError> {
    let preview_width = if preview_width == 0 {
        640
    } else {
        preview_width
    };
    let (duration, _, _) = probe_video(video_path)?;
    let mid = if duration > 0.0 { duration / 2.0 } else { 0.0 };

    let frame_png = work_dir.join("frame_preview.png");
    let mask_png = work_dir.join("mask_preview.png");

    let scale = format!("scale={}:-2", preview_width);
    let frame_status = std::process::Command::new("ffmpeg")
        .args(["-y", "-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{:.3}", mid))
        .arg("-i")
        .arg(video_path)
        .args(["-vframes", "1", "-vf", &scale])
        .arg(&frame_png)
        .status()
        .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg 启动失败: {}", e)))?;
    if !frame_status.success() {
        return Err(SomaError::Ffmpeg("采样帧预览生成失败".into()));
    }

    let mask_status = std::process::Command::new("ffmpeg")
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(mask_path)
        .args(["-vf", &scale])
        .arg(&mask_png)
        .status()
        .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg 启动失败: {}", e)))?;
    if !mask_status.success() {
        return Err(SomaError::Ffmpeg("蒙版预览生成失败".into()));
    }

    Ok((frame_png, mask_png))
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: usize = 64;
    const H: usize = 48;

    /// 合成一帧：平滑移动正弦背景（模拟运动画面，相位逐帧推进使梯度符号
    /// 在帧间翻转、多帧平均后抵消）+ 右下角静态亮块（模拟水印，仅边缘有梯度）
    fn synthetic_frame(i: usize, wm: bool) -> Vec<u8> {
        let mut f = vec![0u8; W * H];
        // 正弦角频率 0.2，相位步进 0.9 rad → 帧间梯度符号快速翻转
        let phase = i as f32 * 4.5;
        for y in 0..H {
            for x in 0..W {
                let v = 128.0 + 60.0 * ((x as f32 + phase) * 0.2).sin();
                f[y * W + x] = v.clamp(0.0, 255.0) as u8;
            }
        }
        if wm {
            for y in 36..44 {
                for x in 48..60 {
                    f[y * W + x] = 230;
                }
            }
        }
        f
    }

    #[test]
    fn test_compute_mask_finds_static_watermark() {
        // 6 帧移动背景 + 静态水印块
        let frames: Vec<Vec<u8>> = (0..6).map(|i| synthetic_frame(i, true)).collect();
        let mask = compute_mask(&frames, W, H, &WatermarkDetectOptions::default()).unwrap();
        assert_eq!(mask.width as usize, W);
        assert_eq!(mask.height as usize, H);
        assert_eq!(mask.frames_used, 6);
        // 水印块中心应被标记（边缘梯度经高斯平滑扩散覆盖块内）
        let center = mask.data[40 * W + 54];
        assert_eq!(center, 255, "水印区域中心应命中蒙版");
        // 运动背景应被抵消：覆盖率仅为水印块占比（约 3%）
        assert!(
            mask.coverage > 0.0 && mask.coverage < 0.2,
            "覆盖率异常: {}",
            mask.coverage
        );
    }

    #[test]
    fn test_compute_mask_moving_content_cancels() {
        // 同样 6 帧移动背景但无水印：蒙版覆盖率应显著更低
        let with_wm: Vec<Vec<u8>> = (0..6).map(|i| synthetic_frame(i, true)).collect();
        let no_wm: Vec<Vec<u8>> = (0..6).map(|i| synthetic_frame(i, false)).collect();
        let m1 = compute_mask(&with_wm, W, H, &WatermarkDetectOptions::default()).unwrap();
        let m2 = compute_mask(&no_wm, W, H, &WatermarkDetectOptions::default()).unwrap();
        assert!(
            m1.coverage > m2.coverage,
            "有水印的覆盖率应高于纯运动画面: {} vs {}",
            m1.coverage,
            m2.coverage
        );
        assert!(
            m2.coverage < 0.05,
            "纯运动画面的蒙版覆盖率应接近 0: {}",
            m2.coverage
        );
    }

    #[test]
    fn test_compute_mask_rejects_few_frames() {
        let frames = vec![synthetic_frame(1, true)];
        let err = compute_mask(&frames, W, H, &WatermarkDetectOptions::default());
        assert!(err.is_err());
    }

    #[test]
    fn test_gaussian_blur_preserves_constant() {
        let src = vec![0.5f32; W * H];
        let out = gaussian_blur(&src, W, H, 2.0);
        assert!(out.iter().all(|v| (v - 0.5).abs() < 1e-4));
    }

    #[test]
    fn test_save_mask_bmp_layout() {
        let mask = WatermarkMask {
            width: W as u32,
            height: H as u32,
            data: vec![255; W * H],
            coverage: 1.0,
            frames_used: 2,
        };
        let dir = std::env::temp_dir().join(format!("soma-wm-{}", soma_core::utils::get_uuid()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mask.bmp");
        save_mask_bmp(&mask, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let row_size = (W * 3 + 3) & !3;
        assert_eq!(bytes.len(), 54 + row_size * H);
        assert_eq!(&bytes[0..2], b"BM");
        let file_size = u32::from_le_bytes(bytes[2..6].try_into().unwrap()) as usize;
        assert_eq!(file_size, bytes.len());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
