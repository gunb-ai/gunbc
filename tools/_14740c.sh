set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
WIT=dag/test/claim/compiler_gate_workflow_witness.dag
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
FNS="the_required_job_checks_out_the_merge_ref,w_RED_the_pull_request_head_ref_is_not_the_checkout_ref,the_live_merge_path_runs_on_the_fleet_and_is_the_required_job,w_RED_a_hosted_job_on_the_merge_path_refuses,w_RED_a_runner_decided_at_dispatch_counts_as_hosted,a_merge_path_disjoint_from_the_hosted_jobs_is_admitted"
cd $REPO
git worktree add -f /tmp/wt740c $HEADSHA 2>&1 | tail -1
cd /tmp/wt740c
export CARGO_TARGET_DIR=$REPO/.t740
CG=/sys/fs/cgroup/ver740c; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=$REPO/.t740/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
echo "=== 0. mutation state of the earlier D1 worktree (was it left dirty?) ==="
if [ -d /tmp/wt740 ]; then git -C /tmp/wt740 status --porcelain -- dag/ | head -3; echo "D1_DIRTY_ABOVE (empty=clean, MUT1-restored-or-never-committed)"; grep -c 'path: \["event", "pull_request", "head", "sha"\]' /tmp/wt740/dag/gunbc/witness/compiler_gate_workflow.dag || true; fi
echo "=== 1. restore check + post-mutation claims on the PRISTINE head (expect PASS=6) ==="
git status --porcelain | head -2
timeout 1800 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c740_post.out 2>&1; echo "[post-restore head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c740_post.out | sed 's/^/  | /'
grep -ac '^PASS ' /tmp/c740_post.out | sed 's/^/  PASS_COUNT=/'
echo "=== 2. regen NO-DRIFT via gunbc run, witnesses.yml (fresh output-dir compare) ==="
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=$REPO/.t740/release/gunbc
rm -rf /tmp/regen740; mkdir -p /tmp/regen740
timeout 1800 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/witnesses.yml > /tmp/c740_drift.out 2>&1; echo "[regen] rc=$?"; tail -4 /tmp/c740_drift.out | cut -c1-230 | sed 's/^/  | /'
echo DONE_14740_C $(date -u +%H:%M:%SZ)
