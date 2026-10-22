#!/usr/bin/env bash
# #14783 D4 (BASE @ 35aaab1d0e..., same head-built binary): single-claim arms that D2 lost to
# the deadline: C2b role canonical plant -> PASS (==0 fail-open at base);
# C3b approval canonical plant -> PASS; C4b approval forged plant -> recorded (expect FAIL: value-closed).
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
rm -rf /sys/fs/cgroup/ver783d 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783d
echo 51539607552 > /sys/fs/cgroup/ver783d/memory.max
RB=$(cat /sys/fs/cgroup/ver783d/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783d memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver783d/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w783hb /tmp/w783b4
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783b)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w783hb $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w783hb || exit 9
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t783b/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) (expect 6ebab24c8db8...)"
git worktree add -q --detach /tmp/w783b4 $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w783b4 || exit 9
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
RO=dag/test/claim/role_singleton_convergence_witness.dag
AP=dag/test/claim/approval_request_client_witness.dag
for f in $RO $AP; do [ "$(git cat-file blob $BASE:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_BASE_BLOB $f -- STOP"; exit 9; }; done
cp -f $RO /tmp/ro783d_orig; cp -f $AP /tmp/ap783d_orig
run() { # $1 tag $2 entry $3 names
  timeout 800 $CB --claim-run --source-root dag --source-root src/v2 --entry $2 --functions "$3" > /tmp/d783_${1}.out 2>&1
  local rc=$?
  echo "  [$1] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/d783_${1}.out) FAIL=$(grep -ac '^FAIL ' /tmp/d783_${1}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/d783_${1}.out) bytes=$(wc -c < /tmp/d783_${1}.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/d783_${1}.out | sed 's/^/    V| /' | cut -c1-240
}
plant() { python3 - "$1" "$2" <<'PY'
import sys
p,anchor=sys.argv[1],sys.argv[2]
s=open(p).read()
n=anchor+'import probe_nonexistent_census_xyz { anything_here}\\n'
assert s.count(anchor)==1, f"anchor {s.count(anchor)}"
open(p,'w').write(s.replace(anchor,n)); print("  plant applied", p)
PY
}
echo "=== base runnable baselines for the two plant-subject claims (single runs): $(date -u +%H:%M:%S)Z"
run base_ap_canon $AP "the_canonical_filing_and_standing_read_routes_compile"
run base_ro_canon $RO "the_canonical_row_taking_apply_compiles_without_admission_refusal"
echo "=== C2b role canonical plant @ base (expect PASS == ==0 fail-open): $(date -u +%H:%M:%S)Z"
plant $RO 'data canonical_apply_source: String = "module probe_canonical_apply\n'
run c2b $RO "the_canonical_row_taking_apply_compiles_without_admission_refusal"
cp -f /tmp/ro783d_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783d_orig)" ] && echo "  RESTORE_RO=YES" || echo "  RESTORE_RO=NO"
echo "=== C3b approval canonical plant @ base (expect PASS): $(date -u +%H:%M:%S)Z"
plant $AP 'data canonical_approval_routes_source: String = "module probe_canonical_approval_routes\n'
run c3b $AP "the_canonical_filing_and_standing_read_routes_compile"
cp -f /tmp/ap783d_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783d_orig)" ] && echo "  RESTORE_AP=YES" || echo "  RESTORE_AP=NO"
echo "=== C4b approval forged plant @ base (contrast: expect FAIL on value, census moot): $(date -u +%H:%M:%S)Z"
python3 - <<'PY'
p='dag/test/claim/approval_request_client_witness.dag'
s=open(p).read()
o='data forged_approval_routes_source: String = "module probe_forged_approval_route'
n='data forged_approval_routes_source: String = "module probe_forged_approval_route\\nimport probe_nonexistent_census_xyz { anything_here}'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("  plant applied forged")
PY
run c4b $AP "no_outside_module_can_mint_a_dial_token_through_the_mint"
cp -f /tmp/ap783d_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783d_orig)" ] && echo "  RESTORE_AP2=YES" || echo "  RESTORE_AP2=NO"
git status --porcelain | head -3
echo DONE_783D $(date -u +%H:%M:%S)Z
