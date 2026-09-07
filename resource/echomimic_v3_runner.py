#!/usr/bin/env python3
"""EchoMimicV3-Flash 推理封装脚本

供 soma 项目通过子进程调用，执行 EchoMimicV3-Flash 数字人口播视频生成。
输入：人像照片 + 音频文件 → 输出：口型同步视频

部署步骤：
  1. 克隆 EchoMimicV3: git clone https://github.com/antgroup/echomimic_v3
  2. 安装依赖: cd echomimic_v3 && pip install -r requirements.txt
  3. 下载模型权重到 {model_dir}/flash/ 目录
  4. 将本脚本放置到 EchoMimicV3 根目录

用法:
  python echomimic_v3_runner.py \
    --portrait photo.jpg \
    --audio audio.wav \
    --outfile output.mp4 \
    --model_dir /path/to/models \
    --resolution 512 \
    --infer_steps 8

输出:
  成功: stdout 输出 {"status": "success", "video_path": "/abs/path/output.mp4"}
  失败: stdout 输出 {"status": "failed", "error": "...", "stderr": "..."}
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import traceback


def build_infer_command(args, echomimic_root, save_path):
    model_base = os.path.join(args.model_dir, "flash")
    wan_model = os.path.join(model_base, "Wan2.1-Fun-V1.1-1.3B-InP")
    transformer = os.path.join(model_base, "transformer", "diffusion_pytorch_model.safetensors")
    wav2vec = os.path.join(model_base, "chinese-wav2vec2-base")
    config_path = os.path.join(echomimic_root, "config", "config.yaml")
    infer_script = os.path.join(echomimic_root, "infer_flash.py")

    python_bin = sys.executable

    cmd = [
        python_bin, infer_script,
        "--image_path", args.portrait,
        "--audio_path", args.audio,
        "--prompt", "A person is speaking.",
        "--num_inference_steps", str(args.infer_steps),
        "--config_path", config_path,
        "--model_name", wan_model,
        "--ckpt_idx", "50000",
        "--transformer_path", transformer,
        "--save_path", save_path,
        "--wav2vec_model_dir", wav2vec,
        "--sampler_name", "Flow_Unipc",
        "--video_length", str(args.video_length),
        "--guidance_scale", "6.0",
        "--audio_guidance_scale", "3.0",
        "--audio_scale", "1.0",
        "--neg_scale", "1.0",
        "--neg_steps", "0",
        "--seed", str(args.seed),
        "--enable_teacache",
        "--teacache_threshold", "0.1",
        "--num_skip_start_steps", "5",
        "--riflex_k", "6",
        "--ulysses_degree", "1",
        "--ring_degree", "1",
        "--weight_dtype", "bfloat16",
        "--sample_size", str(args.resolution), str(args.resolution),
        "--fps", "25",
        "--add_prompt", "",
        "--negative_prompt", "",
        "--shift", "5.0",
    ]
    return cmd


def main():
    parser = argparse.ArgumentParser(description="EchoMimicV3-Flash 推理封装")
    parser.add_argument("--portrait", required=True, help="人像照片路径")
    parser.add_argument("--audio", required=True, help="音频文件路径")
    parser.add_argument("--outfile", required=True, help="输出视频路径")
    parser.add_argument("--model_dir", required=True, help="模型权重根目录（其下应有 flash/ 子目录）")
    parser.add_argument("--resolution", type=int, default=512, help="输出分辨率 (512 或 768)")
    parser.add_argument("--infer_steps", type=int, default=8, help="Flash 推理步数")
    parser.add_argument("--video_length", type=int, default=0, help="视频帧数（0=根据音频自动计算）")
    parser.add_argument("--seed", type=int, default=43, help="随机种子")
    args = parser.parse_args()

    if args.video_length <= 0:
        import torchaudio
        try:
            info = torchaudio.info(args.audio)
            audio_dur = info.num_frames / info.sample_rate
            args.video_length = min(int(audio_dur * 25) + 8, 150)
        except Exception:
            args.video_length = 81
        print(f"auto video_length={args.video_length} (audio_dur={audio_dur:.1f}s)" if 'audio_dur' in dir() else f"fallback video_length={args.video_length}", file=sys.stderr)

    try:
        echomimic_root = os.path.dirname(os.path.abspath(__file__))

        out_dir = os.path.dirname(os.path.abspath(args.outfile))
        os.makedirs(out_dir, exist_ok=True)

        with tempfile.TemporaryDirectory(prefix="emv3_out_") as tmp_save_path:
            cmd = build_infer_command(args, echomimic_root, tmp_save_path)

            env = os.environ.copy()
            env["PYTORCH_CUDA_ALLOC_CONF"] = "max_split_size_mb:128"

            result = subprocess.run(
                cmd,
                cwd=echomimic_root,
                env=env,
                capture_output=True,
                text=True,
                timeout=600,
            )

            if result.returncode != 0:
                print(json.dumps({
                    "status": "failed",
                    "error": f"infer_flash.py exited with code {result.returncode}",
                    "stderr": result.stderr[-2000:] if result.stderr else "",
                }))
                sys.exit(1)

            image_name = os.path.splitext(os.path.basename(args.portrait))[0]
            generated = os.path.join(tmp_save_path, f"{image_name}_output.mp4")

            if not os.path.exists(generated):
                files = os.listdir(tmp_save_path) if os.path.exists(tmp_save_path) else []
                print(json.dumps({
                    "status": "failed",
                    "error": f"输出文件未生成: {generated}",
                    "stderr": f"tmp files: {files}\n{result.stderr[-1000:]}",
                }))
                sys.exit(1)

            shutil.copy2(generated, args.outfile)

            if not os.path.exists(args.outfile):
                raise RuntimeError(f"复制输出文件失败: {args.outfile}")

            print(json.dumps({"status": "success", "video_path": os.path.abspath(args.outfile)}))

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
