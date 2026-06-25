use soma_core::error::SomaError;
use std::path::Path;

pub struct Ffmpeg {
    path: String,
    threads: u32,
    codec: String,
}

const DEFAULT_CODEC: &str = "libx264";
const SUPPORTED_CODECS: &[&str] = &[
    "libx264", "h264_nvenc", "h264_amf", "h264_qsv", "h264_mf", "h264_videotoolbox",
];
const FPS: u32 = 30;

impl Ffmpeg {
    pub fn new(path: &str, threads: u32, codec: &str) -> Self {
        let effective_codec = if SUPPORTED_CODECS.contains(&codec) { codec } else { DEFAULT_CODEC };
        Self { path: path.to_string(), threads, codec: effective_codec.to_string() }
    }

    pub fn get_audio_duration(&self, audio_path: &str) -> Result<f64, SomaError> {
        let output = std::process::Command::new("ffprobe")
            .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", audio_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
        String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
            .map_err(|e| SomaError::Ffmpeg(format!("parse duration failed: {}", e)))
    }

    pub fn concat_clips(&self, clip_files: &[String], output_file: &str) -> Result<(), SomaError> {
        let output_dir = Path::new(output_file).parent().unwrap_or(Path::new("."));
        let concat_list = output_dir.join("ffmpeg-concat-list.txt");

        let mut content = String::new();
        for clip in clip_files {
            let abs = std::fs::canonicalize(clip).unwrap_or_else(|_| PathBuf::from(clip));
            let escaped = abs.to_string_lossy().replace('\\', "/").replace("'", "'\\''");
            content.push_str(&format!("file '{}'\n", escaped));
        }
        std::fs::write(&concat_list, &content).map_err(SomaError::Io)?;

        let result = std::process::Command::new(&self.path)
            .args(&[
                "-y", "-f", "concat", "-safe", "0",
                "-i", concat_list.to_string_lossy().as_ref(),
                "-c:v", &self.codec,
                "-threads", &self.threads.to_string(),
                "-pix_fmt", "yuv420p",
                output_file,
            ])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg concat failed: {}", e)))?;

        let _ = std::fs::remove_file(&concat_list);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            if self.codec != DEFAULT_CODEC {
                return self.concat_clips_fallback(clip_files, output_file, &concat_list);
            }
            return Err(SomaError::Ffmpeg(format!("ffmpeg concat failed: {}", stderr)));
        }
        Ok(())
    }

    fn concat_clips_fallback(&self, clip_files: &[String], output_file: &str, concat_list: &Path) -> Result<(), SomaError> {
        let result = std::process::Command::new(&self.path)
            .args(&[
                "-y", "-f", "concat", "-safe", "0",
                "-i", concat_list.to_string_lossy().as_ref(),
                "-c:v", DEFAULT_CODEC,
                "-threads", &self.threads.to_string(),
                "-pix_fmt", "yuv420p",
                output_file,
            ])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg concat fallback failed: {}", e)))?;
        let _ = std::fs::remove_file(concat_list);
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg concat fallback failed: {}", stderr)));
        }
        Ok(())
    }

    pub fn clip_and_resize(
        &self,
        input_path: &str,
        output_path: &str,
        width: u32,
        height: u32,
        start: f64,
        duration: f64,
    ) -> Result<(), SomaError> {
        let result = std::process::Command::new(&self.path)
            .args(&[
                "-y",
                "-ss", &start.to_string(),
                "-i", input_path,
                "-t", &duration.to_string(),
                "-vf", &format!("scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black", width, height, width, height),
                "-c:v", &self.codec,
                "-an",
                "-pix_fmt", "yuv420p",
                "-r", &FPS.to_string(),
                output_path,
            ])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg clip+resize failed: {}", e)))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg clip+resize failed: {}", stderr)));
        }
        Ok(())
    }

    pub fn add_transition(&self, input_path: &str, output_path: &str, transition: &str, duration: f64) -> Result<(), SomaError> {
        let vf = match transition {
            "FadeIn" => format!("fade=t=in:st=0:d={}", duration),
            "FadeOut" => {
                let dur = self.get_video_duration(input_path)?;
                format!("fade=t=out:st={}:d={}", dur - duration, duration)
            }
            _ => return Ok(()),
        };
        let result = std::process::Command::new(&self.path)
            .args(&["-y", "-i", input_path, "-vf", &vf, "-c:v", &self.codec, "-an", "-pix_fmt", "yuv420p", output_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg transition failed: {}", e)))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg transition failed: {}", stderr)));
        }
        Ok(())
    }

    pub fn generate_video(
        &self,
        video_path: &str,
        audio_path: &str,
        subtitle_path: &str,
        output_path: &str,
        params: &soma_core::models::VideoParams,
    ) -> Result<(), SomaError> {
        let aspect = params.get_video_aspect();
        let (w, _h) = aspect.to_resolution();

        let subtitle_enabled = params.get_subtitle_enabled();
        let font_name = params.font_name.as_deref().unwrap_or("STHeitiMedium.ttc");
        let font_size = params.font_size.unwrap_or(60);
        let text_color = params.text_fore_color.as_deref().unwrap_or("#FFFFFF");
        let stroke_color = params.stroke_color.as_deref().unwrap_or("#000000");
        let stroke_width = params.stroke_width.unwrap_or(1.5);

        let subtitle_position = params.subtitle_position.as_deref().unwrap_or("bottom");
        let y_pos = match subtitle_position {
            "top" => format!("(h*5/100)"),
            "center" => format!("(h/2-th/2)"),
            _ => format!("(h*90/100)"),
        };

        let bgm_file = params.bgm_file.as_deref().unwrap_or("");
        let bgm_volume = params.get_bgm_volume();
        let voice_volume = params.get_voice_volume();

        let mut cmd_args = vec![
            "-y".to_string(),
            "-i".to_string(), video_path.to_string(),
            "-i".to_string(), audio_path.to_string(),
        ];

        let bgm_index = if !bgm_file.is_empty() {
            cmd_args.push("-i".to_string());
            cmd_args.push(bgm_file.to_string());
            Some(2u32)
        } else {
            None
        };

        if subtitle_enabled && !subtitle_path.is_empty() {
            let font_path = soma_core::utils::font_dir().join(font_name);
            let font_path_str = if font_path.exists() {
                font_path.to_string_lossy().to_string()
            } else {
                font_name.to_string()
            };
            let drawtext = format!(
                "drawtext=fontfile='{}':text='{}':fontsize={}:fontcolor={}:borderw={}:bordercolor={}:y={}",
                font_path_str.replace(':', "\\:"), "%{pts\\:text}", font_size, text_color, stroke_width, stroke_color, y_pos
            );
            cmd_args.push("-vf".to_string());
            cmd_args.push(drawtext);
        }

        cmd_args.push("-c:v".to_string());
        cmd_args.push(self.codec.clone());
        cmd_args.push("-c:a".to_string());
        cmd_args.push("aac".to_string());
        cmd_args.push("-b:a".to_string());
        cmd_args.push("192k".to_string());
        cmd_args.push("-pix_fmt".to_string());
        cmd_args.push("yuv420p".to_string());
        cmd_args.push("-shortest".to_string());

        if let Some(bgm_i) = bgm_index {
            cmd_args.push("-filter_complex".to_string());
            cmd_args.push(format!(
                "[1:a]volume={}[a1];[{}:a]volume={}[a2];[a1][a2]amix=inputs=2:duration=longest[aout]",
                voice_volume, bgm_i, bgm_volume
            ));
            cmd_args.push("-map".to_string());
            cmd_args.push("0:v".to_string());
            cmd_args.push("-map".to_string());
            cmd_args.push("[aout]".to_string());
        } else {
            cmd_args.push("-map".to_string());
            cmd_args.push("0:v".to_string());
            cmd_args.push("-map".to_string());
            cmd_args.push("1:a".to_string());
        }

        cmd_args.push(output_path.to_string());

        let result = std::process::Command::new(&self.path)
            .args(cmd_args.iter().map(|s| s.as_str()))
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg generate_video failed: {}", e)))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg generate_video failed: {}", stderr)));
        }
        Ok(())
    }

    pub fn get_video_duration(&self, video_path: &str) -> Result<f64, SomaError> {
        let output = std::process::Command::new("ffprobe")
            .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", video_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
        String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
            .map_err(|e| SomaError::Ffmpeg(format!("parse duration failed: {}", e)))
    }

    pub fn get_video_resolution(&self, video_path: &str) -> Result<(u32, u32), SomaError> {
        let output = std::process::Command::new("ffprobe")
            .args(&["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=s=x:p=0", video_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffprobe resolution failed: {}", e)))?;
        let res_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = res_str.split('x').collect();
        if parts.len() == 2 {
            let w = parts[0].parse::<u32>().unwrap_or(0);
            let h = parts[1].parse::<u32>().unwrap_or(0);
            if w > 0 && h > 0 {
                return Ok((w, h));
            }
        }
        Err(SomaError::Ffmpeg("failed to get video resolution".into()))
    }

    pub fn preprocess_local_materials(&self, materials: &[soma_core::models::MaterialInfo], clip_duration: u32, aspect: &soma_core::models::VideoAspect) -> Result<Vec<String>, SomaError> {
        let (target_w, target_h) = aspect.to_resolution();
        let mut result = Vec::new();

        for mat in materials {
            let path = &mat.url;
            if !Path::new(path).exists() {
                continue;
            }
            let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
            if soma_core::models::FILE_TYPE_IMAGES.contains(&ext) {
                let output = format!("{}.mp4", path.trim_end_matches(&format!(".{}", ext)));
                let dur = clip_duration as f64;
                self.image_to_video(path, &output, dur, target_w, target_h)?;
                result.push(output);
            } else if soma_core::models::FILE_TYPE_VIDEOS.contains(&ext) {
                result.push(path.clone());
            }
        }
        Ok(result)
    }

    fn image_to_video(&self, image_path: &str, output_path: &str, duration: f64, width: u32, height: u32) -> Result<(), SomaError> {
        let result = std::process::Command::new(&self.path)
            .args(&[
                "-y", "-loop", "1",
                "-i", image_path,
                "-t", &duration.to_string(),
                "-vf", &format!("scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black", width, height, width, height),
                "-c:v", &self.codec,
                "-pix_fmt", "yuv420p",
                "-r", &FPS.to_string(),
                output_path,
            ])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg image_to_video failed: {}", e)))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg image_to_video failed: {}", stderr)));
        }
        Ok(())
    }
}

use std::path::PathBuf;
