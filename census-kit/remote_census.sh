#!/bin/bash
# usage: remote_census.sh <shard> <nshards>
set -euo pipefail
SHARD=$1; N=$2; POPB64=$3; BS=${4:-25}; CAPB=${5:-11811160064}; PMAX=${6:-4}
MAIN=${CENSUS_BASE:?set CENSUS_BASE}
HEAD=${CENSUS_HEAD:?set CENSUS_HEAD}
git fetch -q origin $MAIN
git fetch -q origin $HEAD
[ "$(git rev-parse FETCH_HEAD)" = "$HEAD" ] || { echo "REFUSE head sha unfetched: $(git rev-parse FETCH_HEAD)"; exit 3; }
W=$PWD/../cy$SHARD; rm -rf $W; git worktree prune; mkdir -p $W
git worktree add -q --detach $W/main $MAIN; git worktree add -q --detach $W/head $HEAD
[ "$(git -C $W/main rev-parse HEAD)" = "$MAIN" ] && [ "$(git -C $W/head rev-parse HEAD)" = "$HEAD" ] || { echo REFUSE sha; exit 3; }
( cd $W/head && CARGO_TARGET_DIR=$W/tgt cargo build -q --release -p v1-compiler --bin gunbc )
G=$W/tgt/release/gunbc
echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control || echo "subtree_control write failed"; echo "controllers: $(cat /sys/fs/cgroup/cgroup.subtree_control)"
for s in main head; do mkdir -p $W/$s/.scratch; echo "$HARNESS_B64" | base64 -d > $W/$s/.scratch/refusal_census.dag; done
echo "$POPB64" | base64 -d > $W/pop.txt
split -l $BS -d -a 4 $W/pop.txt $W/batch.
echo "shard $SHARD files $(wc -l < $W/pop.txt) batches $(ls $W/batch.* | wc -l)"
run() { side=$1; b=$2; raw=$b.$side.raw; ( CG=/sys/fs/cgroup/gc_${side}_$(basename $b); mkdir -p $CG && echo $CAPB > $CG/memory.max && echo $BASHPID > $CG/cgroup.procs && cd $W/$side && exec $G run --source-root dag --source-root src/v2 --source-root .scratch --entry .scratch/refusal_census.dag --function refusal_census_for_paths --arg "paths_csv=$(paste -sd, $b)" > $raw 2>&1 ) || true; if grep -qE '(ROW|REFUSAL|FRONTEND|CONS|ABSENT) ' $raw; then grep -E '(ROW|REFUSAL|FRONTEND|CONS|ABSENT) ' $raw | sed -E 's/^error: function .* returned .//' | sed "s/^/$side /"; else echo "$side BATCHFAIL $b"; tail -c 1200 $raw | sed "s/^/$side RAW /"; dmesg 2>/dev/null | grep -i "killed process" | tail -2 | sed "s/^/$side OOM /"; fi; }
export -f run; export W G CAPB
CORES=$(nproc); PAR=$(( CORES < PMAX ? CORES : PMAX )); [ $PAR -lt 1 ] && PAR=1; echo "cores $CORES par $PAR"
echo "=== RESULT shard $SHARD"
ls $W/batch.* | awk '{print "main "$1; print "head "$1}' | xargs -P $PAR -L 1 bash -c 'run $0 $1'
echo "=== END shard $SHARD"
