set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
S=/tmp/synth7405
cd $REPO
export CARGO_TARGET_DIR=$REPO/.t740
CG=/sys/fs/cgroup/ver740e; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
if [ ! -d $S/x/.git ]; then echo "MISSING $S/x (cold instance) — recreating"; git clone -q --no-checkout https://github.com/gunb-ai/gunbc $S/x; cd $S/x; git fetch -q origin $HEADSHA; git checkout -q FETCH_HEAD; else cd $S/x; fi
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=$REPO/.t740/release/gunbc
echo "GB_SHA=$(sha256sum $GB|cut -c1-12)"
MAIN=$(git rev-parse main); MERGE=$(git rev-parse main^2 2>/dev/null || true); MERGE=$(git log --format=%H --grep '^merge$' -1 main); CTRL=$(git log --format=%H --grep '^merge_control$' -1 control_merge)
HEADC=$(git log --format=%H --grep '^c2_head_removes_binding$' -1 feature)
echo "HEADC=$HEADC MAIN=$MAIN MERGE=$MERGE CTRL=$CTRL"
for t in main merge control_merge; do
  case $t in main) s=$MAIN;; merge) s=$MERGE;; control_merge) s=$CTRL;; esac
  echo "-- ls-tree $t ($s):"; git ls-tree -r --name-only $s -- dag/test/fx740/ | sed 's/^/  T| /'
done
armfull(){ n=$1; sha=$2; ent=$3;
  rm -rf $S/w2_$n; git worktree add -f --detach $S/w2_$n $sha >/dev/null 2>&1 || { echo "[worktree $n] FAILED"; return; }
  rm -rf $S/o2_$n; mkdir -p $S/o2_$n
  cd $S/w2_$n
  echo "-- entry file present? $(ls -la $ent 2>&1 | head -1)"
  timeout 900 $GB compile --source-root dag --source-root src/v2 --entry $ent --target dag --output-dir $S/o2_$n > $S/comp2_$n.out 2>&1; rc=$?
  echo "[compile $n] rc=$rc bytes=$(wc -c < $S/comp2_$n.out) $(date -u +%H:%M:%SZ)"; sed 's/^/  O| /' $S/comp2_$n.out | cut -c1-230
  cd $S/x; }
armfull main_b  $MAIN  dag/test/fx740/consumer_b.dag
armfull merge_b $MERGE dag/test/fx740/consumer_b.dag
armfull ctrl_b  $CTRL  dag/test/fx740/consumer_b.dag
echo DONE_14740_B6 $(date -u +%H:%M:%SZ)
