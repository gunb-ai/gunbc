set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
export CARGO_TARGET_DIR=$REPO/.t740
CG=/sys/fs/cgroup/ver740b3; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=$REPO/.t740/release/gunbc
echo "BUILD_COMMIT=$HEADSHA GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
S=/tmp/synth740; rm -rf $S; mkdir -p $S; cd $S
git init -q .; git remote add origin $REPO
git fetch -q --depth 1 origin $HEADSHA 2>&1 | tail -1
git checkout -q FETCH_HEAD; git checkout -q -b main
git config user.email v@v.io; git config user.name verifier
mkdir -p dag/test/fx740
cat > dag/test/fx740/fx_lib.dag <<'EOF'
data fx_binding: Int = 7
data fx_other: Int = 8
EOF
cat > dag/test/fx740/consumer_a.dag <<'EOF'
import test.fx740.fx_lib { fx_binding }
test fn a_reads_it_holds: Bool = fx_binding == 7
EOF
git add -A; git commit -qm c1_base
BASE=$(git rev-parse HEAD)
git checkout -q -b feature
cat > dag/test/fx740/fx_lib.dag <<'EOF'
data fx_other: Int = 8
EOF
cat > dag/test/fx740/consumer_a.dag <<'EOF'
import test.fx740.fx_lib { fx_other }
test fn a_reads_it_holds: Bool = fx_other == 8
EOF
git add -A; git commit -qm c2_head_removes_binding
HEADC=$(git rev-parse HEAD)
git checkout -q main
cat > dag/test/fx740/consumer_b.dag <<'EOF'
import test.fx740.fx_lib { fx_binding }
test fn b_reads_it_holds: Bool = fx_binding == 7
EOF
git add -A; git commit -qm c3_main_adds_caller
MAIN=$(git rev-parse HEAD)
git merge -q --no-ff -m merge feature; echo "[git-merge] rc=$?"
MERGE=$(git rev-parse HEAD)
git checkout -q -b control $BASE
echo note > dag/test/fx740/CONTROL.md; git add -A; git commit -qm c4_control
CONTROL=$(git rev-parse HEAD)
git checkout -q -B control_merge $MAIN
git merge -q --no-ff -m merge_control control; echo "[git-merge-control] rc=$?"
CTRL=$(git rev-parse HEAD)
echo "BASE=$BASE HEADC=$HEADC MAIN=$MAIN MERGE=$MERGE CONTROL=$CONTROL CTRL=$CTRL"
arm(){ n=$1; sha=$2; ent=$3;
  rm -rf $S/w_$n; git worktree add -f --detach $S/w_$n $sha >/dev/null 2>&1 || { echo "[worktree $n] FAILED"; return; }
  rm -rf $S/o_$n; mkdir -p $S/o_$n
  cd $S/w_$n
  timeout 1200 $GB compile --source-root dag --source-root src/v2 --entry $ent --target dag --output-dir $S/o_$n > $S/comp_$n.out 2>&1; rc=$?
  echo "[compile $n] rc=$rc entry=$ent out_rs=$(find $S/o_$n -name '*.rs' 2>/dev/null | wc -l) $(date -u +%H:%M:%SZ)"
  grep -aE "not found in module|error:|refus" $S/comp_$n.out | head -5 | cut -c1-220 | sed 's/^/  | /'
  cd $S; }
echo "=== expected: head_alone PASS(b: not present); PASS arms rc=0; MERGE+consumer_b rc!=0 with name 'fx_binding' not found in module 'dag.test.fx740.fx_lib' ==="
arm head_a        $HEADC dag/test/fx740/consumer_a.dag
arm merge_b       $MERGE dag/test/fx740/consumer_b.dag
arm merge_a       $MERGE dag/test/fx740/consumer_a.dag
arm main_b        $MAIN  dag/test/fx740/consumer_b.dag
arm ctrl_a        $CTRL  dag/test/fx740/consumer_a.dag
arm ctrl_b        $CTRL  dag/test/fx740/consumer_b.dag
echo DONE_14740_B3 $(date -u +%H:%M:%SZ)
