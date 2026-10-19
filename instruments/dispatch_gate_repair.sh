#!/usr/bin/env bash
# WHY THIS FILE EXISTS: passing env vars to ctrl-build as separate -- args makes the
# first KEY=VALUE become the -c command string (HEAD_REF is then $0) and the gate script
# never runs -- it exits 0/1 in 13s with zero script output. A quoted wrapper script is
# the only form that survives the arg join. (gate3 ran plainly and worked.)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
nohup ctrl-build --remote -- bash -lc 'bash gate_two_closures_14152.sh' > /tmp/gate_repair_dispatch.log 2>&1 &
echo "dispatched pid=$!"
