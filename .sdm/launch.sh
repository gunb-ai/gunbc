#!/usr/bin/env bash
# Detached launcher for one self-contained remote dispatch.
# Usage: launch.sh <sha> <side> <mode> ; returns immediately.
set -u
SHA="$1"; SIDE="$2"; MODE="${3:-measure}"
N="$(date -u +%Y%m%d%H%M%S)-$RANDOM"
LOG="/tmp/gap_measure/${SIDE}_${MODE}_${N}.log"
D="/tmp/gap_measure/dispatch.env"
mkdir -p /tmp/gap_measure
cd /home/briansrls/.worktrees/still-deer-503-14075 || exit 1
setsid nohup timeout 2400 ctrl-build --remote -- bash .sdm/side.sh "$SHA" "$SIDE" "$MODE" "$N" \
  > "$LOG" 2>&1 < /dev/null &
pid=$!
disown
{ echo "LOG=$LOG"; echo "PID=$pid"; echo "NONCE=$N"; echo "SHA=$SHA"; echo "SIDE=$SIDE"; echo "MODE=$MODE"; echo "STARTED=$(date -u +%H:%M:%S)"; } > "$D"
echo "launched pid=$pid nonce=$N log=$LOG"
