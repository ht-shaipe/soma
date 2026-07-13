# Soma 配置指南

## 配置文件

配置文件路径：`conf/config.toml`

---

## 一、必须配置（核心功能依赖）

| 配置项 | 用途 | 当前值 | 申请地址 | 备注 |
|--------|------|--------|----------|------|
| `llm.api_key` | LLM 脚本生成 | `""` ⚠️ | 见下方LLM表 | 至少配一个LLM |
| `stock.pexels_api_key` | 素材视频搜索 | `""` ⚠️ | https://www.pexels.com/api/ | 免费，200次/时 |
| `stock.pixabay_api_key` | 素材视频搜索 | `""` | https://pixabay.com/api/docs/ | 免费，100次/分 |
| `stock.coverr_api_key` | 素材视频搜索 | `""` | https://coverr.co/api | 免费 |

> ⚠️ 当前 config.toml 中所有 API Key 均为空，核心功能（脚本生成 + 素材搜索）无法使用。

---

## 二、LLM 供应商配置（选配其一即可）

### 免费方案（无需 API Key）

| 供应商 | provider 值 | 默认模型 | 说明 |
|--------|------------|----------|------|
| **Ollama** | `ollama` | llama3 | 本地部署，需先安装 Ollama 并下载模型 |
| **Pollinations** | `pollinations` | openai | 免费 Cloud API，无需 Key |
| **G4F** | `g4f` | gpt-4o-mini | 免费 GPT 接口 |

### 付费方案

| 供应商 | provider 值 | 所需 Key | 默认模型 | 申请地址 | 备注 |
|--------|------------|----------|----------|----------|------|
| **DeepSeek** | `deepseek` | `deepseek_api_key` | deepseek-chat | https://platform.deepseek.com/signup | 注册送额度，价格极低 |
| **OpenAI** | `openai` | `openai_api_key` | gpt-4o-mini | https://platform.openai.com/signup | 按量付费 |
| **Qwen/通义千问** | `qwen` | `qwen_api_key` | qwen-turbo | https://dashscope.console.aliyun.com/ | 需阿里云账号，有免费额度 |
| **Gemini** | `gemini` | `gemini_api_key` | gemini-2.0-flash | https://aistudio.google.com/apikey | 需 Google 账号，有免费额度 |
| **Moonshot/月之暗面** | `moonshot` | `moonshot_api_key` | moonshot-v1-8k | https://platform.moonshot.cn/ | 需中国手机，注册送额度 |
| **Groq** | `groq` | `groq_api_key` | llama-3.1-8b-instant | https://console.groq.com/keys | 免费额度，推理极快 |
| **Grok/xAI** | `grok` | `grok_api_key` | grok-3 | https://console.x.ai/ | 有限测试版 |
| **Azure OpenAI** | `azure` | `azure_api_key` + `azure_base_url` + `api_version` | gpt-4o | https://oai.azure.com/portal | 需 Azure 订阅 + 审批 |
| **MiniMax** | `minimax` | `minimax_api_key` | abab6.5s-chat | https://platform.minimaxi.com/ | 注册送额度 |
| **MiMo/小米** | `mimo` | `mimo_api_key` | mimo | https://mimo.xiaomi.com/ | 小米开发者平台 |
| **Doubao/火山引擎** | `doubao` | `doubao_api_key` | doubao-pro-32k | https://www.volcengine.com/product/doubao | 需火山引擎账号，有免费额度 |
| **Hunyuan/腾讯混元** | `hunyuan` | `hunyuan_api_key` | hunyuan-turbo | https://hunyuan.tencent.com/ | 需腾讯云账号 |
| **Zhipu/智谱** | `zhipu` | `zhipu_api_key` | glm-4-flash | https://bigmodel.cn/ | 需中国手机，注册送 token |
| **Wenxin/百度文心** | `wenxin` | `wenxin_api_key` + `secret_key` | ernie-4.0-8k | https://qianfan.baidubce.com/ | 需百度云账号 |
| **Xunfei/讯飞星火** | `xunfei` | `xunfei_api_key` | generalv3.5 | https://www.xfyun.cn/ | 需中国手机，注册送额度 |

### 聚合/代理方案

| 供应商 | provider 值 | 所需 Key | 默认模型 | 申请地址 | 备注 |
|--------|------------|----------|----------|----------|------|
| **AIHubMix** | `aihubmix` | `aihubmix_api_key` | gpt-4o-mini | https://aihubmix.com | 聚合路由 |
| **EvoLink** | `evolink` | `evolink_api_key` | gpt-4o-mini | https://evolink.ai | 注册送额度，免信用卡 |
| **AIMLAPI** | `aiml` | `aimlapi_api_key` | gpt-4o-mini | https://aimlapi.com | 500+ 模型 |
| **ModelScope/魔搭** | `modelscope` | `modelscope_api_key` | qwen-turbo | https://modelscope.cn/ | 阿里云账号 |
| **Cloudflare Workers AI** | `cloudflare` | `openai_api_key` + `account_id` | llama-3-8b | https://dash.cloudflare.com/ | 需 CF 账号，免费额度 |
| **OneAPI** | `oneapi` | `oneapi_api_key` | gpt-4o-mini | https://github.com/songquanpeng/one-api | 开源自建网关，MIT 协议 |
| **LiteLLM** | `litellm` | `oneapi_api_key` | gpt-4o-mini | https://github.com/BerriAI/litellm | 开源代理，100+ LLM |

---

## 三、TTS 语音合成配置

| TTS 引擎 | provider 值 | 所需 Key | 申请地址 | 免费？ |
|----------|------------|----------|----------|--------|
| **Edge TTS** (默认) | `edge` | 无需 Key | — | ✅ 免费 |
| **Azure Speech** | `azure-v2` | `azure_speech_key` + `azure_speech_region` | https://portal.azure.com/#create/Microsoft.CognitiveServicesSpeechServices | 免费5小时/月 |
| **SiliconFlow** | `siliconflow` | `siliconflow_key` | https://siliconflow.cn/ | 注册送额度 |
| **ElevenLabs** | `elevenlabs` | `elevenlabs_key` | https://elevenlabs.io/app/sign-up | 免费1万字符/月 |
| **MiMo TTS** | `mimo` | `mimo_key`（与 LLM 共用） | https://mimo.xiaomi.com/ | — |
| **Gemini TTS** | `gemini` | `gemini_key`（与 LLM 共用） | https://aistudio.google.com/apikey | — |
| **无语音** | `none` | 无需 Key | — | ✅ 静音模式 |

语音前缀路由（在 voice_name 中使用前缀切换引擎）：
- `siliconflow:` → SiliconFlow TTS
- `elevenlabs:` → ElevenLabs TTS
- `mimo:` → MiMo TTS
- `gemini:` → Gemini TTS
- `azure:` → Azure TTS
- 无前缀 → Edge TTS

---

## 四、素材视频供应商配置

| 供应商 | video_source 值 | 所需 Key | 申请地址 | 免费？ |
|--------|----------------|----------|----------|--------|
| **Pexels** | `pexels` | `pexels_api_key` | https://www.pexels.com/api/ | ✅ 免费，200次/时 |
| **Pixabay** | `pixabay` | `pixabay_api_key` | https://pixabay.com/api/docs/ | ✅ 免费，100次/分 |
| **Coverr** | `coverr` | `coverr_api_key` | https://coverr.co/api | ✅ 免费 |
| **本地素材** | `local` | 无需 Key | — | ✅ 手动上传到 materials/ 目录 |

> 三个在线供应商均支持多 Key 轮换（后端 `pexels_api_keys` 为数组），可用于突破速率限制。

---

## 五、视频生成大模型配置（可选，AI 原生视频生成）

> 视频生成大模型可根据文字/图片直接生成视频片段，区别于「素材搜索拼接」模式。
> 若不配置，Soma 仍可通过素材库（Pexels/Pixabay/Coverr）+ LLM脚本 + TTS 合成视频。

### 供应商一览

| 供应商 | provider 值 | 所需 Key | 默认模型 | 是否免费 | 申请地址 |
|--------|------------|----------|----------|---------|---------|
| **智谱 CogVideoX** | `zhipu_video` | `zhipu_video_api_key` | cogvideox-flash | 有免费模型 | https://open.bigmodel.cn |
| **快手 可灵 Kling** | `kling_video` | `kling_access_key` + `kling_secret_key` | kling-v2-master | 付费 | https://platform.kuaishou.com |
| **MiniMax 海螺 Hailuo** | `minimax_video` | `minimax_video_api_key` | MiniMax-Hailuo-2.3 | 付费（有赠送额度） | https://platform.minimaxi.com |

### 配置示例

```toml
[app]
video_gen_timeout = 300   # 视频生成超时（秒），默认 300

# 智谱 CogVideoX（推荐首选，有免费模型）
zhipu_video_api_key = "your-zhipu-api-key"
zhipu_video_model = "cogvideox-flash"   # 免费；付费可选 cogvideox-3 / vidu2-* / viduq1-*

# 快手 可灵 Kling
kling_access_key = "your-kling-access-key"
kling_secret_key = "your-kling-secret-key"
kling_video_model = "kling-v2-master"   # 可选 kling-v1 / kling-v1-pro

# MiniMax 海螺 Hailuo
minimax_video_api_key = "your-minimax-video-api-key"
minimax_video_model = "MiniMax-Hailuo-2.3"   # 可选 video-01 / T2V-01 / I2V-01
```

---

### 5.1 智谱 CogVideoX（推荐首选）

**为什么推荐**：API 文档最完善，SDK 成熟，且有免费模型 `cogvideox-flash` 可用，支持文生视频、图生视频、首尾帧等多种能力。

**可用模型与价格**

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

**Key 申请步骤**

1. 访问 https://open.bigmodel.cn ，点击右上角「注册/登录」
2. 使用手机号注册并完成验证
3. 新用户注册后自动赠送免费额度（Token）
4. 登录后进入「API Keys」页面：https://open.bigmodel.cn/usercenter/api-keys
5. 点击「创建 API Key」，复制保存（仅显示一次）
6. 将 Key 填入 `zhipu_video_api_key` 配置项
7. 免费模型 `cogvideox-flash` 可直接使用；付费模型需充值

**注意事项**
- 视频生成采用异步模式：提交后返回任务 ID，需轮询查询结果
- 生成的视频 URL 有效期 1 天，需及时下载转存
- 并发限制：V0 账号 5 个、V1 账号 10 个、V2 账号 15 个、V3 账号 20 个
- API 端点：`POST https://open.bigmodel.cn/api/paas/v4/videos/generations`

---

### 5.2 快手 可灵 Kling

**可用模型与价格**

| 模型 | 能力 | 时长 | 分辨率 | 价格（参考） |
|------|------|------|--------|------------|
| `kling-v1` | 文生视频/图生视频 | 5s/10s | 720P/1080P | 0.5-1 元/次 |
| `kling-v1-pro` | 高品质文生视频 | 5s/10s | 1080P | 2-3 元/次 |
| `kling-v2-master` | 最新模型 | 5s/10s | 1080P | 2-3 元/次 |

**Key 申请步骤**

1. 访问 https://platform.kuaishou.com ，点击「注册/登录」
2. 使用手机号注册快手开放平台账号
3. 完成开发者认证（需实名认证）
4. 创建应用，获取 **Access Key** 和 **Secret Key**
5. 将 Access Key 填入 `kling_access_key`，Secret Key 填入 `kling_secret_key`
6. 可灵使用 OAuth 流程生成 access_token 进行 API 调用

**注意事项**
- API 文档需登录后访问，公开资料有限
- 视频生成同样采用异步模式：提交 → 返回 task_id → 轮询查询
- 10 秒视频价格约为 5 秒的 2 倍
- API 基础 URL：`https://api.kuaishou.com`

---

### 5.3 MiniMax 海螺 Hailuo

**可用模型与价格**

| 模型 | 能力 | 时长 | 分辨率 | 价格（参考） |
|------|------|------|--------|------------|
| `MiniMax-Hailuo-2.3` | 文生视频 | 5s | 1080P | 0.5-1 元/次 |
| `video-01` | 文生视频 | — | — | 0.5-1 元/次 |
| `T2V-01` | 文生视频 | 5s | 1080P | 0.5-1 元/次 |
| `I2V-01` | 图生视频 | 5s | 1080P | 0.8-1.5 元/次 |
| `video-01-live2d` | 图生视频/角色动画 | — | 2D 动画 | 1-2 元/次 |
| `S2V-01` | 主体参考生视频 | — | — | 1-2 元/次 |

**Key 申请步骤**

1. 访问 https://platform.minimaxi.com ，点击「注册/登录」
2. 使用邮箱或手机号注册
3. 新用户注册后有赠送额度
4. 登录后进入控制台，在「API Keys」页面创建 Key
5. 将 Key 填入 `minimax_video_api_key` 配置项

**注意事项**
- 国内产品页：https://hailuoai.com ；API 平台：https://platform.minimaxi.com
- 视频生成异步模式：提交 → 返回 task_id → 轮询查询
- API 基础 URL：`https://api.minimax.chat` 或 `https://platform.minimaxi.com/api`

---

### 视频生成模型对比总结

| 维度 | 智谱 CogVideoX | 可灵 Kling | MiniMax Hailuo |
|------|---------------|-----------|----------------|
| **免费模型** | ✅ cogvideox-flash | ❌ | ❌ |
| **文生视频** | ✅ | ✅ | ✅ |
| **图生视频** | ✅ | ✅ | ✅ |
| **首尾帧** | ✅ | — | — |
| **最高分辨率** | 4K | 1080P | 1080P |
| **最长时长** | 10s | 10s | 5s |
| **API 成熟度** | ★★★★★ | ★★★ | ★★★ |
| **起步价格** | 免费 | 0.5 元/次 | 0.5 元/次 |
| **适合场景** | 通用/4K/免费体验 | 快消/电商 | 角色动画 |

> **推荐**：优先使用智谱（免费模型 + API 最完善），可灵和 MiniMax 作为备选。

---

## 六、自动发布配置（可选）

| 配置项 | 用途 | 默认值 | 申请地址 |
|--------|------|--------|----------|
| `ui.upload_post_enabled` | 启用自动发布 | `false` | — |
| `ui.upload_post_api_key` | Upload-Post API Key | `""` | https://app.upload-post.com |
| `ui.upload_post_username` | Upload-Post 账号 | `""` | 同上 |
| `ui.upload_post_platforms` | 目标平台列表 | `[]` | — |
| `ui.upload_post_auto_upload` | 生成后自动上传 | `false` | — |
| `ui.upload_post_youtube_privacy_status` | YouTube 隐私设置 | `"public"` | — |

支持的平台：TikTok、YouTube、Instagram、Facebook 等 12 个平台。

> 注意：当前版本无前端 UI 入口，需通过编辑 config.toml 或直接调用 API 配置。开启 `auto_upload` 后生成视频会自动发布。

---

## 七、基础架构配置（一般无需修改）

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `app.name` | `soma` | 应用名称 |
| `app.version` | `0.1.0` | 版本号 |
| `app.host` | `0.0.0.0` | 监听地址 |
| `app.port` | `8090` | 监听端口 |
| `app.storage_path` | `./storage` | 存储/输出目录 |
| `app.concurrent_tasks` | `2` | 最大并发视频生成任务数 |
| `ffmpeg.path` | `ffmpeg` | FFmpeg 路径（需系统安装） |
| `ffmpeg.threads` | `4` | FFmpeg 线程数 |
| `whisper.provider` | `edge` | 字幕生成引擎（edge/whisper） |
| `whisper.model` | `""` | Whisper 模型大小（base/small/medium/large） |
| `proxy.http` | `""` | HTTP 代理 |
| `proxy.https` | `""` | HTTPS 代理 |

---

## 八、系统依赖

- **FFmpeg**：视频合成核心依赖，需系统安装（`brew install ffmpeg` / `apt install ffmpeg`）
- **Ollama**（可选）：本地 LLM，安装后 `ollama pull llama3` 下载模型

---

## ⚡ 最快上手方案

### 方案 A：零成本全本地
```toml
[llm]
provider = "ollama"
model = "llama3"

[app]
video_source = "local"   # 手动上传素材到 materials/ 目录
```

### 方案 B：最低成本 Cloud
```toml
[llm]
provider = "pollinations"   # 免费，无需 Key
model = "openai"

[stock]
pexels_api_key = "你的Pexels Key"   # https://www.pexels.com/api/ 免费申请
```

### 方案 C：推荐（性价比最优）
```toml
[llm]
provider = "deepseek"
api_key = "你的DeepSeek Key"   # https://platform.deepseek.com/signup 注册送额度
model = "deepseek-chat"
base_url = "https://api.deepseek.com"

[tts]
provider = "edge"             # 默认免费

[stock]
pexels_api_key = "你的Pexels Key"
```
