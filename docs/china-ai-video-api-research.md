# 中国大陆 AI 视频生成 API 调研 (2025-2026)

> 调研时间: 2026年7月 | 数据来源: 各平台官方文档

---

## 1. 智谱 CogVideoX (BigModel/Zhipu)

**平台**: https://open.bigmodel.cn

### 可用模型

| 模型名 | 能力 | 时长 | 分辨率 | 价格 |
|--------|------|------|--------|------|
| `cogvideox-3` | 文生视频/图生视频/首尾帧 | 5s, 10s | 最高4K (3840x2160) | **1 元/次** |
| `cogvideox-flash` | 文生视频 (免费) | - | - | **免费** |
| `vidu2-image` | 图生视频 | 4s | 720P | **1.25 元/次** |
| `vidu2-start-end` | 首尾帧 | 4s | 720P | **1.25 元/次** |
| `vidu2-reference` | 参考生视频 | 4s | 720P | **2.5 元/次** |
| `viduq1-text` | 文生视频 | 5s | 1080P (1920x1080) | **2.5 元/次** |
| `viduq1-image` | 图生视频 | 5s | 1080P | **2.5 元/次** |
| `viduq1-start-end` | 首尾帧 | 5s | 1080P | **2.5 元/次** |

### API 端点

- **视频生成(异步)**: `POST https://open.bigmodel.cn/api/paas/v4/videos/generations`
- **查询异步结果**: 通过返回的 `id` 使用 `client.videos.retrieve_videos_result(id=...)` 查询

### 认证方式

```
Authorization: Bearer {your_api_key}
Content-Type: application/json
```

### 请求格式 (文生视频 - CogVideoX-3)

```json
{
  "model": "cogvideox-3",
  "prompt": "A cat is playing with a ball.",
  "quality": "quality",
  "with_audio": true,
  "size": "1920x1080",
  "fps": 30
}
```

### 请求格式 (图生视频 - Vidu2)

```json
{
  "model": "vidu2-image",
  "image_url": "https://example.com/image.jpg",
  "prompt": "让画面动起来",
  "duration": 4,
  "size": "1280x720",
  "movement_amplitude": "auto",
  "with_audio": true
}
```

### 请求格式 (首尾帧)

```json
{
  "model": "vidu2-start-end",
  "image_url": ["https://example.com/first.jpg", "https://example.com/last.jpg"],
  "prompt": "描述文字",
  "duration": 4,
  "size": "1280x720",
  "movement_amplitude": "auto"
}
```

### 请求格式 (文生视频 - ViduQ1)

```json
{
  "model": "viduq1-text",
  "prompt": "描述文字",
  "style": "anime",
  "duration": 5,
  "aspect_ratio": "16:9",
  "size": "1920x1080",
  "movement_amplitude": "auto"
}
```

### 响应格式

异步模式: 提交后返回任务 ID, 需轮询查询结果

```python
from zai import ZhipuAiClient
client = ZhipuAiClient(api_key="your-api-key")

response = client.videos.generations(...)  # 返回 task id
result = client.videos.retrieve_videos_result(id=response.id)  # 轮询获取结果
```

cURL 查询异步结果:
```bash
curl 'https://open.bigmodel.cn/api/paas/v4/videos/retrieve?id={task_id}' \
  --header 'Authorization: Bearer {your_api_key}'
```

视频 URL 有效期 1 天, 需及时转存。

### 并发限制

| V0 | V1 | V2 | V3 |
|----|----|----|-----|
| 5  | 10 | 15 | 20  |

### 关键参数说明

- `quality`: `"quality"` (质量优先) / `"speed"` (速度优先) — 仅 cogvideox-3
- `fps`: 30 或 60 — 仅 cogvideox-3
- `size`: 支持 `"1920x1080"`, `"1280x720"`, `"3840x2160"` 等
- `aspect_ratio`: 如 `"16:9"` — 仅 viduq1-text
- `style`: `"general"` / `"anime"` — 仅 viduq1-text
- `movement_amplitude`: `"auto"` — Vidu 系列

---

## 2. 可灵 Kling (Kuaishou/快手)

**平台**: https://platform.kuaishou.com

> 注意: 可灵API文档站点有访问限制(需登录), 以下信息基于公开资料。

### 可用模型

| 模型名 | 能力 | 时长 | 分辨率 |
|--------|------|------|--------|
| `kling-v1` | 文生视频/图生视频 | 5s/10s | 720P/1080P |
| `kling-v1-pro` | 高品质文生视频 | 5s/10s | 1080P |
| `kling-v2-master` | 最新模型 | 5s/10s | 1080P |

### API 端点

- **基础URL**: `https://api.kuaishou.com`
- **文生视频**: `POST /v1/kling/text-to-video`
- **图生视频**: `POST /v1/kling/image-to-video`
- **查询任务**: `GET /v1/kling/video-generation/{task_id}`

### 认证方式

```
Authorization: Bearer {access_token}
Content-Type: application/json
```

需在快手开放平台注册应用获取 API Key 和 Secret, 通过 OAuth 流程获取 access_token。

### 请求格式 (文生视频)

```json
{
  "model": "kling-v2-master",
  "prompt": "描述文字",
  "negative_prompt": "不要的内容",
  "duration": "5",
  "aspect_ratio": "16:9",
  "mode": "std"
}
```

### 价格 (参考)

- 标准模式: 约 **0.5-1 元/次** (5s)
- 高品质模式: 约 **2-3 元/次** (5s)
- 10s 视频: 约为 5s 的 2 倍

### 响应格式

异步模式, 返回 task_id, 轮询查询结果获取视频 URL。

---

## 3. 通义万相 / HappyHorse (Alibaba/DashScope 百炼)

**平台**: https://help.aliyun.com/zh/model-studio/ (百炼 Model Studio)

> 注意: 通义万相视频模型已更名为 **HappyHorse** 系列, 在阿里云百炼平台上提供。

### 可用模型

| 模型ID | 能力 | 说明 |
|--------|------|------|
| `happyhorse-1.1-t2v` | 文生视频 | Text-to-Video |
| `happyhorse-1.1-i2v` | 图生视频 | Image-to-Video |
| `happyhorse-1.1-r2v` | 参考生视频 | Reference-to-Video |
| `happyhorse-1.0-video-edit` | 视频编辑 | Video Edit |

### API 端点

- **DashScope 原生URL**: `https://dashscope.aliyuncs.com/api/v1`
- **视频生成**: `POST /services/aigc/text2video/generation`
- **异步查询**: `GET /tasks/{task_id}`
- **OpenAI兼容模式**: `https://{WorkspaceId}.cn-beijing.maas.aliyuncs.com/compatible-mode/v1`

### 认证方式

```
Authorization: Bearer {dashscope_api_key}
Content-Type: application/json
```

API Key 从百炼控制台获取: https://bailian.console.aliyun.com

### 请求格式 (DashScope 原生)

```json
{
  "model": "happyhorse-1.1-t2v",
  "input": {
    "prompt": "描述文字"
  },
  "parameters": {
    "size": "1280x720",
    "duration": 5,
    "seed": 42
  }
}
```

### 价格 (参考)

- happyhorse-1.1-t2v: 约 **0.5-1 元/次**
- happyhorse-1.1-i2v: 约 **0.8-1.5 元/次**
- happyhorse-1.1-r2v: 约 **1-2 元/次**

### 可用区域

华北2(北京)、新加坡、日本(东京)、德国(法兰克福)、美国(弗吉尼亚)

---

## 4. 混元 Hunyuan (Tencent/腾讯)

**平台**: https://cloud.tencent.com/product/hunyuan

> 腾讯混元视频生成 API 目前主要通过腾讯云"泰山创意创作"(taidc)产品提供,
> 尚未完全开放独立的视频生成 API。

### 可用模型

| 模型 | 能力 | 说明 |
|------|------|------|
| HunyuanVideo | 文生视频/图生视频 | 开源模型, API版本有限 |
| Hunyuan-Video-2 | 升级版 | 2025年发布 |

### API 端点

- **腾讯云基础URL**: `https://hunyuan.tencentcloudapi.com`
- **方式**: 腾讯云 API 3.0 (TC3-HMAC-SHA256 签名)

### 认证方式

腾讯云 API 使用 **TC3-HMAC-SHA256** 签名认证:

```
Authorization: TC3-HMAC-SHA256 Credential={SecretId}/...
X-TC-Timestamp: {timestamp}
X-TC-Action: {action}
```

需 SecretId + SecretKey, 从腾讯云控制台获取。

### 价格

- 文生视频: 约 **1-3 元/次** (5s, 1080P)
- 腾讯云按量计费模式

### 状态

API 开放程度有限。HunyuanVideo 开源版本可自行部署。
建议关注腾讯云泰山创意创作产品页面获取最新信息。

---

## 5. 百度 (文心一格 / Vidu)

> **重要**: Vidu 原为生数科技(百度投资)的产品, 但目前 Vidu 的 API
> 已**迁移至智谱平台**, 在 https://open.bigmodel.cn 上以 `vidu2-*` 和 `viduq1-*`
> 系列模型提供 (见第1节)。

### 结论

**百度目前没有独立的公开视频生成 API**。如需使用 Vidu, 请通过智谱平台调用。
百度文心一格主要聚焦图像生成, 暂无视频生成 API。

---

## 6. MiniMax Hailuo/海螺

**平台**: https://platform.minimaxi.com (国际) / https://hailuoai.com (国内)

### 可用模型

| 模型 | 能力 | 说明 |
|------|------|------|
| `video-01` | 文生视频 | 基础模型 |
| `video-01-live2d` | 图生视频/角色动画 | 2D动画风格 |
| `S2V-01` | 主体参考生视频 | Subject Reference |
| `T2V-01` | 文生视频 | Text-to-Video |
| `I2V-01` | 图生视频 | Image-to-Video |

### API 端点

- **基础URL**: `https://api.minimax.chat` 或 `https://platform.minimaxi.com/api`
- **文生视频**: `POST /v1/video_generation`
- **图生视频**: `POST /v1/image_to_video`
- **查询任务**: `GET /v1/query/video_generation?task_id={task_id}`

### 认证方式

```
Authorization: Bearer {api_key}
Content-Type: application/json
```

在 MiniMax 开放平台注册获取 API Key。

### 请求格式 (文生视频)

```json
{
  "model": "T2V-01",
  "prompt": "描述文字",
  "aspect_ratio": "16:9",
  "duration": 5
}
```

### 请求格式 (图生视频)

```json
{
  "model": "I2V-01",
  "prompt": "描述文字",
  "image_url": "https://example.com/image.jpg",
  "duration": 5,
  "aspect_ratio": "16:9"
}
```

### 价格 (参考)

- 文生视频: 约 **0.5-1 元/次**
- 图生视频: 约 **0.8-1.5 元/次**
- 主体参考: 约 **1-2 元/次**

### 响应格式

异步模式, 返回 task_id, 轮询查询结果获取视频 URL。

---

## 总结对比

| 服务商 | 模型 | 文生视频 | 图生视频 | 首尾帧 | 最高分辨率 | 最长时长 | 价格区间 | API成熟度 |
|--------|------|---------|---------|--------|-----------|---------|---------|----------|
| 智谱 | CogVideoX-3 | Y | Y | Y | 4K | 10s | 1元/次 | ★★★★★ |
| 智谱 | Vidu2 | N | Y | Y | 720P | 4s | 1.25元/次 | ★★★★★ |
| 智谱 | ViduQ1 | Y | Y | Y | 1080P | 5s | 2.5元/次 | ★★★★★ |
| 快手 | Kling | Y | Y | - | 1080P | 10s | 0.5-3元/次 | ★★★☆ |
| 阿里 | HappyHorse | Y | Y | - | 1080P | 5s | 0.5-2元/次 | ★★★★ |
| 腾讯 | Hunyuan | Y | Y | - | 1080P | 5s | 1-3元/次 | ★★☆ |
| 百度 | (无) | - | - | - | - | - | - | - |
| MiniMax | Hailuo | Y | Y | - | 1080P | 5s | 0.5-2元/次 | ★★★☆ |

### 推荐选择

- **性价比最高**: 智谱 CogVideoX-3 (1元/次, 4K) 或 CogVideoX-Flash (免费)
- **最高画质**: 智谱 ViduQ1 (1080P, 影视级) 或 CogVideoX-3 (4K)
- **最完整API**: 智谱平台 (文档完整, SDK成熟, 多模型可选)
- **多区域部署**: 阿里百炼 (5个区域可选, OpenAI兼容)
- **电商/快消场景**: 智谱 Vidu2 (低成本720P, 速度快)

> 注: 标注"参考"的价格可能随时间变动, 请访问各平台官网确认最新价格。
