set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14643 @ 3d8c73df part 2 (part 1 = jda18b985: A/B arms landed; runner deadline cut M1 control onward).
# Reuse cached .t643i head build; expected CB aaf3df7aed66... Re-assert staged==head at start
# (part 1 died mid-M1 with a mutation on disk; the worktree is re-created fresh).
OID=3d8c73df80775c0f688c4ebc35f66115cef72afd
FH=dag/test/claim/runner/runner_host_cap_bracket_witness.dag
FS=dag/test/claim/runner/runner_slot_cap_observe_route_witness.dag
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver643j 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver643j
echo 51539607552 > /sys/fs/cgroup/ver643j/memory.max
RB=$(cat /sys/fs/cgroup/ver643j/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver643j memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver643j/cgroup.procs; echo $BASHPID >&11
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w643j
git fetch -q origin +refs/pull/14643/head:refs/verifier/14643j || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14643j)" = "$OID" ] || { echo "PR_REF_MOVED want=$OID STOP"; exit 9; }
git worktree add -q --detach /tmp/w643j $OID || { echo WT_FAIL; exit 9; }
cd /tmp/w643j || exit 9
[ "$(git cat-file blob $OID:$FH | sha256sum | cut -d' ' -f1)" = "$(sha $FH)" ] || { echo "STAGED_FH_FAIL"; exit 9; }
[ "$(git cat-file blob $OID:$FS | sha256sum | cut -d' ' -f1)" = "$(sha $FS)" ] || { echo "STAGED_FS_FAIL"; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t643i CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t643i/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "REUSE_BUILD CB_SHA=$(sha $CB) (part1 was aaf3df7aed66...; must match; if source changed vs part1 cargo rebuilt here — noted)"
FNS_H_H=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $FH | sed 's/^test fn //' | sort | paste -sd,)
FNS_H_S=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $FS | sed 's/^test fn //' | sort | paste -sd,)
runw() { echo "-------- [$5] $(date -u +%H:%M:%S)Z"
  timeout 900 "$3" --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry "$1" --functions "$2" > "$4" 2>&1; local rc=$?
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' "$4") FAIL=$(grep -ac '^FAIL ' "$4") UNOBS=$(grep -ac '^UNOBSERVED ' "$4")"
  grep -aE '^(FAIL|UNOBSERVED) ' "$4" | head -5 | sed 's/^/    V| /' | cut -c1-170
  grep -a 'unresolved import' "$4" | head -1 | sed 's/^/    R| /' | cut -c1-170
}
echo "=== M1 (redo full): restore stale import in host_cap ==="
[ "$(grep -c '^import std.operation_argv { InputTextList, OperationInputBinding }$' $FH)" = "1" ] || { echo "ANCHOR1 STOP"; exit 9; }
sed -i '0,/^import std.operation_argv { InputTextList, OperationInputBinding }$/s//import v2.std.operation_argv { InputTextList, OperationInputBinding }/' $FH
[ "$(grep -c 'import v2.std.operation_argv' $FH)" = "1" ] || { echo "MUT1_NOT_APPLIED"; exit 9; }
runw $FH "$FNS_H_H" $CB /tmp/c643j_m1h.out M1-host-mut
runw $FS "$FNS_H_S" $CB /tmp/c643j_m1s.out M1-slot-control
git cat-file blob HEAD:$FH > $FH
[ "$(sha $FH)" = "$(git cat-file blob $OID:$FH | sha256sum | cut -d' ' -f1)" ] && echo "  RESTORE1_OK" || { echo "RESTORE1_FAIL"; exit 9; }
echo "=== M2: restore stale import in slot_cap only ==="
[ "$(grep -c '^import std.operation_argv { InputTextList }$' $FS)" = "1" ] || { echo "ANCHOR2 STOP"; exit 9; }
sed -i '0,/^import std.operation_argv { InputTextList }$/s//import v2.std.operation_argv { InputTextList }/' $FS
[ "$(grep -c 'import v2.std.operation_argv' $FS)" = "1" ] || { echo "MUT2_NOT_APPLIED"; exit 9; }
runw $FS "$FNS_H_S" $CB /tmp/c643j_m2s.out M2-slot-mut
runw $FH "$FNS_H_H" $CB /tmp/c643j_m2h.out M2-host-control
git cat-file blob HEAD:$FS > $FS
[ "$(sha $FS)" = "$(git cat-file blob $OID:$FS | sha256sum | cut -d' ' -f1)" ] && echo "  RESTORE2_OK" || { echo "RESTORE2_FAIL"; exit 9; }
echo "=== Z: both files green after restores (single run each, quick) ==="
echo "(head-green already proven in part 1 A arms with same tree+binary; restores sha-verified)"
git status --porcelain | sed 's/^/  dirty: /'
echo DONE_643_J
cd /root/workspace/repo-root
git worktree remove --force /tmp/w643j >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
