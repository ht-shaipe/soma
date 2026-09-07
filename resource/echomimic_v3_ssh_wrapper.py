#!/usr/bin/env python3
"""EchoMimicV3-Flash SSH 远程推理包装脚本

在本地运行，通过 SSH 连接远程 GPU 服务器执行 EchoMimicV3-Flash 推理。
需要预先配置 SSH 密钥认证（ssh-copy-id）。

用法:
  python echomimic_v3_ssh_wrapper.py \
    --portrait photo.jpg \
    --audio audio.wav \
    --outfile output.mp4 \
    --model_dir /path/to/models \
    --device cuda:0 \
    --resolution 512 \
    --infer_steps 8 \
    --ssh_host connect.cqa1.seetacloud.com \
    --ssh_port 26322 \
    --ssh_user root \
    --remote_root /root/autodl-tmp/echomimic_v3 \
    --remote_python /root/miniconda3/bin/python

输出:
  成功: stdout 输出 {"status": "success", "video_path": "/abs/path/output.mp4"}
  失败: stdout 输出 {"status": "failed", "error": "...", "stderr": "..."}
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
import traceback
import uuid


def ssh_exec(host, port, user, remote_cmd, timeout=30):
    cmd = [
        "ssh", "-p", str(port),
        "-o", "StrictHostKeyChecking=no",
        "-o", f"ConnectTimeout={min(timeout, 30)}",
        f"{user}@{host}",
        remote_cmd,
    ]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 10)


def scp_upload(host, port, user, local_path, remote_path, timeout=120):
    cmd = [
        "scp", "-P", str(port),
        "-o", "StrictHostKeyChecking=no",
        local_path, f"{user}@{host}:{remote_path}",
    ]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)


def scp_download(host, port, user, remote_path, local_path, timeout=120):
    cmd = [
        "scp", "-P", str(port),
        "-o", "StrictHostKeyChecking=no",
        f"{user}@{host}:{remote_path}", local_path,
    ]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)


def main():
    parser = argparse.ArgumentParser(description="EchoMimicV3-Flash SSH 远程推理包装")
    parser.add_argument("--portrait", required=True, help="人像照片路径")
    parser.add_argument("--audio", required=True, help="音频文件路径")
    parser.add_argument("--outfile", required=True, help="输出视频路径")
    parser.add_argument("--model_dir", required=True, help="模型权重根目录（远程路径）")
    parser.add_argument("--device", default="cuda:0", help="推理设备")
    parser.add_argument("--resolution", type=int, default=512, help="输出分辨率")
    parser.add_argument("--infer_steps", type=int, default=8, help="Flash 推理步数")
    parser.add_argument("--video_length", type=int, default=0, help="视频帧数（0=自动）")
    parser.add_argument("--config_path", default="", help="推理配置文件路径（远程）")
    args = parser.parse_args()

    ssh_host = os.environ.get("EMV3_SSH_HOST", "connect.cqa1.seetacloud.com")
    ssh_port = int(os.environ.get("EMV3_SSH_PORT", "26322"))
    ssh_user = os.environ.get("EMV3_SSH_USER", "root")
    remote_root = os.environ.get("EMV3_REMOTE_ROOT", "/root/autodl-tmp/echomimic_v3")
    remote_python = os.environ.get("EMV3_REMOTE_PYTHON", "/root/miniconda3/bin/python")

    try:
        out_dir = os.path.dirname(os.path.abspath(args.outfile))
        os.makedirs(out_dir, exist_ok=True)

        session_id = uuid.uuid4().hex[:8]
        remote_tmp = f"/tmp/emv3_ssh_{session_id}"
        r = ssh_exec(ssh_host, ssh_port, ssh_user, f"mkdir -p {remote_tmp}")
        if r.returncode != 0:
            raise RuntimeError(f"创建远程临时目录失败: {r.stderr}")

        remote_portrait = f"{remote_tmp}/portrait{os.path.splitext(args.portrait)[1]}"
        remote_audio = f"{remote_tmp}/audio{os.path.splitext(args.audio)[1]}"
        remote_outfile = f"{remote_tmp}/output.mp4"

        r = scp_upload(ssh_host, ssh_port, ssh_user, args.portrait, remote_portrait)
        if r.returncode != 0:
            raise RuntimeError(f"上传人像失败: {r.stderr}")

        r = scp_upload(ssh_host, ssh_port, ssh_user, args.audio, remote_audio)
        if r.returncode != 0:
            raise RuntimeError(f"上传音频失败: {r.stderr}")

        remote_script = f"{remote_root}/echomimic_v3_runner.py"
        runner_cmd = (
            f"cd {remote_root} && "
            f"{remote_python} {remote_script} "
            f"--portrait {remote_portrait} "
            f"--audio {remote_audio} "
            f"--outfile {remote_outfile} "
            f"--model_dir {args.model_dir} "
            f"--resolution {args.resolution} "
            f"--infer_steps {args.infer_steps} "
            f"--video_length {args.video_length}"
        )

        r = ssh_exec(ssh_host, ssh_port, ssh_user, runner_cmd, timeout=600)
        if r.returncode != 0:
            raise RuntimeError(f"远程推理失败: {r.stdout}\n{r.stderr}")

        try:
            result = json.loads(r.stdout.strip().split("\n")[-1])
        except (json.JSONDecodeError, IndexError):
            result = {"status": "failed", "error": f"无法解析远程输出: {r.stdout}"}

        if result.get("status") != "success":
            print(json.dumps(result))
            sys.exit(1)

        r = scp_download(ssh_host, ssh_port, ssh_user, remote_outfile, args.outfile)
        if r.returncode != 0:
            raise RuntimeError(f"下载视频失败: {r.stderr}")

        if not os.path.exists(args.outfile):
            raise RuntimeError(f"输出文件不存在: {args.outfile}")

        ssh_exec(ssh_host, ssh_port, ssh_user, f"rm -rf {remote_tmp}")

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
