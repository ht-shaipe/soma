/// 视频转场特效模块
///
/// 提供独立的转场特效应用函数，包括淡入（FadeIn）和淡出（FadeOut）效果。
/// 使用 FFmpeg 的 fade 视频滤镜实现。
use soma_core::error::SomaError;

/// 为视频应用转场特效
///
/// 根据指定的转场类型，使用 FFmpeg fade 滤镜添加淡入或淡出效果。
/// 不识别的转场类型会被静默跳过。
///
/// # 参数
/// - `input_path`: 输入视频路径
/// - `output_path`: 输出视频路径
/// - `transition`: 转场类型名称（"FadeIn" 或 "FadeOut"）
/// - `duration`: 转场效果持续时间（秒）
/// - `ffmpeg_path`: FFmpeg 可执行文件路径
///
/// # 返回
/// 成功返回 Ok(())，无法识别的转场类型返回 Ok(())（跳过），失败返回 SomaError
pub fn apply_transition(input_path: &str, output_path: &str, transition: &str, duration: f64, ffmpeg_path: &str) -> Result<(), SomaError> {
    // 根据转场类型构建对应的 fade 滤镜参数
    let vf = match transition {
        // fade=t=in:st=0:d=N → 从第 0 秒开始淡入，持续 N 秒
        "FadeIn" => format!("fade=t=in:st=0:d={}", duration),
        "FadeOut" => {
            // 淡出起始时间 = 视频总时长 - 淡出持续时间
            let dur = get_video_duration(input_path, ffmpeg_path)?;
            // fade=t=out:st=START:d=N → 从 START 秒开始淡出，持续 N 秒
            format!("fade=t=out:st={}:d={}", dur - duration, duration)
        }
        // 未识别的转场类型，跳过不做处理
        _ => return Ok(()),
    };
    let result = std::process::Command::new(ffmpeg_path)
        // -y: 覆盖输出
        // -vf: 应用视频滤镜
        // -c:v libx264: 使用 H.264 编码
        // -an: 移除音频
        // -pix_fmt yuv420p: 设置像素格式确保兼容性
        .args(&["-y", "-i", input_path, "-vf", &vf, "-c:v", "libx264", "-an", "-pix_fmt", "yuv420p", output_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg transition failed: {}", e)))?;
    if !result.status.success() {
        return Err(SomaError::Ffmpeg("ffmpeg transition failed".into()));
    }
    Ok(())
}

/// 获取视频文件时长（秒）
///
/// 使用 ffprobe 查询视频文件的 format duration 信息。
///
/// # 参数
/// - `video_path`: 视频文件路径
/// - `ffmpeg_path`: FFmpeg 路径（当前未使用，预留扩展）
///
/// # 返回
/// 成功返回时长（f64 秒），失败返回 SomaError
fn get_video_duration(video_path: &str, ffmpeg_path: &str) -> Result<f64, SomaError> {
    // ffmpeg_path 参数预留，当前使用 ffprobe 独立命令
    let _ = ffmpeg_path;
    let output = std::process::Command::new("ffprobe")
        // -v error: 只输出错误
        // -show_entries format=duration: 只显示时长
        // -of default=noprint_wrappers=1:nokey=1: 仅输出纯数值
        .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", video_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
    String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
        .map_err(|e| SomaError::Ffmpeg(format!("parse duration failed: {}", e)))
}
