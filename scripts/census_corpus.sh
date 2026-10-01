#!/bin/bash
set -u
MODE=${1:-both}; MAXB=${2:-21474836480}
BASE=8e212f9af5db42cec0f3470d99e9db0cc36b7d99; HEAD_SHA=81f200382bd2afd2a604a3f9507f9aaf3a8bdab9
R=$(pwd); O=/tmp/co; rm -rf $O; mkdir -p $O
echo "MemTotal: $(grep MemTotal /proc/meminfo)"
git fetch -q origin $BASE $HEAD_SHA 2>&1 | tail -1
mkdir -p /sys/fs/cgroup/gb 2>/dev/null; echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null
echo $MAXB > /sys/fs/cgroup/gb/memory.max && echo $$ > /sys/fs/cgroup/gb/cgroup.procs && echo "cgroup memory.max=$(cat /sys/fs/cgroup/gb/memory.max)"
for side in base head; do S=$BASE; [ $side = head ] && S=$HEAD_SHA
  W=/tmp/wt_$side; rm -rf $W; git worktree add -qf --detach $W $S
  echo "$side rev-parse=$(git -C $W rev-parse HEAD) dirty=$(git -C $W status --porcelain | wc -l)"
  (cd $W && CARGO_TARGET_DIR=/tmp/tgt cargo build --release -q -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' | head -3)
  mkdir -p /tmp/bin_$side; cp /tmp/tgt/release/gunbc /tmp/bin_$side/; echo "$side gunbc sha256=$(sha256sum /tmp/bin_$side/gunbc | cut -c1-64)"
done
cd /tmp/wt_head
for side in base head; do
  if [ $MODE != emit ]; then t0=$(date +%s); timeout 5000 /tmp/bin_$side/gunbc compile --source-root dag --source-root src/v2 --target dag --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir $O/wc_$side > $O/wc_$side.txt 2>&1
  echo "$side corpus exit=$? secs=$(( $(date +%s)-t0 )) peak=$(cat /sys/fs/cgroup/gb/memory.peak 2>/dev/null) :: $(grep 'blocking error' $O/wc_$side.txt)"
  grep -E '^error\[' $O/wc_$side.txt | sort -u > $O/e_$side.txt; fi
  if [ $MODE != corpus ]; then
  timeout 2500 /tmp/bin_$side/gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/00_compile.dag --target rust --output-dir $O/em_$side > $O/em_$side.txt 2>&1
  echo "$side emit exit=$? :: $(grep 'blocking error' $O/em_$side.txt)"; fi
done
echo "head unidentified advisory: dag=$(grep -c TextRepresentationUnidentifiedAtBoundary $O/wc_head.txt) self-host=$(grep -ci "unidentified" $O/em_head.txt)"
echo "== refusals on head not on base: $(comm -13 $O/e_base.txt $O/e_head.txt | wc -l)"; comm -13 $O/e_base.txt $O/e_head.txt | cut -c1-240 | head -30
echo "== refusals on base not on head: $(comm -23 $O/e_base.txt $O/e_head.txt | wc -l)"; comm -23 $O/e_base.txt $O/e_head.txt | cut -c1-240 | head -10
echo "== emit diff files: $(diff -rq $O/em_base/src $O/em_head/src | wc -l)"; diff -rq $O/em_base/src $O/em_head/src | head -5
(cd $O/em_head && CARGO_TARGET_DIR=/tmp/tgt_em cargo build --release > $O/emb.txt 2>&1; echo "emitted crate build exit=$? errors=$(grep -cE '^error' $O/emb.txt)")
cd $R; git worktree remove -f /tmp/wt_base; git worktree remove -f /tmp/wt_head; echo DONE
