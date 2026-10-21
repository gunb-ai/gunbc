set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14643 @ 3d8c73df (review 84151 head; supersedes deeb798). Fix: stale `import v2.std.operation_argv`
# -> `import std.operation_argv` in BOTH runner_host_cap_bracket_witness.dag (:11) and
# runner_slot_cap_observe_route_witness.dag (:55). Arms:
#  A head: all claims by name from BOTH files -> PASS all, rc 0.
#  B base(201fe0bd = PR base): same names, base-built binary -> refusal naming the unresolved import, zero verdicts.
#  M mutation at head: restore the stale import in EACH file separately -> that file refuses with
#    the named cause while the OTHER file stays all-PASS (wrong-cause control). Restore byte-exact.
OID=$(gh pr view 14643 --repo gunb-ai/gunbc --json headRefOid --jq .headRefOid)
echo "HEADSHA_from_gh=$OID"
BASE=201fe0bd10da30ff0eeabaf2eabfc1b42f5e02f1
FH=dag/test/claim/runner/runner_host_cap_bracket_witness.dag
FS=dag/test/claim/runner/runner_slot_cap_observe_route_witness.dag
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver643i 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver643i
echo 51539607552 > /sys/fs/cgroup/ver643i/memory.max
RB=$(cat /sys/fs/cgroup/ver643i/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver643i memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver643i/cgroup.procs; echo $BASHPID >&11
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w643i_h /tmp/w643i_b
git fetch -q origin +refs/pull/14643/head:refs/verifier/14643i || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14643i)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/14643i) want=$OID STOP"; exit 9; }
git worktree add -q --detach /tmp/w643i_h $OID || { echo WT_H_FAIL; exit 9; }
git worktree add -q --detach /tmp/w643i_b $BASE || { echo WT_B_FAIL; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w643i_h || exit 9
[ "$(git cat-file blob $OID:$FH | sha256sum | cut -d' ' -f1)" = "$(sha $FH)" ] || { echo "H_STAGED_FH"; exit 9; }
[ "$(git cat-file blob $OID:$FS | sha256sum | cut -d' ' -f1)" = "$(sha $FS)" ] || { echo "H_STAGED_FS"; exit 9; }
grep -c 'import v2.std.operation_argv' $FH $FS | sed 's/^/  head_stale_count(both must be 0): /'
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t643i
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CBH=/root/workspace/repo-root/.t643i/release/claim_batch
[ -x $CBH ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_HEAD=$(git rev-parse HEAD) CBH_SHA=$(sha $CBH) $(date -u +%H:%M:%S)Z"
cd /tmp/w643i_b || exit 9
CARGO_TARGET_DIR=/root/workspace/repo-root/.t643ib cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo "BASE_WRONG_COMMIT"; exit 9; }
CBB=/root/workspace/repo-root/.t643ib/release/claim_batch
[ -x $CBB ] || { echo "NO BASE BINARY"; exit 9; }
echo "BUILD_BASE=$(git rev-parse HEAD) CBB_SHA=$(sha $CBB) $(date -u +%H:%M:%S)Z"
cd /tmp/w643i_h || exit 9
FNS_H_H=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $FH | sed 's/^test fn //' | sort | paste -sd,)
FNS_H_S=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $FS | sed 's/^test fn //' | sort | paste -sd,)
echo "name counts: host_cap=$(echo $FNS_H_H | tr ',' '\n' | wc -l) slot_cap=$(echo $FNS_H_S | tr ',' '\n' | wc -l)"
FNS_B_H=$(cd /tmp/w643i_b && grep -aoE '^test fn [a-zA-Z0-9_]+' $FH | sed 's/^test fn //' | sort | paste -sd,)
FNS_B_S=$(cd /tmp/w643i_b && grep -aoE '^test fn [a-zA-Z0-9_]+' $FS | sed 's/^test fn //' | sort | paste -sd,)
[ "$FNS_H_H" = "$FNS_B_H" ] && [ "$FNS_H_S" = "$FNS_B_S" ] && echo "name_lists head==base YES" || echo "name_lists head!=base NO -- report"
runw() { # file fns binary log tag
  echo "-------- [$5] $(date -u +%H:%M:%S)Z"
  timeout 1500 "$3" --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry "$1" --functions "$2" > "$4" 2>&1; local rc=$?
  local P=$(grep -ac '^PASS ' "$4") X=$(grep -ac '^FAIL ' "$4") U=$(grep -ac '^UNOBSERVED ' "$4")
  echo "  rc=$rc PASS=$P FAIL=$X UNOBS=$U"
  [ "$rc" = "0" ] && [ "$P" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
  grep -aE '^(FAIL|UNOBSERVED) ' "$4" | head -6 | sed 's/^/    V| /' | cut -c1-170
  grep -a 'unresolved import' "$4" | head -2 | sed 's/^/    R| /' | cut -c1-170
  return 0
}
echo "=== A. HEAD (head-built binary) both files by name ==="
runw $FH "$FNS_H_H" $CBH /tmp/c643i_hh.out A-host-head
runw $FS "$FNS_H_S" $CBH /tmp/c643i_hs.out A-slot-head
echo "=== B. BASE (base-built binary, stale imports) both files ==="
( cd /tmp/w643i_b && runw() { :; }; timeout 1500 $CBB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $FH --functions "$FNS_B_H" > /tmp/c643i_bh.out 2>&1; rc=$?
  echo "-------- [B-host-base] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c643i_bh.out) FAIL=$(grep -ac '^FAIL ' /tmp/c643i_bh.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c643i_bh.out) $(date -u +%H:%M:%S)Z"
  grep -a 'unresolved import' /tmp/c643i_bh.out | head -2 | sed 's/^/    R| /' | cut -c1-190 )
( cd /tmp/w643i_b && timeout 1500 $CBB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $FS --functions "$FNS_B_S" > /tmp/c643i_bs.out 2>&1; rc=$?
  echo "-------- [B-slot-base] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c643i_bs.out) FAIL=$(grep -ac '^FAIL ' /tmp/c643i_bs.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c643i_bs.out) $(date -u +%H:%M:%S)Z"
  grep -a 'unresolved import' /tmp/c643i_bs.out | head -2 | sed 's/^/    R| /' | cut -c1-190 )
echo "=== M1. MUTATION: restore stale import in host_cap only ==="
A1=$(grep -c '^import std.operation_argv { InputTextList, OperationInputBinding }$' $FH)
[ "$A1" = "1" ] || { echo "ANCHOR1_COUNT=$A1 STOP"; exit 9; }
sed -i '0,/^import std.operation_argv { InputTextList, OperationInputBinding }$/s//import v2.std.operation_argv { InputTextList, OperationInputBinding }/' $FH
[ "$(grep -c 'import v2.std.operation_argv' $FH)" = "1" ] || { echo "MUT1_NOT_APPLIED"; exit 9; }
grep -c 'import v2.std.operation_argv' $FS | xargs echo "  slot stale count (must be 0):"
runw $FH "$FNS_H_H" $CBH /tmp/c643i_m1h.out M1-host-mut
runw $FS "$FNS_H_S" $CBH /tmp/c643i_m1s.out M1-slot-control
git cat-file blob HEAD:$FH > $FH
[ "$(sha $FH)" = "$(git cat-file blob $OID:$FH | sha256sum | cut -d' ' -f1)" ] && echo "  RESTORE1_OK byte-exact" || { echo "RESTORE1_FAIL"; exit 9; }
echo "=== M2. MUTATION: restore stale import in slot_cap only ==="
A2=$(grep -c '^import std.operation_argv { InputTextList }$' $FS)
[ "$A2" = "1" ] || { echo "ANCHOR2_COUNT=$A2 STOP"; exit 9; }
sed -i '0,/^import std.operation_argv { InputTextList }$/s//import v2.std.operation_argv { InputTextList }/' $FS
[ "$(grep -c 'import v2.std.operation_argv' $FS)" = "1" ] || { echo "MUT2_NOT_APPLIED"; exit 9; }
runw $FS "$FNS_H_S" $CBH /tmp/c643i_m2s.out M2-slot-mut
runw $FH "$FNS_H_H" $CBH /tmp/c643i_m2h.out M2-host-control
git cat-file blob HEAD:$FS > $FS
[ "$(sha $FS)" = "$(git cat-file blob $OID:$FS | sha256sum | cut -d' ' -f1)" ] && echo "  RESTORE2_OK byte-exact" || { echo "RESTORE2_FAIL"; exit 9; }
echo "=== Z. FINAL: re-run both files on the head binary after restores ==="
runw $FH "$FNS_H_H" $CBH /tmp/c643i_zh.out Z-host-restored
runw $FS "$FNS_H_S" $CBH /tmp/c643i_zs.out Z-slot-restored
git status --porcelain | sed 's/^/  dirty: /'
echo DONE_643_I
cd /root/workspace/repo-root
git worktree remove --force /tmp/w643i_h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w643i_b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
