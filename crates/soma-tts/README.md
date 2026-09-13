# soma-tts

**语音合成（TTS）与字幕生成层**：统一封装 10 家 TTS 引擎与声音克隆，提供一致的合成接口和字幕产物。

## 引擎矩阵

| 模块 | 引擎 | 密钥 |
|------|------|------|
| `edge_tts` | Microsoft Edge TTS | 免费，无需密钥 |
| `azure_tts` | Azure Speech | speech_key + region |
| `siliconflow_tts` | 硅基流动 | api_key |
| `elevenlabs_tts` | ElevenLabs | api_key + model |
| `mimo_tts` | 小米 MiMo | api_key |
| `gemini_tts` | Google Gemini TTS | api_key |
| `volcengine_tts` | 火山引擎（豆包） | app_id + token |
| `xfyun_tts` | 科大讯飞 | app_id + key + secret |
| `heygem_tts` | HeyGem 商户克隆 | 商户资产（reference audio） |
| `voice_clone_tts` | GPT-SoVITS / CosyVoice / FishSpeech 克隆 | SSH 远程环境 + 参考音频 |
| `voice_clone` | 克隆语音注册与列表 | 远程训练流程 |

## 核心抽象

- `provider.rs`：`SomaTtsProvider` trait —— `synthesize(text, voice, rate, output_path)`，
  返回 `TtsResult { audio_file, audio_duration, subtitle_cues }`。
- `voices.rs`：语音名前缀路由（`siliconflow:` / `elevenlabs:` 等），判断语音归属哪家引擎。
- `subtitle.rs`：字幕生成双路径 —— Whisper 语音识别校正，或 Edge TTS 文本时间轴推算；SRT 读写。

## 调用方式

引擎方法均为 async；阻塞宿主（任务队列线程）通过 `soma_feature::runtime::block_on_async` 桥接。
失败自动重试（指数退避），重试次数由各引擎配置段决定。

## 测试

```bash
cargo test -p soma-tts
```
