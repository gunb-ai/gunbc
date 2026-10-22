#!/usr/bin/env bash
# #14783 G2: the =>true fail-open site, honestly, via base-overlay (#14793-proven pattern).
# The PR touched ONLY 3 witness files; the census machinery is unchanged head-vs-base, so one
# head-built binary serves both trees. BASE arm = head tree with the 3 base witness files overlaid
# (tools/base783.tgz, byte-identical to $BASE blobs). Plant: unterminated string inside the GU
# green-source fixture literal (escaped; witness parse survives, fixture-source compile panics
# -> fixture_imported_corpus_sources -> CensusNotRunnable).
#  claim: call_duplicate_two_anonymous_positional_params_never_collide
#    BASE+plant => census Absent => true => PASS   (the fail-open defect, same binary)
#    HEAD+plant => census_exit ExitFailure 2 => FAIL (the repair)
#    HEAD clean => PASS (sanity: claim holds on a runnable census)
#    HEAD + plant, author's own control claim the_zero_duplicate_claim_disagrees_on_a_source_with_a_duplicate runs alongside.
set -u
REPO=/tmp/c4v/disp2
[ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
HEAD=5a92a871b872124273dd0e168628d8b76df4bc1c
BASE=35aaab1d0e55d54c3923b954a5b93b700196028e
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver783g2 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783g2
echo 51539607552 > /sys/fs/cgroup/ver783g2/memory.max
RB=$(cat /sys/fs/cgroup/ver783g2/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783g2 memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783g2 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783g2)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t783b/release/claim_batch
if [ ! -x $CB ]; then
  rm -rf /tmp/w783gb; git worktree add -q --detach /tmp/w783gb $HEAD || { echo WT_FAIL; exit 9; }
  cd /tmp/w783gb && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
  cd $REPO || exit 9
fi
[ -x $CB ] || { echo "NO_BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB) (expect 6ebab24c8db8... if warm)"
GU=dag/test/claim/guarantee_probe_corpus_witness.dag
C1=call_duplicate_two_anonymous_positional_params_never_collide
C2=the_zero_duplicate_claim_disagrees_on_a_source_with_a_duplicate
# base overlay tree
rm -rf /tmp/w783bo; git worktree add -q --detach /tmp/w783bo $HEAD || { echo WTB_FAIL; exit 9; }
tar xzf tools/base783.tgz -C /tmp/w783bo || exit 9
for f in dag/test/claim/guarantee_probe_corpus_witness.dag dag/test/claim/approval_request_client_witness.dag dag/test/claim/role_singleton_convergence_witness.dag; do
  [ "$(sha /tmp/w783bo/$f)" = "$(sha tools/stage/basetree/$f)" ] || { echo "OVERLAY_MISMATCH $f -- STOP"; exit 9; }
done
echo BASE_OVERLAY_BYTE_IDENTICAL_TO_BASE_BLOBS
plant_unterm() { python3 - "$1" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
anchor='data call_duplicate_green_source: String = "module probe_call_duplicate_green\\n'
assert s.count(anchor)==1, f"anchor {s.count(anchor)}"
open(p,'w').write(s.replace(anchor, anchor+'fn broken() -> Int { let s = \\"abc\\n')); print("  plant unterm applied")
PY
}
run() { # tag tree fns...
  local tag=$1 tree=$2; shift 2
  cd $tree || exit 9
  timeout 850 $CB --claim-run --source-root dag --source-root src/v2 --entry $GU --functions "$@" > /tmp/g783_$tag.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/g783_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/g783_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/g783_$tag.out) BYTES=$(wc -c < /tmp/g783_$tag.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) |cause|Unobserved|exit ' /tmp/g783_$tag.out | head -6 | sed 's/^/    V| /' | cut -c1-230
  cd $REPO || exit 9
}
echo "=== ARM b_clean: base overlay, NO plant (expect PASS=1): $(date -u +%H:%M:%S)Z"
run b_clean /tmp/w783bo $C1
echo "=== ARM b_plant: base overlay + unterm plant (expect PASS=1 = fail-open defect): $(date -u +%H:%M:%S)Z"
plant_unterm /tmp/w783bo/$GU || exit 9
run b_plant /tmp/w783bo $C1
echo "=== ARM h_plant: head tree + same plant (expect FAIL = repair): $(date -u +%H:%M:%S)Z"
rm -rf /tmp/w783bh; git worktree add -q --detach /tmp/w783bh $HEAD || { echo WTH_FAIL; exit 9; }
[ "$(sha /tmp/w783bh/$GU)" = "$(sha tools/stage/basetree/../basetree/dag/test/claim/guarantee_probe_corpus_witness.dag)" ] && { echo "HEAD_TREE_HAS_BASE_BLOB -- STOP"; exit 9; } || true
plant_unterm /tmp/w783bh/$GU || exit 9
run h_plant /tmp/w783bh $C1 $C2
echo "=== ARM h_clean: head tree, no plant (expect PASS=2): $(date -u +%H:%M:%S)Z"
git -C /tmp/w783bh checkout -q -- $GU
run h_clean /tmp/w783bh $C1 $C2
echo DONE_G2
