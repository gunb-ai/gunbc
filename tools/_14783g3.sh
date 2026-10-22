#!/usr/bin/env bash
# #14783 G3: head-side arms with FULL output (G2 head arms died rc=2/186B, cause unknown).
# Same head-built binary 6ebab24c8db8. Arms: h_clean_c1 (C1 alone, expect PASS), h_clean_c2 (C2 alone, expect PASS),
# h_plant_c1 (C1 + unterm plant; expect FAIL = repair). Dumps full claim_batch output for diagnosis.
set -u
REPO=/tmp/c4v/disp2
[ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
HEAD=5a92a871b872124273dd0e168628d8b76df4bc1c
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver783g3 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783g3
echo 51539607552 > /sys/fs/cgroup/ver783g3/memory.max
RB=$(cat /sys/fs/cgroup/ver783g3/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783g3 memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783g3 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783g3)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t783b/release/claim_batch
[ -x $CB ] || { echo NO_BINARY_BUILDING; rm -rf /tmp/w783gb3; git worktree add -q --detach /tmp/w783gb3 $HEAD && cd /tmp/w783gb3 && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2; cd $REPO; }
echo "CB_SHA=$(sha $CB) (expect 6ebab24c8db8...)"
GU=dag/test/claim/guarantee_probe_corpus_witness.dag
C1=call_duplicate_two_anonymous_positional_params_never_collide
C2=the_zero_duplicate_claim_disagrees_on_a_source_with_a_duplicate
rm -rf /tmp/w783h3; git worktree add -q --detach /tmp/w783h3 $HEAD || { echo WT_FAIL; exit 9; }
run() { local tag=$1; shift
  cd /tmp/w783h3 || exit 9
  timeout 850 $CB --claim-run --source-root dag --source-root src/v2 --entry $GU --functions "$@" > /tmp/g7833_$tag.out 2>&1
  echo "  [$tag] rc=$? PASS=$(grep -ac '^PASS ' /tmp/g7833_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/g7833_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/g7833_$tag.out) BYTES=$(wc -c < /tmp/g7833_$tag.out)"
  echo "    ---- full output ----"; head -25 /tmp/g7833_$tag.out | sed 's/^/    O| /' | cut -c1-230
  cd $REPO || exit 9
}
echo "=== h_clean_c1: $(date -u +%H:%M:%S)Z"; run h_clean_c1 $C1
echo "=== h_clean_c2: $(date -u +%H:%M:%S)Z"; run h_clean_c2 $C2
echo "=== h_plant_c1: $(date -u +%H:%M:%S)Z"
python3 - /tmp/w783h3/$GU <<'PY' || exit 9
import sys
p=sys.argv[1]; s=open(p).read()
anchor='data call_duplicate_green_source: String = "module probe_call_duplicate_green\\n'
assert s.count(anchor)==1, f"anchor {s.count(anchor)}"
open(p,'w').write(s.replace(anchor, anchor+'fn broken() -> Int { let s = \\"abc\\n')); print("  plant unterm applied")
PY
run h_plant_c1 $C1
echo "=== h_plant_c2 (author red-side claim under the same plant): $(date -u +%H:%M:%S)Z"
run h_plant_c2 $C2
echo DONE_G3
