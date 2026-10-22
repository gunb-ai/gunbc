set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt740b4 $HEADSHA 2>&1 | tail -1
cd /tmp/wt740b4
export CARGO_TARGET_DIR=$REPO/.t740
CG=/sys/fs/cgroup/ver740b4; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=$REPO/.t740/release/gunbc
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
S=/tmp/synth7404; rm -rf $S; mkdir -p $S
git clone -q --no-checkout $REPO $S/x 2>/dev/null || git clone -q --no-checkout https://github.com/gunb-ai/gunbc $S/x
cd $S/x
git fetch -q origin $HEADSHA 2>&1 | tail -1
git checkout -q FETCH_HEAD 2>/dev/null || git checkout -q $HEADSHA
git checkout -q -b main
git config user.email v@v.io; git config user.name verifier
mkdir -p dag/test/fx740
printf 'module test.fx740.fx_lib\n\ndata fx_binding: Int = 7\ndata fx_other: Int = 8\n' > dag/test/fx740/fx_lib.dag
printf 'module test.fx740.consumer_a\n\nimport test.fx740.fx_lib { fx_binding }\n\ntest fn a_reads_it_holds: Bool = fx_binding == 7\n' > dag/test/fx740/consumer_a.dag
git add -A; git commit -qm c1_base
BASE=$(git rev-parse HEAD)
git checkout -q -b feature
printf 'module test.fx740.fx_lib\n\ndata fx_other: Int = 8\n' > dag/test/fx740/fx_lib.dag
printf 'module test.fx740.consumer_a\n\nimport test.fx740.fx_lib { fx_other }\n\ntest fn a_reads_it_holds: Bool = fx_other == 8\n' > dag/test/fx740/consumer_a.dag
git add -A; git commit -qm c2_head_removes_binding
HEADC=$(git rev-parse HEAD)
git checkout -q main
printf 'module test.fx740.consumer_b\n\nimport test.fx740.fx_lib { fx_binding }\n\ntest fn b_reads_it_holds: Bool = fx_binding == 7\n' > dag/test/fx740/consumer_b.dag
git add -A; git commit -qm c3_main_adds_caller
MAIN=$(git rev-parse HEAD)
git merge -q --no-ff -m merge feature; echo "[git-merge] rc=$?"
MERGE=$(git rev-parse HEAD)
git checkout -q -b control $BASE
echo note > dag/test/fx740/CONTROL.md; git add -A; git commit -qm c4_control
git checkout -q -B control_merge $MAIN
git merge -q --no-ff -m merge_control control; echo "[git-merge-control] rc=$?"
CTRL=$(git rev-parse HEAD)
echo "BASE=$BASE HEADC=$HEADC MAIN=$MAIN MERGE=$MERGE CTRL=$CTRL"
arm(){ n=$1; sha=$2; ent=$3;
  rm -rf $S/w_$n; git worktree add -f --detach $S/w_$n $sha >/dev/null 2>&1 || { echo "[worktree $n] FAILED"; return; }
  rm -rf $S/o_$n; mkdir -p $S/o_$n
  cd $S/w_$n
  timeout 1500 $GB compile --source-root dag --source-root src/v2 --entry $ent --target dag --output-dir $S/o_$n > $S/comp_$n.out 2>&1; rc=$?
  echo "[compile $n] rc=$rc entry=$ent out_rs=$(find $S/o_$n -name '*.rs' 2>/dev/null | wc -l) bytes=$(wc -c < $S/comp_$n.out) $(date -u +%H:%M:%SZ)"
  grep -aE "not found in module|error:|refus|fx_binding|PASS|FAIL" $S/comp_$n.out | head -6 | cut -c1-220 | sed 's/^/  | /'
  cd $S/x; }
echo "=== expected: head_a rc=0 (head compiles alone); main_b rc=0; merge_b rc!=0 naming fx_binding; merge_a rc=0; ctrl_a/ctrl_b rc=0 ==="
arm head_a  $HEADC dag/test/fx740/consumer_a.dag
arm main_b  $MAIN  dag/test/fx740/consumer_b.dag
arm merge_b $MERGE dag/test/fx740/consumer_b.dag
arm merge_a $MERGE dag/test/fx740/consumer_a.dag
arm ctrl_a  $CTRL  dag/test/fx740/consumer_a.dag
arm ctrl_b  $CTRL  dag/test/fx740/consumer_b.dag
echo DONE_14740_B4 $(date -u +%H:%M:%SZ)
