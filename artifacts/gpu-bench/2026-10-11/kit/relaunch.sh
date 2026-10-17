#!/bin/bash
# usage: relaunch.sh LABEL KV TP SHORT WARM UTIL SEQS MODEL EXTRA — kill everything, relaunch vauto
cd /workspace/gpu-bench
tmux kill-session -t fn 2>/dev/null; tmux kill-session -t vauto 2>/dev/null
pkill -9 -f "[v]llm serve"; pkill -9 -f "[V]LLM::"; pkill -9 -f "[E]ngineCore"; sleep 8
nvidia-smi --query-gpu=memory.used --format=csv,noheader | tr "\n" " "; echo
mkdir -p failed && for f in serve_$1.log vauto.log vauto_fn.log; do [ -f $f ] && mv $f failed/$(date +%H%M)_$f; done
tmux new -d -s vauto "GPU_UTIL=$6 MAX_SEQS=$7 MODEL=$8 EXTRA=\"$9\" ./vauto.sh $1 $2 $3 $4 $5 > vauto.log 2>&1"
tmux ls | cut -c1-8
