set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14680 HEAD arm at 45d56bd (post-heal head, keen-owl gave new heads). Runner has cgroup2, root,
# but no bounded cgroup files -> claim_batch panics HostBudgetUnreadable (entry_resolve.rs:1733).
# FIX: create a bounded subtree and run each claim_batch INSIDE it (echo $$ > cgroup.procs).
# Scope: build at head (print commit+sha), 7/7 PASS by name, positives-not-weakened (named
# operation resolution), then the kind-eligibility revert mutation (sole anchor verified 1).
HEADSHA=45d56bd96f2b52f5014c29fb2de4e4acdb8d0c20   # gh pr view 14680 --json headRefOid
F=src/v2/test/claim/normalize/service_spine.dag
RES=src/v2/compiler/resolve.dag
W=/root/workspace/ver680h
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w680h $W
git fetch -q origin +refs/pull/14680/head:refs/verifier/14680h2 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14680h2)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14680h2) STOP"; exit 9; }
mkdir -p /sys/fs/cgroup/verif680
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
echo 51539607552 > /sys/fs/cgroup/verif680/memory.max || { echo CGROUP_FAIL; exit 9; }
RB=$(cat /sys/fs/cgroup/verif680/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verif680 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verif680/cgroup.procs; echo $BASHPID >&11
git worktree add -q --detach /tmp/w680h $HEADSHA || { echo WT_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w680h || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WRONG_COMMIT; exit 9; }
echo "=== build claim_batch AT HEAD ==="
( CARGO_TARGET_DIR=$W cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CB=$W/release/claim_batch; [ -x $CB ] || { echo NO_CB; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $F | sed 's/^test fn //' | sort | paste -sd,)
echo "names_from_file=$(echo $FNS | tr ',' '\n' | wc -l)"
runf () { # out fns
  timeout 1400 $CB --claim-run --source-root dag --source-root src/v2 --entry $F --functions "$2" > "$1" 2>&1; echo $?
}
echo "=== A. HEAD 7/7 by name ==="
rc=$(runf /tmp/c680h_head.out "$FNS"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680h_head.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680h_head.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c680h_head.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c680h_head.out | sed 's/^/    V| /' | cut -c1-140
for n in $(echo $FNS | tr ',' ' '); do
  echo "    $n: PASS=$(grep -ac "^PASS $n\$" /tmp/c680h_head.out) FAIL=$(grep -ac "^FAIL $n\$" /tmp/c680h_head.out)"
done
grep -a 'HostBudgetUnreadable' /tmp/c680h_head.out | head -1 | sed 's/^/    PANIC| /' | cut -c1-140
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c680h_head.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== C. MUTATION: revert kind eligibility at the qualified door ==="
cp $RES /tmp/c680h_r.orig; HB=$(sha $RES)
A=$(grep -c 'match symbol_index_lexical_eligible($' $RES); echo "  ANCHOR=$A (expect 1)"
[ "$A" = "1" ] || { echo "  ANCHOR_BAD skip-mutation"; echo DONE_680_H; exit 0; }
python3 - <<'PY'
p="src/v2/compiler/resolve.dag"
s=open(p).read()
old="""  match symbol_index_lexical_eligible(
    index: ctx.namespace.symbol_index,
    required: ctx.reference_category,
    candidates: symbol_index_absolute_candidates(index: ctx.namespace.symbol_index, qualified_path: path)
  ) {"""
n=s.count(old)
assert n==1, "block anchor count %d -- SKIP LOUDLY" % n
new="""  match symbol_index_absolute_candidates(index: ctx.namespace.symbol_index, qualified_path: path) {"""
open(p,'w').write(s.replace(old,new))
print("KIND_ELIGIBILITY_REVERTED block=1")
PY
cmp -s $RES /tmp/c680h_r.orig && { echo "  !! MUTATION_NOOP"; echo DONE_680_H; exit 0; } || echo "  mutated $(sha $RES|cut -c1-12)"
( CARGO_TARGET_DIR=$W cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
rc=$(runf /tmp/c680h_mut.out "$FNS"); echo "  MUT rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680h_mut.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680h_mut.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c680h_mut.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c680h_mut.out | sed 's/^/    V| /' | cut -c1-140
echo "=== restore ==="
cp /tmp/c680h_r.orig $RES
cmp -s $RES /tmp/c680h_r.orig && [ "$(sha $RES)" = "$HB" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO"
( CARGO_TARGET_DIR=$W cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
rc=$(runf /tmp/c680h_post.out "$FNS"); echo "  POST rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680h_post.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680h_post.out)"
git status --porcelain | grep -v '^??' | sed 's/^/    PORCELAIN| /'
echo DONE_680_H
cd /root/workspace/repo-root
git worktree remove --force /tmp/w680h >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
