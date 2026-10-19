#!/usr/bin/env bash
# TAG=mytag dispatch_poll.sh   -- one short dispatch: is it done, and what does it say so far
set -u
TAG="${TAG:-stageA}"
OUT="/tmp/detached_${TAG}.out"; RC="/tmp/detached_${TAG}.rc"
ctrl-build --remote -- bash -lc "
  echo SIZE=\$(wc -c < $OUT 2>/dev/null || echo 0)
  echo RC=\$(cat $RC 2>/dev/null || echo RUNNING)
  pgrep -fa 'cargo test|cargo build' | head -2 | sed 's/^/RUNNING /'
  echo '--- MARKERS ---'
  grep -aE '^T=|^ROW |^SUMMARY|^SEED |^TARGET_CACHE|^NPROC|^MEM |^FATAL|Aborting|^REGEN|^ABORT' $OUT 2>/dev/null | head -50
  echo '--- TAIL ---'
  tail -c 2500 $OUT 2>/dev/null
"
