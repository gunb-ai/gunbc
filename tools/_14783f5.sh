#!/usr/bin/env bash
# #14783 D5: base single-claim runs with 1500s budgets (the slow RO pair + approval canonical + forged):
# base_ro_canon (== 0 claim), c2b (role canonical + unknown-import plant), c3b (approval canonical plant),
# c4b (approval forged plant). Same head-built binary (deterministic 6ebab24c8db8...).
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEAD=5a92a871b872124273dd0e168628d8b76df4bc1c
BASE=35aaab1d0e559b54a7e386be1b684b1d8b97e555
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver783f5 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783f5
echo 51539607552 > /sys/fs/cgroup/ver783f5/memory.max
RB=$(cat /sys/fs/cgroup/ver783f5/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783f5 memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w783f5
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783f5 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783f5)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t783b/release/claim_batch
if [ ! -x $CB ]; then
  echo "no binary in target dir; building at head"
  rm -rf /tmp/w783f5build; git worktree add -q --detach /tmp/w783f5build $HEAD || { echo NO_BINARY_NO_BUILD; exit 9; }
  cd /tmp/w783f5build && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
  cd $REPO
fi
[ -x $CB ] || { echo NO_BINARY; exit 9; }
echo "CB_SHA=$(sha $CB)"
git worktree add -q --detach /tmp/w783f5 $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w783f5 || exit 9
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
RO=dag/test/claim/role_singleton_convergence_witness.dag
AP=dag/test/claim/approval_request_client_witness.dag
for f in $RO $AP; do [ "$(git cat-file blob $BASE:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_BASE_BLOB $f -- STOP"; exit 9; }; done
cp -f $RO /tmp/ro783f5_orig; cp -f $AP /tmp/ap783f5_orig
run1() { # $1 tag $2 entry $3 single claim
  timeout 1450 $CB --claim-run --source-root dag --source-root src/v2 --entry $2 --functions "$3" > /tmp/f783f5_${1}.out 2>&1
  local rc=$?
  echo "  [$1] rc=$rc PASS=$(grep -ac "^PASS $3\$" /tmp/f783f5_${1}.out) FAIL=$(grep -ac "^FAIL $3\$" /tmp/f783f5_${1}.out) UNOBS=$(grep -ac "^UNOBSERVED $3\$" /tmp/f783f5_${1}.out) bytes=$(wc -c < /tmp/f783f5_${1}.out)"
  grep -aE "^(PASS|FAIL|UNOBSERVED) $3\$" /tmp/f783f5_${1}.out | sed 's/^/    V| /' | cut -c1-240
}
echo "=== [base_ro_canon] base runnable, 1450s: $(date -u +%H:%M:%S)Z"
run1 base_ro_canon $RO the_canonical_row_taking_apply_compiles_without_admission_refusal
echo "=== [c2b] base + unknown-import plant on canonical source: $(date -u +%H:%M:%S)Z"
python3 - <<'PY'
p='dag/test/claim/role_singleton_convergence_witness.dag'
s=open(p).read()
a='data canonical_apply_source: String = "module probe_canonical_apply\n'
assert s.count(a)==1
open(p,'w').write(s.replace(a, a+'import probe_nonexistent_census_xyz { anything_here}\\n')); print("  plant c2b applied")
PY
run1 c2b $RO the_canonical_row_taking_apply_compiles_without_admission_refusal
cp -f /tmp/ro783f5_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783f5_orig)" ] && echo "  RESTORE_RO=YES" || echo "  RESTORE_RO=NO"
echo "=== [c3b] base + approval canonical plant: $(date -u +%H:%M:%S)Z"
python3 - <<'PY'
p='dag/test/claim/approval_request_client_witness.dag'
s=open(p).read()
a='data canonical_approval_routes_source: String = "module probe_canonical_approval_routes\\n'
assert s.count(a)==1
open(p,'w').write(s.replace(a, a+'import probe_nonexistent_census_xyz { anything_here}\\n')); print("  plant c3b applied")
PY
run1 c3b $AP the_canonical_filing_and_standing_read_routes_compile
cp -f /tmp/ap783f5_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783f5_orig)" ] && echo "  RESTORE_AP=YES" || echo "  RESTORE_AP=NO"
echo "=== [c4b] base + approval forged plant (value-closed contrast): $(date -u +%H:%M:%S)Z"
python3 - <<'PY'
p='dag/test/claim/approval_request_client_witness.dag'
s=open(p).read()
o='data forged_approval_routes_source: String = "module probe_forged_approval_route'
assert s.count(o)==1
open(p,'w').write(s.replace(o, o+'\\nimport probe_nonexistent_census_xyz { anything_here}')); print("  plant c4b applied")
PY
run1 c4b $AP no_outside_module_can_mint_a_dial_token_through_the_mint
cp -f /tmp/ap783f5_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783f5_orig)" ] && echo "  RESTORE_AP2=YES" || echo "  RESTORE_AP2=NO"
git status --porcelain | head -3
echo DONE_783F5 $(date -u +%H:%M:%S)Z
