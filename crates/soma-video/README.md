# soma-video

**视频合成与处理层**：FFmpeg 命令封装与成片合成，全部能力均为同步阻塞调用（进程超时保护）。

## 两大核心

### `ffmpeg.rs` —— Ffmpeg 原语

| 原语 | 用途 |
|------|------|
| `generate_video` | 渲染成片：视频 + 配音 + 字幕 + BGM（字幕样式/描边/背景/位置全部可配） |
| `combine_videos`（在 compose.rs） | 多段素材按转场模式拼接并与音频时长对齐 |
| `concat_videos` / `concat_audios` | 多段无损拼接（视频/音频） |
| `clip_and_resize` | 区间截取 + 按目标画幅智能裁切（模糊背景填充） |
| `add_transition` | 淡入/淡出转场 |
| `add_watermark` | 右下角半透明水印 |
| `preprocess_local_materials` | 本地素材预处理（图片转视频、路径规整） |
| `image_to_video` | 静态图 → 视频 |
| `get_video_duration` / `get_video_resolution` / `get_audio_duration` | 媒体信息探测（ffprobe） |

### `compose.rs` —— VideoComposer

成片编排器：`combine_videos`（拼接对齐）→ `get_bgm_file`（BGM 解析：指定文件 / 歌曲目录随机）→
`generate_video`（渲染成品）。`video.compose` 与 `video.render` 功能点的底层实现。

## 编码器

启动时探测可用硬编编码器（`SUPPORTED_CODECS`：libx264 / h264_videotoolbox / h264_nvenc 等），
不支持的编码名自动回退默认。所有 FFmpeg 进程带超时保护（`run_with_timeout`）。

## 测试

```bash
cargo test -p soma-video
```
