#!/usr/bin/env bash
# usage: serve_ninfer.sh KV_CAPACITY_TOKENS [CONCURRENCY=8] [KV_DTYPE=nvfp4]
# Find the largest KV_CAPACITY that starts: try high, drop by 50000 on OOM. Watch the "capacity |" log line.
WS=${WS:-/workspace}
exec $WS/ninfer/build/apps/ninfer-serve $WS/models/ninfer/qwen3_8_27b_nvfp4.ninfer \
  --host 127.0.0.1 --port ${PORT:-8080} --device ${DEVICE:-0} --max-context ${MAXCTX:-262144} --kv-capacity ${1:?kv capacity} \
  --max-concurrency ${2:-8} --kv-dtype ${3:-nvfp4} --device-state-slots 2 \
  --spec mtp --draft-tokens 3 --lm-head-draft --preserve-thinking --host-context-mib ${HOST_MIB:-32768}
