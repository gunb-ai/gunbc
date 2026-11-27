#!/bin/bash
# john: after the sm_60/70 build -> start Strata over all three cards (cfg-john3) -> smoke test -> warm150k (TURNS=6, DELTA=2000) with 1 s GPU log.
T=~/strata-test; P=~/strata-pool; L=~/llm-logs
until [ -x $T/strata-main/build-sm60/strata ]; do pgrep -f "strata_build_sm60.sh" >/dev/null || { echo "BUILD NOT RUNNING AND NOT DONE"; tail -20 $T/build-sm60.out; exit 1; }; sleep 20; done
R=$L/results/strata3-$(date -u +%Y%m%dT%H%M%SZ); mkdir -p $R; echo $R > $L/strata3-latest
ev(){ echo "$(date -u +%FT%TZ) $*" | tee -a $R/run.log; }
nvidia-smi --query-gpu=timestamp,index,name,temperature.gpu,power.draw,power.limit,clocks.sm,clocks.mem,utilization.gpu,memory.used,clocks_event_reasons.active,clocks_event_reasons.sw_power_cap,clocks_event_reasons.sw_thermal_slowdown,clocks_event_reasons.hw_slowdown,clocks_event_reasons.hw_thermal_slowdown --format=csv,noheader -l 1 > $R/gpu.csv 2>&1 & G=$!
trap 'kill $G 2>/dev/null' EXIT
ev "power limits: $(nvidia-smi --query-gpu=index,name,power.limit --format=csv,noheader | tr '\n' ';')"
ev "starting strata server (${CFG:-cfg-john3.json})"
if pgrep -f "[s]erve/server.py --engine strata --config $P/${CFG:-cfg-john3.json}" >/dev/null; then ev "server already running: reusing it"; else
  cd $T/strata-main && setsid nohup $T/strata/.venv/bin/python serve/server.py --engine strata --config $P/${CFG:-cfg-john3.json} --host 127.0.0.1 --port 18081 > $R/server.log 2>&1 < /dev/null &
  sleep 5
fi
KEY=$(cat $P/api_key)
for i in $(seq 1 720); do c=$(curl -s -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $KEY" http://127.0.0.1:18081/v1/models); [ "$c" = 200 ] && break; pgrep -f "[s]erve/server.py --engine strata --config $P/${CFG:-cfg-john3.json}" >/dev/null || { ev "SERVER EXITED"; tail -30 $R/server.log; tail -30 $P/engine.log; exit 1; }; sleep 5; done
ev "server ready (/v1/models $c) after $((i*5)) s"; ev "split: $(grep -E "layer split( auto)?:? (K=|explicit|stages|CUDA[0-9] layers)|--layer-split" $P/engine.log | tail -4 | tr "\n" " " | cut -c1-500)"
ans=$(curl -s -H "Authorization: Bearer $KEY" -H 'Content-Type: application/json' http://127.0.0.1:18081/v1/chat/completions -d '{"messages":[{"role":"user","content":"What is 17*23? Answer with just the number."}],"max_tokens":300,"temperature":0}' | python3 -c 'import json,sys;print(json.load(sys.stdin)["choices"][0]["message"].get("content"))')
ev "SMOKE 17*23 -> $ans"
[ "$ans" = "391" ] || { ev "SMOKE FAILED: refusing to benchmark"; exit 1; }
ev "warm150k start"
cd $L && TPW=1.2488 TURNS=6 DELTA=2000 PORTS=18081 python3 warm150k_john.py $P/api_key ${ARM:-john-strata3-v100x2-p100} 1 150000 512 > $R/warm150k.jsonl 2> $R/warm150k.err
ev "warm150k done rc=$?"; ev "STRATA3 DONE"
