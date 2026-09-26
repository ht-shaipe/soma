/// 视频转场特效模块
///
/// 提供独立的转场特效应用函数，包括淡入、淡出、滑入、滑出效果，
/// 以及 Shuffle 随机混合模式。使用 FFmpeg 滤镜实现。
use soma_core::error::SomaError;

/// 为视频应用转场特效
///
/// - `codec`: 视频编码器名称（如 "libx264"）
pub fn apply_transition(input_path: &str, output_path: &str, transition: &str, duration: f64, ffmpeg_path: &str, side: &str, codec: &str) -> Result<(), SomaError> {
    let vf = match transition {
        "FadeIn" => format!("fade=t=in:st=0:d={}", duration),
        "FadeOut" => {
            let dur = get_video_duration(input_path, ffmpeg_path)?;
            format!("fade=t=out:st={}:d={}", (dur - duration).max(0.0), duration)
        }
        "Dissolve" => {
            let dur = get_video_duration(input_path, ffmpeg_path)?;
            let in_dur = duration.min(dur * 0.3);
            format!("fade=t=in:st=0:d={},fade=t=out:st={}:d={}", in_dur, dur - duration, duration)
        }
        "SlideIn" => build_slide_in_filter(duration, side)?,
        "SlideOut" => build_slide_out_filter(input_path, duration, side, ffmpeg_path)?,
        _ => {
            log::warn!("未识别的转场类型: {}, 跳过转场处理", transition);
            return Ok(());
        }
    };
    if transition == "SlideIn" {
        let (w, h) = get_video_resolution(input_path, ffmpeg_path)?;
        let total_dur = get_video_duration(input_path, ffmpeg_path)?;
        let result = std::process::Command::new(ffmpeg_path)
            .args([
                "-y",
                "-f", "lavfi", "-i", &format!("color=c=black:s={}x{}:duration={:.3}", w, h, total_dur),
                "-i", input_path,
                "-filter_complex", &format!("[1:v]setpts=PTS-STARTPTS[fg];[0:v][fg]overlay=x='{}':y='{}':shortest=1",
                    build_slide_x_expr(duration, side, w, h, true),
                    build_slide_y_expr(duration, side, w, h, true)
                ),
                "-c:v", codec, "-an", "-pix_fmt", "yuv420p", output_path,
            ])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg slide_in failed: {}", e)))?;
        if !result.status.success() {
            return Err(SomaError::Ffmpeg("ffmpeg slide_in failed".into()));
        }
    } else {
        let result = std::process::Command::new(ffmpeg_path)
            .args(["-y", "-i", input_path, "-vf", &vf, "-c:v", codec, "-an", "-pix_fmt", "yuv420p", output_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg transition failed: {}", e)))?;
        if !result.status.success() {
            return Err(SomaError::Ffmpeg("ffmpeg transition failed".into()));
        }
    }
    Ok(())
}

/// 随机选择一种转场效果并应用（Shuffle 模式）
pub fn apply_shuffle_transition(input_path: &str, output_path: &str, duration: f64, ffmpeg_path: &str, codec: &str) -> Result<(), SomaError> {
    use rand::Rng;
    let mut rng = rand::rng();
    let transitions = ["FadeIn", "FadeOut", "Dissolve", "SlideIn", "SlideOut"];
    let sides = ["left", "right", "top", "bottom"];
    let t_idx = rng.random_range(0..transitions.len());
    let s_idx = rng.random_range(0..sides.len());
    apply_transition(input_path, output_path, transitions[t_idx], duration, ffmpeg_path, sides[s_idx], codec)
}

fn build_slide_in_filter(duration: f64, _side: &str) -> Result<String, SomaError> {
    Ok(format!("fade=t=in:st=0:d={}", duration))
}

fn build_slide_out_filter(input_path: &str, duration: f64, side: &str, ffmpeg_path: &str) -> Result<String, SomaError> {
    let (w, h) = get_video_resolution(input_path, ffmpeg_path)?;
    let total_dur = get_video_duration(input_path, ffmpeg_path)?;
    let start_t = (total_dur - duration).max(0.0);
    let x_expr = build_slide_x_expr_slideout(duration, side, w, start_t);
    let y_expr = build_slide_y_expr_slideout(duration, side, h, start_t);
    Ok(format!(
        "color=c=black:s={}x{}:duration={:.3}[bg];[0:v]setpts=PTS-STARTPTS[fg];[bg][fg]overlay=x='{}':y='{}':shortest=1",
        w, h, total_dur, x_expr, y_expr
    ))
}

fn build_slide_x_expr(duration: f64, side: &str, w: u32, _h: u32, is_in: bool) -> String {
    match side {
        "left" if is_in => format!("if(lt(t,{duration}),{neg_w}+{w}*t/{duration},0)", neg_w = -(w as i64), w = w, duration = duration),
        "right" if is_in => format!("if(lt(t,{duration}),{w}-{w}*t/{duration},0)", w = w, duration = duration),
        _ => "0".to_string(),
    }
}

fn build_slide_y_expr(duration: f64, side: &str, _w: u32, h: u32, is_in: bool) -> String {
    match side {
        "top" if is_in => format!("if(lt(t,{duration}),{neg_h}+{h}*t/{duration},0)", neg_h = -(h as i64), h = h, duration = duration),
        "bottom" if is_in => format!("if(lt(t,{duration}),{h}-{h}*t/{duration},0)", h = h, duration = duration),
        _ => "0".to_string(),
    }
}

fn build_slide_x_expr_slideout(duration: f64, side: &str, w: u32, start_t: f64) -> String {
    match side {
        "left" => format!("if(gte(t,{start_t}),-{w}*(t-{start_t})/{duration},0)", w = w, start_t = start_t, duration = duration),
        "right" => format!("if(gte(t,{start_t}),{w}*(t-{start_t})/{duration},0)", w = w, start_t = start_t, duration = duration),
        _ => "0".to_string(),
    }
}

fn build_slide_y_expr_slideout(duration: f64, side: &str, h: u32, start_t: f64) -> String {
    match side {
        "top" => format!("if(gte(t,{start_t}),-{h}*(t-{start_t})/{duration},0)", h = h, start_t = start_t, duration = duration),
        "bottom" => format!("if(gte(t,{start_t}),{h}*(t-{start_t})/{duration},0)", h = h, start_t = start_t, duration = duration),
        _ => "0".to_string(),
    }
}

fn get_video_duration(video_path: &str, _ffmpeg_path: &str) -> Result<f64, SomaError> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", video_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
    String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
        .map_err(|e| SomaError::Ffmpeg(format!("parse duration failed: {}", e)))
}

fn get_video_resolution(video_path: &str, _ffmpeg_path: &str) -> Result<(u32, u32), SomaError> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=s=x:p=0", video_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe resolution failed: {}", e)))?;
    let res_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = res_str.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse::<u32>().unwrap_or(0);
        let h = parts[1].parse::<u32>().unwrap_or(0);
        if w > 0 && h > 0 { return Ok((w, h)); }
    }
    Err(SomaError::Ffmpeg("failed to get video resolution".into()))
}
