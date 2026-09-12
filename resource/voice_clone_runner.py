#!/usr/bin/env python3
"""声音克隆推理封装脚本

供 soma 项目通过 SSH 远程调用，执行声音克隆 TTS 推理。
输入：参考音频 + 参考文本 + 目标文本 → 输出：克隆音色合成的语音

部署步骤：
  1. 创建 Python 3.10/3.11 虚拟环境: python3.10 -m venv venv
  2. 激活环境: source venv/bin/activate
  3. 安装 PyTorch CUDA >= 12.1: pip install torch --index-url https://download.pytorch.org/whl/cu121
  4. 安装对应克隆模型库:
     - GPT-SoVITS: pip install gpt-sovits
     - CosyVoice: pip install cosyvoice
     - Fish-Speech: pip install fish-speech
  5. 下载模型权重到 {model_dir}/{模型名}/ 目录
  6. 将本脚本放置到运行环境根目录

用法:
  python voice_clone_runner.py \
    --reference_audio ref.wav \
    --reference_text "参考音频对应的文本" \
    --target_text "要合成的目标文本" \
    --outfile output.mp3 \
    --clone_model gpt_sovits \
    --model_dir /path/to/models \
    --device cuda:0 \
    --rate 1.0 \
    --volume 1.0

输出:
  成功: stdout 输出 {"status": "success", "audio_path": "/abs/path/output.mp3", "duration": 5.2}
  失败: stdout 输出 {"status": "failed", "error": "...", "stderr": "..."}
"""

import argparse
import json
import os
import subprocess
import sys
import traceback

os.environ.setdefault("MODELSCOPE_LOG_LEVEL", "40")
os.environ.setdefault("TRANSFORMERS_VERBOSITY", "error")
os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")

import logging
logging.getLogger("modelscope").setLevel(logging.ERROR)
logging.getLogger("transformers").setLevel(logging.ERROR)


def clone_gpt_sovits(reference_audio, reference_text, target_text, outfile, model_dir, device, rate, volume):
    import torch
    from GPT_SoVITS.inference import TTS

    model_path = os.path.join(model_dir, "GPT-SoVITS")
    gpt_path = os.path.join(model_path, "gpt.pth")
    sovits_path = os.path.join(model_path, "sovits.pth")

    tts = TTS(gpt_path=gpt_path, sovits_path=sovits_path, device=device)

    tts.set_reference(reference_audio, reference_text)

    audio = tts.synthesize(target_text, speed=rate, volume=volume)

    torchaudio_save(audio, outfile, sample_rate=tts.sample_rate)
    duration = len(audio) / tts.sample_rate
    return duration


def ensure_wav_24k(audio_path):
    """确保参考音频为 WAV 24kHz mono 格式，CosyVoice 的 load_wav 使用 soundfile 后端仅支持 WAV。"""
    import os
    import tempfile
    import subprocess
    import torchaudio

    ext = os.path.splitext(audio_path)[1].lower()
    if ext == ".wav":
        try:
            info = torchaudio.info(audio_path)
            if info.sample_rate == 24000 and info.num_channels == 1:
                return audio_path
        except Exception:
            pass

    tmp = tempfile.NamedTemporaryFile(suffix=".wav", delete=False)
    tmp.close()
    subprocess.run(
        ["ffmpeg", "-y", "-i", audio_path, "-ar", "24000", "-ac", "1", tmp.name],
        check=True, capture_output=True,
    )
    return tmp.name


def clone_cosyvoice(reference_audio, reference_text, target_text, outfile, model_dir, device, rate, volume):
    import sys
    import torch
    import torchaudio

    third_party = os.path.join(os.environ.get("PYTHONPATH", "").split(":")[0], "third_party", "Matcha-TTS")
    if os.path.isdir(third_party):
        sys.path.append(third_party)

    from cosyvoice.cli.cosyvoice import AutoModel

    wav_path = ensure_wav_24k(reference_audio)

    model = AutoModel(model_dir=model_dir)

    if reference_text:
        result = model.inference_zero_shot(target_text, reference_text, wav_path, speed=rate)
    else:
        result = model.inference_cross_lingual(target_text, wav_path, speed=rate)

    audio_chunks = []
    sample_rate = model.sample_rate
    for chunk in result:
        audio_chunks.append(chunk["tts_speech"])

    audio = torch.cat(audio_chunks, dim=1)
    audio = audio * volume

    torchaudio.save(outfile, audio, sample_rate)
    duration = audio.shape[1] / sample_rate
    return duration


def clone_fish_speech(reference_audio, reference_text, target_text, outfile, model_dir, device, rate, volume):
    import torch
    from fish_speech.inference import TTS

    model_path = os.path.join(model_dir, "Fish-Speech")
    tts = TTS(model_path=model_path, device=device)

    tts.set_reference(reference_audio, reference_text)

    audio = tts.synthesize(target_text, speed=rate, volume=volume)

    torchaudio_save(audio, outfile, sample_rate=tts.sample_rate)
    duration = len(audio) / tts.sample_rate
    return duration


def torchaudio_save(audio, path, sample_rate):
    import torch
    import torchaudio

    if not isinstance(audio, torch.Tensor):
        audio = torch.tensor(audio)
    if audio.dim() == 1:
        audio = audio.unsqueeze(0)
    torchaudio.save(path, audio, sample_rate)


def load_audio(path):
    import torchaudio

    audio, sr = torchaudio.load(path)
    return audio.squeeze(0)


def main():
    parser = argparse.ArgumentParser(description="声音克隆推理封装")
    parser.add_argument("--reference_audio", required=True, help="参考音频文件路径")
    parser.add_argument("--reference_text", default="", help="参考音频对应文本")
    parser.add_argument("--target_text", required=True, help="要合成的目标文本")
    parser.add_argument("--outfile", required=True, help="输出音频文件路径")
    parser.add_argument("--clone_model", default="gpt_sovits", choices=["gpt_sovits", "cosyvoice", "fish_speech"])
    parser.add_argument("--model_dir", required=True, help="模型权重根目录")
    parser.add_argument("--device", default="cuda:0", help="推理设备")
    parser.add_argument("--rate", type=float, default=1.0, help="语速倍率")
    parser.add_argument("--volume", type=float, default=1.0, help="音量倍率")
    args = parser.parse_args()

    try:
        out_dir = os.path.dirname(os.path.abspath(args.outfile))
        os.makedirs(out_dir, exist_ok=True)

        if args.clone_model == "gpt_sovits":
            duration = clone_gpt_sovits(
                args.reference_audio, args.reference_text, args.target_text,
                args.outfile, args.model_dir, args.device, args.rate, args.volume
            )
        elif args.clone_model == "cosyvoice":
            duration = clone_cosyvoice(
                args.reference_audio, args.reference_text, args.target_text,
                args.outfile, args.model_dir, args.device, args.rate, args.volume
            )
        elif args.clone_model == "fish_speech":
            duration = clone_fish_speech(
                args.reference_audio, args.reference_text, args.target_text,
                args.outfile, args.model_dir, args.device, args.rate, args.volume
            )
        else:
            raise ValueError(f"不支持的克隆模型: {args.clone_model}")

        if not os.path.exists(args.outfile):
            raise RuntimeError(f"输出文件未生成: {args.outfile}")

        print(json.dumps({
            "status": "success",
            "audio_path": os.path.abspath(args.outfile),
            "duration": round(duration, 3),
        }))
        sys.exit(0)

    except Exception as e:
        error_msg = str(e)
        stderr_msg = traceback.format_exc()
        print(json.dumps({
            "status": "failed",
            "error": error_msg,
            "stderr": stderr_msg,
        }))
        sys.stderr.write(stderr_msg)
        sys.exit(1)


if __name__ == "__main__":
    main()