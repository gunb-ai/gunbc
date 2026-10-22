set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=9d4661bc432010fb78b1636865460e270b54bbaa
WIT=dag/test/claim/compiler_gate_workflow_witness.dag
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt740 $HEADSHA 2>&1 | tail -1
cd /tmp/wt740
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t740
CG=/sys/fs/cgroup/ver740; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t740/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
FNS="the_required_job_checks_out_the_merge_ref,w_RED_the_pull_request_head_ref_is_not_the_checkout_ref,the_live_merge_path_runs_on_the_fleet_and_is_the_required_job,w_RED_a_hosted_job_on_the_merge_path_refuses,w_RED_a_runner_decided_at_dispatch_counts_as_hosted,a_merge_path_disjoint_from_the_hosted_jobs_is_admitted"
echo "=== A. 6 claims by name, head (expect PASS=6) ==="
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c740_head.out 2>&1; echo "[head] rc=$?"; grep -aE 'PASS|FAIL|UNOBS|refused|error' /tmp/c740_head.out | tail -12 | cut -c1-220
echo "=== B. emitted yml matches the expression (regen gate, NO-DRIFT) ==="
grep -n 'ref: ${{ github.sha }}' .github/workflows/witnesses.yml | head -2
timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $GATE --functions "main_wet_one" > /dev/null 2>&1 || true
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t740/release/gunbc
timeout 1500 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/witnesses.yml > /tmp/c740_drift.out 2>&1; echo "[regen witnesses.yml] rc=$?"; tail -6 /tmp/c740_drift.out | cut -c1-220
echo "=== C. MUTATION: restore head-sha checkout ref (expect BOTH new claims FAIL, siblings PASS) ==="
python3 - <<'PY'
p="/tmp/wt740/dag/gunbc/witness/compiler_gate_workflow.dag"
s=open(p).read()
a='data compiler_gate_checkout_ref: Expression = ContextAccess { context: Github, path: ["sha"] }'
assert s.count(a)==1, ("anchor count", s.count(a))
open(p,"w").write(s.replace(a,'data compiler_gate_checkout_ref: Expression = ContextAccess { context: Github, path: ["event", "pull_request", "head", "sha"] }')); print("MUT applied")
PY
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c740_mut.out 2>&1; echo "[mut] rc=$?"; grep -aE 'PASS|FAIL|UNOBS' /tmp/c740_mut.out | tail -12 | cut -c1-220
echo "--- regen gate under mutation (expect DRIFT: emitted yml no longer matches) ---"
timeout 1500 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/witnesses.yml > /tmp/c740_drift_mut.out 2>&1; echo "[regen mut] rc=$?"; tail -6 /tmp/c740_drift_mut.out | cut -c1-220
echo "=== D. restore byte-exact ==="
git checkout -- dag/gunbc/witness/compiler_gate_workflow.dag
git status --porcelain | head -3; echo "DIRTY_ABOVE (empty = byte-exact)"
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c740_post.out 2>&1; echo "[post] rc=$?"; grep -aE 'PASS|FAIL|UNOBS' /tmp/c740_post.out | tail -12 | cut -c1-220
echo DONE_14740_A $(date -u +%H:%M:%SZ)
