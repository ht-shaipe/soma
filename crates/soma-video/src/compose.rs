/// 视频合成编排模块
///
/// 提供 [`VideoComposer`] 结构体，负责视频合成的高层编排逻辑：
/// - 将多个素材视频按音频时长裁剪、缩放并拼接
/// - 调用 FFmpeg 生成最终视频（含音频、字幕、背景音乐）
/// - 根据配置获取背景音乐文件
use soma_core::error::SomaError;
use soma_core::models::VideoAspect;
use crate::ffmpeg::Ffmpeg;

/// 视频合成编排器
///
/// 封装 Ffmpeg 实例，提供视频合成的业务流程编排，
/// 包括素材裁剪拼接、最终视频生成和背景音乐选择。
pub struct VideoComposer {
    /// FFmpeg 命令行封装实例
    ffmpeg: Ffmpeg,
}

impl VideoComposer {
    /// 创建 VideoComposer 实例
    ///
    /// # 参数
    /// - `ffmpeg`: 已配置的 Ffmpeg 实例
    pub fn new(ffmpeg: Ffmpeg) -> Self {
        Self { ffmpeg }
    }

    /// 获取内部 Ffmpeg 实例的引用
    pub fn ffmpeg(&self) -> &Ffmpeg {
        &self.ffmpeg
    }

    /// 将多个视频素材拼接为一段与音频时长匹配的视频
    ///
    /// 根据音频时长计算所需视频总时长，依次裁剪每个素材视频片段
    /// （限制单片段最大时长），缩放到目标分辨率后拼接为完整视频。
    ///
    /// # 参数
    /// - `video_paths`: 素材视频路径列表
    /// - `audio_path`: 音频文件路径（用于确定所需视频时长）
    /// - `output_path`: 输出视频路径
    /// - `video_aspect`: 目标视频宽高比
    /// - `max_clip_duration`: 单个片段最大时长（秒）
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回 SomaError
    pub fn combine_videos(
        &self,
        video_paths: &[String],
        audio_path: &str,
        output_path: &str,
        video_aspect: &VideoAspect,
        max_clip_duration: u32,
        transition_mode: &str,
    ) -> Result<(), SomaError> {
        let audio_duration = self.ffmpeg.get_audio_duration(audio_path)?;
        let (target_w, target_h) = video_aspect.to_resolution();
        let required_duration = audio_duration + 0.1;

        let output_dir = std::path::Path::new(output_path).parent().unwrap_or(std::path::Path::new("."));
        std::fs::create_dir_all(output_dir).map_err(SomaError::Io)?;

        let mut segments: Vec<(String, f64)> = Vec::new();
        let mut accumulated: f64 = 0.0;
        for (i, video_path) in video_paths.iter().enumerate() {
            if accumulated >= required_duration {
                break;
            }
            let clip_dur = self.ffmpeg.get_video_duration(video_path).unwrap_or(max_clip_duration as f64);
            let mut start: f64 = 0.0;
            let mut seg_idx: usize = 0;
            while start < clip_dur && accumulated < required_duration {
                let remaining = required_duration - accumulated;
                let seg_dur = (max_clip_duration as f64).min(clip_dur - start).min(remaining);
                if seg_dur <= 0.0 {
                    break;
                }
                let clip_output = output_dir.join(format!("temp-seg-{}-{}.mp4", i, seg_idx)).to_string_lossy().to_string();
                self.ffmpeg.clip_and_resize(video_path, &clip_output, target_w, target_h, start, seg_dur)?;
                segments.push((clip_output, seg_dur));
                start += seg_dur;
                accumulated += seg_dur;
                seg_idx += 1;
            }
        }

        let total_seg_duration: f64 = segments.iter().map(|(_, d)| *d).sum();
        if total_seg_duration < required_duration && !segments.is_empty() {
            let base_count = segments.len();
            let mut idx = 0;
            while accumulated < required_duration {
                let remaining = required_duration - accumulated;
                let (ref src_path, dur) = segments[idx % base_count];
                let use_dur = dur.min(remaining);
                segments.push((src_path.clone(), use_dur));
                accumulated += use_dur;
                idx += 1;
            }
        }

        // 对每个片段应用转场特效（none/shuffle 以外跳过）
        let transition_duration = 1.0;
        let ffmpeg_path = &self.ffmpeg.path;
        let codec = &self.ffmpeg.codec;
        if transition_mode != "none" && !transition_mode.is_empty() {
            let mut transitioned = Vec::new();
            for (seg_i, (ref seg_path, seg_dur)) in segments.iter().enumerate() {
                let trans_output = output_dir.join(format!("temp-trans-{}.mp4", seg_i)).to_string_lossy().to_string();
                if transition_mode == "Shuffle" {
                    crate::effects::apply_shuffle_transition(seg_path, &trans_output, transition_duration, ffmpeg_path, codec)?;
                } else {
                    crate::effects::apply_transition(seg_path, &trans_output, transition_mode, transition_duration, ffmpeg_path, "left", codec)?;
                }
                transitioned.push((trans_output, *seg_dur));
            }
            // 清理原始片段
            let mut deleted: std::collections::HashSet<String> = std::collections::HashSet::new();
            for (p, _) in &segments {
                if deleted.insert(p.clone()) {
                    let _ = std::fs::remove_file(p);
                }
            }
            segments = transitioned;
        }

        let clip_files: Vec<String> = segments.iter().map(|(p, _)| p.clone()).collect();

        if clip_files.is_empty() {
            return Err(SomaError::Ffmpeg("no video clips to combine".into()));
        }

        if clip_files.len() == 1 {
            let _ = std::fs::copy(&clip_files[0], output_path);
            let _ = std::fs::remove_file(&clip_files[0]);
            return Ok(());
        }

        self.ffmpeg.concat_clips(&clip_files, output_path)?;

        let mut deleted: std::collections::HashSet<String> = std::collections::HashSet::new();
        for f in &clip_files {
            if deleted.insert(f.clone()) {
                let _ = std::fs::remove_file(f);
            }
        }

        Ok(())
    }

    /// 生成最终视频（合并视频、音频、字幕和背景音乐）
    ///
    /// 委托给 Ffmpeg::generate_video 执行实际的合并操作。
    ///
    /// # 参数
    /// - `combined_path`: 已拼接的无声视频路径
    /// - `audio_path`: 配音音频路径
    /// - `subtitle_path`: 字幕文件路径
    /// - `output_path`: 输出视频路径
    /// - `params`: 视频参数配置
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回 SomaError
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

    /// 获取背景音乐文件路径
    ///
    /// 优先使用用户指定的背景音乐文件，依次在歌曲目录和原始路径中查找。
    /// 若未指定文件但 bgm_type 为 "random"，则从歌曲目录中随机选取一个 mp3 文件。
    ///
    /// # 参数
    /// - `bgm_type`: 背景音乐类型，"random" 表示随机选取
    /// - `bgm_file`: 指定的背景音乐文件名或路径
    ///
    /// # 返回
    /// 找到返回文件路径字符串，未找到返回空字符串
    pub fn get_bgm_file(&self, bgm_type: &str, bgm_file: &str) -> String {
        if !bgm_file.is_empty() {
            // 优先在歌曲目录中查找指定文件
            let song_dir = soma_core::utils::song_dir();
            let resolved = song_dir.join(bgm_file);
            if resolved.exists() {
                return resolved.to_string_lossy().to_string();
            }
            // 其次尝试作为绝对/相对路径直接查找
            if std::path::Path::new(bgm_file).exists() {
                return bgm_file.to_string();
            }
            // 文件不存在，返回空
            return String::new();
        }

        // bgm_type 为 "random" 时，从歌曲目录随机选取 mp3
        if bgm_type == "random" {
            let song_dir = soma_core::utils::song_dir();
            if let Ok(entries) = std::fs::read_dir(&song_dir) {
                // 收集所有 mp3 文件
                let files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("mp3"))
                    .map(|e| e.path().to_string_lossy().to_string())
                    .collect();
                if !files.is_empty() {
                    use rand::Rng;
                    // 随机选取一个文件
                    let idx = rand::rng().random_range(0..files.len());
                    return files[idx].clone();
                }
            }
        }
        String::new()
    }
}
