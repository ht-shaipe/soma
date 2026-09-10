#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Live2D 卡通口播视频渲染脚本

供 soma 项目通过子进程调用，执行 Live2D 模型口型同步视频生成。
输入：音频文件 + Live2D 模型目录 → 输出：口型同步视频

部署步骤：
  1. 安装 Python 3.8+
  2. pip install live2d-py glfw pillow numpy
  3. pip install phonemizer（需 espeak-ng）或 aeneas
  4. 确保 FFmpeg 已安装并在 PATH 中

用法:
  python3 live2d_runner.py \
    --audio audio.wav \
    --model_dir /path/to/model \
    --outfile output.mp4 \
    --fps 30 \
    --width 1080 \
    --height 1920 \
    --render_threads 1

输出:
  成功: stdout 输出 {"status": "success", "video_path": "...", "frames_count": N, "duration": F}
  失败: stdout 输出 {"status": "failed", "error": "...", "stderr": "..."}
"""

import argparse
import atexit
import bisect
import json
import logging
import os
import subprocess
import sys
import tempfile
import traceback

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    stream=sys.stderr,
)
logger = logging.getLogger("live2d_runner")

VISEME_TABLE = {
    "a":  (1.0,  0.0, 0.5),
    "o":  (0.8, -0.8, 0.3),
    "e":  (0.6, -0.3, 0.4),
    "i":  (0.3,  0.8, 0.2),
    "u":  (0.3, -0.9, 0.1),
    "v":  (0.3, -0.8, 0.1),
    "b":  (0.1,  0.0, 0.3),
    "p":  (0.1,  0.0, 0.3),
    "m":  (0.1,  0.0, 0.3),
    "f":  (0.2, -0.5, 0.2),
    "d":  (0.3,  0.3, 0.3),
    "t":  (0.3,  0.3, 0.3),
    "n":  (0.2,  0.2, 0.3),
    "l":  (0.3,  0.5, 0.3),
    "g":  (0.2, -0.2, 0.3),
    "k":  (0.2, -0.2, 0.3),
    "h":  (0.3, -0.3, 0.4),
    "j":  (0.3,  0.6, 0.2),
    "q":  (0.3,  0.6, 0.2),
    "x":  (0.3,  0.5, 0.2),
    "zh": (0.3, -0.4, 0.3),
    "ch": (0.3, -0.4, 0.3),
    "sh": (0.3, -0.4, 0.3),
    "r":  (0.3, -0.5, 0.3),
    "z":  (0.3, -0.3, 0.3),
    "c":  (0.3, -0.3, 0.3),
    "s":  (0.3, -0.3, 0.4),
    "w":  (0.3, -0.7, 0.1),
    "y":  (0.3,  0.7, 0.2),
}

NEUTRAL_VISEME = (0.0, 0.0, 0.0)

_cleanup_paths = []


def _atexit_cleanup():
    for path in _cleanup_paths:
        try:
            if os.path.isdir(path):
                import shutil
                shutil.rmtree(path, ignore_errors=True)
            elif os.path.exists(path):
                os.remove(path)
        except Exception:
            pass


atexit.register(_atexit_cleanup)

_orig_fd = os.dup(1)
os.dup2(2, 1)


def emit_success(video_path, frames_count, duration):
    data = json.dumps({
        "status": "success",
        "video_path": os.path.abspath(video_path),
        "frames_count": int(frames_count),
        "duration": round(float(duration), 3),
    }) + "\n"
    os.write(_orig_fd, data.encode())


def emit_failed(error, stderr_text=""):
    data = json.dumps({
        "status": "failed",
        "error": str(error),
        "stderr": str(stderr_text)[-2000:],
    }) + "\n"
    os.write(_orig_fd, data.encode())
    sys.exit(1)


def find_model3_json(model_dir):
    for name in os.listdir(model_dir):
        if name.endswith(".model3.json"):
            return os.path.join(model_dir, name)
    return None


def init_opengl(width, height):
    try:
        import glfw
        glfw.init()
        window = glfw.create_window(width, height, "live2d_render", None, None)
        if window is None:
            emit_failed("GLFW 窗口创建失败")
            return None
        glfw.make_context_current(window)
        glfw.window_hint(glfw.VISIBLE, glfw.FALSE)
        return window
    except Exception as e:
        emit_failed(f"OpenGL 初始化失败: {e}", traceback.format_exc())
        return None


def load_model(model_dir):
    try:
        from live2d.v3 import init as l2d_init, glInit, LAppModel
    except ImportError as e:
        emit_failed(f"live2d-py 未安装: {e}")
        return None

    model3_path = find_model3_json(model_dir)
    if model3_path is None:
        emit_failed(f"模型目录缺少 .model3.json: {model_dir}")
        return None

    try:
        l2d_init()
        glInit()
        model = LAppModel()
        model.LoadModelJson(model3_path)
        model.CreateRenderer()
        logger.info("模型加载成功: %s", model3_path)
        return model
    except Exception as e:
        emit_failed(f"模型加载失败: {e}", traceback.format_exc())
        return None


def get_audio_duration(audio_path):
    try:
        import torchaudio
        info = torchaudio.info(audio_path)
        return info.num_frames / info.sample_rate
    except Exception:
        pass

    try:
        result = subprocess.run(
            ["ffprobe", "-v", "quiet", "-print_format", "json",
             "-show_format", audio_path],
            capture_output=True, text=True, timeout=30,
        )
        if result.returncode == 0:
            meta = json.loads(result.stdout)
            return float(meta["format"]["duration"])
    except Exception:
        pass

    emit_failed("无法确定音频时长（需安装 torchaudio 或 ffprobe）")
    return 0.0


def extract_phonemes(audio_path, audio_duration):
    phonemes = []

    try:
        import phonemizer
        from phonemizer.separator import Separator
        sep = Separator(phone="|", word="", syllable="")
        text = " ".join(["a"] * max(1, int(audio_duration / 0.3)))
        phn_str = phonemizer.phonemize(text, separator=sep, backend="espeak", language="cmn")
        phn_list = [p for p in phn_str.replace(" ", "").split("|") if p]
        if phn_list:
            seg_dur = audio_duration / len(phn_list)
            for i, p in enumerate(phn_list):
                phonemes.append((p, i * seg_dur, (i + 1) * seg_dur))
            logger.info("phonemizer 提取音素 %d 个", len(phonemes))
            return phonemes
    except Exception as e:
        logger.warning("phonemizer 不可用: %s，使用回退方案", e)

    n_segments = max(1, int(audio_duration / 0.3))
    seg_dur = audio_duration / n_segments
    dummy_phonemes = ["a", "o", "e", "i", "u", "n", "m", "s", "sh", "h"]
    for i in range(n_segments):
        p = dummy_phonemes[i % len(dummy_phonemes)]
        phonemes.append((p, i * seg_dur, (i + 1) * seg_dur))
    logger.info("回退方案生成音素 %d 个", len(phonemes))
    return phonemes


def map_viseme(phonemes):
    viseme_seq = []
    for phoneme, start_t, end_t in phonemes:
        p = phoneme.lower().strip()
        params = VISEME_TABLE.get(p, VISEME_TABLE.get(p[0] if p else "", NEUTRAL_VISEME))
        viseme_seq.append((params[0], params[1], params[2], start_t, end_t))
    return viseme_seq


def _interpolate_viseme(viseme_seq, time_point):
    if not viseme_seq:
        return NEUTRAL_VISEME

    start_times = [v[3] for v in viseme_seq]
    idx = bisect.bisect_right(start_times, time_point) - 1

    if idx < 0:
        return viseme_seq[0][:3]
    if idx >= len(viseme_seq):
        return viseme_seq[-1][:3]

    curr = viseme_seq[idx]
    if idx + 1 < len(viseme_seq):
        nxt = viseme_seq[idx + 1]
        span = nxt[3] - curr[3]
        if span > 0:
            ratio = (time_point - curr[3]) / span
            ratio = max(0.0, min(1.0, ratio))
            return (
                curr[0] + (nxt[0] - curr[0]) * ratio,
                curr[1] + (nxt[1] - curr[1]) * ratio,
                curr[2] + (nxt[2] - curr[2]) * ratio,
            )
    return curr[:3]


def render_frames(model, viseme_seq, audio_duration,
                  fps, width, height, frames_dir):
    total_frames = int(audio_duration * fps)
    if total_frames <= 0:
        emit_failed("帧数为 0，音频时长过短或 fps 过低")
        return 0

    logger.info("开始渲染 %d 帧 (%.2fs × %dfps)",
                total_frames, audio_duration, fps)

    from OpenGL.GL import glReadPixels, GL_RGB, GL_UNSIGNED_BYTE
    from PIL import Image

    model.Resize(width, height)

    for idx in range(total_frames):
        t = idx / fps
        mouth_open_y, mouth_form, mouth_open_x = _interpolate_viseme(viseme_seq, t)
        try:
            model.SetParameterValue("ParamMouthOpenY", mouth_open_y)
            model.SetParameterValue("ParamMouthForm", mouth_form)
            model.Update()
            model.Draw()

            pixels = glReadPixels(0, 0, width, height, GL_RGB, GL_UNSIGNED_BYTE)
            img = Image.frombytes('RGB', (width, height), pixels)
            img = img.transpose(Image.FLIP_TOP_BOTTOM)
            frame_path = os.path.join(frames_dir, f"frame_{idx:06d}.png")
            img.save(frame_path)

            if (idx + 1) % 100 == 0:
                logger.info("已渲染 %d/%d 帧", idx + 1, total_frames)
        except Exception as e:
            emit_failed(f"帧 {idx} 渲染失败: {e}", traceback.format_exc())
            return 0

    logger.info("渲染完成，共 %d 帧", total_frames)
    return total_frames


def compose_video(frames_dir, audio_path, outfile, fps):
    cmd = [
        "ffmpeg", "-y",
        "-framerate", str(fps),
        "-i", os.path.join(frames_dir, "frame_%06d.png"),
        "-i", audio_path,
        "-c:v", "libx264",
        "-pix_fmt", "yuv420p",
        "-c:a", "aac",
        "-shortest",
        outfile,
    ]
    logger.info("FFmpeg 合成: %s", " ".join(cmd))
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    except FileNotFoundError:
        emit_failed("FFmpeg 未安装或不在 PATH 中")
        return False
    except subprocess.TimeoutExpired:
        emit_failed("FFmpeg 合成超时")
        return False

    if result.returncode != 0:
        emit_failed(f"FFmpeg 合成失败 (退出码 {result.returncode})", result.stderr)
        return False

    if not os.path.exists(outfile) or os.path.getsize(outfile) == 0:
        emit_failed("合成输出文件异常：文件不存在或大小为 0")
        return False

    logger.info("FFmpeg 合成成功: %s", outfile)
    return True


def main():
    parser = argparse.ArgumentParser(description="Live2D 卡通口播视频渲染")
    parser.add_argument("--audio", required=True, help="音频文件路径")
    parser.add_argument("--model_dir", required=True, help="Live2D 模型目录路径")
    parser.add_argument("--outfile", required=True, help="输出视频路径")
    parser.add_argument("--fps", type=int, default=30, help="渲染帧率 [24, 60]")
    parser.add_argument("--width", type=int, default=1080, help="输出宽度")
    parser.add_argument("--height", type=int, default=1920, help="输出高度")
    parser.add_argument("--render_threads", type=int, default=1, help="渲染线程数")
    args = parser.parse_args()

    fps = max(24, min(60, args.fps))
    width = max(256, min(3840, args.width))
    height = max(256, min(3840, args.height))

    if not os.path.exists(args.audio):
        emit_failed(f"音频文件不存在: {args.audio}")
        return
    if not os.path.isdir(args.model_dir):
        emit_failed(f"模型目录不存在: {args.model_dir}")
        return

    out_dir = os.path.dirname(os.path.abspath(args.outfile))
    os.makedirs(out_dir, exist_ok=True)

    window = init_opengl(width, height)
    if window is None:
        return

    model = load_model(args.model_dir)
    if model is None:
        return

    audio_duration = get_audio_duration(args.audio)
    if audio_duration <= 0:
        emit_failed(f"音频时长无效: {audio_duration}")
        return
    logger.info("音频时长: %.2fs", audio_duration)

    phonemes = extract_phonemes(args.audio, audio_duration)
    if not phonemes:
        emit_failed("音素提取结果为空")
        return

    viseme_seq = map_viseme(phonemes)
    logger.info("viseme 映射完成，共 %d 段", len(viseme_seq))

    with tempfile.TemporaryDirectory(prefix="live2d_frames_") as frames_dir:
        _cleanup_paths.append(frames_dir)
        try:
            frames_count = render_frames(
                model, viseme_seq, audio_duration,
                fps, width, height, frames_dir,
            )
            if frames_count == 0:
                return

            if not compose_video(frames_dir, args.audio, args.outfile, fps):
                return

            _cleanup_paths.remove(frames_dir)
            emit_success(args.outfile, frames_count, audio_duration)
        except Exception as e:
            emit_failed(f"未预期错误: {e}", traceback.format_exc())


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        emit_failed(f"未预期错误: {e}", traceback.format_exc())
