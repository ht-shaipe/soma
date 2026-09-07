#!/usr/bin/env python3
"""EchoMimicV3-Flash 模型权重下载脚本

供 soma 项目通过子进程调用，从 ModelScope 或 HuggingFace 下载 EchoMimicV3-Flash 模型权重。
支持断点续传，下载完成后校验文件完整性。

用法:
  python echomimic_v3_downloader.py \
    --model_source huggingface \
    --target_dir /path/to/models \
    --repo_id BadToBest/EchoMimicV3

输出:
  下载中: stdout 输出 {"status": "downloading", "downloaded_bytes": N, "total_bytes": M, "speed": S}
  成功:   stdout 输出 {"status": "success", "total_bytes": M}
  失败:   stdout 输出 {"status": "failed", "error": "..."}
"""

import argparse
import json
import os
import sys
import time
import traceback


REQUIRED_FILES = [
    "flash/Wan2.1-Fun-V1.1-1.3B-InP",
    "flash/chinese-wav2vec2-base",
    "flash/transformer/diffusion_pytorch_model.safetensors",
]


def check_integrity(target_dir):
    missing = []
    for rel_path in REQUIRED_FILES:
        full_path = os.path.join(target_dir, rel_path)
        if not os.path.exists(full_path):
            missing.append(rel_path)
        elif os.path.isfile(full_path) and os.path.getsize(full_path) == 0:
            missing.append(rel_path + " (空文件)")
    return missing


def download_from_huggingface(repo_id, target_dir):
    from huggingface_hub import snapshot_download

    snapshot_path = snapshot_download(
        repo_id=repo_id,
        local_dir=target_dir,
        resume_download=True,
    )
    return snapshot_path


def download_from_modelscope(repo_id, target_dir):
    from modelscope.hub.snapshot_download import snapshot_download

    snapshot_path = snapshot_download(
        model_id=repo_id,
        local_dir=target_dir,
    )
    return snapshot_path


def main():
    parser = argparse.ArgumentParser(description="EchoMimicV3-Flash 模型下载")
    parser.add_argument("--model_source", default="modelscope", choices=["modelscope", "huggingface"], help="下载源")
    parser.add_argument("--target_dir", required=True, help="模型权重目标目录")
    parser.add_argument("--repo_id", default="BadToBest/EchoMimicV3", help="模型仓库 ID")
    args = parser.parse_args()

    try:
        os.makedirs(args.target_dir, exist_ok=True)

        missing = check_integrity(args.target_dir)
        if not missing:
            total_bytes = 0
            for root, _, files in os.walk(args.target_dir):
                for fname in files:
                    total_bytes += os.path.getsize(os.path.join(root, fname))
            print(json.dumps({"status": "success", "total_bytes": total_bytes}))
            sys.exit(0)

        print(json.dumps({"status": "downloading", "downloaded_bytes": 0, "total_bytes": 0, "speed": 0}))

        if args.model_source == "huggingface":
            download_from_huggingface(args.repo_id, args.target_dir)
        else:
            download_from_modelscope(args.repo_id, args.target_dir)

        missing = check_integrity(args.target_dir)
        if missing:
            raise RuntimeError(f"下载完成后文件校验失败，缺失: {', '.join(missing)}")

        total_bytes = 0
        for root, _, files in os.walk(args.target_dir):
            for fname in files:
                total_bytes += os.path.getsize(os.path.join(root, fname))

        print(json.dumps({"status": "success", "total_bytes": total_bytes}))
        sys.exit(0)

    except Exception as e:
        error_msg = str(e)
        stderr_msg = traceback.format_exc()
        print(json.dumps({"status": "failed", "error": error_msg}))
        sys.stderr.write(stderr_msg)
        sys.exit(1)


if __name__ == "__main__":
    main()