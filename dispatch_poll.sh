#!/usr/bin/env bash
set -u
TAG="${TAG:-stageA}"
OUT="/tmp/detached_${TAG}.out"; DONE="/tmp/detached_${TAG}.done"
ctrl-build --remote -- bash -lc "
  echo SIZE=\$(wc -c < $OUT 2>/dev/null || echo 0)
  echo DONE=\$(test -f $DONE && echo yes || echo no)
  pgrep -fa 'cargo test' | head -3 | sed 's/^/RUNNING /'
  echo '--- HEAD ---'; grep -aE '^T=|^ROW |^SUMMARY|^SEED |^TARGET_CACHE|^NPROC|^MEM |FATAL|Aborting' $OUT 2>/dev/null | head -40
  echo '--- TAIL ---'; tail -c 3000 $OUT 2>/dev/null
"
