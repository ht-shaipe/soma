# Soma 技术文档索引

> 本目录是 Obsidian 知识库，记录项目的技术调研、方案评估与落地实践。
> 文档遵循「调研 → 选型 → 落地 → 复盘」的演进链路，与代码模块一一对应。

## 文档地图

### 版本规划

| 文档 | 日期 | 主题 |
|------|------|------|
| [0.1.2升级计划_功能点独立化与Tauri改造_2026-09-12](0.1.2升级计划_功能点独立化与Tauri改造_2026-09-12.md) | 2026-09-12 | 0.1.2 版本完整计划：流水线步骤拆分为独立功能点 + Tauri v2 桌面应用改造（分支 `0.1.2`） |

### 方案调研与评估

| 文档 | 日期 | 主题 | 对应代码 |
|------|------|------|----------|
| [数字人开源自部署方案评估_2026-09-02](数字人开源自部署方案评估_2026-09-02.md) | 2026-09-02 | 「照片 + 文案 → 口播视频」开源方案选型，结论：主力选 EchoMimicV3-Flash（Apache 2.0，单张 4090 可跑） | `soma-stock/src/digital_human/` |
| [EchoMimicV3-Flash成本评估_2026-09-03](EchoMimicV3-Flash成本评估_2026-09-03.md) | 2026-09-03 | GPU 服务器与推理成本测算 | — |
| [china-ai-video-api-research](china-ai-video-api-research.md) | 2026-07 | 中国大陆 AI 视频生成 API 调研（智谱/可灵/通义万相/混元/百度 Vidu/MiniMax） | `soma-stock/src/aivideo/` |

### 实践系列文章（articles/）

| 文章 | 主题 | 对应代码 |
|------|------|----------|
| [01_从云端API到开源自部署的选型之路](articles/01_从云端API到开源自部署的选型之路.md) | 数字人口播方案选型全过程（云端 API 贵 → 开源自部署） | — |
| [02_EchoMimicV3_GPU服务器部署的6个坑](articles/02_EchoMimicV3_GPU服务器部署的6个坑.md) | AutoDL GPU 服务器环境坑（ModelScope/HuggingFace 下载等） | `resource/echomimic_v3_*.py` |
| [03_Rust异步调用Python_GPU推理](articles/03_Rust异步调用Python_GPU推理.md) | `block_on_async` 模式详解（tokio::spawn panic 解法） | `soma-stock/src/digital_human/echomimic_v3.rs` |
| [04_声音克隆TTS集成踩坑](articles/04_声音克隆TTS集成踩坑.md) | CosyVoice 声音克隆的多格式音频处理与路径解析 | `soma-tts/src/voice_clone_tts.rs` |
| [05_长视频分段生成与音画同步](articles/05_长视频分段生成与音画同步.md) | 150 帧（6 秒）限制下的分段生成、拼接与 0.7 秒音画偏差对齐 | `soma-server/src/service/segment_dh_video.rs` |

## 演进时间线

```
2026-06  立项调研（模型与API对接需求分析.md）→ 基础短视频流水线
2026-07  接入 AI 原生视频生成（智谱/可灵/MiniMax）→ 流程重构为 6 步流水线 → 密钥治理
2026-09  数字人口播：方案评估 → EchoMimicV3-Flash 落地 → HeyGem/声音克隆 → Live2D 卡通数字人（纯 CPU）
```

## 相关入口

- 项目主文档：[../README.md](../README.md)
- 配置指南：[../conf/CONFIG_GUIDE.md](../conf/CONFIG_GUIDE.md)
- 立项调研：[../模型与API对接需求分析.md](../模型与API对接需求分析.md)
