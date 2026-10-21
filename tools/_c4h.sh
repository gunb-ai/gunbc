set -u
cd /root/workspace/repo-root 2>/dev/null || true
# C4-H (jolly-boar-495, PR 14696, head f4cc1b8e2aa9, stacked on #14692) — manager + jolly ask.
# Run A: bounded subtree memory.max=16G READ BACK, normal verdicts rc=0 PASS, no gate text.
# Run B: subtree whose memory.max=max (unbounded, read back) with BUILDBUDDY_RUN_ID unset ->
#        rc=2, stderr 'claim_batch: UNOBSERVED cause=HostBudgetUnreadable ...', no panic, no 137.
HEADSHA=f4cc1b8e2aa96934764551dd8ad2c298253371f4   # gh pr view 14696 --json headRefOid
F=dag/test/claim/react_jsx_emit.dag
FN=witness_safe_page_emits_ok
W=/root/workspace/ver696
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w696
git fetch -q origin +refs/pull/14696/head:refs/verifier/14696 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14696)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14696) STOP"; exit 9; }
git worktree add -q --detach /tmp/w696 $HEADSHA || { echo WT_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w696 || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WRONG_COMMIT; exit 9; }
( CARGO_TARGET_DIR=$W cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CB=$W/release/claim_batch; [ -x $CB ] || { echo NO_CB; exit 9; }
CBF=$(sha $CB)
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA256=$CBF"
echo "=== cgroup setup ==="
echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
mkdir -p /sys/fs/cgroup/verA /sys/fs/cgroup/verB
echo 17179869184 > /sys/fs/cgroup/verA/memory.max 2>/dev/null || true
MA=$(cat /sys/fs/cgroup/verA/memory.max 2>/dev/null)
MB=$(cat /sys/fs/cgroup/verB/memory.max 2>/dev/null)
echo "READBACK verA.memory.max=$MA (expect 17179869184)"
echo "READBACK verB.memory.max=$MB (expect max)"
[ "$MA" = "17179869184" ] || echo "!! verA NOT BOUND — Run A invalid"
[ "$MB" = "max" ] || echo "!! verB not unbounded — Run B invalid (got $MB)"
echo "=== RUN A: bounded 16G, normal verdicts ==="
exec 11>>/sys/fs/cgroup/verA/cgroup.procs; echo $BASHPID >&11; exec 11>&-
timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $F --functions "$FN" > /tmp/c696_A.out 2>&1; rcA=$?
echo "  rcA=$rcA PASS=$(grep -ac '^PASS ' /tmp/c696_A.out) FAIL=$(grep -ac '^FAIL ' /tmp/c696_A.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c696_A.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c696_A.out | head -3 | sed 's/^/    V| /' | cut -c1-150
echo "  A_hostbudget_hits=$(grep -ac 'HostBudgetUnreadable' /tmp/c696_A.out) (expect 0)"
echo "  A_panic_hits=$(grep -ac 'panicked' /tmp/c696_A.out) (expect 0)"
echo "=== RUN B: unbounded subtree + BUILDBUDDY_RUN_ID unset -> typed UNOBSERVED rc=2 ==="
exec 12>>/sys/fs/cgroup/verB/cgroup.procs; echo $BASHPID >&12; exec 12>&-
echo "  inside verB: $(cat /proc/self/cgroup)"
env -u BUILDBUDDY_RUN_ID sh -c "exec timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $F --functions '$FN'" > /tmp/c696_B.out 2>&1; rcB=$?
echo "  rcB=$rcB (expect 2; NEVER 137 or panic)"
echo "  B_UNOBSERVED_cause_lines=$(grep -ac 'UNOBSERVED cause=HostBudgetUnreadable' /tmp/c696_B.out)"
grep -a 'UNOBSERVED' /tmp/c696_B.out | head -3 | sed 's/^/    V| /' | cut -c1-200
echo "  B_panic_hits=$(grep -ac 'panicked\|Panic' /tmp/c696_B.out) (expect 0)"
echo "  B_output_policy_refusal_hits=$(grep -ac 'OUTPUT-POLICY REFUSAL' /tmp/c696_B.out)"
echo "  B PASS/FAIL lines=$(grep -acE '^(PASS|FAIL) ' /tmp/c696_B.out)"
echo "  B log head:"; head -6 /tmp/c696_B.out | sed 's/^/    B| /' | cut -c1-190
echo DONE_C4H
cd /root/workspace/repo-root
git worktree remove --force /tmp/w696 >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
