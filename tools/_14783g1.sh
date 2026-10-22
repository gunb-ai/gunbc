#!/usr/bin/env bash
# #14783 G1: the =>true fail-open site, honestly. Plant an UNTERMINATED STRING inside the
# fixture source literal (escaped quote, no raw newline -> the WITNESS still parses), at BASE and
# at HEAD, same head-built binary. Site-1 claim: call_duplicate_two_anonymous_positional_params_never_collide
#   BASE: census_any_count_for_class -> CensusNotRunnable maps to none, Absent => true  => PASS (defect)
#   HEAD: census_verdict/census_exit -> CensusUnobserved -> ExitFailure 2 -> harness FAIL (repair)
# A head-PASS would mean the plant was inert (census still Observed) and I report it as such.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
HEAD=5a92a871b872124273dd0e168628d8b76df4bc1c
BASE=35aaab1d0e55d54c3923b954a5b93b700196028e
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver783g 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783g
echo 51539607552 > /sys/fs/cgroup/ver783g/memory.max
RB=$(cat /sys/fs/cgroup/ver783g/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783g memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783g || { echo FETCH_FAIL; exit 9; }
git cat-file -e $BASE^{commit} 2>/dev/null || git fetch -q origin $BASE || git fetch -q --deepen=20 origin refs/pull/14783/head || true
git cat-file -e $BASE^{commit} 2>/dev/null || { echo "BASE_UNREACHABLE -- STOP"; exit 9; }
[ "$(git rev-parse refs/verifier/f783g)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t783b/release/claim_batch
if [ ! -x $CB ]; then
  rm -rf /tmp/w783gb; git worktree add -q --detach /tmp/w783gb $HEAD || { echo WT_FAIL; exit 9; }
  cd /tmp/w783gb || exit 9
  cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
  cd $REPO || exit 9
fi
[ -x $CB ] || { echo "NO_BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB) (expect 6ebab24c8db8... if warm)"
CLAIM=call_duplicate_two_anonymous_positional_params_never_collide
GU=dag/test/claim/guarantee_probe_corpus_witness.dag
plant_unterm() { python3 - "$1" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
anchor='data call_duplicate_green_source: String = "module probe_call_duplicate_green\\n'
assert s.count(anchor)==1, f"anchor {s.count(anchor)}"
open(p,'w').write(s.replace(anchor, anchor+'fn broken() -> Int { let s = \\"abc\\n')); print("  plant unterm applied")
PY
}
run1() { # $1 tag $2 tree
  cd $2 || exit 9
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $GU --functions $CLAIM > /tmp/g783_$1.out 2>&1
  local rc=$?
  echo "  [$1] rc=$rc PASS=$(grep -ac "^PASS $CLAIM\$" /tmp/g783_$1.out) FAIL=$(grep -ac "^FAIL $CLAIM\$" /tmp/g783_$1.out) UNOBS=$(grep -ac "^UNOBSERVED $CLAIM\$" /tmp/g783_$1.out) BYTES=$(wc -c < /tmp/g783_$1.out)"
  grep -aE "^(PASS|FAIL|UNOBSERVED) |cause|CensusUnobserved|exit" /tmp/g783_$1.out | grep -iv "pass " | head -4 | sed 's/^/    V| /' | cut -c1-230
  cd $REPO || exit 9
}
echo "=== BASE + unterm plant (expect PASS = the =>true fail-open defect): $(date -u +%H:%M:%S)Z"
rm -rf /tmp/w783bbase; git worktree add -q --detach /tmp/w783bbase $BASE || { echo WTB_FAIL; exit 9; }
plant_unterm /tmp/w783bbase/$GU || exit 9
run1 b_unterm /tmp/w783bbase
echo "=== HEAD + same unterm plant (expect FAIL = the repair): $(date -u +%H:%M:%S)Z"
rm -rf /tmp/w783bhead; git worktree add -q --detach /tmp/w783bhead $HEAD || { echo WTH_FAIL; exit 9; }
[ "$(git cat-file blob $HEAD:$GU | sha256sum | cut -d' ' -f1)" = "$(sha /tmp/w783bhead/$GU)" ] || { echo "NOT_HEAD_BLOB -- STOP"; exit 9; }
plant_unterm /tmp/w783bhead/$GU || exit 9
run1 h_unterm /tmp/w783bhead
echo "=== HEAD no plant (sanity: claim PASSes on a clean census): $(date -u +%H:%M:%S)Z"
git -C /tmp/w783bhead checkout -q -- $GU
run1 h_clean /tmp/w783bhead
echo DONE_G1
