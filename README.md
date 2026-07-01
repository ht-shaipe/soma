# Soma

AI 驱动的短视频自动生成工具 —— 输入主题，自动完成脚本撰写、配音、素材采集、字幕合成与视频拼接，一键生成成品视频。

## 功能特性

- **AI 脚本生成**：支持 DeepSeek、OpenAI、Qwen、Gemini、Moonshot、Grok、Ollama 等 25+ 种 LLM 提供商
- **智能语音合成 (TTS)**：Edge TTS、SiliconFlow、MiMo、ElevenLabs、Azure 多引擎配音，支持中/英/多语种语音
- **素材自动采集**：对接 Pexels、Pixabay、Coverr 素材库，按关键词自动搜索并下载视频片段
- **视频合成**：基于 FFmpeg 实现视频裁剪、拼接、字幕叠加、音轨混音（配音 + 背景音乐）
- **灵活配置**：支持多种画幅比例（16:9 / 9:16 / 1:1）、拼接模式、字幕样式自定义
- **任务管理**：异步任务队列，支持并发任务处理、进度跟踪与状态查询
- **Web 界面**：Vue 3 + Element Plus 可视化操作界面，支持中英文双语

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust + Actix-Web 4 |
| 前端 | Vue 3 + TypeScript + Vite + Element Plus |
| 视频处理 | FFmpeg |
| 语音合成 | Edge TTS / SiliconFlow / ElevenLabs / Azure / MiMo |
| AI 模型 | DeepSeek / OpenAI / Qwen / Gemini 等 25+ 提供商 |

## 项目结构

```
soma/
├── conf/                   # 配置文件目录
│   ├── config.toml         # 主配置文件
│   └── CONFIG_GUIDE.md     # 完整配置指南（含 API Key 申请地址）
├── crates/                 # Rust 工作空间
│   ├── soma-core/          # 核心模块（配置、模型、工具、错误定义）
│   ├── soma-tts/           # 语音合成（Edge TTS / SiliconFlow / MiMo / ElevenLabs / Azure）
│   ├── soma-stock/         # 素材采集（Pexels / Pixabay / Coverr）
│   ├── soma-video/         # 视频处理（FFmpeg 拼接 / 字幕 / 混音 / 转场）
│   └── soma-server/        # Web 服务（API 路由 / Handler / 任务流水线）
├── resource/               # 静态资源
│   ├── fonts/              # 字体文件
│   └── songs/              # 背景音乐
├── web/                    # Vue 3 前端项目
└── Cargo.toml              # Rust 工作空间配置
```

## 快速上手教程

### 第 1 步：环境准备

| 依赖 | 最低版本 | 安装方式 | 检查命令 |
|------|---------|---------|---------|
| **Rust** | Edition 2021 | [rustup.rs](https://rustup.rs) | `rustc --version` |
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org) | `node --version` |
| **FFmpeg** | 4.0+ | `brew install ffmpeg` (macOS) / `apt install ffmpeg` (Ubuntu) | `ffmpeg -version` |

### 第 2 步：获取项目

```bash
git clone <repo-url> && cd soma
```

### 第 3 步：申请 API Key

Soma 核心功能需要两类 API Key：

#### 3.1 LLM API Key（脚本生成，必配一个）

| 推荐方案 | provider | 是否免费 | 申请地址 |
|---------|----------|---------|---------|
| **Ollama**（本地） | `ollama` | ✅ 完全免费 | 安装 Ollama 后 `ollama pull llama3` |
| **Pollinations** | `pollinations` | ✅ 免费 | 无需申请，直接使用 |
| **DeepSeek** | `deepseek` | 注册送额度 | https://platform.deepseek.com/signup |
| **Groq** | `groq` | 免费额度 | https://console.groq.com/keys |
| **Qwen/通义千问** | `qwen` | 有免费额度 | https://dashscope.console.aliyun.com/ |

> 其他 20+ LLM 供应商详见 [conf/CONFIG_GUIDE.md](conf/CONFIG_GUIDE.md)

#### 3.2 素材库 API Key（视频搜索，至少配一个）

| 推荐方案 | 是否免费 | 申请地址 |
|---------|---------|---------|
| **Pexels** | ✅ 免费，200次/时 | https://www.pexels.com/api/ |
| **Pixabay** | ✅ 免费，100次/分 | https://pixabay.com/api/docs/ |

> 也可以使用本地素材（`video_source = "local"`），无需 Key

### 第 4 步：编辑配置

```bash
vim conf/config.toml
```

#### 方案 A：零成本全本地

```toml
[llm]
provider = "ollama"
model = "llama3"

[app]
video_source = "local"
```

> 需先安装 Ollama：https://ollama.com ，然后 `ollama pull llama3`

#### 方案 B：Cloud 免费方案

```toml
[llm]
provider = "pollinations"
model = "openai"

[stock]
pexels_api_key = "你从 Pexels 申请的 Key"
```

#### 方案 C：推荐方案（性价比最优）

```toml
[llm]
provider = "deepseek"
api_key = "sk-xxx"                 # 从 https://platform.deepseek.com 获取
model = "deepseek-chat"
base_url = "https://api.deepseek.com"

[tts]
provider = "edge"                  # 免费，无需 Key
voice_name = "zh-CN-XiaoxiaoNeural"

[stock]
pexels_api_key = "你从 Pexels 申请的 Key"
```

### 第 5 步：启动后端

```bash
# 编译并运行（首次编译约 3-5 分钟）
cargo run --release
```

看到以下输出即表示启动成功：

```
[INFO] Soma v0.1.0 starting on 0.0.0.0:8080
[INFO] Storage path: ./storage
```

后端 API 运行在 `http://localhost:8080`

### 第 6 步：启动前端

新开一个终端：

```bash
cd web

# 安装依赖
npm install

# 开发模式启动（自动代理 /api 到后端 8080）
npm run dev
```

前端运行在 `http://localhost:5173`，自动代理 API 请求到后端。

### 第 7 步：开始使用

1. 浏览器打开 `http://localhost:5173`
2. 进入 **设置** 页面，确认 API Key 已生效（也可在此修改配置）
3. 进入 **任务** 页面，点击 **新建任务**
4. 填写视频主题（如："介绍人工智能的发展历程"），选择语言、段落数、画幅比例等
5. 点击 **生成**，等待流水线执行
6. 生成完成后可在任务列表中预览/下载成品视频

### 第 8 步：生成视频的完整流水线

```
输入主题："人工智能发展史"
    │
    ▼
┌──────────────┐
│ 1. LLM 脚本  │  AI 根据主题撰写视频旁白脚本
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ 2. 关键词提取 │  AI 从脚本提取素材搜索关键词
└──────┬───────┘
       │
       ├─────────────────────┐
       ▼                     ▼
┌──────────────┐   ┌───────────────┐
│ 3. TTS 配音   │   │ 4. 素材搜索下载 │
└──────┬───────┘   └───────┬───────┘
       │                   │
       └─────────┬─────────┘
                 ▼
       ┌──────────────┐
       │ 5. 字幕生成   │  基于配音音频生成 SRT 字幕
       └──────┬───────┘
              ▼
       ┌──────────────┐
       │ 6. 视频合成   │  素材+配音+字幕+背景音乐 → 成品视频
       └──────┬───────┘
              ▼
         🎬 成品视频
```

### 可选：TTS 语音配置

默认使用 **Edge TTS**（免费，无需 Key）。如需更高质量的语音：

| TTS 引擎 | 所需 Key | 申请地址 | 免费额度 |
|----------|----------|----------|---------|
| Azure Speech | `azure_speech_key` + `azure_speech_region` | https://portal.azure.com | 5小时/月 |
| SiliconFlow | `siliconflow_key` | https://siliconflow.cn | 注册送额度 |
| ElevenLabs | `elevenlabs_key` | https://elevenlabs.io | 1万字符/月 |

### 可选：生产部署

```bash
# 构建前端生产版本
cd web && npm run build

# 后端会自动托管 web/dist 下的前端静态文件
# 只需启动后端即可：
cargo run --release

# 访问 http://your-server:8080
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
EXPOSE 8080
CMD ["soma-server"]
```

## 工作流程

```
用户输入主题
    │
    ▼
┌─────────────┐    ┌──────────────┐    ┌───────────────┐
│  LLM 脚本生成 │───▶│  TTS 语音合成  │───▶│  素材搜索下载   │
└─────────────┘    └──────────────┘    └───────────────┘
                                              │
              ┌───────────────┐               │
              │  字幕 + 视频合成  │◀──────────────┘
              └───────┬───────┘
                      │
                      ▼
                 成品视频输出
```

## API 接口

所有接口统一通过 `/api/v1/{module}` 路由访问：

| 模块 | 说明 |
|------|------|
| `videos/create` | 创建视频生成任务 |
| `tasks/list` | 获取任务列表 |
| `tasks/get` | 查询任务状态 |
| `tasks/delete` | 删除任务 |
| `scripts/generate` | 生成视频脚本 |
| `terms/*` | 关键词提取 |
| `musics/list` | 获取背景音乐列表 |
| `materials/list` | 获取素材列表 |
| `materials/fonts` | 获取系统字体列表 |
| `stream/play` | 视频流播放 |
| `stream/download` | 视频下载 |
| `config/get` | 获取当前配置 |
| `config/save` | 保存配置 |
| `voice/preview` | 预览语音 |
| `llm/*` | LLM 直接调用 |

## 常见问题

### Q: 启动报错 "FFmpeg not found"
确保 FFmpeg 已安装并在 PATH 中：`ffmpeg -version`

### Q: 生成任务失败 "LLM chat failed"
检查 `conf/config.toml` 中 LLM 的 `api_key` 是否正确，`provider` 是否匹配

### Q: 视频无素材画面（黑屏）
检查素材库 API Key 是否配置，或使用本地素材模式（`video_source = "local"`，素材放 `materials/` 目录）

### Q: 语音合成失败
默认 Edge TTS 需要网络连接。如网络受限，可切换到本地 Ollama + 本地 TTS

### Q: 如何添加背景音乐？
将 MP3 文件放入 `resource/songs/` 目录，生成视频时可在界面中选择

### Q: 如何添加自定义字体？
将字体文件放入 `resource/fonts/` 目录，系统会自动识别（同时也会扫描系统字体）

## 配置完整参考

详见 [conf/CONFIG_GUIDE.md](conf/CONFIG_GUIDE.md) — 包含所有 25+ LLM 供应商、7 种 TTS 引擎、3 个素材库的配置方法与 API Key 申请地址。

## License

MIT
