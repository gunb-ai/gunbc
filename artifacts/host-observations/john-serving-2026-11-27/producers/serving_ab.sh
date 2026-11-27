#!/bin/bash
# john serving A/B (blower in middle): S1 = Strata on the two passive cards only (blower idle); cooldown; S2 = Strata on all three.
# gpu indices are nvidia-smi PCI order: 0 = V100-16 blower @01:00, 1 = P100 @81:00, 2 = V100-32 @c1:00.
P=~/strata-pool
cool(){ for i in $(seq 1 60); do m=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits | sort -n | tail -1); [ "$m" -le 60 ] && break; sleep 10; done; echo "$(date -u +%FT%TZ) cooldown done: hottest ${m}C"; }
stop(){ for p in $(pgrep -f "[s]erve/server.py"); do kill $p; done; sleep 10; }
echo "$(date -u +%FT%TZ) SERVING AB start"
stop; cool
CFG=cfg-john-passive.json ARM=john-ab-S1-passive-only $P/strata3_run.sh; echo "$(date -u +%FT%TZ) S1 dir $(cat ~/llm-logs/strata3-latest)"
stop; cool
CFG=cfg-john3.json ARM=john-ab-S2-all-three $P/strata3_run.sh; echo "$(date -u +%FT%TZ) S2 dir $(cat ~/llm-logs/strata3-latest)"
echo "$(date -u +%FT%TZ) SERVING AB DONE"
