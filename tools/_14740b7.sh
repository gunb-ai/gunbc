set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt740g $HEADSHA 2>&1 | tail -1
cd /tmp/wt740g
export CARGO_TARGET_DIR=$REPO/.t740
CG=/sys/fs/cgroup/ver740g; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=$REPO/.t740/release/gunbc
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
S=/tmp/fx740; rm -rf $S; mkdir -p $S/x
git clone -q --no-checkout https://github.com/gunb-ai/gunbc $S/x
cd $S/x
git fetch -q origin $HEADSHA; git checkout -q FETCH_HEAD
git checkout -q -B main
git config user.email v@v.io; git config user.name verifier
mk(){ printf 'module %s\n\n%s\n' "$1" "$2" > "$3"; }
mkdir -p dag/test/fx740
mk test.fx740.fx_lib 'data fx_binding: Int = 7
data fx_other: Int = 8' dag/test/fx740/fx_lib.dag
mk test.fx740.consumer_a 'import test.fx740.fx_lib { fx_binding }

test fn a_reads_it_holds() -> Bool { fx_binding == 7 }' dag/test/fx740/consumer_a.dag
git add -A; git commit -qm c1_base || exit 9
git rev-parse HEAD > /tmp/fx_shas; BASE=$(cat /tmp/fx_shas)
# HEAD branch: removes fx_binding; its own consumer updated (head alone compiles)
git checkout -q -B feature
mk test.fx740.fx_lib 'data fx_other: Int = 8' dag/test/fx740/fx_lib.dag
mk test.fx740.consumer_a 'import test.fx740.fx_lib { fx_other }

test fn a_reads_it_holds() -> Bool { fx_other == 8 }' dag/test/fx740/consumer_a.dag
git add -A; git commit -qm c2_head_removes_binding || exit 9
HEADC=$(git rev-parse HEAD)
# main: NEW caller of fx_binding
git checkout -q -B main $BASE
mk test.fx740.consumer_b 'import test.fx740.fx_lib { fx_binding }

test fn b_reads_it_holds() -> Bool { fx_binding == 7 }' dag/test/fx740/consumer_b.dag
git add -A; git commit -qm c3_main_adds_caller || exit 9
MAIN=$(git rev-parse HEAD)
# ASSERT trees now (the b5 failure mode was silent: entry missing in the checked-out tree)
echo "-- assert trees --"
for pair in "MAIN:$MAIN" "HEADC:$HEADC"; do
  n=${pair%%:*}; s=${pair##*:}
  echo "[$n $s]"; git ls-tree -r --name-only $s -- dag/test/fx740/ | sed 's/^/  T| /'
done
git ls-tree --name-only $MAIN -- dag/test/fx740/consumer_b.dag | grep -q consumer_b || { echo "ASSERT FAIL: consumer_b not in MAIN tree"; exit 8; }
# merge ref: main merges feature (head's removal + main's new caller)
git checkout -q -B mergebranch $MAIN
git merge -q --no-ff -m merge feature || { echo "MERGE CONFLICT"; git status --short; exit 9; }
MERGE=$(git rev-parse HEAD)
git ls-tree --name-only $MERGE -- dag/test/fx740/consumer_b.dag | grep -q consumer_b || { echo "ASSERT FAIL: consumer_b not in MERGE tree"; exit 8; }
# control: nothing main depends on
git checkout -q -B control $BASE
echo note > dag/test/fx740/CONTROL.md; git add -A; git commit -qm c4_control || exit 9
git checkout -q -B control_merge $MAIN
git merge -q --no-ff -m merge_control control || exit 9
CTRL=$(git rev-parse HEAD)
echo "BASE=$BASE HEADC=$HEADC MAIN=$MAIN MERGE=$MERGE CTRL=$CTRL"
arm(){ n=$1; sha=$2; ent=$3; exp=$4;
  rm -rf $S/w_$n; git worktree add -f --detach $S/w_$n $sha >/dev/null 2>&1 || { echo "[compile $n] WORKTREE FAILED"; return; }
  test -f $S/w_$n/$ent || { echo "[compile $n] ENTRY MISSING in $sha tree"; ls $S/w_$n/$ent; return; }
  rm -rf $S/o_$n; mkdir -p $S/o_$n
  cd $S/w_$n
  timeout 900 $GB compile --source-root dag --source-root src/v2 --entry $ent --target dag --output-dir $S/o_$n > $S/comp_$n.out 2>&1; rc=$?
  echo "[compile $n] rc=$rc expect=$exp entry=$ent @ $sha $(date -u +%H:%M:%SZ)"
  grep -aE "not found in module|error:|refus|PASS|FAIL" $S/comp_$n.out | head -5 | cut -c1-220 | sed 's/^/  | /'
  cd $S/x; }
echo "=== expected: head_a 0, main_b 0, merge_b NONZERO naming fx_binding in module test.fx740.fx_lib, merge_a 0, ctrl_a 0, ctrl_b 0 ==="
arm head_a  $HEADC dag/test/fx740/consumer_a.dag 0
arm main_b  $MAIN  dag/test/fx740/consumer_b.dag 0
arm merge_b $MERGE dag/test/fx740/consumer_b.dag nonzero
arm merge_a $MERGE dag/test/fx740/consumer_a.dag 0
arm ctrl_a  $CTRL  dag/test/fx740/consumer_a.dag 0
arm ctrl_b  $CTRL  dag/test/fx740/consumer_b.dag 0
echo DONE_14740_B7 $(date -u +%H:%M:%SZ)
