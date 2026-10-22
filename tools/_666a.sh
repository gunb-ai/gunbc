set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14666 name the two ambiguous literals. Head d3d44cf (gh re-resolved; moved from 13e0374 —
# disclosed). Delta = 2 files 3 lines: GcpPolicy literal name + import DeclarationRef + literal.
# Manager scope: ledger_row_coherence 18/18 under claim_batch at head; gunbc compile clean of the
# TWO named ambiguity errors (they appear at base); the #14607 tail error is separate (#14667).
HEADSHA=d3d44cfed33af0feede633829f7cf574f648ecc7   # gh pr view 14666 --json headRefOid
BASESHA=2538f832391c23c0209ce39c42df00f7492906c4   # git merge-base head origin/main
LRC=dag/test/claim/ledger_row_coherence.dag
echo "HEADSHA=$HEADSHA BASESHA=$BASESHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w666h /tmp/w666b
git fetch -q origin +refs/pull/14666/head:refs/verifier/14666h || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14666h)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14666h) STOP"; exit 9; }
git cat-file -e "$BASESHA^{commit}" || git fetch -q origin $BASESHA
git worktree add -q --detach /tmp/w666h $HEADSHA || { echo WT_H_FAIL; exit 9; }
git worktree add -q --detach /tmp/w666b $BASESHA || { echo WT_B_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
echo "=== build claim_batch at HEAD ==="
( cd /tmp/w666h && CARGO_TARGET_DIR=/tmp/w666h/.t666h cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1 )
CBH=/tmp/w666h/.t666h/release/claim_batch; GBH=/tmp/w666h/.t666h/release/gunbc
[ -x $CBH ] && [ -x $GBH ] || { echo NO_BINS_H; exit 9; }
echo "CBH_SHA=$(sha $CBH|cut -c1-12) GBH_SHA=$(sha $GBH|cut -c1-12)"
echo "=== A. HEAD: 18 claims by name (names from FILE at run time) ==="
cd /tmp/w666h || exit 9
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $LRC | sed 's/^test fn //' | sort | paste -sd,)
N=$(echo $FNS | tr ',' '\n' | wc -l); echo "  names_from_file=$N"
timeout 1500 $CBH --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $LRC --functions "$FNS" > /tmp/c666_head.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c666_head.out) FAIL=$(grep -ac '^FAIL ' /tmp/c666_head.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c666_head.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c666_head.out | sed 's/^/    V| /' | cut -c1-160
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c666_head.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== B. gunbc compile route at HEAD: author's command; grep ambiguity for the two literals ==="
cd /tmp/w666h || exit 9
( CARGO_TARGET_DIR=/tmp/w666h/.t666h timeout 2700 $GBH test //gunbc/instruments:native-process-memory-census > /tmp/c666_gunbc_h.log 2>&1 ); rc=$?
echo "  gunbc-rc=$rc (author: 101, exit 1, 1 PASS 8 SKIPPED — tail error #14607 expected, NOT this PR)"
grep -ac 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log | sed 's/^/  H| ambiguity_total_hits=/;s/^/  /'
grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log | grep -acE "DeclarationRef" | sed 's/^/  H| DeclarationRef_hits(expect 0)=/'
grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log | grep -acE "GcpPolicy" | sed 's/^/  H| GcpPolicy_hits(expect 0)=/'
grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log | head -4 | sed 's/^/    G| /' | cut -c1-190
grep -aciE 'tail|14607' /tmp/c666_gunbc_h.log | sed 's/^/  H| tail_error_lines_seen=/'
tail -6 /tmp/c666_gunbc_h.log | sed 's/^/    T| /' | cut -c1-170
echo "=== C. BASE: gunbc compile shows the TWO named ambiguity errors (other-shape control) ==="
git worktree add -q --detach /tmp/w666b2 $BASESHA >/dev/null 2>&1 || true
cd /tmp/w666b || exit 9
( CARGO_TARGET_DIR=/tmp/w666b/.t666b timeout 2700 cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1 )
GBB=/tmp/w666b/.t666b/release/gunbc; [ -x $GBB ] || { echo NO_GBB; exit 9; }
( timeout 2700 $GBB test //gunbc/instruments:native-process-memory-census > /tmp/c666_gunbc_b.log 2>&1 ); rc=$?
echo "  gunbc-rc=$rc"
grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_b.log | grep -acE "DeclarationRef" | sed 's/^/  B| DeclarationRef_hits(expect>=1)=/'
grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_b.log | grep -acE "GcpPolicy" | sed 's/^/  B| GcpPolicy_hits(expect>=1)=/'
grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_b.log | head -6 | sed 's/^/    G| /' | cut -c1-190
tail -6 /tmp/c666_gunbc_b.log | sed 's/^/    T| /' | cut -c1-170
echo DONE_666_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w666h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w666b >/dev/null 2>&1 || true
git worktree remove --force /tmp/w666b2 >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
