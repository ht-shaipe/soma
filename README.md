# Soma

AI 驱动的短视频自动生成工具 —— 输入主题，自动完成脚本撰写、配音、素材采集、字幕合成与视频拼接，一键生成成品视频。

## 功能特性

- **AI 脚本生成**：支持 DeepSeek、OpenAI、Qwen、Gemini、Moonshot、Grok、Ollama 等 20+ 种 LLM 提供商
- **智能语音合成 (TTS)**：Edge TTS、SiliconFlow、MiMo、ElevenLabs 多引擎配音，支持中/英/多语种语音
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
| 语音合成 | Edge TTS / SiliconFlow / ElevenLabs |
| AI 模型 | DeepSeek / OpenAI / Qwen / Gemini 等 |

## 项目结构

```
soma/
├── conf/                   # 配置文件目录
│   └── config.toml
├── crates/                 # Rust 工作空间
│   ├── soma-core/          # 核心模块（配置、模型、工具、错误定义）
│   ├── soma-tts/           # 语音合成（Edge TTS / SiliconFlow / MiMo / ElevenLabs）
│   ├── soma-stock/         # 素材采集（Pexels / Pixabay / Coverr）
│   ├── soma-video/         # 视频处理（FFmpeg 拼接 / 字幕 / 混音）
│   └── soma-server/        # Web 服务（API 路由 / Handler / 任务流水线）
├── resource/               # 静态资源（字体 / 背景音乐）
├── web/                    # Vue 3 前端项目
└── Cargo.toml              # Rust 工作空间配置
```

## 快速开始

### 环境要求

- Rust (Edition 2021)
- Node.js 18+
- FFmpeg（确保在系统 PATH 中可用）

### 后端

```bash
# 克隆仓库
git clone <repo-url> && cd soma

# 编辑配置文件，填入所需的 API Key
vim conf/config.toml

# 编译运行
cargo run --release
```

服务默认启动在 `http://0.0.0.0:8080`。

### 前端

```bash
cd web

# 安装依赖
npm install

# 开发模式启动
npm run dev

# 构建生产版本
npm run build
```

## 配置说明

编辑 [conf/config.toml](conf/config.toml) 进行配置：

```toml
[app]
name = "soma"
host = "0.0.0.0"
port = 8080
storage_path = "./storage"
concurrent_tasks = 2

[llm]
provider = "deepseek"          # LLM 提供商
model = "deepseek-chat"
api_key = ""                   # 填入你的 API Key
base_url = "https://api.deepseek.com"

[tts]
provider = "edge"              # TTS 引擎: edge / siliconflow / elevenlabs
voice_name = "zh-CN-XiaoxiaoNeural"

[stock]
pexels_api_key = ""            # Pexels API Key
pixabay_api_key = ""           # Pixabay API Key
coverr_api_key = ""            # Coverr API Key

[ffmpeg]
path = "ffmpeg"
threads = 4
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
| `stream/play` | 视频流播放 |

## License

MIT
