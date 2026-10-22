set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=$1
WIT=dag/test/claim/instruments/instrument_dispatch_workflow_witness.dag
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
YML=.github/workflows/instrument_dispatch.yml
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt613j $HEADSHA 2>&1 | tail -1
cd /tmp/wt613j
git rev-parse HEAD | grep -q "^$HEADSHA" || { echo "PR_REF_MOVED/GUARD FAIL"; exit 9; }
export CARGO_TARGET_DIR=$REPO/.t613j
CG=/sys/fs/cgroup/ver613j; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=$REPO/.t613j/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
FNS=$(grep -oE '^test fn [A-Za-z_0-9]+' $WIT | awk '{print $3}' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $FNS | tr ',' '\n' | wc -l)"
timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c613j.out 2>&1; echo "[head] rc=$?"
grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c613j.out | sed 's/^/  | /'
grep -ac '^PASS ' /tmp/c613j.out | sed 's/^/PASS_COUNT=/'; grep -ac '^FAIL ' /tmp/c613j.out | sed 's/^/FAIL_COUNT=/'
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=$REPO/.t613j/release/gunbc
echo "GB_SHA=$(sha256sum $GB|cut -c1-12)"
echo "=== REGEN idempotency/NO-DRIFT for $YML ==="
cp $YML /tmp/committed613j.yml
timeout 1500 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$YML > /tmp/c613j_regen.out 2>&1; echo "[regen] rc=$?"; grep -aE 'Filesystem.Write|refus|error' /tmp/c613j_regen.out | tail -3 | cut -c1-200 | sed 's/^/  | /'
if diff -q /tmp/committed613j.yml $YML >/dev/null 2>&1; then echo "NO-DRIFT=YES"; else echo "DRIFT=YES:"; diff /tmp/committed613j.yml $YML | head -15 | sed 's/^/  D| /'; fi
echo "=== DESIGN.md delta check (local file present; hash for cross-check) ==="
sha256sum DESIGN.md | sed 's/^/DESIGN_SHA=/'
echo DONE_613J $(date -u +%H:%M:%SZ)
