#!/usr/bin/env bash
# Start a script on the runner DETACHED so the client's 45-minute cap cannot kill it, then
# poll it with dispatch_poll.sh in later short dispatches. The runner's filesystem persists
# across dispatches (that persistence is also what let another session's dirty .dag edits
# leak into an earlier measurement in this lane), so the output file and the cargo target
# dir both survive.
#   TAG=mytag dispatch_detached.sh <script> <args...>
set -u
TAG="${TAG:-stageA}"
SCRIPT="$1"; shift
OUT="/tmp/detached_${TAG}.out"
RC="/tmp/detached_${TAG}.rc"
ctrl-build --remote -- bash -lc "
  rm -f $OUT $RC
  mkdir -p /tmp
  nohup bash -c 'bash $SCRIPT $*; echo \$? > $RC' > $OUT 2>&1 < /dev/null &
  echo STARTED pid=\$! tag=$TAG out=$OUT
"
