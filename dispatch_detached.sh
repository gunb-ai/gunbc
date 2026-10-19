#!/usr/bin/env bash
# Run a script on the runner DETACHED, so the client's 45-minute cap cannot kill it, and
# poll its result file in later short dispatches. The runner's filesystem persists between
# dispatches (/root/workspace/repo-root is shared and outlives a dispatch -- that is how
# another session's dirty .dag edits leaked into a measurement earlier in this lane), so
# the output file survives.
#   dispatch_detached.sh <script> <args...>      -> starts it, prints JOB tag
#   dispatch_poll.sh <tag>                       -> prints progress + tail
set -u
TAG="${TAG:-stageA}"
SCRIPT="$1"; shift
OUT="/tmp/detached_${TAG}.out"
DONE="/tmp/detached_${TAG}.done"
ctrl-build --remote -- bash -lc "rm -f $OUT $DONE; nohup bash $SCRIPT $* > $OUT 2>&1 < /dev/null & echo STARTED pid=\$! out=$OUT"
