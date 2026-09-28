#!/bin/bash
# usage: wave.sh <wave> <files-per-shard> <batch-size> <cap-bytes> <parallel> [max-shards]
S=${KIT_DIR:?set KIT_DIR}
W=$1; PER=$2; BS=$3; CAP=$4; PAR=$5; MAXS=${6:-32}; D=$S/wave$W; mkdir -p $D
cd /home/briansrls/.worktrees/gunbc/bright-boar-848
for i in $(seq 0 $((MAXS-1))); do
  a=$(( 1 + i*PER )); b=$(( a + PER - 1 ))
  pop=$(sed -n "${a},${b}p" $S/remaining.txt)
  [ -z "$pop" ] && break
  B64=$(printf '%s\n' "$pop" | base64 -w0)
  ( CTRL_BUILD_RUNNER_EXEC_PROPERTIES=EstimatedMemory=48GB ctrl-build --remote --timeout 60m -- bash -c "CENSUS_BASE=$CENSUS_BASE CENSUS_HEAD=$CENSUS_HEAD HARNESS_B64=$(base64 -w0 $S/refusal_census.dag); $(cat $S/remote_census.sh)" _ $W$i 1 "$B64" $BS $CAP $PAR > $D/s$i.out 2>&1; echo "s$i rc=$?" >> $D/done.txt ) &
  sleep 2
done
wait
grep -h -E "^(main|head) " $D/s*.out > $D/all.txt; wc -l < $D/all.txt
