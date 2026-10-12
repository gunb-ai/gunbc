#!/usr/bin/env bash
# usage: run.sh LABEL [SHORT_NS=1,4,8] [WARM_NS=1,4,8] [URL=http://127.0.0.1:8080] [MODEL=qwen3.8-27b] [KEY=x]
# Run with ONE server up. Short test 4 min total; warm test warms max(WARM_NS) x ~174k sessions (~7 min), then 8 min cycling the Ns.
set -e; cd "$(dirname "$0")"
L=${1:?label}; S=${2:-1,4,8}; W=${3:-1,4,8}; U=${4:-http://127.0.0.1:8080}; M=${5:-qwen3.8-27b}; K=${6:-x}
nvidia-smi --query-gpu=name,memory.total,memory.used,clocks.max.mem --format=csv | tee results_$L.txt
python3 longbench.py $U $K $M short_$L.jsonl 4 $S
python3 longctx.py  $U $K $M warm_$L.jsonl 222000 8 $W
{ echo "== $L short"; python3 summarize.py short_$L.jsonl; echo "== $L warm ~174k"; python3 summarize_ctx.py warm_$L.jsonl; } | tee -a results_$L.txt
