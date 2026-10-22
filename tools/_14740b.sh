set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt740b $HEADSHA 2>&1 | tail -1
cd /tmp/wt740b
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t740
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t740/release/gunbc
CG=/sys/fs/cgroup/ver740b; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
# ---- #14675-shape git fixture: head removes a binding main has a NEW caller of ----
R=/tmp/gf740r; rm -rf $R; mkdir -p $R
git init -q --bare $R/origin.git
git clone -q $R/origin.git $R/seed 2>/dev/null; cd $R/seed
git config user.email v@v.io; git config user.name verifier
mkdir -p fx
cat > fx/fx_lib.dag <<'EOF'
data fx_binding: Int = 7
data fx_other: Int = 8
EOF
cat > fx/consumer_a.dag <<'EOF'
import fx.fx_lib { fx_binding }
test fn a_reads_it_holds: Bool = fx_binding == 7
EOF
git add -A; git commit -qm c1_base
BASE=$(git rev-parse HEAD); git branch -M main
# HEAD branch: removes fx_binding and updates its only consumer (head alone compiles)
git checkout -qb feature
cat > fx/fx_lib.dag <<'EOF'
data fx_other: Int = 8
EOF
cat > fx/consumer_a.dag <<'EOF'
import fx.fx_lib { fx_other }
test fn a_reads_it_holds: Bool = fx_other == 8
EOF
git add -A; git commit -qm c2_head_removes_binding
HEAD_SHA=$(git rev-parse HEAD)
# main: adds a NEW caller of fx_binding
git checkout -q main
cat > fx/consumer_b.dag <<'EOF'
import fx.fx_lib { fx_binding }
test fn b_reads_it_holds: Bool = fx_binding == 7
EOF
git add -A; git commit -qm c3_main_adds_caller
MAIN=$(git rev-parse HEAD)
# merge ref: head merged with main (head's removal + main's new caller)
git merge -q --no-ff -m merge feature >/dev/null 2>&1; rc=$?
echo "[merge] rc=$rc"; [ $rc -eq 0 ] || git status --short
MERGE=$(git rev-parse HEAD)
# control branch: touches nothing main uses, merged with main
git checkout -q -b control $BASE
echo "# control note" > CONTROL.md; git add -A; git commit -qm c4_control
git checkout -q main 2>/dev/null || true
git checkout -q -b control_merge $MAIN
git merge -q --no-ff -m merge_control control >/dev/null 2>&1
CTRL=$(git rev-parse HEAD)
# publish everything by sha
git push -q origin main feature control control_merge
git update-ref refs/pull/1/merge $MERGE
git push -q origin +refs/pull/1/merge:refs/pull/1/merge
git -C $R/origin.git config uploadpack.allowAnySHA1InWant true
echo "BASE=$BASE HEAD=$HEAD_SHA MAIN=$MAIN MERGE=$MERGE CTRL=$CTRL"
# ---- fetch arms (git-level: does the ref resolve at all) ----
arm(){ n=$1; shift; rm -rf $R/w_$n; mkdir -p $R/w_$n; cd $R/w_$n
  git init -q .; git remote add origin $R/origin.git
  git fetch -q --depth 1 origin "$@" > fetch.out 2>&1; rc=$?
  echo "[fetch $n] rc=$rc args=$*"; tail -2 fetch.out | sed 's/^/  | /'; cd $R/seed; }
arm head_alone $HEAD_SHA
arm head_plus_main $HEAD_SHA $MAIN
arm merge_alone $MERGE
arm control_merge $CTRL
# ---- compile arms on the composed trees (the actual #14675 shape) ----
comp(){ n=$1; sha=$2; tree=$3
  rm -rf $R/c_$n; mkdir -p $R/c_$n; cd $R/c_$n
  git init -q .; git remote add origin $R/origin.git
  git fetch -q --depth 1 origin $sha >/dev/null 2>&1; git checkout -q FETCH_HEAD
  timeout 600 $GB compile --source-root . --source-root $REPO/../repo-root/dag --source-root $REPO/../repo-root/src/v2 --entry $tree --target dag --output-dir $R/o_$n > comp.out 2>&1; rc=$?
  echo "[compile $n] rc=$rc entry=$tree $(date -u +%H:%M:%SZ)"
  grep -aE "not found in module|MissingExport|error:" comp.out | head -4 | cut -c1-200 | sed 's/^/  | /'
  cd $R/seed; }
# use the fixture's OWN roots: entries are fixture files; std/ modules come from the head tree via worktree copy
cp -r /tmp/wt740b/dag $R/c_dag 2>/dev/null || true
compall(){ n=$1; sha=$2; ent=$3
  rm -rf $R/c_$n; mkdir -p $R/c_$n
  git -C $R/seed worktree add -f --detach $R/c_$n $sha >/dev/null 2>&1 || { cd $R/seed; git worktree add -f --detach $R/c_$n $sha >/dev/null 2>&1; }
  cd $R/c_$n
  # std library from the head build tree (not part of the fixture)
  ln -sfn /tmp/wt740b/dag/std $R/c_$n/std 2>/dev/null || true
  timeout 600 $GB compile --source-root . --entry $ent --target dag --output-dir $R/o_$n > comp.out 2>&1; rc=$?
  echo "[compile $n] rc=$rc entry=$ent $(date -u +%H:%M:%SZ)"
  grep -aE "not found in module|error:|refus" comp.out | head -5 | cut -c1-200 | sed 's/^/  | /'
  cd $R/seed; }
compall head_alone $HEAD_SHA fx/consumer_a.dag
compall head_plus_main_merge $MERGE fx/consumer_b.dag
compall control_merge $CTRL fx/consumer_a.dag
compall control_merge_b $CTRL fx/consumer_b.dag
echo DONE_14740_B $(date -u +%H:%M:%SZ)
