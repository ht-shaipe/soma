#!/usr/bin/env python3
"""SadTalker 推理封装脚本

供 soma 项目通过子进程调用，执行 SadTalker 数字人口播视频生成。
输入：人像照片 + 音频文件 → 输出：口型同步视频

部署步骤：
  1. 创建 Python 虚拟环境: python3 -m venv venv
  2. 激活环境: source venv/bin/activate
  3. 克隆 SadTalker: git clone https://github.com/OpenTalker/SadTalker
  4. 安装依赖: cd SadTalker && pip install -r requirements.txt
  5. 下载模型权重到 checkpoints/ 和 gfpgan/weights/ 目录
  6. 将本脚本放置到 SadTalker 根目录

用法:
  python sadtalker_runner.py \
    --portrait photo.jpg \
    --audio audio.mp3 \
    --outfile output.mp4 \
    --checkpoint_dir checkpoints/ \
    --size 256

输出:
  成功: stdout 输出 {"status": "success", "video_path": "/abs/path/output.mp4"}
  失败: stdout 输出 {"status": "failed", "error": "...", "stderr": "..."}
"""

import argparse
import json
import os
import shutil
import sys
import tempfile
import traceback

def main():
    parser = argparse.ArgumentParser(description="SadTalker 推理封装")
    parser.add_argument("--portrait", required=True, help="人像照片路径")
    parser.add_argument("--audio", required=True, help="音频文件路径")
    parser.add_argument("--outfile", required=True, help="输出视频路径")
    parser.add_argument("--checkpoint_dir", required=True, help="模型权重目录")
    parser.add_argument("--device", default="cpu", help="推理设备 (cpu/cuda, 自动检测)")
    parser.add_argument("--still", default="false", help="still 模式")
    parser.add_argument("--full", default="false", help="全图增强")
    parser.add_argument("--size", type=int, default=256, help="输出分辨率")
    parser.add_argument("--pose_style", type=int, default=0, help="姿态风格")
    parser.add_argument("--exp_scale", type=float, default=1.0, help="表情缩放")
    parser.add_argument("--batch_size", type=int, default=2, help="batch size")
    args = parser.parse_args()

    try:
        from src.gradio_demo import SadTalker

        still_mode = args.still.lower() == "true"
        full_enhancer = args.full.lower() == "true"

        sadtalker = SadTalker(
            checkpoint_path=args.checkpoint_dir,
            config_path='src/config',
            lazy_load=False,
        )

        out_dir = os.path.dirname(os.path.abspath(args.outfile))
        os.makedirs(out_dir, exist_ok=True)

        tmp_portrait = os.path.join(out_dir, f"_tmp_portrait_{os.getpid()}.jpg")
        tmp_audio = os.path.join(out_dir, f"_tmp_audio_{os.getpid()}{os.path.splitext(args.audio)[1]}")
        shutil.copy2(args.portrait, tmp_portrait)
        shutil.copy2(args.audio, tmp_audio)

        try:
            result_path = sadtalker.test(
                tmp_portrait,
                tmp_audio,
                preprocess="full" if full_enhancer else "resize",
                still_mode=still_mode,
                use_enhancer=full_enhancer,
                batch_size=args.batch_size,
                size=args.size,
                pose_style=args.pose_style,
                exp_scale=args.exp_scale,
                result_dir=out_dir,
            )

            if result_path and os.path.exists(result_path):
                shutil.move(result_path, args.outfile)
            else:
                raise RuntimeError(f"输出文件未生成: {result_path}")

            print(json.dumps({"status": "success", "video_path": os.path.abspath(args.outfile)}))
        finally:
            for f in [tmp_portrait, tmp_audio]:
                if os.path.exists(f):
                    os.remove(f)

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
