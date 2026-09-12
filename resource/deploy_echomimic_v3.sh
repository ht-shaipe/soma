#!/bin/bash
# EchoMimicV3-Flash 一键部署脚本
# 在 AutoDL GPU 服务器上执行：bash deploy_echomimic_v3.sh

set -e

ECHO_DIR="/root/autodl-tmp/echomimic_v3"
MODEL_DIR="/root/autodl-tmp/echomimic_models"

echo "=== 1. 检查环境 ==="
source /root/miniconda3/etc/profile.d/conda.sh
conda activate base
python3 -c "import torch; print(f'torch={torch.__version__}, cuda={torch.cuda.is_available()}, cuda_ver={torch.version.cuda}')"

echo "=== 2. 安装依赖（跳过 torch/tensorflow/retina-face）==="
cd $ECHO_DIR
# 注释掉会覆盖已有 torch 的包
sed -i 's/^torch>=2.1.2/#torch>=2.1.2/' requirements.txt
sed -i 's/^tensorflow==2.15.0/#tensorflow==2.15.0/' requirements.txt
sed -i 's/^retina-face==0.0.17/#retina-face==0.0.17/' requirements.txt

pip install -r requirements.txt --timeout 300 -i https://pypi.tuna.tsinghua.edu.cn/simple

# 单独安装 torchdiffeq 和 torchsde（不重装 torch）
pip install torchdiffeq torchsde --timeout 300 -i https://pypi.tuna.tsinghua.edu.cn/simple --no-deps

echo "=== 3. 下载模型权重 ==="
mkdir -p $MODEL_DIR/flash

# 3.1 下载 Wan2.1-Fun-V1.1-1.3B-InP
echo "--- 下载 Wan2.1-Fun-V1.1-1.3B-InP ---"
pip install huggingface_hub modelscope --timeout 300 -i https://pypi.tuna.tsinghua.edu.cn/simple
python3 -c "
from huggingface_hub import snapshot_download
snapshot_download('alibaba-pai/Wan2.1-Fun-V1.1-1.3B-InP', local_dir='$MODEL_DIR/flash/Wan2.1-Fun-V1.1-1.3B-InP')
print('Wan2.1 下载完成')
"

# 3.2 下载 chinese-wav2vec2-base
echo "--- 下载 chinese-wav2vec2-base ---"
python3 -c "
from modelscope.hub.snapshot_download import snapshot_download
snapshot_download('TencentGameMate/chinese-wav2vec2-base', local_dir='$MODEL_DIR/flash/chinese-wav2vec2-base')
print('chinese-wav2vec2-base 下载完成')
"

# 3.3 下载 EchoMimicV3-Flash transformer 权重
echo "--- 下载 EchoMimicV3-Flash 权重 ---"
python3 -c "
from huggingface_hub import snapshot_download
snapshot_download('BadToBest/EchoMimicV3', local_dir='$MODEL_DIR/echomimic_v3_repo', allow_patterns=['echomimicv3-flash-pro/*'])
print('EchoMimicV3-Flash 下载完成')
"

# 将 transformer 权重放到正确位置
if [ -d "$MODEL_DIR/echomimic_v3_repo/echomimicv3-flash-pro/transformer" ]; then
    cp -r $MODEL_DIR/echomimic_v3_repo/echomimicv3-flash-pro/transformer $MODEL_DIR/flash/transformer
fi

echo "=== 4. 验证模型文件 ==="
python3 -c "
import os
base = '$MODEL_DIR/flash'
checks = [
    'Wan2.1-Fun-V1.1-1.3B-InP',
    'chinese-wav2vec2-base',
    'transformer/diffusion_pytorch_model.safetensors',
]
for f in checks:
    path = os.path.join(base, f)
    exists = os.path.exists(path)
    size = os.path.getsize(path) if os.path.isfile(path) else 0
    status = '✅' if exists else '❌'
    print(f'{status} {f} ({size} bytes)')
"

echo "=== 5. 部署完成 ==="
echo "模型目录: $MODEL_DIR"
echo "代码目录: $ECHO_DIR"
echo ""
echo "soma config.toml 配置:"
echo "[digital_human]"
echo 'provider = "echomimic_v3"'
echo ""
echo "[digital_human.echomimic_v3]"
echo "env_path = \"$ECHO_DIR\""
echo "model_path = \"$MODEL_DIR\""
echo 'device = "cuda:0"'
echo 'python_path = "python3"'