#!/bin/bash
# usage: auto.sh LABEL START LANES KV NS  — wait for model, find largest KV capacity that starts, run the bench
cd /workspace/gpu-bench
L=$1; c=$2; LANES=$3; KV=$4; NS=$5
until [ -x /workspace/ninfer/build/apps/ninfer-serve ] && ! pgrep -x ninja >/dev/null && [ -f /workspace/models/ninfer/qwen3_8_27b_nvfp4.ninfer ]; do sleep 10; done
while :; do
  pkill -f ninfer-serve; sleep 5
  ./serve_ninfer.sh $c $LANES $KV > serve.log 2>&1 &
  until curl -sf http://127.0.0.1:8081/health >/dev/null || grep -q FATAL serve.log; do sleep 5; done
  if curl -sf http://127.0.0.1:8081/health >/dev/null; then echo "CAPACITY_OK $c"; break; fi
  echo "failed $c: $(grep FATAL serve.log)"; c=$((c-50000)); [ $c -lt 300000 ] && { echo NO_CAPACITY; exit 1; }
done
grep -E "capacity \||context \|" serve.log
cp serve.log serve_$L.log
sed "s#> telemetry.csv#> telemetry_$L.csv#; s#>> telemetry.csv#>> telemetry_$L.csv#" sample.sh > sample_$L.sh; chmod +x sample_$L.sh; ./sample_$L.sh & SP=$!
curl -s http://127.0.0.1:8081/metrics > metrics_before_$L.prom
./run.sh $L $NS $NS http://127.0.0.1:8081 qwen3.8-27b
echo RUN_EXIT=$?
curl -s http://127.0.0.1:8081/metrics > metrics_after_$L.prom; grep -E "restores_total|replayed_tokens_total" metrics_after_$L.prom
kill $SP; cp serve.log serve_$L.log
