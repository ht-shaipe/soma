/// FFmpeg 命令行封装模块
///
/// 封装了 FFmpeg/ffprobe 的常用命令调用，包括：
/// - 视频片段裁剪与缩放（clip_and_resize）
/// - 视频片段拼接（concat_clips）
/// - 转场特效添加（add_transition）
/// - 最终视频生成（generate_video）：合并视频、音频、字幕和背景音乐
/// - 图片转视频（image_to_video）
/// - 媒体信息查询（时长、分辨率）
use soma_core::error::SomaError;
use std::path::Path;
use std::time::Duration;

const FFMPEG_TIMEOUT_SECS: u64 = 600;

fn run_with_timeout(cmd: &mut std::process::Command) -> Result<std::process::Output, SomaError> {
    let mut child = cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg spawn failed: {}", e)))?;
    let timeout = Duration::from_secs(FFMPEG_TIMEOUT_SECS);
    match child.wait_timeout(timeout) {
        Ok(Some(status)) => {
            let output = child.wait_with_output()
                .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg output read failed: {}", e)))?;
            if !status.success() {
                return Ok(output);
            }
            Ok(output)
        }
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(SomaError::Ffmpeg(format!("ffmpeg timed out after {}s", FFMPEG_TIMEOUT_SECS)))
        }
        Err(e) => {
            let _ = child.kill();
            Err(SomaError::Ffmpeg(format!("ffmpeg wait error: {}", e)))
        }
    }
}

trait ChildWaitTimeout {
    fn wait_timeout(&mut self, timeout: Duration) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl ChildWaitTimeout for std::process::Child {
    fn wait_timeout(&mut self, timeout: Duration) -> std::io::Result<Option<std::process::ExitStatus>> {
        match self.try_wait() {
            Ok(Some(status)) => return Ok(Some(status)),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
        let start = std::time::Instant::now();
        loop {
            match self.try_wait() {
                Ok(Some(status)) => return Ok(Some(status)),
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// 将 #RRGGBB 颜色字符串转换为 ASS 字幕格式 &H00BBGGRR
fn hex_to_ass_color(hex: &str) -> String {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = &clean[0..2];
        let g = &clean[2..4];
        let b = &clean[4..6];
        format!("&H00{}{}{}", b, g, r)
    } else {
        "&H00FFFFFF".to_string()
    }
}

/// 将 #RRGGBB 颜色转换为 ASS 格式 &HAABBGGRR（带透明度）
fn hex_to_ass_color_with_alpha(hex: &str, alpha: &str) -> String {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = &clean[0..2];
        let g = &clean[2..4];
        let b = &clean[4..6];
        format!("&H{}{}{}{}", alpha, b, g, r)
    } else {
        format!("&H{}FFFFFF", alpha)
    }
}

/// 从 VideoParams.text_background_color 解析出 #RRGGBB 颜色字符串
///
/// text_background_color 可为：
/// - bool true → "#000000"（黑色背景）
/// - bool false / null → 空字符串（无背景）
/// - 字符串 "#RRGGBB" → 该颜色
fn resolve_background_color(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Bool(true) => "#000000".to_string(),
        serde_json::Value::Bool(false) => String::new(),
        serde_json::Value::String(s) => {
            if s.starts_with('#') && s.len() == 7 {
                s.clone()
            } else if !s.is_empty() {
                format!("#{}", s)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

/// FFmpeg 命令行封装结构体
///
/// 封装了 FFmpeg 可执行文件路径、编码线程数和视频编码器配置，
/// 提供视频处理的各种便捷方法。
pub struct Ffmpeg {
    /// FFmpeg 可执行文件路径
    pub path: String,
    /// 编码使用的线程数
    pub threads: u32,
    /// 视频编码器名称（如 libx264、h264_nvenc 等）
    pub codec: String,
}

/// 默认视频编码器：H.264 软编码
const DEFAULT_CODEC: &str = "libx264";
/// 支持的视频编码器列表，涵盖软编码和各平台硬编码
const SUPPORTED_CODECS: &[&str] = &[
    "libx264", "h264_nvenc", "h264_amf", "h264_qsv", "h264_mf", "h264_videotoolbox",
];
/// 默认输出帧率
const FPS: u32 = 30;

impl Ffmpeg {
    /// 创建 Ffmpeg 实例
    ///
    /// # 参数
    /// - `path`: FFmpeg 可执行文件路径
    /// - `threads`: 编码线程数
    /// - `codec`: 视频编码器名称，若不在支持列表中则回退为默认编码器 libx264
    ///
    /// # 返回
    /// 配置好的 Ffmpeg 实例
    pub fn new(path: &str, threads: u32, codec: &str) -> Self {
        let effective_codec = if SUPPORTED_CODECS.contains(&codec) { codec } else { DEFAULT_CODEC };
        Self { path: path.to_string(), threads, codec: effective_codec.to_string() }
    }

    /// 获取音频文件时长（秒）
    ///
    /// 使用 ffprobe 查询音频文件的 format duration 信息。
    ///
    /// # 参数
    /// - `audio_path`: 音频文件路径
    ///
    /// # 返回
    /// 成功返回时长（f64 秒），失败返回 SomaError
    pub fn get_audio_duration(&self, audio_path: &str) -> Result<f64, SomaError> {
        let output = std::process::Command::new("ffprobe")
            // -v error: 只输出错误信息
            // -show_entries format=duration: 只显示 format 中的 duration 字段
            // -of default=noprint_wrappers=1:nokey=1: 不打印包裹行和键名，仅输出数值
            .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", audio_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
        String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
            .map_err(|e| SomaError::Ffmpeg(format!("parse duration failed: {}", e)))
    }

    /// 拼接多个视频片段为一个完整视频
    ///
    /// 使用 FFmpeg 的 concat 分离器，先将所有片段路径写入临时文件列表，
    /// 再通过 concat 协议拼接。若指定编码器失败，自动回退为默认编码器重试。
    ///
    /// # 参数
    /// - `clip_files`: 待拼接的视频片段路径列表
    /// - `output_file`: 输出视频文件路径
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回 SomaError
    pub fn concat_clips(&self, clip_files: &[String], output_file: &str) -> Result<(), SomaError> {
        let output_dir = Path::new(output_file).parent().unwrap_or(Path::new("."));
        // 临时拼接列表文件路径
        let concat_list = output_dir.join("ffmpeg-concat-list.txt");

        // 构建 concat 列表文件内容，每行格式为：file '绝对路径'
        let mut content = String::new();
        for clip in clip_files {
            // 转换为绝对路径以确保 concat 分离器能正确解析
            let abs = std::fs::canonicalize(clip).unwrap_or_else(|_| PathBuf::from(clip));
            // 转义路径中的反斜杠和单引号，避免 concat 列表解析错误
            let escaped = abs.to_string_lossy().replace('\\', "/").replace("'", "'\\''");
            content.push_str(&format!("file '{}'\n", escaped));
        }
        std::fs::write(&concat_list, &content).map_err(SomaError::Io)?;

        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&[
                "-y",
                "-f", "concat",
                "-safe", "0",
                "-max_delay", "500000",
                "-i", concat_list.to_string_lossy().as_ref(),
                "-c:v", &self.codec,
                "-threads", &self.threads.to_string(),
                "-pix_fmt", "yuv420p",
                output_file,
            ]))?;

        // 拼接完成后删除临时列表文件
        let _ = std::fs::remove_file(&concat_list);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            // 若当前编码器不是默认编码器，尝试回退到默认编码器重新拼接
            if self.codec != DEFAULT_CODEC {
                return self.concat_clips_fallback(clip_files, output_file, &concat_list);
            }
            return Err(SomaError::Ffmpeg(format!("ffmpeg concat failed: {}", stderr)));
        }
        Ok(())
    }

    /// 拼接回退方法：使用默认编码器 libx264 重新拼接
    ///
    /// 当指定编码器拼接失败时，回退到 libx264 软编码重新尝试。
    fn concat_clips_fallback(&self, _clip_files: &[String], output_file: &str, concat_list: &Path) -> Result<(), SomaError> {
        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&[
                "-y", "-f", "concat", "-safe", "0",
                "-i", concat_list.to_string_lossy().as_ref(),
                "-c:v", DEFAULT_CODEC,
                "-threads", &self.threads.to_string(),
                "-pix_fmt", "yuv420p",
                output_file,
            ]))?;
        // 清理临时列表文件
        let _ = std::fs::remove_file(concat_list);
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg concat fallback failed: {}", stderr)));
        }
        Ok(())
    }

    /// 裁剪视频片段并缩放到指定分辨率
    ///
    /// 从源视频中截取指定时间段，同时缩放到目标分辨率。
    /// 缩放时保持原始宽高比，不足部分用黑边填充（letterbox/pillarbox）。
    ///
    /// # 参数
    /// - `input_path`: 输入视频路径
    /// - `output_path`: 输出视频路径
    /// - `width`: 目标宽度（像素）
    /// - `height`: 目标高度（像素）
    /// - `start`: 截取起始时间（秒）
    /// - `duration`: 截取时长（秒）
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回 SomaError
    pub fn clip_and_resize(
        &self,
        input_path: &str,
        output_path: &str,
        width: u32,
        height: u32,
        start: f64,
        duration: f64,
    ) -> Result<(), SomaError> {
        let vf = format!(
            "split[original][bg];[bg]scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h},gblur=sigma=20[blurred];\
             [original]scale={w}:{h}:force_original_aspect_ratio=decrease[fg];\
             [blurred][fg]overlay=(W-w)/2:(H-h)/2",
            w = width, h = height
        );
        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&[
                "-y",
                "-ss", &start.to_string(),
                "-i", input_path,
                "-t", &duration.to_string(),
                "-vf", &vf,
                "-c:v", &self.codec,
                "-an",
                "-pix_fmt", "yuv420p",
                "-r", &FPS.to_string(),
                output_path,
            ]))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg clip+resize failed: {}", stderr)));
        }
        Ok(())
    }

    /// 为视频添加转场特效（淡入/淡出）
    ///
    /// 使用 FFmpeg 的 fade 滤镜为视频添加淡入或淡出效果。
    ///
    /// # 参数
    /// - `input_path`: 输入视频路径
    /// - `output_path`: 输出视频路径
    /// - `transition`: 转场类型，"FadeIn" 为淡入，"FadeOut" 为淡出，其他值不做处理
    /// - `duration`: 转场持续时间（秒）
    ///
    /// # 返回
    /// 成功返回 Ok(())，无法识别的转场类型也返回 Ok(())（跳过），失败返回 SomaError
    pub fn add_transition(&self, input_path: &str, output_path: &str, transition: &str, duration: f64) -> Result<(), SomaError> {
        let vf = match transition {
            "FadeIn" => format!("fade=t=in:st=0:d={}", duration),
            "FadeOut" => {
                let dur = self.get_video_duration(input_path)?;
                format!("fade=t=out:st={}:d={}", dur - duration, duration)
            }
            "Dissolve" => {
                let dur = self.get_video_duration(input_path)?;
                format!("fade=t=in:st=0:d={},fade=t=out:st={}:d={}", duration.min(dur * 0.3), dur - duration, duration)
            }
            _ => return Ok(()),
        };
        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&["-y", "-i", input_path, "-vf", &vf, "-c:v", &self.codec, "-an", "-pix_fmt", "yuv420p", output_path]))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg transition failed: {}", stderr)));
        }
        Ok(())
    }

    /// 生成最终视频（合并视频流、音频流、字幕和背景音乐）
    ///
    /// 这是视频生成的核心方法，将无声视频、配音音频、可选字幕和可选背景音乐
    /// 合成为最终的视频文件。
    ///
    /// # 参数
    /// - `video_path`: 无声视频文件路径
    /// - `audio_path`: 配音音频文件路径
    /// - `subtitle_path`: 字幕文件路径（当前通过 drawtext 滤镜实时渲染，此参数暂未直接使用）
    /// - `output_path`: 输出视频文件路径
    /// - `params`: 视频参数，包含分辨率、字幕样式、音量等配置
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回 SomaError
    pub fn generate_video(
        &self,
        video_path: &str,
        audio_path: &str,
        subtitle_path: &str,
        output_path: &str,
        params: &soma_core::models::VideoParams,
    ) -> Result<(), SomaError> {
        let aspect = params.get_video_aspect();
        let (_w, _h) = aspect.to_resolution();

        // 字幕参数，带默认值
        let subtitle_enabled = params.get_subtitle_enabled();
        let font_name = params.font_name.as_deref().unwrap_or("STHeitiMedium.ttc");
        let font_size = params.font_size.unwrap_or(26);
        let text_color = params.text_fore_color.as_deref().unwrap_or("#FFFFFF");
        let stroke_color = params.stroke_color.as_deref().unwrap_or("#000000");
        let stroke_width = params.stroke_width.unwrap_or(3.0);

        let subtitle_position = params.subtitle_position.as_deref().unwrap_or("bottom");

        // 背景音乐和音量参数
        let bgm_file = params.bgm_file.as_deref().unwrap_or("");
        let bgm_volume = params.get_bgm_volume();
        let voice_volume = params.get_voice_volume();

        // 构建 FFmpeg 命令参数
        let mut cmd_args = vec![
            "-y".to_string(),
            "-i".to_string(), video_path.to_string(),   // 输入0：视频流
            "-i".to_string(), audio_path.to_string(),    // 输入1：配音音频流
        ];

        // 如果有背景音乐，添加第三个输入流
        let bgm_index = if !bgm_file.is_empty() {
            cmd_args.push("-stream_loop".to_string());
            cmd_args.push("-1".to_string());
            cmd_args.push("-i".to_string());
            cmd_args.push(bgm_file.to_string());         // 输入2：背景音乐流
            Some(2u32)
        } else {
            None
        };

        // 如果启用字幕，使用 subtitles 滤镜加载 SRT 文件渲染字幕
        if subtitle_enabled && !subtitle_path.is_empty() {
            // 检测 FFmpeg 是否支持 subtitles 滤镜（需要 libass）
            let has_subtitles_filter = std::process::Command::new(&self.path)
                .args(["-filters"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains("subtitles"))
                .unwrap_or(false);
            if !has_subtitles_filter {
                log::warn!("FFmpeg 不支持 subtitles 滤镜（缺少 libass），跳过字幕渲染");
            } else {
            // 优先从字体目录查找字体文件，找不到则直接使用字体名称
            let font_path = soma_core::utils::font_dir().join(font_name);
            let font_path_str = if font_path.exists() {
                font_path.to_string_lossy().to_string()
            } else {
                font_name.to_string()
            };
            // subtitles 滤镜会按 SRT 时间轴逐段渲染字幕文本
            // force_style: 设置字幕样式（字体、大小、颜色、描边、位置等）
            //   FontName: 字体文件路径
            //   FontSize: 字体大小
            //   PrimaryColour: 字体颜色（ASS 格式 &H00BBGGRR，注意 BGR 顺序且高位字节 00=不透明）
            //   OutlineColour: 描边颜色（同上 BGR 格式）
            //   Outline: 描边宽度
            //   Alignment: 对齐方式 2=底部居中, 5=上方居中, 6=上方左对齐, 8=顶部居中, 9=顶部左对齐
            //   MarginV: 垂直边距（像素）
            let (alignment, margin_v) = match subtitle_position {
                "top" => (8, 30),
                "center" => (5, 0),
                "custom" => {
                    let pos = params.custom_position.unwrap_or(70.0) as u32;
                    (2, pos)
                },
                _ => (2, 30),
            };
            // 将 #RRGGBB 颜色转为 ASS 的 &H00BBGGRR 格式
            let ass_text_color = hex_to_ass_color(text_color);
            let ass_stroke_color = hex_to_ass_color(stroke_color);

            // 字幕背景样式
            // BackColour: 背景颜色（ASS 格式 &HAABBGGRR，AA=透明度 00=不透明 FF=全透明）
            // BorderStyle: 3=不透明底框背景, 4=不透明底框+描边
            let bg_style = if let Some(ref bg_color_val) = params.text_background_color {
                let bg_hex = resolve_background_color(bg_color_val);
                if !bg_hex.is_empty() {
                    let is_rounded = params.rounded_subtitle_background.unwrap_or(false);
                    // 圆角背景: 半透明(alpha=140 ≈ 0x8C)，否则不透明
                    let alpha = if is_rounded { "8C" } else { "00" };
                    let ass_bg = hex_to_ass_color_with_alpha(&bg_hex, alpha);
                    let border_type = if stroke_width > 0.0 { 4 } else { 3 };
                    // Shadow: 背景扩展量，模拟圆角/矩形的内边距
                    let shadow = if is_rounded { (font_size as f32 * 0.15).ceil() as u32 } else { (font_size as f32 * 0.3).ceil() as u32 };
                    format!(",BackColour={},BorderStyle={},Shadow={}", ass_bg, border_type, shadow)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let style = format!(
                "FontName={},FontSize={},PrimaryColour={},OutlineColour={},Outline={},Alignment={},MarginV={}{}",
                font_path_str.replace('\\', "\\\\").replace(':', "\\:"), font_size, ass_text_color, ass_stroke_color, stroke_width, alignment, margin_v, bg_style
            );
            let escaped_sub = subtitle_path.replace('\\', "/").replace(':', "\\:");
            let escaped_style = style.replace(',', "\\,").replace(':', "\\:");
            let sub_filter = format!("subtitles={}:force_style={}", escaped_sub, escaped_style);
            cmd_args.push("-vf".to_string());
            cmd_args.push(sub_filter);
            }
        }

        // 编码参数
        cmd_args.push("-c:v".to_string());
        cmd_args.push(self.codec.clone());      // 视频编码器
        cmd_args.push("-c:a".to_string());
        cmd_args.push("aac".to_string());       // 音频编码器：AAC
        cmd_args.push("-b:a".to_string());
        cmd_args.push("192k".to_string());      // 音频比特率：192kbps
        cmd_args.push("-pix_fmt".to_string());
        cmd_args.push("yuv420p".to_string());   // 像素格式
        cmd_args.push("-shortest".to_string()); // 以最短的流为准截断输出

        // 音频混合处理：背景音乐与配音混音
        if let Some(bgm_i) = bgm_index {
            let video_dur = self.get_video_duration(video_path).unwrap_or(60.0);
            let fade_start = (video_dur - 3.0).max(0.0);
            cmd_args.push("-filter_complex".to_string());
            cmd_args.push(format!(
                "[1:a]volume={}[a1];[{}:a]volume={},atrim=1:duration={:.3},afade=t=out:st={:.3}:d=3[a2];[a1][a2]amix=inputs=2:duration=longest[aout]",
                voice_volume, bgm_i, bgm_volume, video_dur, fade_start
            ));
            // 映射视频流和混合后的音频流
            cmd_args.push("-map".to_string());
            cmd_args.push("0:v".to_string());    // 取输入0的视频流
            cmd_args.push("-map".to_string());
            cmd_args.push("[aout]".to_string()); // 取混合后的音频流
        } else {
            // 无背景音乐时，直接映射视频和配音音频
            cmd_args.push("-map".to_string());
            cmd_args.push("0:v".to_string());    // 取输入0的视频流
            cmd_args.push("-map".to_string());
            cmd_args.push("1:a".to_string());    // 取输入1的音频流
        }

        cmd_args.push(output_path.to_string());

        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(cmd_args.iter().map(|s| s.as_str())))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg generate_video failed: {}", stderr)));
        }
        Ok(())
    }

    /// 获取视频文件时长（秒）
    ///
    /// 使用 ffprobe 查询视频文件的 format duration 信息。
    ///
    /// # 参数
    /// - `video_path`: 视频文件路径
    ///
    /// # 返回
    /// 成功返回时长（f64 秒），失败返回 SomaError
    pub fn get_video_duration(&self, video_path: &str) -> Result<f64, SomaError> {
        let output = std::process::Command::new("ffprobe")
            .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", video_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
        String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
            .map_err(|e| SomaError::Ffmpeg(format!("parse duration failed: {}", e)))
    }

    /// 获取视频文件分辨率
    ///
    /// 使用 ffprobe 查询视频流的宽高信息。
    ///
    /// # 参数
    /// - `video_path`: 视频文件路径
    ///
    /// # 返回
    /// 成功返回 (宽度, 高度)，失败返回 SomaError
    pub fn get_video_resolution(&self, video_path: &str) -> Result<(u32, u32), SomaError> {
        let output = std::process::Command::new("ffprobe")
            // -select_streams v:0: 选择第一个视频流
            // -show_entries stream=width,height: 显示宽高
            // -of csv=s=x:p=0: 用 x 分隔输出，不含前缀
            .args(&["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=s=x:p=0", video_path])
            .output()
            .map_err(|e| SomaError::Ffmpeg(format!("ffprobe resolution failed: {}", e)))?;
        let res_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // 解析 "宽x高" 格式的输出
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

    /// 预处理本地素材：将图片转为视频，过滤非视频文件
    ///
    /// 遍历素材列表，将图片文件转换为指定分辨率和时长的视频片段，
    /// 视频文件直接保留路径。不存在的文件会被跳过。
    ///
    /// # 参数
    /// - `materials`: 素材信息列表
    /// - `clip_duration`: 图片转视频时的单片段时长（秒）
    /// - `aspect`: 目标视频宽高比
    ///
    /// # 返回
    /// 成功返回处理后的视频文件路径列表，失败返回 SomaError
    pub fn preprocess_local_materials(&self, materials: &[soma_core::models::MaterialInfo], clip_duration: u32, aspect: &soma_core::models::VideoAspect) -> Result<Vec<String>, SomaError> {
        let (target_w, target_h) = aspect.to_resolution();
        let mut result = Vec::new();

        for mat in materials {
            let path = &mat.url;
            // 跳过不存在的文件
            if !Path::new(path).exists() {
                continue;
            }
            let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
            if soma_core::models::FILE_TYPE_IMAGES.contains(&ext) {
                // 图片文件：转换为同名的 .mp4 视频
                let output = format!("{}.mp4", path.trim_end_matches(&format!(".{}", ext)));
                let dur = clip_duration as f64;
                self.image_to_video(path, &output, dur, target_w, target_h)?;
                result.push(output);
            } else if soma_core::models::FILE_TYPE_VIDEOS.contains(&ext) {
                // 视频文件：直接使用原始路径
                result.push(path.clone());
            }
        }
        Ok(result)
    }

    /// 将静态图片转换为视频
    ///
    /// 使用 FFmpeg 的 -loop 1 参数循环播放图片，生成指定时长和分辨率的视频。
    /// 缩放策略与 clip_and_resize 相同：等比缩放 + 黑边填充居中。
    ///
    /// # 参数
    /// - `image_path`: 输入图片路径
    /// - `output_path`: 输出视频路径
    /// - `duration`: 视频时长（秒）
    /// - `width`: 目标宽度
    /// - `height`: 目标高度
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回 SomaError
    fn image_to_video(&self, image_path: &str, output_path: &str, duration: f64, width: u32, height: u32) -> Result<(), SomaError> {
        // 图片缩放效果：从1.0逐渐放大到1.0+clip_duration*0.03
        // 使用 zoompan 滤镜实现动态缩放，zoom 从 1.0 线性增长
        // zoom='min(zoom+0.0005,1.2)' 每帧增大0.0005，最大1.2倍
        // d=帧数 指定动画总帧数，x/y 居中
        let total_frames = (duration * FPS as f64).ceil() as u32;
        let zoom_expr = format!("min(zoom+0.0005,{:.3})", 1.0 + duration * 0.03);
        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&[
                "-y",
                "-loop", "1",
                "-i", image_path,
                "-vf", &format!(
                    "zoompan=z='{}':d={}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':s={}x{}:fps={}",
                    zoom_expr, total_frames, width, height, FPS
                ),
                "-c:v", &self.codec,
                "-pix_fmt", "yuv420p",
                "-t", &duration.to_string(),
                output_path,
            ]))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg image_to_video failed: {}", stderr)));
        }
        Ok(())
    }

    /// 为视频叠加水印图片（右下角半透明）
    ///
    /// - `input_path`: 输入视频路径
    /// - `watermark_path`: 水印图片路径
    /// - `output_path`: 输出视频路径
    pub fn add_watermark(&self, input_path: &str, watermark_path: &str, output_path: &str) -> Result<(), SomaError> {
        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&[
                "-y",
                "-i", input_path,
                "-i", watermark_path,
                "-filter_complex", "[1:v]format=rgba,colorchannelmixer=aa=0.5[wm];[0:v][wm]overlay=W-w-10:H-h-10",
                "-c:v", &self.codec,
                "-pix_fmt", "yuv420p",
                "-c:a", "copy",
                output_path,
            ]))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg add_watermark failed: {}", stderr)));
        }
        Ok(())
    }

    /// 拼接片头/片尾视频（将多段视频顺序拼接为一段）
    ///
    /// - `segments`: 视频文件路径列表（按顺序拼接）
    /// - `output_path`: 输出视频路径
    pub fn concat_videos(&self, segments: &[&str], output_path: &str) -> Result<(), SomaError> {
        if segments.is_empty() {
            return Err(SomaError::Ffmpeg("concat_videos: 空片段列表".into()));
        }
        if segments.len() == 1 {
            std::fs::copy(segments[0], output_path)
                .map_err(|e| SomaError::Ffmpeg(format!("复制视频失败: {}", e)))?;
            return Ok(());
        }
        let tmp_dir = std::env::temp_dir().join("soma_concat");
        std::fs::create_dir_all(&tmp_dir).ok();
        let list_path = tmp_dir.join("concat_list.txt");
        let mut list_content = String::new();
        for seg in segments {
            let escaped = seg.replace("'", "'\\''");
            list_content.push_str(&format!("file '{}'\n", escaped));
        }
        std::fs::write(&list_path, &list_content)
            .map_err(|e| SomaError::Ffmpeg(format!("写入 concat 列表失败: {}", e)))?;

        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&[
                "-y",
                "-f", "concat", "-safe", "0",
                "-i", list_path.to_str().unwrap_or(""),
                "-c", "copy",
                output_path,
            ]))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg concat failed: {}", stderr)));
        }
        Ok(())
    }

    /// 拼接多个音频文件为一个
    ///
    /// - `audio_files`: 音频文件路径列表（按顺序拼接）
    /// - `output_file`: 输出音频路径
    pub fn concat_audios(&self, audio_files: &[String], output_file: &str) -> Result<(), SomaError> {
        if audio_files.is_empty() {
            return Err(SomaError::VideoGen("音频拼接输入为空".into()));
        }
        if audio_files.len() == 1 {
            std::fs::copy(&audio_files[0], output_file)
                .map_err(|e| SomaError::Ffmpeg(format!("复制音频失败: {}", e)))?;
            return Ok(());
        }

        let output_dir = Path::new(output_file).parent().unwrap_or(Path::new("."));
        let concat_list = output_dir.join("ffmpeg-audio-concat-list.txt");

        let mut content = String::new();
        for audio in audio_files {
            let abs = std::fs::canonicalize(audio).unwrap_or_else(|_| PathBuf::from(audio));
            let escaped = abs.to_string_lossy().replace('\\', "/").replace("'", "'\\''");
            content.push_str(&format!("file '{}'\n", escaped));
        }
        std::fs::write(&concat_list, &content).map_err(SomaError::Io)?;

        let result = run_with_timeout(std::process::Command::new(&self.path)
            .args(&[
                "-y",
                "-f", "concat",
                "-safe", "0",
                "-i", concat_list.to_string_lossy().as_ref(),
                "-c", "copy",
                output_file,
            ]))?;

        let _ = std::fs::remove_file(&concat_list);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Ffmpeg(format!("ffmpeg concat_audios failed: {}", stderr)));
        }
        Ok(())
    }
}

use std::path::PathBuf;
