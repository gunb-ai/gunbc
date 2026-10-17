#!/usr/bin/env bash
# Detached launcher. Launches from the CLEAN harness worktree so BuildBuddy mirrors
# no dirty files (the old 444-file patch upload on every retry is gone).
# Usage: launch.sh <sha> <side> <mode> [harness-ref]
set -u
HERE="$(cd "$(dirname "$0")/.." && pwd)"      # the harness worktree root
SHA="$1"; SIDE="$2"; MODE="${3:-measure}"; HARNESS="${4:-sd503-instrument}"
N="$(date -u +%Y%m%d%H%M%S)-$RANDOM"
LOG="/tmp/gap_measure/${SIDE}_${MODE}_${N}.log"
cd "$HERE" || exit 1
[ -f .sdm/side.sh ] || { echo "no .sdm/side.sh in $HERE"; exit 1; }
setsid nohup timeout 2400 ctrl-build --remote -- bash .sdm/side.sh "$SHA" "$SIDE" "$MODE" "$HARNESS" "$N" \
  > "$LOG" 2>&1 < /dev/null &
pid=$!
disown
{ echo "LOG=$LOG"; echo "PID=$pid"; echo "NONCE=$N"; echo "SHA=$SHA"; echo "SIDE=$SIDE"; echo "MODE=$MODE"; echo "HARNESS=$HARNESS"; echo "STARTED=$(date -u +%H:%M:%S)"; } > /tmp/gap_measure/dispatch.env
echo "launched pid=$pid nonce=$N log=$LOG dirty=$(git status --porcelain | wc -l)"
