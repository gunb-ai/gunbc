set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14680 service_spine value-position rewrite. Manager scope: (1) all 7 claims PASS at head;
# (2) culprit pair: PASS at dae01be7434, FAIL at 226bb079ecc (#13832); (3) type-position refusal
# pin goes RED if kind eligibility is reverted (my mutation: drop the symbol_index_lexical_eligible
# filter at the qualified door resolve_declared_path_reading — sole anchor, count verified 1);
# (4) rewritten positives still resolve the named operation. Standing rule: build AT each judged
# head; print build commit + binary sha.
HEADSHA=76f7ad37cbc883bd5eeb54519d6677bbe60b6958   # gh pr view 14680 --json headRefOid
PASS_SHA=dae01be7434
FAIL_SHA=226bb079ecc1ac1bc0883145783442c8808f500a   # #13832
F=src/v2/test/claim/normalize/service_spine.dag
RES=src/v2/compiler/resolve.dag
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w680h /tmp/w680p /tmp/w680f
git fetch -q origin +refs/pull/14680/head:refs/verifier/14680h || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14680h)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14680h) STOP"; exit 9; }
git cat-file -e "$PASS_SHA^{commit}" 2>/dev/null || git fetch -q origin $PASS_SHA || true
git cat-file -e "$FAIL_SHA^{commit}" 2>/dev/null || git fetch -q origin $FAIL_SHA || true
git worktree add -q --detach /tmp/w680h $HEADSHA || { echo WT_H_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w680h || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "=== build claim_batch AT HEAD ==="
( CARGO_TARGET_DIR=/tmp/w680h/.t680 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBH=/tmp/w680h/.t680/release/claim_batch; [ -x $CBH ] || { echo NO_CBH; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CBH_SHA=$(sha $CBH)"
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $F | sed 's/^test fn //' | sort | paste -sd,)
echo "names_from_file count=$(echo $FNS | tr ',' '\n' | wc -l)"
runf () { # cb out fns
  timeout 1500 $1 --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $F --functions "$3" > "$2" 2>&1; echo $?
}
echo "=== A. HEAD: all claims by name, expect 7/7 PASS ==="
rc=$(runf $CBH /tmp/c680_head.out "$FNS"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680_head.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680_head.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c680_head.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c680_head.out | sed 's/^/    V| /' | cut -c1-140
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c680_head.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== B. culprit pair ==="
for R in "$PASS_SHA p" "$FAIL_SHA f"; do set -- $R; S=$1; T=$2
  git worktree add -q --detach /tmp/w680$T $S || { echo "WT_$T fail(try fetch)"; git fetch -q origin $S && git worktree add -q --detach /tmp/w680$T $S; } || { echo "WT_$T SKIP"; continue; }
  cd /tmp/w680$T || exit 9
  echo "  rev=$T $(git rev-parse HEAD)"
  ( CARGO_TARGET_DIR=/tmp/w680$T/.t cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
  CB=/tmp/w680$T/.t/release/claim_batch; [ -x $CB ] || { echo "  NO_CB_$T"; cd /tmp/w680h; continue; }
  echo "  BUILD_COMMIT_$T=$(git rev-parse HEAD) CB_SHA_$T=$(sha $CB)"
  FN2=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $F | sed 's/^test fn //' | sort | paste -sd,)
  rc=$(runf $CB /tmp/c680_$T.out "$FN2"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680_$T.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680_$T.out)"
  grep -aE '^FAIL ' /tmp/c680_$T.out | head -3 | sed 's/^/    V| /' | cut -c1-140
  cd /tmp/w680h
done
echo "=== C. MUTATION: revert kind eligibility at the qualified door ==="
cd /tmp/w680h || exit 9
cp $RES /tmp/c680_r.orig; HB=$(sha $RES)
A=$(grep -c 'match symbol_index_lexical_eligible($' $RES); echo "  ANCHOR=$A (expect 1)"
[ "$A" = "1" ] || { echo "  ANCHOR_BAD skip-mutation"; echo DONE_680_A; exit 0; }
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
cmp -s $RES /tmp/c680_r.orig && { echo "  !! MUTATION_NOOP"; echo DONE_680_A; exit 0; } || echo "  mutated $(sha $RES|cut -c1-12) was ${HB:0:12}"
( CARGO_TARGET_DIR=/tmp/w680h/.t680 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
rc=$(runf $CBH /tmp/c680_mut.out "$FNS"); echo "  MUT rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680_mut.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680_mut.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c680_mut.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c680_mut.out | sed 's/^/    V| /' | cut -c1-140
echo "  EXPECT exactly 1 FAIL: a_service_namespace_does_not_bind_bare_on_the_resolve_route_holds (the type-position refusal pin). Positives (imported_service resolves t.s.F.Run, ss_route_qualified p.s.E.Get) must still PASS."
echo "=== restore ==="
cp /tmp/c680_r.orig $RES
cmp -s $RES /tmp/c680_r.orig && [ "$(sha $RES)" = "$HB" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO"
( CARGO_TARGET_DIR=/tmp/w680h/.t680 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
rc=$(runf $CBH /tmp/c680_post.out "$FNS"); echo "  POST rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680_post.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680_post.out)"
git status --porcelain | grep -v '^??' | sed 's/^/    PORCELAIN| /'
echo DONE_680_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w680h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w680p >/dev/null 2>&1 || true
git worktree remove --force /tmp/w680f >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
