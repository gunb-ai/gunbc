#!/usr/bin/env bash
# Usage: dispatch_rows.sh <sha> <label> <timeout> <row> [row...]
# Positional row names are passed through ctrl-build's arg join; env-prefix args are NOT
# safe there (the first KEY=VALUE becomes bash -lc's command string and the script dies
# instantly with 13s exit). Row timeout is positional too, for the same reason.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
SHA="$1"; shift; LABEL="$1"; shift; ROWTO="$1"; shift
ARGS=""; for a in "$@"; do ARGS="$ARGS $a"; done
nohup ctrl-build --remote -- bash -lc "ROW_TIMEOUT=$ROWTO bash repair_tests_14152.sh $SHA $LABEL$ARGS" > "/tmp/rows_${LABEL}_dispatch.log" 2>&1 &
echo "dispatched label=$LABEL rows=$* pid=$!"
