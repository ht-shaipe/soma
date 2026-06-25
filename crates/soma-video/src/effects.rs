use soma_core::error::SomaError;

pub fn apply_transition(input_path: &str, output_path: &str, transition: &str, duration: f64, ffmpeg_path: &str) -> Result<(), SomaError> {
    let vf = match transition {
        "FadeIn" => format!("fade=t=in:st=0:d={}", duration),
        "FadeOut" => {
            let dur = get_video_duration(input_path, ffmpeg_path)?;
            format!("fade=t=out:st={}:d={}", dur - duration, duration)
        }
        _ => return Ok(()),
    };
    let result = std::process::Command::new(ffmpeg_path)
        .args(&["-y", "-i", input_path, "-vf", &vf, "-c:v", "libx264", "-an", "-pix_fmt", "yuv420p", output_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg transition failed: {}", e)))?;
    if !result.status.success() {
        return Err(SomaError::Ffmpeg("ffmpeg transition failed".into()));
    }
    Ok(())
}

fn get_video_duration(video_path: &str, ffmpeg_path: &str) -> Result<f64, SomaError> {
    let _ = ffmpeg_path;
    let output = std::process::Command::new("ffprobe")
        .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", video_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
    String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
        .map_err(|e| SomaError::Ffmpeg(format!("parse duration failed: {}", e)))
}
