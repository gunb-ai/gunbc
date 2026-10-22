set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
WIT=dag/test/claim/compiler_gate_workflow_witness.dag
FNS="the_required_job_checks_out_the_merge_ref,w_RED_the_pull_request_head_ref_is_not_the_checkout_ref,the_live_merge_path_runs_on_the_fleet_and_is_the_required_job,w_RED_a_hosted_job_on_the_merge_path_refuses,w_RED_a_runner_decided_at_dispatch_counts_as_hosted,a_merge_path_disjoint_from_the_hosted_jobs_is_admitted"
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt740f $HEADSHA 2>&1 | tail -1
cd /tmp/wt740f
export CARGO_TARGET_DIR=$REPO/.t740
CG=/sys/fs/cgroup/ver740f; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=$REPO/.t740/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
echo "=== MUT: restore head-sha checkout ref (expect: the 2 new merge-ref claims FAIL; siblings PASS) ==="
grep -c 'data compiler_gate_checkout_ref: Expression = ContextAccess { context: Github, path: \["sha"\] }' dag/gunbc/witness/compiler_gate_workflow.dag
python3 - <<'PY'
p="/tmp/wt740f/dag/gunbc/witness/compiler_gate_workflow.dag"
s=open(p).read()
a='data compiler_gate_checkout_ref: Expression = ContextAccess { context: Github, path: ["sha"] }'
assert s.count(a)==1, ("anchor count", s.count(a))
open(p,"w").write(s.replace(a,'data compiler_gate_checkout_ref: Expression = ContextAccess { context: Github, path: ["event", "pull_request", "head", "sha"] }')); print("MUT applied")
PY
timeout 1800 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c740_mut.out 2>&1; echo "[mut] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c740_mut.out | sed 's/^/  | /'
echo "POST-RESTORE-CHECK"
git checkout -- dag/gunbc/witness/compiler_gate_workflow.dag
git status --porcelain -- dag/ | head -2; echo "DIRTY_ABOVE (empty = byte-exact)"
timeout 1800 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c740_post2.out 2>&1; echo "[post] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c740_post2.out | sed 's/^/  | /'
echo DONE_14740_D $(date -u +%H:%M:%SZ)
