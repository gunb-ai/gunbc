#!/usr/bin/env bash
# Usage: dispatch_rows_poll.sh <label> -- nohup'd, because output flushes only at completion
# and a foreground poll of a 40-minute dispatch just holds this session hostage.
set -uo pipefail
L="$1"
for i in $(seq 1 140); do
  if grep -aq "ALLDONE\|Action failed\|Command failed" "/tmp/rows_${L}_dispatch.log" 2>/dev/null; then echo SETTLED; break; fi
  sleep 20
done
echo polled
