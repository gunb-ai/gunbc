#!/bin/bash
# john burn test, generic over however many GPUs are present: each card alone (fp16, DUR s), cooldowns, then all together.
# Per-card safety stop at (that card's shutdown temp - 2 C). Power caps are whatever gpu-power-cap.service set. 1 s nvidia-smi log.
export CUDA_DEVICE_ORDER=PCI_BUS_ID
T=~/thermal; DUR=${DUR:-300}; COOL=${COOL:-120}
R=$T/results/run-$(date -u +%Y%m%dT%H%M%SZ); mkdir -p $R; L=$R/run.log
ev(){ echo "$(date -u +%FT%TZ) $*" | tee -a $L; }
nvidia-smi -q -d POWER,TEMPERATURE,CLOCK > $R/limits.txt
N=$(nvidia-smi --query-gpu=index --format=csv,noheader | wc -l)
declare -A LIM; while IFS=', ' read i st; do LIM[$i]=$(( ${st%%.*} - 2 )); done < <(nvidia-smi --query-gpu=index,temperature.gpu.tlimit --format=csv,noheader,nounits 2>/dev/null | grep -v "N/A")
for i in $(seq 0 $((N-1))); do [ -z "${LIM[$i]}" ] && LIM[$i]=$(( $(nvidia-smi -i $i -q -d TEMPERATURE | awk -F: "/Shutdown Temp/{print \$2+0; exit}") - 2 )); done
ev "cards: $(nvidia-smi --query-gpu=index,name,power.limit --format=csv,noheader | tr '\n' ';') guard limits: $(for i in "${!LIM[@]}"; do printf 'gpu%s=%sC ' $i ${LIM[$i]}; done)"
nvidia-smi --query-gpu=timestamp,index,name,temperature.gpu,power.draw,power.limit,clocks.sm,clocks.mem,utilization.gpu,memory.used,clocks_event_reasons.active,clocks_event_reasons.sw_power_cap,clocks_event_reasons.sw_thermal_slowdown,clocks_event_reasons.hw_slowdown,clocks_event_reasons.hw_thermal_slowdown --format=csv,noheader -l 1 > $R/gpu.csv 2>&1 & GS=$!
( while sleep 2; do nvidia-smi --query-gpu=index,temperature.gpu --format=csv,noheader,nounits | while IFS=', ' read i t; do
    if [ "$t" -ge "${LIM[$i]:-85}" ]; then ev "GUARD TRIP: gpu$i at ${t}C (limit ${LIM[$i]}C)"; pkill -f "$T/burnin.py $i "; fi; done; done ) & GD=$!
trap 'kill $GS $GD 2>/dev/null' EXIT
burn(){ PH=$1; shift; ev "PHASE $PH start"; P=""; for g in "$@"; do $T/venv/bin/python $T/burnin.py $g $DUR fp16 > $R/burn-$PH-g$g.log 2>&1 & P="$P $!"; done; wait $P
  for g in "$@"; do echo "  gpu$g: $(grep mismatches $R/burn-$PH-g$g.log | head -1 | sed 's/.*verify //') | $(tail -1 $R/burn-$PH-g$g.log)" | tee -a $L; done; ev "PHASE done"; }
ev "idle baseline"; sleep 20
[ -z "${ONLY_ALL:-}" ] && for g in $(seq 0 $((N-1))); do burn "g$g-alone" $g; sleep $COOL; done
burn "all" $(seq 0 $((N-1))); sleep 60
ev "RUN DONE $R"
