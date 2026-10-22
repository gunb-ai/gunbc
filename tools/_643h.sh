set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14643 HEAD-only re-run at deeb798 (base + mutation arms VERIFIED in dispatch jfc53a21c: base
# rc=1 with the PR's exact 'unresolved import: module v2.std.operation_argv not found' at :55:1;
# mutation rc=1 same named cause; restore cmp-identical). Head arm died rc=137 at my 16G cgroup
# bound (resolve > 16G; runner sets BUILDBUDDY_RUN_ID so ExternallyBounded standing is active).
# This dispatch: 48G bound, reuse built binary.
HEADSHA=deeb798e5512bc0534ef0b7dcbb300bb90c17780
F=dag/test/claim/runner/runner_slot_cap_observe_route_witness.dag
W=/root/workspace/ver643
mkdir -p /sys/fs/cgroup/verif643
echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
grep -qw memory /sys/fs/cgroup/cgroup.subtree_control || echo "CGROUP-NOMEM-WARN"
echo 51539607552 > /sys/fs/cgroup/verif643/memory.max 2>/dev/null || true
[ "$(cat /sys/fs/cgroup/verif643/memory.max 2>/dev/null)" != "max" ] && echo "CGROUP 48G ok ($(cat /sys/fs/cgroup/verif643/memory.max))" || echo "CGROUP_FAIL arms-may-panic"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w643h3
git fetch -q origin +refs/pull/14643/head:refs/verifier/14643c2 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14643c2)" = "$HEADSHA" ] || { echo PR_REF_MOVED STOP; exit 9; }
git worktree add -q --detach /tmp/w643h3 $HEADSHA || { echo WT_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w643h3 || exit 9
CB=$W/release/claim_batch
[ -x $CB ] || ( CARGO_TARGET_DIR=$W cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $F | sed 's/^test fn //' | sort | paste -sd,)
echo "names=$(echo $FNS | tr ',' '\n' | wc -l)"
exec 11>>/sys/fs/cgroup/verif643/cgroup.procs; echo $BASHPID >&11; exec 11>&-
timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $F --functions "$FNS" > /tmp/c643_head2.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c643_head2.out) FAIL=$(grep -ac '^FAIL ' /tmp/c643_head2.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c643_head2.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c643_head2.out | sed 's/^/    V| /' | cut -c1-140
grep -a 'HostBudget' /tmp/c643_head2.out | head -1 | cut -c1-150
grep -a 'peak' /tmp/c643_head2.out | head -2 | cut -c1-150
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c643_head2.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo DONE_643_H2
cd /root/workspace/repo-root
git worktree remove --force /tmp/w643h3 >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
