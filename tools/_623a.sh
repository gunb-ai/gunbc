set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14623 HEAD-side dispatch: head claims + breadth census + import-removal mutation.
# Companion: _623b.sh (BASE refuse arms + other-shape control).
HEADSHA=e919b92b3ea5c7d38c3a70bfb49de005a3ba0c23   # gh pr view 14623 --json headRefOid
F=dag/test/claim/react_jsx_emit.dag
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w623h
git fetch -q origin +refs/pull/14623/head:refs/verifier/14623 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14623)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14623) STOP"; exit 9; }
git worktree add -q --detach /tmp/w623h $HEADSHA || { echo WT_H_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
echo "=== build claim_batch at HEAD ==="
( cd /tmp/w623h && CARGO_TARGET_DIR=/tmp/w623h/.t623h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBH=/tmp/w623h/.t623h/release/claim_batch; [ -x $CBH ] || { echo NO_CBH; exit 9; }
echo "CBH_SHA=$(sha $CBH | cut -c1-12)"
FNS_H=$(grep -aoE '^test fn [a-zA-Z0-9_]+' /tmp/w623h/$F | sed 's/^test fn //' | sort | paste -sd,)
echo "FNS_HEAD=$FNS_H count=$(echo $FNS_H | tr ',' '\n' | wc -l)"
run3 () { timeout 1500 $3 --claim-run --source-root "$2" --source-root src/v2 --source-root src/v1 --entry "$1" --functions "$5" > "$4" 2>&1; echo $?; }
cd /tmp/w623h || exit 9
echo "=== A. HEAD: 4 claims by name, expect rc=0 4/4 PASS ==="
rc=$(run3 $F dag $CBH /tmp/c623_head.out "$FNS_H"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c623_head.out) FAIL=$(grep -ac '^FAIL ' /tmp/c623_head.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c623_head.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c623_head.out | sed 's/^/    V| /' | cut -c1-150
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c623_head.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== E. census files at head (PR breadth claim): first claim each, expect no resolve error ==="
for f in markdown_markup_xss_witness markdown_roundtrip markdown_source_emit markup_document_comment_witness html_emit_witness html_roundtrip; do
  p=dag/test/claim/$f.dag
  [ -f "$p" ] || { echo "  $f MISSING (disclose)"; continue; }
  n=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $p | sed 's/^test fn //' | head -1)
  [ -z "$n" ] && { echo "  $f NO_TEST_FN (disclose)"; continue; }
  rc=$(run3 $p dag $CBH /tmp/c623_c_$f.out "$n"); echo "  $f first=$n rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c623_c_$f.out) FAIL=$(grep -ac '^FAIL ' /tmp/c623_c_$f.out) nodecl=$(grep -aci 'no declaration is indexed' /tmp/c623_c_$f.out)"
done
echo "=== D. MUTATION at head copy: delete the gunbhub_serve import line -> must refuse again ==="
cp $F /tmp/c623_f.orig
A=$(grep -c '^import gunbc.gunbhub_serve { gunbhub_browse_page, gunbhub_hostile_page }$' $F)
echo "  ANCHOR=$A (expect 1)"
[ "$A" = "1" ] || { echo "  ANCHOR_BAD skip-mutation"; echo DONE_623_A; exit 0; }
sed -i '/^import gunbc.gunbhub_serve { gunbhub_browse_page, gunbhub_hostile_page }$/d' $F
rc=$(run3 $F dag $CBH /tmp/c623_mut.out "$FNS_H"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c623_mut.out) FAIL=$(grep -ac '^FAIL ' /tmp/c623_mut.out)"
echo "  mut_cause=$(grep -ac "no declaration is indexed at 'gunbc.gunbhub_serve.gunbhub_browse_page'" /tmp/c623_mut.out)"
cp /tmp/c623_f.orig $F
cmp -s $F /tmp/c623_f.orig && echo "  RESTORE_IDENTICAL=YES" || echo "  RESTORE_IDENTICAL=NO"
git status --porcelain -- $F | sed 's/^/    PORCELAIN| /'
echo DONE_623_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w623h >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
