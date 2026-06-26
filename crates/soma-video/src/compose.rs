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
    ) -> Result<(), SomaError> {
        // 获取音频时长，视频总时长需略大于音频时长
        let audio_duration = self.ffmpeg.get_audio_duration(audio_path)?;
        let (target_w, target_h) = video_aspect.to_resolution();
        // 加 0.1 秒余量，避免视频比音频略短
        let required_duration = audio_duration + 0.1;

        let output_dir = std::path::Path::new(output_path).parent().unwrap_or(std::path::Path::new("."));
        std::fs::create_dir_all(output_dir).map_err(SomaError::Io)?;

        let mut clip_files: Vec<String> = Vec::new();
        let mut total_duration: f64 = 0.0;

        // 逐个裁剪素材视频，直到累计时长满足要求
        for (i, video_path) in video_paths.iter().enumerate() {
            if total_duration >= required_duration {
                break;
            }
            // 获取素材实际时长，若失败则使用最大片段时长
            let clip_dur = self.ffmpeg.get_video_duration(video_path).unwrap_or(max_clip_duration as f64);
            // 取素材时长和最大片段时长的较小值
            let actual_dur = clip_dur.min(max_clip_duration as f64);

            // 裁剪并缩放到目标分辨率
            let clip_output = output_dir.join(format!("temp-clip-{}.mp4", i + 1)).to_string_lossy().to_string();
            self.ffmpeg.clip_and_resize(video_path, &clip_output, target_w, target_h, 0.0, actual_dur)?;
            clip_files.push(clip_output);
            total_duration += actual_dur;
        }

        // 只有一个片段时直接复制，无需拼接
        if clip_files.len() == 1 {
            let _ = std::fs::copy(&clip_files[0], output_path);
            let _ = std::fs::remove_file(&clip_files[0]);
            return Ok(());
        }

        // 拼接所有片段
        self.ffmpeg.concat_clips(&clip_files, output_path)?;

        // 清理临时片段文件
        for f in &clip_files {
            let _ = std::fs::remove_file(f);
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
