#!/usr/bin/env bash
# Build NInfer + fetch model. Needs: CUDA 13.1 toolkit, cmake>=3.28, ninja, ffmpeg+curl dev libs, git, python3.
set -euo pipefail
WS=${WS:-/workspace}; mkdir -p $WS/models/ninfer
export PATH=/usr/local/cuda/bin:$PATH
command -v apt-get >/dev/null && { apt-get update -q && DEBIAN_FRONTEND=noninteractive apt-get install -yq pkg-config libavformat-dev libavcodec-dev libavutil-dev libswscale-dev libcurl4-openssl-dev; }
command -v hf >/dev/null || pip install -q -U "huggingface_hub[cli]"
[ -d $WS/ninfer ] || git clone https://github.com/neroued/ninfer $WS/ninfer
cd $WS/ninfer && git checkout 81c8ce09 2>/dev/null || echo "WARN: commit 81c8ce09 not found, using HEAD"
# pip-only CUDA images lack nvtx3 headers; link them in if missing
if [ ! -e /usr/local/cuda/include/nvtx3 ]; then
  n=$(python3 -c 'import nvidia,os;print(os.path.dirname(nvidia.__path__[0]))' 2>/dev/null || true)
  f=$(find / -path '*include/nvtx3' -type d 2>/dev/null | head -1 || true); [ -n "$f" ] && ln -s "$f" /usr/local/cuda/include/nvtx3
fi
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build -j
hf download neroued/Qwen3.8-27B-nvfp4-NInfer qwen3_8_27b_nvfp4.ninfer --local-dir $WS/models/ninfer
ls -la $WS/models/ninfer/qwen3_8_27b_nvfp4.ninfer $WS/ninfer/build/apps/ninfer-serve
echo SETUP_OK
