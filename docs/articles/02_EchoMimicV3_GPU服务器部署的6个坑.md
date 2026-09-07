# EchoMimicV3-Flash GPU 服务器部署：环境搭建的 6 个坑

> 选定 EchoMimicV3-Flash 后，在 AutoDL GPU 服务器上部署环境。看起来就是装个 Python + 下载权重，实际踩了 6 个坑，每个都花了不少时间。

## 坑一：ModelScope vs HuggingFace，国内下载的抉择

EchoMimicV3-Flash 的权重在 ModelScope 和 HuggingFace 都有。国内服务器直连 HuggingFace 不稳，第一反应是用 `hf-mirror.com`：

```bash
export HF_ENDPOINT=https://hf-mirror.com
huggingface-cli download antgroup/echomimic_v3_flash --local-dir ./models
```

但 EchoMimicV3 的权重分散在多个子目录（Wan2.1-Fun-V1.1-1.3B-InP、chinese-wav2vec2-base、transformer/），手动拼路径容易错。

**最终方案**：直接用 ModelScope：

```python
from modelscope import snapshot_download
model_dir = snapshot_download('antgroup/echomimic_v3_flash')
```

ModelScope 是阿里达摩院的，国内 CDN 速度快，蚂蚁的模型在 ModelScope 上维护得最好。**国内部署首选 ModelScope，不要折腾 HF。**

## 坑二：CUDA 版本不匹配

AutoDL 的 PyTorch 镜像默认 CUDA 12.1，但 EchoMimicV3-Flash 依赖的一些算子需要特定版本。报错信息类似：

```
RuntimeError: CUDA error: no kernel image is available for execution on the device
```

这是因为 PyTorch 编译的 CUDA 版本与服务器驱动不匹配。AutoDL 不同镜像的 CUDA 版本不同，选错了就报错。

**解决方案**：选择 `PyTorch 2.1 + CUDA 12.1` 镜像，EchoMimicV3-Flash 官方 README 明确测试过 CUDA ≥ 12.1。

```bash
# 在 AutoDL 创建实例时选择镜像
# PyTorch 2.1.0 + Python 3.10 + CUDA 12.1
```

## 坑三：显存不够，不是 12G 就行

官方标称 12G 显存，但横评实测分辨率=768 时峰值 **33.7G**。这是什么情况？

官方仓库里有三种脚本，分别用于不同场景：

| 脚本 | 用途 | 显存 | 你该用哪个 |
|------|------|------|-----------|
| `app.py` | 标准 Gradio demo，全量推理 | 33G+（768 分辨率） | 显存够时首选 |
| `app_mm.py` | 低显存模式，offload 到 CPU | 12G | 显存不够时用 |
| `infer_flash.py` | 纯推理脚本（无 Gradio UI） | 同 app.py | **项目集成用这个** |

在 24G 的 RTX 4090 上，有两个选择：

```python
# 方案 A：低显存模式（12G 够，但慢）
python app_mm.py --resolution 512

# 方案 B：标准模式降分辨率（24G 够，推荐）
python infer_flash.py --resolution 512  # 而非 768
```

**实测**：4090 24G 跑 `resolution=512` 标准模式稳定，显存占用约 18-22G。512 分辨率画质够用（输出 480×848 竖屏）。项目集成用 `infer_flash.py`（无 UI、可命令行调用、输出 JSON）。

## 坑四：FFmpeg libass 缺失，字幕烧录报错

本地 Mac 上字幕烧录正常，GPU 服务器上报错：

```
Unknown encoder 'libass'
```

AutoDL 镜像的 FFmpeg 是精简版，不含 `libass` 滤镜（字幕烧录需要）。

**解决方案**：

```bash
# 方案 A：装完整版 FFmpeg
conda install -c conda-forge ffmpeg

# 方案 B：静态编译版（推荐，不污染 conda 环境）
wget https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz
tar xf ffmpeg-release-amd64-static.tar.xz
cp ffmpeg-*/ffmpeg /usr/local/bin/
cp ffmpeg-*/ffprobe /usr/local/bin/
```

**注意**：本地 Mac 用 `static-ffmpeg`（pip 包），GPU 服务器用静态编译版，两边 FFmpeg 版本和滤镜支持要一致，否则本地能跑服务器报错。

## 坑五：SSH 并行连接被拒绝

为了加速长视频生成，尝试并行 SSH 连接 GPU 服务器同时跑多个推理。结果：

```
ssh: connect to host connect.cqa1.seetacloud.com port 26322: Connection refused
```

AutoDL 的 SSH 有**并发连接数限制**（通常 3-5 个），而且 GPU 推理本身也不适合并行（显存会被多个进程瓜分）。

**解决方案**：**串行执行，不要并行**。在 Rust 侧用 `Semaphore` 控制并发为 1：

```rust
use tokio::sync::Semaphore;
let semaphore = Arc::new(Semaphore::new(1)); // 串行
```

如果需要吞吐量，租多台 GPU 服务器分片，而不是在一台上并行。

## 坑六：模型加载慢，冷启动 4 分钟

首次推理时，模型加载 + CUDA 编译要 **4 分钟**。之后推理只要 30-60 秒（10s 成片）。如果每次推理都重新加载模型，效率极低。

### 最终方案：SSH 启动 FastAPI + HTTP 调用

```
┌─ 本地 soma ────────────────────────────┐
│  1. SSH 检查 FastAPI 是否已启动         │
│     （curl localhost:8000/health）      │
│  2. 未启动 → SSH 后台启动 FastAPI       │
│     （nohup python echomimic_service.py）│
│  3. HTTP POST /submit 提交推理请求      │
│  4. HTTP GET /query 轮询结果            │
│  5. HTTP GET /download 下载视频         │
└─────────────────────────────────────────┘
         │ SSH（仅启动服务时用）
         ▼
┌─ GPU 服务器 ───────────────────────────┐
│  FastAPI 服务（常驻）                   │
│  模型加载一次，常驻显存                  │
│  后续请求直接推理，无需重新加载          │
└─────────────────────────────────────────┘
```

模型在 FastAPI 启动时加载一次，后续 HTTP 调用直接推理，冷启动只发生在服务器重启时。

### 为什么不用纯 SSH 或纯 FastAPI？

**纯 SSH**（每次 SSH 执行 `python infer_flash.py`）：每次都是新进程，模型无法常驻，每次都要等 4 分钟冷启动。对于 30 秒一条的视频，4 分钟冷启动不可接受。

**纯 FastAPI**（手动在服务器上启动服务）：需要额外维护一个 HTTP 服务，AutoDL 服务器关机后服务丢失，每次开机要手动启动。soma 侧也要改成 HTTP provider 而非 SSH provider，改动量大。

**混合方案**兼顾两者：SSH 负责"确保服务在线"，HTTP 负责"高效推理"。soma 侧仍用 SSH 接口，只是 SSH 命令从"执行推理"变成"启动服务 + curl 调用"。

## 总结

| 坑 | 耗时 | 解决方案 |
|---|---|---|
| ModelScope vs HF | 1h | 国内用 ModelScope |
| CUDA 版本 | 2h | 选 PyTorch 2.1 + CUDA 12.1 镜像 |
| 显存不够 | 1h | 降分辨率到 512 |
| FFmpeg libass | 0.5h | 装静态编译版 FFmpeg |
| SSH 并行限制 | 1h | 串行执行 + Semaphore |
| 模型冷启动慢 | 2h | FastAPI 常驻或 SSH+HTTP 混合 |

**总耗时约 7-8 小时**，大部分时间花在试错和等下载上。如果有人提前告诉你这些坑，2 小时就能搞定。

这就是写这篇文章的目的。

---

*本文基于 soma 项目开发实践。下一篇：《Rust 异步调用 Python GPU 推理：block_on_async 模式详解》*