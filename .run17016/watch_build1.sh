#!/usr/bin/env bash
# Blocking watcher: prints when the build stage out file shows completion, with tail. NO kills.
for i in $(seq 1 160); do
  R=$(ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no briansrls@srv1.tailecbe08.ts.net 'F=$HOME/scratch/bold-crane-719/run/logs/build1_861da81430d.out; if [ -f "$F" ]; then tail -c 4000 "$F"; else echo "OUT_MISSING"; fi' 2>/dev/null)
  if printf '%s' "$R" | grep -qE "CARGO_RC=|STAGE1_RC=|VOID_BUILD_NOT_FRESH|BUILD_RC=|LEG_RC="; then
    echo "BUILD_DONE_MARKER at $(date -u +%FT%TZ)"; printf '%s\n' "$R" | tail -25; exit 0
  fi
  sleep 30
done
echo "WATCHER_TIMEOUT_80MIN at $(date -u +%FT%TZ)"
