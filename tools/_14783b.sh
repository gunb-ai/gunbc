#!/usr/bin/env bash
# #14783 D2 (BASE side @ 35aaab1d0e559b54a7e386be1b684b1d8b97e555, head-built binary):
# base baselines (GU minus the PR-added RED name), and the fail-open demos:
# C1b guarantee green-source plant -> claim PASS at base (=>true fail-open);
# C2b role canonical plant -> PASS at base (==0 fail-open);
# C3b approval canonical plant -> PASS at base;
# C4b approval forged plant -> FAIL at base too (value-closed contrast).
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
rm -rf /sys/fs/cgroup/ver783b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783b
echo 51539607552 > /sys/fs/cgroup/ver783b/memory.max
RB=$(cat /sys/fs/cgroup/ver783b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783b memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver783b/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w783hb /tmp/w783b2
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783b)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w783hb $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w783hb || exit 9
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t783b/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z (same source as D1 6ebab24c8db8...)"
git worktree add -q --detach /tmp/w783b2 $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w783b2 || exit 9
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
GU=dag/test/claim/guarantee_probe_corpus_witness.dag
RO=dag/test/claim/role_singleton_convergence_witness.dag
AP=dag/test/claim/approval_request_client_witness.dag
for f in $GU $RO $AP; do [ "$(git cat-file blob $BASE:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_BASE_BLOB $f -- STOP"; exit 9; }; done
cp -f $GU /tmp/gu783b_orig; cp -f $RO /tmp/ro783b_orig; cp -f $AP /tmp/ap783b_orig
C1=a_cursor_turn_on_srv2_binds_the_taken_snapshot_through_the_env_wrapper
run() { # $1 tag $2 entry $3 names $4 timeout
  timeout ${4:-600} $CB --claim-run --source-root dag --source-root src/v2 --entry $2 --functions "$3" > /tmp/b783_${1}.out 2>&1
  local rc=$?
  echo "  [$1] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/b783_${1}.out) FAIL=$(grep -ac '^FAIL ' /tmp/b783_${1}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/b783_${1}.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/b783_${1}.out | sed 's/^/    V| /' | cut -c1-240
}
echo "=== BASE baselines (GU minus PR-added RED): $(date -u +%H:%M:%S)Z"
run baseGU $GU "call_duplicate_two_anonymous_positional_params_never_collide" 900
run baseRO $RO "no_caller_supplied_spec_reaches_the_role_singleton_assessment,an_altered_spec_is_refused_at_the_assessor_seal,the_canonical_row_taking_apply_compiles_without_admission_refusal" 900
run baseAP $AP "no_outside_module_can_mint_a_dial_token_through_the_mint,no_outside_module_can_write_a_dial_token_literal,no_outside_module_can_call_the_live_origin_admission,no_outside_module_can_call_the_filing_step,the_canonical_filing_and_standing_read_routes_compile" 900
plant() { # $1 file $2 anchor-line $3 tag
  python3 - "$1" "$2" <<'PY'
import sys
p,anchor=sys.argv[1],sys.argv[2]
s=open(p).read()
o=anchor
n=anchor+'import probe_nonexistent_census_xyz { anything_here}\\n'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("  plant applied", p)
PY
}
echo "=== C1b guarantee green plant @ base (expect PASS = =>true fail-open): $(date -u +%H:%M:%S)Z"
plant $GU 'data call_duplicate_green_source: String = "module probe_call_duplicate_green\n'
run c1b $GU "call_duplicate_two_anonymous_positional_params_never_collide" 900
cp -f /tmp/gu783b_orig $GU; [ "$(sha $GU)" = "$(sha /tmp/gu783b_orig)" ] && echo "  RESTORE_GU=YES" || echo "  RESTORE_GU=NO"
echo "=== C2b role canonical plant @ base (expect PASS = ==0 fail-open):"
plant $RO 'data canonical_apply_source: String = "module probe_canonical_apply\n'
run c2b $RO "the_canonical_row_taking_apply_compiles_without_admission_refusal" 900
cp -f /tmp/ro783b_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783b_orig)" ] && echo "  RESTORE_RO=YES" || echo "  RESTORE_RO=NO"
echo "=== C3b approval canonical plant @ base (expect PASS = fail-open):"
plant $AP 'data canonical_approval_routes_source: String = "module probe_canonical_approval_routes\n'
run c3b $AP "the_canonical_filing_and_standing_read_routes_compile" 900
cp -f /tmp/ap783b_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783b_orig)" ] && echo "  RESTORE_AP=YES" || echo "  RESTORE_AP=NO"
echo "=== C4b approval forged plant @ base (expect FAIL both: value-closed contrast):"
python3 - <<'PY'
p='dag/test/claim/approval_request_client_witness.dag'
s=open(p).read()
o='data forged_approval_routes_source: String = "module probe_forged_approval_route'
n='data forged_approval_routes_source: String = "module probe_forged_approval_route\\nimport probe_nonexistent_census_xyz { anything_here}'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("  plant applied forged")
PY
run c4b $AP "no_outside_module_can_mint_a_dial_token_through_the_mint" 900
cp -f /tmp/ap783b_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783b_orig)" ] && echo "  RESTORE_AP2=YES" || echo "  RESTORE_AP2=NO"
git status --porcelain | head -3
echo DONE_783B $(date -u +%H:%M:%S)Z
