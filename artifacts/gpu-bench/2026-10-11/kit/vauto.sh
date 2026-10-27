#!/bin/bash
# usage: vauto.sh LABEL KV TP SHORT_NS WARM_NS — serve vLLM, bench, keep metrics + telemetry
cd /workspace/gpu-bench
L=$1; KV=$2; export TP=$3; S=$4; W=$5
pkill -f "[n]infer-serve"; pkill -f "[v]llm serve"; sleep 5
./serve_vllm.sh $KV > serve_$L.log 2>&1 &
until curl -sf http://127.0.0.1:8081/health >/dev/null; do sleep 10; pgrep -f "vllm serve" >/dev/null || { echo SERVE_DIED; tail -30 serve_$L.log; exit 1; }; done
echo SERVE_OK; grep -iE "KV cache|GPU KV|Maximum concurrency|memory" serve_$L.log | tail -6
sed "s#> telemetry.csv#> telemetry_$L.csv#; s#>> telemetry.csv#>> telemetry_$L.csv#" sample.sh > sample_$L.sh; chmod +x sample_$L.sh; ./sample_$L.sh & SP=$!
[ "$TP" -gt 1 ] && { nvidia-smi dmon -s t -d 1 -o T > pcie_$L.log & DP=$!; }
curl -s http://127.0.0.1:8081/metrics > metrics_before_$L.prom
./run.sh $L $S $W http://127.0.0.1:8081 qwen
echo RUN_EXIT=$?
curl -s http://127.0.0.1:8081/metrics > metrics_after_$L.prom; grep -E "prefix_cache_(hits|queries)_total|num_preemptions_total" metrics_after_$L.prom
kill $SP $DP 2>/dev/null
