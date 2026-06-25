use soma_core::error::SomaError;
use soma_core::models::VideoAspect;
use crate::ffmpeg::Ffmpeg;

pub struct VideoComposer {
    ffmpeg: Ffmpeg,
}

impl VideoComposer {
    pub fn new(ffmpeg: Ffmpeg) -> Self {
        Self { ffmpeg }
    }

    pub fn combine_videos(
        &self,
        video_paths: &[String],
        audio_path: &str,
        output_path: &str,
        video_aspect: &VideoAspect,
        max_clip_duration: u32,
    ) -> Result<(), SomaError> {
        let audio_duration = self.ffmpeg.get_audio_duration(audio_path)?;
        let (target_w, target_h) = video_aspect.to_resolution();
        let required_duration = audio_duration + 0.1;

        let output_dir = std::path::Path::new(output_path).parent().unwrap_or(std::path::Path::new("."));
        std::fs::create_dir_all(output_dir).map_err(SomaError::Io)?;

        let mut clip_files: Vec<String> = Vec::new();
        let mut total_duration: f64 = 0.0;

        for (i, video_path) in video_paths.iter().enumerate() {
            if total_duration >= required_duration {
                break;
            }
            let clip_dur = self.ffmpeg.get_video_duration(video_path).unwrap_or(max_clip_duration as f64);
            let actual_dur = clip_dur.min(max_clip_duration as f64);

            let clip_output = output_dir.join(format!("temp-clip-{}.mp4", i + 1)).to_string_lossy().to_string();
            self.ffmpeg.clip_and_resize(video_path, &clip_output, target_w, target_h, 0.0, actual_dur)?;
            clip_files.push(clip_output);
            total_duration += actual_dur;
        }

        if clip_files.len() == 1 {
            let _ = std::fs::copy(&clip_files[0], output_path);
            let _ = std::fs::remove_file(&clip_files[0]);
            return Ok(());
        }

        self.ffmpeg.concat_clips(&clip_files, output_path)?;

        for f in &clip_files {
            let _ = std::fs::remove_file(f);
        }

        Ok(())
    }

    pub fn generate_video(
        &self,
        combined_path: &str,
        audio_path: &str,
        subtitle_path: &str,
        output_path: &str,
        params: &soma_core::models::VideoParams,
    ) -> Result<(), SomaError> {
        self.ffmpeg.generate_video(combined_path, audio_path, subtitle_path, output_path, params)
    }

    pub fn get_bgm_file(&self, bgm_type: &str, bgm_file: &str) -> String {
        if !bgm_file.is_empty() {
            let song_dir = soma_core::utils::song_dir();
            let resolved = song_dir.join(bgm_file);
            if resolved.exists() {
                return resolved.to_string_lossy().to_string();
            }
            if std::path::Path::new(bgm_file).exists() {
                return bgm_file.to_string();
            }
            return String::new();
        }

        if bgm_type == "random" {
            let song_dir = soma_core::utils::song_dir();
            if let Ok(entries) = std::fs::read_dir(&song_dir) {
                let files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("mp3"))
                    .map(|e| e.path().to_string_lossy().to_string())
                    .collect();
                if !files.is_empty() {
                    use rand::Rng;
                    let idx = rand::rng().random_range(0..files.len());
                    return files[idx].clone();
                }
            }
        }
        String::new()
    }
}
