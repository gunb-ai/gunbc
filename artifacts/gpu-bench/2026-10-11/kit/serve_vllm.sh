#!/usr/bin/env bash
# usage: serve_vllm.sh [KV_DTYPE=fp8_e4m3]   (nvfp4 KV is unsupported for head_size 256; alt: turboquant_4bit_nc)
# pip install vllm  (we used 0.30)
exec vllm serve ${MODEL:-nvidia/Qwen3.8-27B-NVFP4} --served-model-name qwen --port ${PORT:-8080} --tensor-parallel-size ${TP:-1} \
  --kv-cache-dtype ${1:-fp8_e4m3} --max-model-len 262144 --max-num-seqs ${MAX_SEQS:-32} \
  --max-num-batched-tokens 32768 --enable-chunked-prefill --enable-prefix-caching \
  --gpu-memory-utilization ${GPU_UTIL:-0.90} --speculative-config '{"method":"mtp","num_speculative_tokens":3}' ${EXTRA:-}
