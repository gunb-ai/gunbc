#!/bin/bash
# john A/B: is the blower card a net thermal contributor or drain for its passive neighbours?
# A = passive cards loaded, blower idle; cooldown to near baseline; B = passive cards + blower loaded. Same DUR, same caps.
# fp16 matmul burn (burnin.py); per-card guard at shutdown-2 C; 1 s nvidia-smi log. Fans are logged separately from srv2 (bmc_fan_poll.sh).
export CUDA_DEVICE_ORDER=PCI_BUS_ID
T=~/thermal; DUR=${DUR:-600}
R=$T/results/ab-$(date -u +%Y%m%dT%H%M%SZ); mkdir -p $R; echo $R > $T/ab-latest; L=$R/run.log
ev(){ echo "$(date -u +%FT%TZ) $*" | tee -a $L; }
BLOWER=$(nvidia-smi --query-gpu=index,name --format=csv,noheader | awk -F', ' '/V100-SXM2-16GB/{print $1}')
PASSIVE=$(nvidia-smi --query-gpu=index,name --format=csv,noheader | awk -F', ' '!/V100-SXM2-16GB/{print $1}' | tr '\n' ' ')
declare -A LIM; while IFS=', ' read i s; do LIM[$i]=$(( s - 2 )); done < <(for i in $(nvidia-smi --query-gpu=index --format=csv,noheader); do echo "$i, $(nvidia-smi -i $i -q -d TEMPERATURE | awk -F: '/Shutdown Temp/{print $2+0; exit}')"; done)
ev "layout: $(nvidia-smi --query-gpu=index,pci.bus_id,name,power.limit --format=csv,noheader | tr '\n' ';') blower=gpu$BLOWER passive=gpu${PASSIVE// / gpu} guards: $(for i in "${!LIM[@]}"; do printf 'gpu%s=%sC ' $i ${LIM[$i]}; done)"
nvidia-smi --query-gpu=timestamp,index,name,temperature.gpu,power.draw,power.limit,clocks.sm,clocks.mem,utilization.gpu,memory.used,clocks_event_reasons.active,clocks_event_reasons.sw_power_cap,clocks_event_reasons.sw_thermal_slowdown,clocks_event_reasons.hw_slowdown,clocks_event_reasons.hw_thermal_slowdown,fan.speed --format=csv,noheader -l 1 > $R/gpu.csv 2>&1 & GS=$!
( while sleep 2; do nvidia-smi --query-gpu=index,temperature.gpu --format=csv,noheader,nounits | while IFS=', ' read i t; do
    if [ "$t" -ge "${LIM[$i]:-85}" ]; then ev "GUARD TRIP: gpu$i at ${t}C (limit ${LIM[$i]}C)"; pkill -f "$T/[b]urnin.py $i "; fi; done; done ) & GD=$!
trap 'kill $GS $GD 2>/dev/null' EXIT
burn(){ PH=$1; shift; ev "PHASE $PH start (cards: $*)"; P=""; for g in "$@"; do $T/venv/bin/python $T/burnin.py $g $DUR fp16 > $R/burn-$PH-g$g.log 2>&1 & P="$P $!"; done; wait $P
  for g in "$@"; do echo "  gpu$g: $(tail -1 $R/burn-$PH-g$g.log)" | tee -a $L; done; ev "PHASE $PH done"; }
cool(){ ev "cooldown"; base=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits | sort -n | tail -1); for i in $(seq 1 90); do m=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits | sort -n | tail -1); [ "$m" -le "$1" ] && break; sleep 10; done; ev "cooldown done: hottest card ${m}C"; }
ev "idle baseline 60 s"; sleep 60; B0=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits | sort -n | tail -1); ev "baseline hottest ${B0}C"
burn A_passive_only $PASSIVE
cool $(( B0 + 5 ))
burn B_passive_plus_blower $PASSIVE $BLOWER
ev "AB DONE $R"
