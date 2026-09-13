# soma-stock

**素材采集层**：多源视频素材的搜索、下载、校验与 AI 视频生成，数字人引擎的健康探测也在此层。

## 素材源

| 模块 | 来源 | 说明 |
|------|------|------|
| `pexels` / `pixabay` / `coverr` | 免费实拍素材站 | 关键词搜索 → 下载 → 本地校验 |
| `aivideo/` | 智谱 CogVideoX、可灵 Kling、MiniMax 海螺 | 文生视频 / 图生视频，串行提交 + 并发轮询 + 部分失败容忍 |

## 关键入口

- `search_videos(source, keyword, aspect, min_duration, keys…) -> Vec<MaterialInfo>`：只搜不下载
- `download_videos(save_dir, terms, …) -> Vec<String>`：批量搜索下载，按关键词顺序轮询候选，
  直到总时长覆盖音频需求；下载后校验文件有效性，无效自动清理
- `save_video(url, save_dir) -> String`：单 URL 下载（供 `material.download` 功能点复用）
- `generate_ai_videos(save_dir, terms, source, aspect, clip_duration, conf, portrait) -> (Vec<String>, Vec<AiVideoSegmentLog>)`：
  AI 视频批量生成，返回逐段日志（`AiVideoSegmentLog`）供任务详情展示

## 数字人引擎探测（`digital_human/`）

Live2D / HeyGem / SadTalker / EchoMimicV3 / HeyGen 各引擎的就绪检查
（Python 环境、模型文件、FFmpeg、远程服务连通性），供 `preflight_check` 与任务前置校验使用。

## 调用方式

入口均为 async；素材下载自动读取系统代理（HTTP_PROXY/HTTPS_PROXY）。

## 测试

```bash
cargo test -p soma-stock
```
