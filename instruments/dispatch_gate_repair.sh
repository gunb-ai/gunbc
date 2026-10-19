#!/usr/bin/env bash
# WHY THIS FILE EXISTS: env vars must live INSIDE the quoted bash -lc string. Passing them as
# separate ctrl-build args word-splits them -- `bash -lc LOGS=/tmp/x HEAD_REF=... bash s.sh`
# makes LOGS=/tmp/x the command string and HEAD_REF=... $0, so the gate exits in 13 seconds
# with zero output. Omitting them entirely is worse: LOGS is a required variable
# (`LOGS=${LOGS:?LOGS required}`) and the run dies with exit 1. Both failures look like
# "the gate says nothing" unless you read the log, which is what happened twice.
# The script lives at the repo ROOT on the instruments line, not under instruments/.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
LOGS="${1:-/tmp/gate_repair}"
SID="${2:-both}"
rm -rf "$LOGS"; mkdir -p "$LOGS"
nohup ctrl-build --remote -- bash -lc "LOGS=$LOGS SIDE_FILTER=$SID HEAD_REF=refs/heads/session/gentle-carp-16 MAIN_REF=refs/heads/main bash gate_two_closures_14152.sh" > "/tmp/gate_$(basename $LOGS)_dispatch.log" 2>&1 &
echo "dispatched LOGS=$LOGS SIDE=$SID pid=$!"
