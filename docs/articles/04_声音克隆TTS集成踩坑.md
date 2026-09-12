# 声音克隆 TTS 集成：多格式音频处理与路径解析踩坑记

> 用 CosyVoice 做声音克隆，用户上传的参考音频格式五花八门：WAV、MP3、M4A、AAC、OGG、FLAC。路径解析也踩了坑：绝对路径、相对路径、已有路径，三种情况三种处理。

## 需求

声音克隆 TTS 的输入：

- **参考音频**：用户上传的一段录音（5-10 秒），用来克隆音色
- **参考文本**：参考音频对应的文字内容
- **目标文本**：要合成的文案

输出：克隆音色的语音音频。

我们用 CosyVoice2 作为克隆引擎，部署在 GPU 服务器上，通过 SSH 远程调用。

## 坑一：参考音频格式不统一

CosyVoice 要求输入 **24kHz 单声道 WAV**。但用户上传的格式五花八门：

| 格式 | 实际遇到 | 问题 |
|------|---------|------|
| WAV | me.wav（24kHz mono） | ✅ 直接可用 |
| M4A | xie2.m4a（MP4 容器） | ❌ CosyVoice 不认 |
| MP3 | 常见 | ❌ 需转换 |
| AAC | iPhone 录音 | ❌ 需转换 |
| OGG | 网页录音 | ❌ 需转换 |
| FLAC | 音乐库 | ❌ 需转换 |

### 解决方案：FFmpeg 统一转 WAV

```python
def ensure_wav_24k(input_path: str, output_path: str) -> str:
    """确保音频为 24kHz 单声道 WAV，否则用 ffmpeg 转换"""
    if input_path.endswith('.wav'):
        # 检查采样率
        probe = subprocess.run(
            ['ffprobe', '-v', 'error', '-show_entries', 'stream=sample_rate,channels',
             '-of', 'csv=p=0', input_path],
            capture_output=True, text=True
        )
        parts = probe.stdout.strip().split(',')
        if len(parts) >= 2 and parts[0] == '24000' and parts[1] == '1':
            return input_path  # 已经是 24kHz mono WAV

    # 用 ffmpeg 转换
    subprocess.run([
        'ffmpeg', '-y', '-i', input_path,
        '-ar', '24000', '-ac', '1', '-acodec', 'pcm_s16le',
        output_path
    ], check=True)
    return output_path
```

**关键参数**：
- `-ar 24000`：采样率 24kHz（CosyVoice 要求）
- `-ac 1`：单声道
- `-acodec pcm_s16le`：16 位 PCM 编码

### Rust 侧格式校验

在 Rust 侧也要放开格式限制，不能只允许 WAV：

```rust
const ALLOWED_EXTENSIONS: &[&str] = &[
    "wav", "mp3", "m4a", "aac", "ogg", "flac",
];

fn validate_reference_audio(path: &str) -> Result<(), SomaError> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(SomaError::Config(format!(
            "参考音频格式不支持: {}，支持: {:?}", ext, ALLOWED_EXTENSIONS
        )));
    }
    Ok(())
}
```

## 坑二：M4A 不是音频文件？

用户上传 `xie2.m4a`，文件大小 148KB，看起来正常。但 FFmpeg 探测报错：

```
[mov,mp4,m4a,3gp,3g2,mj2] Error: could not find corresponding tag for codec
```

M4A 本质是 **MP4 容器**的音频流，FFmpeg 能解但需要完整的解码链。GPU 服务器上的 FFmpeg 是精简版，缺少 AAC 解码器。

**解决方案**：装完整版 FFmpeg（见上一篇文章的坑四）。或者用 Python 的 `pydub`：

```python
from pydub import AudioSegment
audio = AudioSegment.from_file(input_path, format="m4a")
audio.set_frame_rate(24000).set_channels(1).export(output_path, format="wav")
```

但 `pydub` 底层还是调 FFmpeg，所以根本解决方案还是**装完整版 FFmpeg**。

## 坑三：参考音频路径解析

这是最隐蔽的坑。用户提供的参考音频路径有三种情况：

1. **绝对路径**：`/Users/tynan/workspace/.../me.wav`
2. **相对路径**：`me.wav`（相对于 `storage/voice_clone_refs/`）
3. **已存在的路径**：`storage/voice_clone_refs/me.wav`

原来的代码只处理了情况 2：

```rust
let ref_audio = storage_dir("voice_clone_refs", false)
    .join(ref_audio_raw)
    .to_string_lossy()
    .to_string();
```

如果用户传绝对路径（情况 1），拼接后变成 `storage/voice_clone_refs//Users/tynan/.../me.wav`，路径错误。

如果用户传已存在的相对路径（情况 3），拼接后路径重复。

三种情况的对比：

| 用户输入 | 原代码结果 | 正确结果 |
|---------|-----------|---------|
| `/Users/tynan/.../me.wav` | `voice_clone_refs//Users/tynan/.../me.wav`（错） | 原样使用 |
| `me.wav` | `voice_clone_refs/me.wav` ✓ | 同左 |
| `storage/voice_clone_refs/me.wav` | `voice_clone_refs/storage/voice_clone_refs/me.wav`（错） | 原样使用 |

### 正确的路径解析

```rust
let ref_audio = if Path::new(ref_audio_raw).is_absolute() 
    || Path::new(ref_audio_raw).exists() 
{
    // 情况 1 & 3：绝对路径或已存在的路径，直接用
    ref_audio_raw.to_string()
} else {
    // 情况 2：相对路径，拼接到 voice_clone_refs 目录
    storage_dir("voice_clone_refs", false)
        .join(ref_audio_raw)
        .to_string_lossy()
        .to_string()
};
```

判断顺序很重要：**先判断绝对路径/已存在，再回退到拼接**。如果反过来，已存在的相对路径会被错误拼接。

## 坑四：SSH 传输参考音频

参考音频在本地 Mac 上，CosyVoice 在 GPU 服务器上。需要先把参考音频传过去。

SSH 调用流程：

```
本地 Mac                          GPU 服务器
  │                                  │
  │  1. scp 参考音频到服务器          │
  │─────────────────────────────────▶│
  │                                  │
  │  2. ssh 执行推理脚本             │
  │     (传入服务器上的音频路径)      │
  │─────────────────────────────────▶│
  │                                  │ 3. CosyVoice 推理
  │                                  │
  │  4. 返回合成音频路径              │
  │◀─────────────────────────────────│
  │                                  │
  │  5. scp 合成音频回本地            │
  │◀─────────────────────────────────│
```

代码实现：

```python
def ssh_exec(host, port, user, remote_cmd, timeout=30):
    cmd = [
        "ssh", "-p", str(port),
        "-o", "StrictHostKeyChecking=no",
        "-o", f"ConnectTimeout={min(timeout, 30)}",
        f"{user}@{host}", remote_cmd,
    ]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 10)

def scp_upload(host, port, user, local_path, remote_path, timeout=120):
    cmd = [
        "scp", "-P", str(port),
        "-o", "StrictHostKeyChecking=no",
        local_path, f"{user}@{host}:{remote_path}"
    ]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
```

**坑**：SSH 连接被拒绝（`Connection refused`）。AutoDL 服务器关机后 SSH 不可达，但报错信息不明确。

**解决方案**：在 SSH 调用前先做连通性检查：

```python
def check_ssh_available(host, port, user, timeout=5):
    result = ssh_exec(host, port, user, "echo ok", timeout=timeout)
    return result.returncode == 0 and "ok" in result.stdout
```

## 坑五：CosyVoice API 版本差异

**现象**：按官方 README 写的代码报错：

```
AttributeError: 'CosyVoice' object has no attribute 'inference_sft'
```

或者反过来：

```
AttributeError: 'CosyVoice2' object has no attribute 'inference_zero_shot'
```

CosyVoice2 的 API 有两个版本：

```python
# 旧版 API（CosyVoice 1.x）
model = CosyVoice2(model_path)
output = model.inference_sft(text, speaker_id)

# 新版 API（CosyVoice 2.0.5+）
from cosyvoice.cli.cosyvoice import CosyVoice
model = CosyVoice(model_path)
output = model.inference_zero_shot(text, prompt_text, prompt_audio)
```

版本不同 API 不同，文档不完善，只能看源码。

**解决方案**：在 `voice_clone_runner.py` 里做版本探测：

```python
try:
    # 尝试新版 API
    from cosyvoice.cli.cosyvoice import CosyVoice
    model = CosyVoice(model_path)
    USE_NEW_API = True
except ImportError:
    # 回退到旧版
    from cosyvoice.cli.cosyvoice2 import CosyVoice2
    model = CosyVoice2(model_path)
    USE_NEW_API = False
```

## 总结

| 坑 | 耗时 | 解决方案 |
|---|---|---|
| 音频格式不统一 | 2h | FFmpeg 统一转 24kHz mono WAV |
| M4A 解码失败 | 1h | 装完整版 FFmpeg |
| 路径解析三种情况 | 2h | 先判断绝对/已存在，再回退拼接 |
| SSH 传输参考音频 | 1h | scp 上传 + ssh 推理 + scp 下载 |
| CosyVoice API 版本 | 1h | 版本探测 + try/except 回退 |

声音克隆看起来简单（"不就是传个参考音频吗"），实际从格式到路径到传输到 API，每一步都有坑。希望这篇记录能帮你少走弯路。

---

*本文基于 soma 项目开发实践。下一篇：《数字人长视频分段生成：6 秒限制下的拼接策略与音画同步》*