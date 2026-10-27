#!/bin/bash
# samples GPU + host memory every 5s into telemetry.csv until killed
cd /workspace/gpu-bench
echo "ts,gpu_index,gpu_mem_used_mib,gpu_mem_total_mib,gpu_util_pct,mem_util_pct,power_w,sm_clock_mhz,mem_clock_mhz,temp_c,host_used_mib,host_avail_mib,swap_used_mib" > telemetry.csv
while :; do
  t=$(date -u +%FT%TZ); h=$(free -m | awk '/Mem:/{u=$3;a=$7} /Swap:/{s=$3} END{print u","a","s}')
  nvidia-smi --query-gpu=index,memory.used,memory.total,utilization.gpu,utilization.memory,power.draw,clocks.sm,clocks.mem,temperature.gpu --format=csv,noheader,nounits | tr -d ' ' | while read g; do echo "$t,$g,$h"; done >> telemetry.csv
  sleep 5
done
