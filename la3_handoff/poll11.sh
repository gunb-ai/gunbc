#!/bin/bash
# look-ahead-3 poller v11: polls origin/seed-map-by-declaration every 120s.
# Fires (exit 0, prints CONVERGED_HEAD=<sha>) only on a head whose SEED and
# MIRROR both carry the fused Map rules (renamed 5b37c7fb17f2:
#   declaration_resolved_container -> map_builtin_spelling
#   unbound_container_diagnostics  -> unimported_map_diagnostics).
# Other heads append to /tmp/la3/poll11.ignored.
cd /home/briansrls/.worktrees/gunbc/jolly-koi-408 || exit 2
PAT='declaration_resolved_container|unbound_container_diagnostics|map_builtin_spelling|unimported_map_diagnostics'
LAST=""
while :; do
  git fetch -q origin seed-map-by-declaration 2>/dev/null
  H=$(git rev-parse FETCH_HEAD 2>/dev/null) || { sleep 120; continue; }
  if [ "$H" != "$LAST" ]; then
    LAST=$H
    S=$(git show $H:src/v1/04_resolve.dag 2>/dev/null | grep -cE "$PAT")
    M=$(git show $H:src/v1/stage0/src/v1_compiler_infer_resolve.rs 2>/dev/null | grep -cE "$PAT")
    if [ "${S:-0}" -ge 1 ] && [ "${M:-0}" -ge 1 ]; then
      echo "CONVERGED_HEAD=$H SEED_HITS=$S MIRROR_HITS=$M"
      exit 0
    fi
    echo "$(date -u +%H:%M:%S) ignoring head $H (seed=$S mirror=$M)" >> /tmp/la3/poll11.ignored
  fi
  sleep 120
done
