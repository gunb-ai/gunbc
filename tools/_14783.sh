#!/usr/bin/env bash
# #14783 @ 5a92a871b872124273dd0e168628d8b76df4bc1c (base = merge-base 35aaab1d0e559b54a7e386be1b684b1d8b97e555)
# Arms: A baseline head (guarantee 2 named + RED, role 5, approval 6, all by name);
# B baseline base same binary (converted claims PASS on runnable census);
# C unrunnable-census plants (unresolvable import in probe source):
#   guarantee green-source plant: head FAIL (exit 2) vs base PASS  <-- the =>true fail-open demo
#   role canonical-source plant:  head FAIL vs base PASS           <-- ==0 fail-open demo
#   approval canonical plant:     head FAIL vs base PASS
#   approval forged plant:        head FAIL, base also FAIL (value-fail-closed, contrast)
#   guarantee red-source plant:   RED head FAIL (CensusUnobserved != CensusDisagreed)
# D static: probe API unchanged vs merge-base; no new local algebra (grep diff).
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver783 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783
echo 51539607552 > /sys/fs/cgroup/ver783/memory.max
RB=$(cat /sys/fs/cgroup/ver783/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver783/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t783
sha() { sha256sum "$1" | cut -d' ' -f1; }
HEAD=5a92a871b872124273dd0e168628d8b76df4bc1c
BASE=35aaab1d0e559b54a7e386be1b684b1d8b97e555
[ ${#HEAD} -eq 40 ] && [ ${#BASE} -eq 40 ] || { echo "OID LEN STOP"; exit 9; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w783h /tmp/w783b
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783)" = "$HEAD" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f783) want=$HEAD -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w783h $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w783h || exit 9
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo WT_NOT_AT_HEAD; exit 9; }
for f in dag/test/claim/guarantee_probe_corpus_witness.dag dag/test/claim/role_singleton_convergence_witness.dag dag/test/claim/approval_request_client_witness.dag; do
  [ "$(git cat-file blob $HEAD:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
echo "STAGED_OK 3 head blobs"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
CB=/root/workspace/repo-root/.t783/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
GU=dag/test/claim/guarantee_probe_corpus_witness.dag
RO=dag/test/claim/role_singleton_convergence_witness.dag
AP=dag/test/claim/approval_request_client_witness.dag
cp -f $GU /tmp/gu783_orig; cp -f $RO /tmp/ro783_orig; cp -f $AP /tmp/ap783_orig
N_GU="call_duplicate_two_anonymous_positional_params_never_collide,the_zero_duplicate_claim_disagrees_on_a_source_with_a_duplicate"
N_RO="no_caller_supplied_spec_reaches_the_role_singleton_assessment,an_altered_spec_is_refused_at_the_assessor_seal,the_canonical_row_taking_apply_compiles_without_admission_refusal,no_outside_module_can_mint_a_warrant,no_outside_module_can_write_a_warrant_literal"
N_AP="no_outside_module_can_mint_a_dial_token_through_the_mint,no_outside_module_can_write_a_dial_token_literal,no_outside_module_can_call_the_live_origin_admission,no_outside_module_can_call_the_filing_step,the_canonical_filing_and_standing_read_routes_compile"
run3() { # $1 tag; runs 3 entries at CWD tree
  for spec in "GU:$GU:$N_GU" "RO:$RO:$N_RO" "AP:$AP:$N_AP"; do
    IFS=: read lbl entry names <<< "$spec"
    timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $entry --functions "$names" > /tmp/f783_${1}_${lbl}.out 2>&1
    local rc=$?
    echo "  [$1 $lbl] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/f783_${1}_${lbl}.out) FAIL=$(grep -ac '^FAIL ' /tmp/f783_${1}_${lbl}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f783_${1}_${lbl}.out)"
    grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_${1}_${lbl}.out | sed 's/^/    V| /' | cut -c1-230
  done
}
echo "=== A. baseline HEAD: $(date -u +%H:%M:%S)Z"; run3 headA
echo "=== B. baseline BASE (same binary): $(date -u +%H:%M:%S)Z"
cd /tmp/w783h
git worktree add -q --detach /tmp/w783b $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w783b || exit 9
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
for f in $GU $RO $AP; do
  [ "$(git cat-file blob $BASE:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || echo "  BASE_BLOB_MISMATCH $f"
done
run3 baseA
echo "=== C1. plant guarantee green source (unresolvable import) -- HEAD"
cd /tmp/w783h || exit 9
python3 - <<'PY'
p='dag/test/claim/guarantee_probe_corpus_witness.dag'
s=open(p).read()
o='data call_duplicate_green_source: String = "module probe_call_duplicate_green\\n'
n=o+'import probe_nonexistent_census_xyz { anything_here }\\n'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("plant applied green")
PY
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $GU --functions "call_duplicate_two_anonymous_positional_params_never_collide" > /tmp/f783_c1h.out 2>&1; echo "  [C1 head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c1h.out | sed 's/^/    V| /' | cut -c1-260
echo "=== C1b. same plant at BASE (=>true fail-open demo):"
cd /tmp/w783b || exit 9
python3 - <<'PY'
p='dag/test/claim/guarantee_probe_corpus_witness.dag'
s=open(p).read()
o='data call_duplicate_green_source: String = "module probe_call_duplicate_green\\n'
n=o+'import probe_nonexistent_census_xyz { anything_here }\\n'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("plant applied green base")
PY
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $GU --functions "call_duplicate_two_anonymous_positional_params_never_collide" > /tmp/f783_c1b.out 2>&1; echo "  [C1 base] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c1b.out | sed 's/^/    V| /' | cut -c1-260
echo "=== C2. plant role canonical_apply_source: HEAD then BASE"
cd /tmp/w783h || exit 9
python3 - <<'PY'
for base in ('/tmp/w783h','/tmp/w783b'):
    p=base+'/dag/test/claim/role_singleton_convergence_witness.dag'
    s=open(p).read()
    o='data canonical_apply_source: String = "module probe_canonical_apply\\n'
    n=o+'import probe_nonexistent_census_xyz { anything_here }\\n'
    assert s.count(o)==1, f"anchor {s.count(o)} at {base}"
    open(p,'w').write(s.replace(o,n))
print("plant applied canonical (both trees)")
PY
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $RO --functions "the_canonical_row_taking_apply_compiles_without_admission_refusal" > /tmp/f783_c2h.out 2>&1; echo "  [C2 head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c2h.out | sed 's/^/    V| /' | cut -c1-260
cd /tmp/w783b || exit 9
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $RO --functions "the_canonical_row_taking_apply_compiles_without_admission_refusal" > /tmp/f783_c2b.out 2>&1; echo "  [C2 base] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c2b.out | sed 's/^/    V| /' | cut -c1-260
echo "=== C3. plant approval canonical_approval_routes_source: HEAD then BASE"
cd /tmp/w783h || exit 9
python3 - <<'PY'
for base in ('/tmp/w783h','/tmp/w783b'):
    p=base+'/dag/test/claim/approval_request_client_witness.dag'
    s=open(p).read()
    o='data canonical_approval_routes_source: String = "module probe_canonical_approval_routes\\n'
    n=o+'import probe_nonexistent_census_xyz { anything_here }\\n'
    assert s.count(o)==1, f"anchor {s.count(o)} at {base}"
    open(p,'w').write(s.replace(o,n))
print("plant applied approval canonical (both trees)")
PY
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $AP --functions "the_canonical_filing_and_standing_read_routes_compile" > /tmp/f783_c3h.out 2>&1; echo "  [C3 head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c3h.out | sed 's/^/    V| /' | cut -c1-260
cd /tmp/w783b || exit 9
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $AP --functions "the_canonical_filing_and_standing_read_routes_compile" > /tmp/f783_c3b.out 2>&1; echo "  [C3 base] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c3b.out | sed 's/^/    V| /' | cut -c1-260
echo "=== C4. plant approval forged source (>=1 site): head FAIL, base FAIL too (value-closed contrast)"
cd /tmp/w783h || exit 9
python3 - <<'PY'
for base in ('/tmp/w783h','/tmp/w783b'):
    p=base+'/dag/test/claim/approval_request_client_witness.dag'
    s=open(p).read()
    o='data forged_approval_routes_source: String = "module probe_forged_approval_route'
    n='data forged_approval_routes_source: String = "module probe_forged_approval_route\\nimport probe_nonexistent_census_xyz { anything_here }'
    i=s.find(o); assert s.count(o)==1, f"anchor {s.count(o)}"
    open(p,'w').write(s.replace(o,n))
print("plant applied forged (both trees)")
PY
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $AP --functions "no_outside_module_can_mint_a_dial_token_through_the_mint" > /tmp/f783_c4h.out 2>&1; echo "  [C4 head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c4h.out | sed 's/^/    V| /' | cut -c1-260
cd /tmp/w783b || exit 9
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $AP --functions "no_outside_module_can_mint_a_dial_token_through_the_mint" > /tmp/f783_c4b.out 2>&1; echo "  [C4 base] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c4b.out | sed 's/^/    V| /' | cut -c1-260
echo "=== C5. plant guarantee RED source: head RED FAIL (Unobserved != Disagreed)"
cd /tmp/w783h || exit 9
python3 - <<'PY'
p='dag/test/claim/guarantee_probe_corpus_witness.dag'
s=open(p).read()
o='data call_duplicate_red_source: String = "module probe_call_duplicate\\n'
n=o+'import probe_nonexistent_census_xyz { anything_here }\\n'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("plant applied red src")
PY
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $GU --functions "the_zero_duplicate_claim_disagrees_on_a_source_with_a_duplicate" > /tmp/f783_c5h.out 2>&1; echo "  [C5 head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f783_c5h.out | sed 's/^/    V| /' | cut -c1-260
echo "=== D. statics: probe API + no parallel algebra + restore"
[ "$(git cat-file blob $HEAD:dag/gunbc/compile_census_probe.dag | sha256sum | cut -d' ' -f1)" = "$(git cat-file blob $BASE:dag/gunbc/compile_census_probe.dag | sha256sum | cut -d' ' -f1)" ] && echo "  PROBE_API_IDENTICAL head vs base (14578 API, no edits)" || echo "  PROBE_CHANGED -- report"
git diff $BASE $HEAD -- dag/gunbc/compile_census_probe.dag | grep -ac "^+" | sed 's/^/  probe_diff_added_lines=/'
git diff $BASE $HEAD -- $GU $RO $AP | grep -cE "^\+(data|type|fn census)" | sed 's/^/  new_local_algebra_lines=/'
cd /tmp/w783h
cp -f /tmp/gu783_orig $GU; cp -f /tmp/ro783_orig $RO; cp -f /tmp/ap783_orig $AP
O1=$([ "$(sha $GU)" = "$(sha /tmp/gu783_orig)" ] && echo Y || echo N); O2=$([ "$(sha $RO)" = "$(sha /tmp/ro783_orig)" ] && echo Y || echo N); O3=$([ "$(sha $AP)" = "$(sha /tmp/ap783_orig)" ] && echo Y || echo N)
echo "  RESTORE head-tree gu=$O1 ro=$O2 ap=$O3"; git status --porcelain | head -3
echo DONE_14783 $(date -u +%H:%M:%S)Z
