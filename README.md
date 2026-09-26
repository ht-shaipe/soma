# Soma

AI 驱动的短视频自动生成工具 —— 输入主题，自动完成脚本撰写、配音、素材采集、字幕合成与视频拼接，一键生成成品视频；同时支持**数字人口播视频**：一张人像照片 + 一段文案，生成开口说话、口型同步的口播视频。

## 功能特性

### 短视频自动生成

- **AI 脚本生成**：支持 DeepSeek、OpenAI、Qwen、Gemini、Moonshot、Grok、Ollama 等 25+ 种 LLM 提供商
- **需求理解与分镜**：LLM 先解析风格/情绪/受众（意图理解），再产出带视觉提示词、镜头运动、转场建议的分镜脚本
- **智能语音合成 (TTS)**：Edge TTS、Azure、SiliconFlow、ElevenLabs、Gemini、MiMo、火山引擎、科大讯飞、Fish-Speech S2（含旁白情感标签）等 11 种引擎，支持中/英/多语种
- **声音克隆**：对接 SSH 远程 GPU 推理服务，支持 GPT-SoVITS / CosyVoice / Fish-Speech 克隆自定义音色
- **素材自动采集**：对接 Pexels、Pixabay、Coverr 素材库，按关键词自动搜索下载视频片段，支持多 Key 轮换突破速率限制
- **AI 视频生成**：支持智谱 CogVideoX（免费 flash 模型）、快手可灵 Kling、MiniMax 海螺 Hailuo 等视频生成大模型，直接生成视频片段
- **视频合成**：基于 FFmpeg 实现视频裁剪、拼接、转场特效、字幕叠加、音轨混音（配音 + 背景音乐）、水印
- **灵活配置**：多种画幅比例（16:9 / 9:16 / 1:1）、拼接模式、转场模式、字幕样式自定义
- **任务管理**：异步任务队列（并发/排队上限可配）、进度跟踪、状态查询、SQLite 持久化（服务重启不丢任务）、支持任意步骤断点续跑
- **社交发布**：通过 Upload-Post 一键发布至 TikTok、YouTube 等 12 个平台

### 数字人口播

输入一张人像照片（≤10MB）+ 一段文案（≤1000 字），生成口型与配音同步的口播视频，支持 5 种引擎：

| 引擎 | provider 值 | 部署形态 | 成本 |
|------|------------|---------|------|
| **HeyGen** | `heygen` | 云端 API | 付费（按量） |
| **SadTalker** | `sadtalker` | 本地推理（CPU/GPU） | 免费 |
| **EchoMimicV3-Flash** | `echomimic_v3` | 本地 GPU（≥12G 显存） | 免费 |
| **HeyGem** | `heygem` | Docker 三容器（fun-asr + fish-speech + duix.avatar） | 免费 |
| **Live2D 卡通数字人** | `live2d` | 纯 CPU 渲染 | 零成本 |

- 长文案自动**分段生成**（单段上限 150 帧 / 6 秒），逐段 TTS + 口播渲染后 FFmpeg 拼接，自动音画对齐
- 内置敏感词过滤（`resource/sensitive_words.txt`），可自定义词库
- Live2D 模式纯 CPU 运行，无需 GPU，支持自定义模型上传

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust + Actix-Web 4 |
| 前端 | Vue 3 + TypeScript + Vite + Element Plus + Pinia + Vue I18n（中英双语） |
| 视频处理 | FFmpeg（拼接/转场/字幕/混音/水印） |
| 语音合成 | Edge TTS / Azure / SiliconFlow / ElevenLabs / Gemini / MiMo / 火山引擎 / 讯飞 / Fish-Speech S2 / HeyGem / 声音克隆 |
| AI 模型 | DeepSeek / OpenAI / Qwen / Gemini 等 25+ 提供商 |
| 视频生成 | 智谱 CogVideoX / 快手可灵 Kling / MiniMax 海螺 |
| 数字人 | HeyGen / SadTalker / EchoMimicV3-Flash / HeyGem / Live2D |
| 任务持久化 | SQLite（WAL 模式） |

## 项目结构

```
soma/
├── conf/                   # 配置文件目录
│   ├── config.toml         # 主配置文件（含密钥，已 gitignore）
│   ├── config.toml.example # 配置模板（含完整注释）
│   └── CONFIG_GUIDE.md     # 完整配置指南（含 API Key 申请地址）
├── crates/                 # Rust 工作空间
│   ├── soma-core/          # 核心模块（配置、模型、错误、敏感词过滤）
│   ├── soma-tts/           # 语音合成（10 种引擎 + 声音克隆 + Whisper 字幕）
│   ├── soma-stock/         # 素材采集（Pexels/Pixabay/Coverr）+ AI 视频生成（智谱/可灵/MiniMax）+ 数字人 Provider（5 种）
│   ├── soma-video/         # 视频处理（FFmpeg 拼接/转场/字幕/混音/水印）
│   └── soma-server/        # Web 服务（API 路由/任务队列/流水线/SQLite 持久化）
├── docs/                   # 技术文档（Obsidian 知识库：方案调研、成本评估、实践文章）
├── resource/               # 静态资源与 Python 推理封装脚本
│   ├── *_runner.py         # 数字人/声音克隆推理脚本（Rust 子进程/SSH 调用）
│   ├── sensitive_words.txt # 数字人文案敏感词库
│   ├── fonts/              # 字体文件
│   └── songs/              # 背景音乐
├── storage/                # 运行时存储（自动创建）
│   ├── tasks.db            # SQLite 任务持久化
│   ├── tasks/{task_id}/    # 每任务产物（音频/字幕/分段视频/成品）
│   ├── portraits/          # 数字人人像照片
│   └── songs/ fonts/ cache_videos/
├── web/                    # Vue 3 前端项目
├── Makefile                # 常用命令（dev/build/prod/test 等）
└── Cargo.toml              # Rust 工作空间配置
```

## 快速上手教程

### 第 1 步：环境准备

| 依赖 | 最低版本 | 安装方式 | 检查命令 |
|------|---------|---------|---------|
| **Rust** | Edition 2021 | [rustup.rs](https://rustup.rs) | `rustc --version` |
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org) | `node --version` |
| **FFmpeg** | 4.0+ | `brew install ffmpeg` (macOS) / `apt install ffmpeg` (Ubuntu) | `ffmpeg -version` |

> 数字人本地引擎额外依赖 Python 3.8+（SadTalker/EchoMimicV3/Live2D/声音克隆均通过 Python 推理脚本驱动）。

### 第 2 步：获取项目

```bash
git clone <repo-url> && cd soma
cp conf/config.toml.example conf/config.toml
```

### 第 3 步：申请 API Key

Soma 核心功能需要两类 API Key：

#### 3.1 LLM API Key（脚本生成，必配一个）

| 推荐方案 | llm_provider | 是否免费 | 申请地址 |
|---------|--------------|---------|---------|
| **Ollama**（本地） | `ollama` | 完全免费 | 安装 Ollama 后 `ollama pull llama3` |
| **Pollinations** | `pollinations` | 免费 | 无需申请，直接使用 |
| **DeepSeek** | `deepseek` | 注册送额度 | https://platform.deepseek.com/signup |
| **Groq** | `groq` | 免费额度 | https://console.groq.com/keys |
| **Qwen/通义千问** | `qwen` | 有免费额度 | https://dashscope.console.aliyun.com/ |

> 其他 20+ LLM 供应商详见 [conf/CONFIG_GUIDE.md](conf/CONFIG_GUIDE.md)

#### 3.2 素材库 API Key（视频搜索，至少配一个）

| 推荐方案 | 是否免费 | 申请地址 |
|---------|---------|---------|
| **Pexels** | 免费，200次/时 | https://www.pexels.com/api/ |
| **Pixabay** | 免费，100次/分 | https://pixabay.com/api/docs/ |

> 也可以使用本地素材（`video_source = "local"`），无需 Key。在线素材库均支持多 Key 轮换。

#### 3.3 视频生成大模型 API Key（AI 原生视频，可选）

> 视频生成大模型可根据文字/图片直接生成视频片段，区别于「素材搜索拼接」模式。不配置时仍可通过素材库合成视频。

| 推荐方案 | 是否免费 | 申请地址 | 配置项 |
|---------|---------|---------|--------|
| **智谱 CogVideoX** | flash 模型免费 | https://open.bigmodel.cn | `zhipu_video_api_key` |
| **快手 可灵 Kling** | 付费 | https://platform.kuaishou.com | `kling_access_key` + `kling_secret_key` |
| **MiniMax 海螺 Hailuo** | 注册送额度 | https://platform.minimaxi.com | `minimax_video_api_key` |

> 详见 [conf/CONFIG_GUIDE.md 第五章](conf/CONFIG_GUIDE.md)

### 第 4 步：编辑配置

```bash
vim conf/config.toml
```

> 注意：LLM / 素材库 / 视频生成等配置统一放在 `[app]` 段，字段为 `{provider}_api_key` 扁平命名，由 `llm_provider` 指定当前启用的 LLM。

#### 方案 A：零成本全本地

```toml
[app]
llm_provider = "ollama"
ollama_model_name = "llama3"
video_source = "local"
```

> 需先安装 Ollama：https://ollama.com ，然后 `ollama pull llama3`

#### 方案 B：Cloud 免费方案

```toml
[app]
llm_provider = "pollinations"
pexels_api_keys = ["你从 Pexels 申请的 Key"]
```

#### 方案 C：推荐方案（性价比最优）

```toml
[app]
llm_provider = "deepseek"
deepseek_api_key = "sk-xxx"                 # 从 https://platform.deepseek.com 获取
deepseek_model_name = "deepseek-chat"
pexels_api_keys = ["你从 Pexels 申请的 Key"]
# TTS 默认使用 Edge TTS（免费，无需 Key）
```

#### 方案 D：AI 视频生成模式（用大模型直接生成视频片段）

```toml
[app]
llm_provider = "deepseek"
deepseek_api_key = "sk-xxx"
deepseek_model_name = "deepseek-chat"
pexels_api_keys = ["你从 Pexels 申请的 Key"]

# 视频生成大模型（可选，替代或补充素材拼接）
zhipu_video_api_key = "你的智谱 Key"   # https://open.bigmodel.cn 免费申请
zhipu_video_model = "cogvideox-flash"  # 免费；付费可选 cogvideox-3 / viduq1-*
video_gen_timeout = 300                 # 生成超时（秒）
```

#### 方案 E：数字人口播模式

```toml
[digital_human]
provider = "live2d"        # 零成本卡通数字人；或 heygen / sadtalker / echomimic_v3 / heygem

[digital_human.live2d]
models_dir = "./storage/live2d_models"
script_path = "resource/live2d_runner.py"
# 其余保持默认即可
```

> 各数字人引擎的部署要求与完整配置见 [conf/CONFIG_GUIDE.md 数字人章节](conf/CONFIG_GUIDE.md)。EchoMimicV3-Flash 需要 ≥12G 显存的 GPU；Live2D 纯 CPU 即可运行。

### 第 5 步：启动服务

推荐使用 Makefile：

```bash
make install   # 安装前后端依赖
make dev       # 并行启动后端 (8090) 与前端开发服务器 (5273)
```

也可以手动启动：

```bash
# 后端（首次编译约 3-5 分钟）
cargo run --release

# 前端（新开终端）
cd web && npm install && npm run dev
```

看到以下输出即表示后端启动成功：

```
[INFO] Soma v0.1.0 starting on 0.0.0.0:8090
[INFO] Storage path: ./storage
```

后端 API 运行在 `http://localhost:8090`，前端运行在 `http://localhost:5273`（自动代理 `/api` 到后端）。

**常用 make 命令**：`make dev`（开发）、`make build`（前后端 release 构建）、`make prod`（构建前端后启动后端，单进程托管）、`make test`、`make fmt`、`make lint`、`make clean`。

### 第 6 步：开始使用

1. 浏览器打开 `http://localhost:5273`
2. 按引导向导确认 API Key 已生效（也可在 **设置** 页面修改配置）
3. 生成短视频：进入创作向导，填写视频主题（如"介绍人工智能的发展历程"），选择语言、段落数、画幅比例等，点击生成
4. 生成数字人口播：上传一张人像照片（Live2D 模式选择卡通模型），输入口播文案（≤1000 字），选择语音，点击生成
5. 任务页面实时查看进度，完成后可预览/下载成品视频

### 生成视频的完整流水线

短视频采用 6 步流水线，支持 `stop_at` 在任意步骤暂停（用于调试/预览），已完成步骤自动跳过（断点续跑）：

```
输入主题："人工智能发展史"
    │
    ▼
┌──────────────────┐
│ 1. 需求理解       │  LLM 解析风格/情绪/受众 → 意图参数
└────────┬─────────┘
         ▼
┌──────────────────┐
│ 2. 文案创作       │  AI 根据主题 + 意图撰写旁白脚本
└────────┬─────────┘
         ▼
┌──────────────────┐
│ 3. 分镜脚本       │  生成场景/视觉提示词/搜索关键词/镜头/转场
└────────┬─────────┘
         ▼
┌──────────────────────────────┐
│ 4. 素材生成（按 video_source） │
│  ├─ 素材库搜索  Pexels/Pixabay/Coverr
│  └─ AI 视频生成  智谱/可灵/MiniMax
└────────┬─────────────────────┘
         ▼
┌──────────────────┐
│ 5. 配音 + 字幕    │  TTS 语音合成 → SRT 字幕（Edge 对齐 或 Whisper）
└────────┬─────────┘
         ▼
┌──────────────────┐
│ 6. 视频合成       │  素材 + 配音 + 字幕 + BGM + 转场 → 成品视频
└────────┬─────────┘
         ▼
    🎬 成品视频
```

数字人口播为独立三阶段流水线（文案不经过 LLM 改写）：

```
人像照片 + 口播文案
    │
    ▼
配音音频 (30%) ──▶ 口播视频 (90%) ──▶ 字幕/BGM 合成 (100%)
   TTS            数字人引擎渲染        FFmpeg
                    │
                    ├─ 长文案自动分段（单段 ≤150 帧/6 秒），逐段渲染后拼接
                    └─ 进度按阶段百分比上报，前端轮询展示
```

### 可选：TTS 语音配置

默认使用 **Edge TTS**（免费，无需 Key）。如需更高质量或特定音色：

| TTS 引擎 | 所需 Key | 申请地址 | 免费额度 |
|----------|----------|----------|---------|
| Azure Speech | `[azure]` 段 `speech_key` + `speech_region` | https://portal.azure.com | 5小时/月 |
| SiliconFlow | `[siliconflow]` 段 `api_key` | https://siliconflow.cn | 注册送额度 |
| ElevenLabs | `[elevenlabs]` 段 `api_key` | https://elevenlabs.io | 1万字符/月 |
| 火山引擎 | `[volcengine]` 段 `app_id` + `access_token` | https://console.volcengine.com/speech | 有免费额度 |
| 科大讯飞 | `[xfyun]` 段 `app_id` + `api_key` + `api_secret` | https://www.xfyun.cn | 注册送额度 |
| Fish-Speech S2 | `[fishspeech]` 段 `base_url`（自部署）或 `api_key`（云端） | https://fish.audio | 自部署免费 / 云端按量 |

> 支持在 `voice_name` 中用引擎前缀路由（如 `siliconflow:xxx`、`azure:xxx`、`volcengine:xxx`），无前缀默认 Edge TTS。声音克隆音色通过 `/api/v1/voices/clone` 上传参考音频训练。
>
> **Fish-Speech S2** 兼容自部署（`python tools/api_server.py`）与 Fish Audio 云端；在 `[app]` 段开启 `narration_emotion_tags = true` 后，LLM 会在旁白中插入 `[whisper]` `[excited]` 等情感标签，由 Fish-Speech 渲染为真实情绪（其他引擎自动剥离）。详见 CONFIG_GUIDE.md。

### 可选：生产部署

```bash
# 构建前端生产版本
cd web && npm run build

# 后端会自动托管 web/dist 下的前端静态文件
# 只需启动后端即可：
cargo run --release

# 访问 http://your-server:8090
```

### 可选：Docker 部署

```dockerfile
# Dockerfile 示例
FROM rust:1.77 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ffmpeg ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/soma-server /usr/local/bin/
COPY --from=builder /app/conf /app/conf
COPY --from=builder /app/resource /app/resource
COPY --from=builder /app/web/dist /app/web/dist
WORKDIR /app
EXPOSE 8090
CMD ["soma-server"]
```

> HeyGem 数字人引擎官方推荐 Docker 三容器部署（fun-asr + fish-speech-ziming + duix.avatar），详见 [conf/CONFIG_GUIDE.md](conf/CONFIG_GUIDE.md)。

## API 接口

除少数显式端点外，所有接口通过 `POST /api/v1/{module}` 统一入口访问，请求体中携带 `module` 与 `method` 字段进行二级分发。

### 显式端点

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/materials/upload` | 上传视频素材（multipart） |
| POST | `/api/v1/musics/upload` | 上传背景音乐 |
| POST | `/api/v1/audio/upload` | 上传音频 |
| POST | `/api/v1/portraits/upload` | 上传数字人人像照片（≤10MB） |
| POST | `/api/v1/voices/clone` | 声音克隆（上传参考音频） |
| GET | `/api/v1/voices/preview` | 预览语音 |
| GET | `/api/v1/subtitles/preview` | 预览字幕 |
| GET | `/storage/*` | 静态文件服务 |

### module.method 分发端点

| module | 可用 method | 说明 |
|--------|------------|------|
| `videos` | create / draft / updateconfig / start / fetchmaterials / generatenarration / generateaudio | 视频任务创建与启动 |
| `tasks` | list / get / delete / stop | 任务管理 |
| `digital_human` | create / draft / start | 数字人任务 |
| `dh_tasks` | list / get / delete | 数字人任务管理 |
| `scripts` | generate | LLM 脚本生成 |
| `terms` | generate / extract | 关键词提取 |
| `storyboard` | generate | 分镜脚本生成 |
| `intent` | generate | 需求理解 |
| `social` | generate | 社交发布元数据生成 |
| `musics` | list | 背景音乐列表 |
| `materials` | list / fonts | 素材列表 / 字体列表 |
| `portraits` | list / delete | 人像照片管理 |
| `voices` | list / list_cloned / delete_cloned | 语音列表 / 克隆音色管理 |
| `stream` | play / download | 视频流播放 / 下载 |
| `config` | get / save | 配置读写 |
| `upload` | upload / status | Upload-Post 跨平台发布 |

## 常见问题

### Q: 启动报错 "FFmpeg not found"
确保 FFmpeg 已安装并在 PATH 中：`ffmpeg -version`

### Q: 生成任务失败 "LLM chat failed"
检查 `conf/config.toml` 中 `[app]` 段对应 provider 的 `*_api_key` 是否正确，`llm_provider` 是否匹配

### Q: 视频无素材画面（黑屏）
检查素材库 API Key 是否配置，或使用本地素材模式（`video_source = "local"`，素材通过页面上传）

### Q: 语音合成失败
默认 Edge TTS 需要网络连接。如网络受限，可切换到本地 Ollama + 本地 TTS

### Q: 如何添加背景音乐？
将 MP3 文件放入 `storage/songs/` 目录（或通过页面/`/api/v1/musics/upload` 上传），生成视频时在界面中选择

### Q: 如何添加自定义字体？
将字体文件放入 `storage/fonts/` 目录，系统会自动识别（同时也会扫描系统字体）

### Q: 数字人生成失败？
- 检查 `[digital_human]` 段 `provider` 与对应子段配置是否完整
- 本地引擎（SadTalker/EchoMimicV3/Live2D）依赖 Python 环境，确认 `python_path` / `env_path` / `model_path` 正确；`preflight_check = true` 可在提交前自动预检
- EchoMimicV3-Flash 需要 ≥12G 显存，确认 GPU 可用（`nvidia-smi`）
- 文案超过 1000 字会被截断，请控制长度

### Q: 声音克隆如何使用？
1. 配置 `[digital_human.voice_clone]` 段（SSH 远程 GPU 推理服务）
2. 前端「声音克隆」面板上传参考音频，等待训练完成
3. 生成任务时选择克隆音色

## 配置完整参考

详见 [conf/CONFIG_GUIDE.md](conf/CONFIG_GUIDE.md) — 包含所有 25+ LLM 供应商、10 种 TTS 引擎、3 个素材库、3 个视频生成大模型、5 种数字人引擎、声音克隆的配置方法与 API Key 申请地址。

## 技术文档

[docs/](docs/) 目录为 Obsidian 知识库，包含数字人方案调研、成本评估与系列实践文章：

- [docs/README.md](docs/README.md) — 文档索引（调研 → 落地链路）
- 数字人开源自部署方案评估（选型：EchoMimicV3-Flash）
- EchoMimicV3-Flash 成本评估（GPU 服务器测算）
- 中国大陆 AI 视频生成 API 调研
- articles/ — 5 篇实践文章（GPU 部署踩坑、Rust 异步调 Python、声音克隆集成、长视频分段与音画同步等）

## License

MIT
