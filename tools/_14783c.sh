#!/usr/bin/env bash
# #14783 D3 (HEAD side @ 5a92a871b87, head-built binary):
# RO remaining 2 claims (D1 timed out after 3/5 at 1500s; each gets its own 1500s run);
# head plants: C1 green->claim FAIL (rc=2, CensusUnobserved); C2 canonical->FAIL; C3 canonical->FAIL;
# C4 forged->FAIL (contrast; base side shows FAIL too in D2); C5 RED->FAIL.
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
rm -rf /sys/fs/cgroup/ver783c 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783c
echo 51539607552 > /sys/fs/cgroup/ver783c/memory.max
RB=$(cat /sys/fs/cgroup/ver783c/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783c memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver783c/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t783c
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w783hc
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783c || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783c)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w783hc $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w783hc || exit 9
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo WT_NOT_AT_HEAD; exit 9; }
GU=dag/test/claim/guarantee_probe_corpus_witness.dag
RO=dag/test/claim/role_singleton_convergence_witness.dag
AP=dag/test/claim/approval_request_client_witness.dag
for f in $GU $RO $AP; do [ "$(git cat-file blob $HEAD:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }; done
cp -f $GU /tmp/gu783c_orig; cp -f $RO /tmp/ro783c_orig; cp -f $AP /tmp/ap783c_orig
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t783c/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
run() { # $1 tag $2 entry $3 names $4 timeout
  timeout ${4:-600} $CB --claim-run --source-root dag --source-root src/v2 --entry $2 --functions "$3" > /tmp/c783_${1}.out 2>&1
  local rc=$?
  echo "  [$1] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c783_${1}.out) FAIL=$(grep -ac '^FAIL ' /tmp/c783_${1}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c783_${1}.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c783_${1}.out | sed 's/^/    V| /' | cut -c1-260
}
echo "=== RO remaining 2 claims @ head (own 1500s each): $(date -u +%H:%M:%S)Z"
run ro4 $RO "no_outside_module_can_mint_a_warrant" 1500
run ro5 $RO "no_outside_module_can_write_a_warrant_literal" 1500
plant() { python3 - "$1" "$2" <<'PY'
import sys
p,anchor=sys.argv[1],sys.argv[2]
s=open(p).read()
n=anchor+'import probe_nonexistent_census_xyz { anything_here}\\n'
assert s.count(anchor)==1, f"anchor {s.count(anchor)}"
open(p,'w').write(s.replace(anchor,n)); print("  plant applied", p)
PY
}
echo "=== C1 guarantee green plant @ HEAD (expect FAIL rc=2 CensusUnobserved reported as FAIL): $(date -u +%H:%M:%S)Z"
plant $GU 'data call_duplicate_green_source: String = "module probe_call_duplicate_green\n'
run c1h $GU "call_duplicate_two_anonymous_positional_params_never_collide" 900
cp -f /tmp/gu783c_orig $GU; [ "$(sha $GU)" = "$(sha /tmp/gu783c_orig)" ] && echo "  RESTORE_GU=YES" || echo "  RESTORE_GU=NO"
echo "=== C5 guarantee RED-source plant @ HEAD (RED claims CensusUnobserved == 0 -> FAIL):"
plant $GU 'data call_duplicate_red_source: String = "module probe_call_duplicate\n'
run c5h $GU "the_zero_duplicate_claim_disagrees_on_a_source_with_a_duplicate" 900
cp -f /tmp/gu783c_orig $GU; [ "$(sha $GU)" = "$(sha /tmp/gu783c_orig)" ] && echo "  RESTORE_GU2=YES" || echo "  RESTORE_GU2=NO"
echo "=== C2 role canonical plant @ HEAD (expect FAIL; D1 baseline 3/5 PASS at same binary):"
plant $RO 'data canonical_apply_source: String = "module probe_canonical_apply\n'
run c2h $RO "the_canonical_row_taking_apply_compiles_without_admission_refusal" 900
cp -f /tmp/ro783c_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783c_orig)" ] && echo "  RESTORE_RO=YES" || echo "  RESTORE_RO=NO"
echo "=== C3 approval canonical plant @ HEAD (expect FAIL; baseline PASS):"
plant $AP 'data canonical_approval_routes_source: String = "module probe_canonical_approval_routes\n'
run c3h $AP "the_canonical_filing_and_standing_read_routes_compile" 900
cp -f /tmp/ap783c_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783c_orig)" ] && echo "  RESTORE_AP=YES" || echo "  RESTORE_AP=NO"
echo "=== C4 approval forged plant @ HEAD (expect FAIL; contrast with value-closed claims):"
python3 - <<'PY'
p='dag/test/claim/approval_request_client_witness.dag'
s=open(p).read()
o='data forged_approval_routes_source: String = "module probe_forged_approval_route'
n='data forged_approval_routes_source: String = "module probe_forged_approval_route\\nimport probe_nonexistent_census_xyz { anything_here}'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("  plant applied forged")
PY
run c4h $AP "no_outside_module_can_mint_a_dial_token_through_the_mint" 900
cp -f /tmp/ap783c_orig $AP; [ "$(sha $AP)" = "$(sha /tmp/ap783c_orig)" ] && echo "  RESTORE_AP2=YES" || echo "  RESTORE_AP2=NO"
echo "=== statics:"
[ "$(git cat-file blob $HEAD:dag/gunbc/compile_census_probe.dag | sha256sum | cut -d' ' -f1)" = "$(git cat-file blob $BASE:dag/gunbc/compile_census_probe.dag | sha256sum | cut -d' ' -f1)" ] && echo "  PROBE_API_IDENTICAL head vs merge-base (14578 API, no edits)" || echo "  PROBE_CHANGED -- REPORT"
git diff $BASE $HEAD -- dag/gunbc/compile_census_probe.dag | grep -ac "^+" | sed 's/^/  probe_diff_added_lines=/' || true
git diff $BASE $HEAD -- $GU $RO $AP | grep -cE "^\+(data|type|fn census)" | sed 's/^/  new_local_algebra_lines=/' || true
git status --porcelain | head -3
echo DONE_783C $(date -u +%H:%M:%S)Z
