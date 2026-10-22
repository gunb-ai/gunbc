#!/usr/bin/env bash
# #14783 G4: the RO/AP fail-open sites, same proven machinery as G2/G3 (base overlay + escaped
# unterm-string plant inside the canonical fixture source the ==0 claim reads).
# Fail-open shape: at BASE `Absent => 0` makes an unrunnable census SATISFY `== 0` => PASS (defect).
# At HEAD census_exit maps NotRunnable to ExitFailure 2 => FAIL (repair, interim mapping per body).
# Arms (single claim per invocation -- two names in one --functions was G2's head rc=2 bug):
#   ro_b_plant  the_canonical_row_taking_apply_compiles_without_admission_refusal @ base overlay + plant (expect PASS)
#   ro_h_plant  same @ head + plant (expect FAIL)
#   ap_b_plant  the_canonical_filing_and_standing_read_routes_compile @ base overlay + plant (expect PASS)
#   ap_h_plant  same @ head + plant (expect FAIL)
# Plus a >=1 site under the same plant, to show base FAIL / head FAIL-with-refusal (both red, but
# only head says WHY): ro_b_plant_ge an_altered_spec_is_refused_at_the_assessor_seal @ base + plant (expect FAIL).
set -u
REPO=/tmp/c4v/disp2
[ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
HEAD=5a92a871b872124273dd0e168628d8b76df4bc1c
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver783g4 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783g4
echo 51539607552 > /sys/fs/cgroup/ver783g4/memory.max
RB=$(cat /sys/fs/cgroup/ver783g4/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783g4 memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783g4 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783g4)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t783b/release/claim_batch
[ -x $CB ] || { echo NO_BINARY_BUILDING; rm -rf /tmp/w783gb4; git worktree add -q --detach /tmp/w783gb4 $HEAD && cd /tmp/w783gb4 && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2; cd $REPO; }
echo "CB_SHA=$(sha $CB) (expect 6ebab24c8db8...)"
RO=dag/test/claim/role_singleton_convergence_witness.dag
AP=dag/test/claim/approval_request_client_witness.dag
ROCL=the_canonical_row_taking_apply_compiles_without_admission_refusal
APCL=the_canonical_filing_and_standing_read_routes_compile
ROGE=an_altered_spec_is_refused_at_the_assessor_seal
# trees
rm -rf /tmp/w783b4b; git worktree add -q --detach /tmp/w783b4b $HEAD || { echo WTB_FAIL; exit 9; }
tar xzf tools/base783.tgz -C /tmp/w783b4b || exit 9
for f in $RO $AP dag/test/claim/guarantee_probe_corpus_witness.dag; do
  [ "$(sha /tmp/w783b4b/$f)" = "$(sha tools/stage/basetree/$f)" ] || { echo "OVERLAY_MISMATCH $f -- STOP"; exit 9; }
done
echo BASE_OVERLAY_BYTE_IDENTICAL
rm -rf /tmp/w783b4h; git worktree add -q --detach /tmp/w783b4h $HEAD || { echo WTH_FAIL; exit 9; }
plant() { python3 - "$1" "$2" <<'PY'
import sys
p,mod=sys.argv[1],sys.argv[2]
s=open(p).read()
anchor='data %s: String = "module probe_%s\\n' % (mod.replace('_canonical','_canonical'), mod)
# explicit anchors instead
PY
python3 - "$1" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
import re
for anchor in ['data canonical_apply_source: String = "module probe_canonical_apply\\n',
               'data canonical_approval_routes_source: String = "module probe_canonical_approval_routes\\n']:
    if s.count(anchor)==1:
        open(p,'w').write(s.replace(anchor, anchor+'fn broken() -> Int { let s = \\"abc\\n')); print("  plant unterm applied at", anchor[:52]); sys.exit(0)
print("  ANCHOR_NOT_FOUND -- STOP"); sys.exit(1)
PY
}
run() { local tag=$1 tree=$2 entry=$3 fn=$4
  cd $tree || exit 9
  timeout 1400 $CB --claim-run --source-root dag --source-root src/v2 --entry $entry --functions $fn > /tmp/g7834_$tag.out 2>&1
  echo "  [$tag] rc=$? PASS=$(grep -ac '^PASS ' /tmp/g7834_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/g7834_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/g7834_$tag.out) BYTES=$(wc -c < /tmp/g7834_$tag.out) $(date -u +%H:%M:%S)Z"
  grep -aE '^(PASS|FAIL|UNOBSERVED) |census|Census|exit' /tmp/g7834_$tag.out | grep -aiv 'floor-|cost-partition|pass ' | head -5 | sed 's/^/    V| /' | cut -c1-235
  cd $REPO || exit 9
}
echo "=== ro_b_plant (expect PASS = ==0 fail-open defect): $(date -u +%H:%M:%S)Z"
plant /tmp/w783b4b/$RO || exit 9
run ro_b_plant /tmp/w783b4b $RO $ROCL
echo "=== ro_h_plant (expect FAIL = repair): $(date -u +%H:%M:%S)Z"
plant /tmp/w783b4h/$RO || exit 9
run ro_h_plant /tmp/w783b4h $RO $ROCL
echo "=== ap_b_plant (expect PASS = ==0 fail-open defect): $(date -u +%H:%M:%S)Z"
plant /tmp/w783b4b/$AP || exit 9
run ap_b_plant /tmp/w783b4b $AP $APCL
echo "=== ap_h_plant (expect FAIL = repair): $(date -u +%H:%M:%S)Z"
plant /tmp/w783b4h/$AP || exit 9
run ap_h_plant /tmp/w783b4h $AP $APCL
echo DONE_G4
