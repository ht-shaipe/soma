# MoneyPrinterTurbo 模型与 API 对接需求分析

> 项目地址：`/Users/shaipe/workspace/ai/agv/MoneyPrinterTurbo`
> 分析日期：2026-06-25

---

## 一、项目概述

MoneyPrinterTurbo 是一个 AI 驱动的短视频自动生成工具，输入主题即可自动完成脚本撰写、配音、素材采集、字幕合成与视频拼接。

核心流水线：

```
用户输入主题 → LLM脚本生成 → TTS语音合成 → 素材搜索下载 → 字幕生成 → 视频合成
```

---

## 二、必要模型与服务（运行必须对接）

### 1. LLM 大语言模型（必要，至少选1个）

用于：脚本生成、搜索关键词提取、社交发布文案生成

| 提供商 | provider 值 | 默认模型 | 是否免费 | 申请地址 |
|--------|------------|----------|---------|---------|
| **OpenAI** | `openai` | gpt-4o-mini | 付费 | https://platform.openai.com/api-keys |
| **DeepSeek** | `deepseek` | deepseek-chat | 付费（有赠送额度） | https://platform.deepseek.com/api_keys |
| **Qwen/通义千问** | `qwen` | qwen-max | 付费（有免费额度） | https://dashscope.console.aliyun.com/apiKey |
| **Moonshot/Kimi** | `moonshot` | moonshot-v1-8k | 付费（有赠送额度） | https://platform.moonshot.cn/console/api-keys |
| **Gemini** | `gemini` | gemini-2.5-flash | 有免费额度 | https://ai.google.dev/ |
| **Azure OpenAI** | `azure` | gpt-35-turbo | 付费 | https://learn.microsoft.com/azure/ai-services/openai/ |
| **Ollama（本地）** | `ollama` | llama3 | **免费** | https://ollama.com/library |
| **Groq** | `groq` | llama-3.3-70b-versatile | 有免费额度 | https://console.groq.com/keys |
| **Grok** | `grok` | grok-4.3 | 付费 | https://api.x.ai/ |
| **AIHubMix** | `aihubmix` | gpt-5.4-mini | 付费 | https://aihubmix.com/?aff=CEve |
| **AIML API** | `aimlapi` | openai/gpt-4o-mini | 付费 | https://aimlapi.com/app/keys |
| **OneAPI** | `oneapi` | 自定义 | 付费 | https://github.com/songquanpeng/one-api |
| **Cloudflare** | `cloudflare` | 自定义 | 付费 | https://dash.cloudflare.com/ |
| **MiniMax** | `minimax` | MiniMax-M3 | 付费 | https://platform.minimax.io |
| **EvoLink** | `evolink` | gpt-5.5 | 付费 | https://evolink.ai/dashboard/keys |
| **MiMo（小米）** | `mimo` | mimo-v2.5-pro | 付费 | https://platform.xiaomimimo.com/docs/zh-CN/quick-start/first-api-call |
| **文心一言** | `ernie` | 自定义 | 付费 | https://aip.baidubce.com/ |
| **ModelScope（魔搭）** | `modelscope` | Qwen/Qwen3-32B | 有免费额度 | https://modelscope.cn/docs/model-service/API-Inference/intro |
| **Pollinations** | `pollinations` | openai-fast | **免费** | https://pollinations.ai/ （无需Key） |
| **LiteLLM** | `litellm` | 自定义 | 取决后端 | https://docs.litellm.ai/docs/providers |
| **g4f** | `g4f` | gpt-3.5-turbo | 免费（不稳定） | https://github.com/xtekky/gpt4free |

> **推荐最低配置**：DeepSeek（价格低、中文质量好）或 Ollama（完全免费本地运行）

---

### 2. TTS 语音合成（必要，至少选1个）

用于：将脚本文本转为配音音频

| 引擎 | 前缀标识 | 是否免费 | 需要Key | 申请地址 |
|------|---------|---------|--------|---------|
| **Edge TTS (Azure V1)** | 无前缀 | **免费** | 否 | 无需申请，开箱即用 |
| **Azure TTS V2** | 名字含 `-V2` | 付费 | 是 | https://portal.azure.com/#view/Microsoft_Azure_ProjectOxford/CognitiveServicesHub/~/SpeechServices |
| **SiliconFlow** | `siliconflow:` | 有免费额度 | 是 | https://siliconflow.cn |
| **Gemini TTS** | `gemini:` | 有免费额度 | 是（复用 Gemini API Key） | 同 Gemini LLM |
| **MiMo TTS** | `mimo:` | 付费 | 是（复用 MiMo API Key） | 同 MiMo LLM |
| **ElevenLabs** | `elevenlabs:` | 有免费额度 | 是 | https://elevenlabs.io/app/settings/api-keys |
| **无配音** | `no-voice` | 免费 | 否 | 无需申请 |

> **推荐最低配置**：Edge TTS 免费，无需任何 Key，支持中英文多语种，开箱即用

---

### 3. 视频素材库 API（必要，至少选1个）

用于：按关键词搜索和下载无版权视频素材

| 素材库 | source 值 | 是否免费 | 限制 | 申请地址 |
|--------|----------|---------|------|---------|
| **Pexels** | `pexels` | **免费** | 200次/时 | https://www.pexels.com/api/ |
| **Pixabay** | `pixabay` | **免费** | 5000次/时 | https://pixabay.com/api/docs/ |
| **Coverr** | `coverr` | 免费 | Demo: 50次/时 | https://coverr.co/developers?ctx=header_navigation |
| **本地素材** | `local` | 免费 | 无 | 无需申请 |

> **推荐最低配置**：Pexels（免费、素材丰富、竖屏素材多）

---

## 三、可选模型与服务

### 4. Whisper 语音识别（可选）

用于：更精确的字幕时间轴对齐（仅当 `subtitle_provider = "whisper"` 时启用）

| 项目 | 说明 |
|------|------|
| 模型 | faster-whisper（local） |
| 推荐模型大小 | large-v3（约3GB）/ large-v3-turbo（约250MB） |
| 是否需要Key | 否（本地模型） |
| 下载方式 | 自动从 HuggingFace 下载，或手动从网盘下载 |

国内下载备选：
- 百度网盘：https://pan.baidu.com/s/11h3Q6tsDtjQKTjUu3sc5cA?pwd=xjs9
- 夸克网盘：https://pan.quark.cn/s/3ee3d991d64b

> **说明**：默认使用 `edge` 模式（基于 Edge TTS 时间戳），无需 GPU 和额外模型。仅在字幕质量不佳时切换 whisper。

---

### 5. Upload-Post 跨平台发布（可选）

用于：视频生成后自动发布到 TikTok / Instagram / YouTube Shorts

| 项目 | 说明 |
|------|------|
| 是否必要 | 否 |
| 申请地址 | https://upload-post.com |
| 文档 | https://docs.upload-post.com |

---

### 6. Redis 状态管理（可选）

用于：任务队列状态持久化（默认使用内存）

| 项目 | 说明 |
|------|------|
| 是否必要 | 否（`enable_redis = false` 时使用内存） |
| 安装 | https://redis.io/download |

---

### 7. 代理配置（可选）

用于：无法直连海外 API 时的网络代理

```toml
[proxy]
http = "http://user:pass@proxy:1234"
https = "http://user:pass@proxy:1234"
```

---

## 四、最低运行配置（零成本方案）

```toml
[app]
video_source = "pexels"
pexels_api_keys = ["你的Pexels免费Key"]

llm_provider = "ollama"           # 或 pollinations（也免费）

[whisper]
# 不启用，使用 edge 模式

# Ollama 配置（本地运行，完全免费）
ollama_base_url = "http://localhost:11434"
ollama_model_name = "llama3"

# TTS 使用 Edge TTS（免费，无需 Key）
# 默认即可，无需额外配置
```

**零成本方案需要安装**：
1. Ollama：https://ollama.com （下载后运行 `ollama pull llama3`）
2. Pexels 注册获取免费 API Key

---

## 五、推荐运行配置（性价比方案）

```toml
[app]
video_source = "pexels"
pexels_api_keys = ["你的Pexels免费Key"]
llm_provider = "deepseek"

# DeepSeek（中文质量好，价格极低）
deepseek_api_key = "sk-xxx"
deepseek_model_name = "deepseek-chat"

# TTS 使用 Edge TTS（免费）
# 默认即可
```

---

## 六、各 API Key 申请步骤

### 6.1 Pexels（免费，推荐首选）

1. 访问 https://www.pexels.com/api/
2. 填写注册表单（姓名、邮箱、用途说明）
3. 邮箱验证后即可获得 API Key
4. 免费版限制：200次/小时，足够个人使用

### 6.2 Pixabay（免费）

1. 访问 https://pixabay.com/api/docs/
2. 注册账号
3. 登录后在 API文档页 获取 Key
4. 免费版限制：5000次/小时

### 6.3 Coverr（免费）

1. 访问 https://coverr.co/developers?ctx=header_navigation
2. 注册即可获取 Key
3. Demo 计划：50次/小时；付费版 2000次/小时

### 6.4 DeepSeek（推荐 LLM，性价比最高）

1. 访问 https://platform.deepseek.com/
2. 注册账号（手机号/邮箱）
3. 进入 API Keys 页面创建 Key
4. 新用户通常有赠送额度
5. 价格：输入 ¥1/百万Token，输出 ¥2/百万Token

### 6.5 OpenAI

1. 访问 https://platform.openai.com/api-keys
2. 注册 OpenAI 账号
3. 绑定信用卡
4. 创建 API Key
5. 最低使用 gpt-4o-mini 模型（价格低）

### 6.6 Qwen/通义千问

1. 访问 https://dashscope.console.aliyun.com/apiKey
2. 登录阿里云账号
3. 开通 DashScope 服务
4. 创建 API Key
5. 有免费额度（qwen-turbo 等模型免费）

### 6.7 Moonshot/Kimi

1. 访问 https://platform.moonshot.cn/console/api-keys
2. 注册账号
3. 创建 API Key
4. 新用户赠送额度

### 6.8 Gemini

1. 访问 https://ai.google.dev/
2. 使用 Google 账号登录
3. 创建项目并获取 API Key
4. 免费版有限额（15 RPM）

### 6.9 Groq（免费额度大，速度快）

1. 访问 https://console.groq.com/keys
2. 注册账号
3. 创建 API Key
4. 免费版限制：14400请求/天

### 6.10 Ollama（完全免费本地运行）

1. 访问 https://ollama.com/download 下载安装
2. 安装后运行命令拉取模型：`ollama pull llama3`
3. 无需 API Key，本地服务自动运行在 http://localhost:11434

### 6.11 SiliconFlow TTS

1. 访问 https://siliconflow.cn
2. 注册账号
3. 获取 API Key
4. 有免费额度

### 6.12 ElevenLabs TTS

1. 访问 https://elevenlabs.io/app/settings/api-keys
2. 注册账号
3. 创建 API Key
4. 免费版：10000字符/月
5. 在 Voice Library 中标记喜欢的声音（favorites）

### 6.13 Azure TTS V2

1. 访问 https://portal.azure.com/
2. 创建"语音服务"资源
3. 获取订阅密钥（speech_key）和区域（speech_region）
4. 有免费额度（500万字/月）

### 6.14 MiMo TTS（小米）

1. 访问 https://platform.xiaomimimo.com/docs/zh-CN/quick-start/first-api-call
2. 注册账号
3. 获取 API Key（LLM 和 TTS 共用）

---

## 八、视频生成大模型（新增，AI 原生视频生成）

> 2026-07 新增：除素材搜索拼接模式外，Soma 现已支持通过视频生成大模型直接生成视频片段。
> 以下为三大已对接供应商的 Key 申请说明。

### 8.1 智谱 CogVideoX（推荐首选，有免费模型）

| 模型 | 能力 | 时长 | 分辨率 | 价格 |
|------|------|------|--------|------|
| `cogvideox-flash` | 文生视频 | — | — | **免费** |
| `cogvideox-3` | 文生视频/图生视频/首尾帧 | 5s, 10s | 最高 4K | 1 元/次 |
| `vidu2-image` | 图生视频 | 4s | 720P | 1.25 元/次 |
| `vidu2-start-end` | 首尾帧 | 4s | 720P | 1.25 元/次 |
| `vidu2-reference` | 参考生视频 | 4s | 720P | 2.5 元/次 |
| `viduq1-text` | 文生视频 | 5s | 1080P | 2.5 元/次 |
| `viduq1-image` | 图生视频 | 5s | 1080P | 2.5 元/次 |
| `viduq1-start-end` | 首尾帧 | 5s | 1080P | 2.5 元/次 |

**Key 申请步骤**：

1. 访问 https://open.bigmodel.cn ，注册账号（手机号）
2. 新用户注册赠送免费额度
3. 登录后进入 API Keys 页面：https://open.bigmodel.cn/usercenter/api-keys
4. 点击「创建 API Key」并复制保存（仅显示一次）
5. 填入 Soma 配置：`zhipu_video_api_key = "your-key"`
6. 默认模型 `cogvideox-flash` 免费；付费模型需充值

**Soma 配置**：
```toml
zhipu_video_api_key = "your-zhipu-api-key"
zhipu_video_model = "cogvideox-flash"
```

---

### 8.2 快手 可灵 Kling

| 模型 | 能力 | 时长 | 分辨率 | 价格（参考） |
|------|------|------|--------|------------|
| `kling-v1` | 文生视频/图生视频 | 5s/10s | 720P/1080P | 0.5-1 元/次 |
| `kling-v1-pro` | 高品质文生视频 | 5s/10s | 1080P | 2-3 元/次 |
| `kling-v2-master` | 最新模型 | 5s/10s | 1080P | 2-3 元/次 |

**Key 申请步骤**：

1. 访问 https://platform.kuaishou.com ，注册快手开放平台账号
2. 完成开发者认证（需实名认证）
3. 创建应用，获取 Access Key 和 Secret Key
4. 填入 Soma 配置：`kling_access_key` 和 `kling_secret_key`

**Soma 配置**：
```toml
kling_access_key = "your-access-key"
kling_secret_key = "your-secret-key"
kling_video_model = "kling-v2-master"
```

---

### 8.3 MiniMax 海螺 Hailuo

| 模型 | 能力 | 时长 | 分辨率 | 价格（参考） |
|------|------|------|--------|------------|
| `MiniMax-Hailuo-2.3` | 文生视频 | 5s | 1080P | 0.5-1 元/次 |
| `T2V-01` | 文生视频 | 5s | 1080P | 0.5-1 元/次 |
| `I2V-01` | 图生视频 | 5s | 1080P | 0.8-1.5 元/次 |
| `video-01-live2d` | 图生视频/角色动画 | — | 2D 动画 | 1-2 元/次 |
| `S2V-01` | 主体参考生视频 | — | — | 1-2 元/次 |

**Key 申请步骤**：

1. 访问 https://platform.minimaxi.com ，注册账号
2. 新用户注册后有赠送额度
3. 在控制台「API Keys」页面创建 Key
4. 填入 Soma 配置：`minimax_video_api_key`

**Soma 配置**：
```toml
minimax_video_api_key = "your-minimax-video-api-key"
minimax_video_model = "MiniMax-Hailuo-2.3"
```

---

### 视频生成模型对比

| 维度 | 智谱 CogVideoX | 可灵 Kling | MiniMax Hailuo |
|------|---------------|-----------|----------------|
| 免费模型 | ✅ cogvideox-flash | ❌ | ❌ |
| 文生视频 | ✅ | ✅ | ✅ |
| 图生视频 | ✅ | ✅ | ✅ |
| 首尾帧 | ✅ | — | — |
| 最高分辨率 | 4K | 1080P | 1080P |
| 最长时长 | 10s | 10s | 5s |
| API 成熟度 | ★★★★★ | ★★★ | ★★★ |
| 起步价格 | 免费 | 0.5 元/次 | 0.5 元/次 |

> **推荐**：优先使用智谱（免费 + API 最完善），可灵和 MiniMax 作为备选。

---

## 九、与 Soma 项目对比

| 维度 | MoneyPrinterTurbo | Soma |
|------|-------------------|------|
| 语言 | Python | Rust |
| LLM 提供商数量 | 20+ | 7+（通过 ai-llm-kit） |
| TTS 引擎 | 7种（Edge/Azure V2/SiliconFlow/Gemini/MiMo/ElevenLabs/静音） | 4种（Edge/SiliconFlow/MiMo/ElevenLabs） |
| 素材库 | 3种（Pexels/Pixabay/Coverr） | 3种（Pexels/Pixabay/Coverr） |
| 视频生成大模型 | 不支持 | 3种（智谱CogVideoX/可灵Kling/MiniMax Hailuo） |
| 字幕方式 | Edge时间戳 / Whisper本地模型 | Edge时间戳 / Whisper |
| 跨平台发布 | 支持（Upload-Post） | 不支持 |
| WebUI | Streamlit | Vue 3 + Element Plus |
| 视频处理 | MoviePy 2.x + Pillow | FFmpeg 直接调用 |

两者所需对接的模型和服务基本一致，核心都是 **1个LLM + 1个TTS + 1个素材库**。Soma 额外支持 **AI 原生视频生成**（智谱/可灵/MiniMax），可替代或补充素材拼接模式。
