set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14629 roadmap_hierarchy_view dead Document arm. Claim: at base the file refuses to COMPILE
# (variant 'Document' not found in type 'Fragment'); at head all test fns PASS. Statics (local):
# delta base..head = 1 line deletion (the Document arm of hv_nav_hrefs); file has 12 test fns at head.
HEADSHA=97b8bb1d35e46c08b344910b8b5b3390cedea899   # gh pr view 14629 --json headRefOid
BASESHA=647d770c78c3489d5430dfeb3f3399e3d9cd9312   # git merge-base head origin/main (fetched)
F=dag/test/claim/roadmap/roadmap_hierarchy_view_witness.dag
echo "HEADSHA=$HEADSHA BASESHA=$BASESHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w629h /tmp/w629b
git fetch -q origin +refs/pull/14629/head:refs/verifier/14629 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14629)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14629) STOP"; exit 9; }
git cat-file -e "$BASESHA^{commit}" || git fetch -q origin $BASESHA
git worktree add -q --detach /tmp/w629h $HEADSHA || { echo WT_H_FAIL; exit 9; }
git worktree add -q --detach /tmp/w629b $BASESHA || { echo WT_B_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
echo "=== build claim_batch at HEAD ==="
( cd /tmp/w629h && CARGO_TARGET_DIR=/tmp/w629h/.t629h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBH=/tmp/w629h/.t629h/release/claim_batch; [ -x $CBH ] || { echo NO_CBH; exit 9; }
echo "CBH_SHA=$(sha $CBH | cut -c1-12)"
echo "=== build claim_batch at BASE ==="
( cd /tmp/w629b && CARGO_TARGET_DIR=/tmp/w629b/.t629b cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBB=/tmp/w629b/.t629b/release/claim_batch; [ -x $CBB ] || { echo NO_CBB; exit 9; }
echo "CBB_SHA=$(sha $CBB | cut -c1-12)"
FNS_H=$(grep -aoE '^test fn [a-zA-Z0-9_]+' /tmp/w629h/$F | sed 's/^test fn //' | sort | paste -sd,)
echo "FNS_HEAD=$FNS_H count=$(echo $FNS_H | tr ',' '\n' | wc -l)"
FNS_B=$(grep -aoE '^test fn [a-zA-Z0-9_]+' /tmp/w629b/$F | sed 's/^test fn //' | sort | paste -sd,)
echo "FNS_BASE_count=$(echo $FNS_B | tr ',' '\n' | wc -l) identical=$([ "$FNS_H" = "$FNS_B" ] && echo YES || echo NO)"
runw () { # entry cb out fns wetflag
  local extra=""; [ "$5" = "wet" ] && extra="--wet"
  timeout 1500 $2 --claim-run --source-root dag --source-root src/v2 --source-root src/v1 $extra --entry "$1" --functions "$4" > "$3" 2>&1; echo $?
}
echo "=== A. HEAD hermetic: all 12 by name, expect rc=0 all PASS ==="
cd /tmp/w629h || exit 9
rc=$(runw $F $CBH /tmp/c629_head_h.out "$FNS_H" hermetic); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c629_head_h.out) FAIL=$(grep -ac '^FAIL ' /tmp/c629_head_h.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c629_head_h.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c629_head_h.out | sed 's/^/    V| /' | cut -c1-160
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c629_head_h.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== A2. HEAD wet (author route): expect rc=0 all PASS ==="
rc=$(runw $F $CBH /tmp/c629_head_w.out "$FNS_H" wet); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c629_head_w.out) FAIL=$(grep -ac '^FAIL ' /tmp/c629_head_w.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c629_head_w.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c629_head_w.out | sed 's/^/    V| /' | cut -c1-160
echo "=== B. BASE: same file+names -> COMPILE/RESOLVE refusal, named cause ==="
cd /tmp/w629b || exit 9
rc=$(runw $F $CBB /tmp/c629_base.out "$FNS_B" wet); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c629_base.out) FAIL=$(grep -ac '^FAIL ' /tmp/c629_base.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c629_base.out)"
echo "  cause_hits Document-not-found=$(grep -ac "Document' not found in type 'Fragment'\|variant 'Document' not found" /tmp/c629_base.out)"
grep -aiE "error|refus|not found" /tmp/c629_base.out | head -6 | sed 's/^/    E| /' | cut -c1-190
[ "$rc" = "0" ] && echo "  !! BASE DID NOT REFUSE (rc=0)"
[ "$rc" != "0" ] && [ "$(grep -ac "Document' not found in type 'Fragment'\|variant 'Document' not found" /tmp/c629_base.out)" = "0" ] && echo "  !! REFUSED WITH WRONG/UNNAMED CAUSE"
echo "=== C. OTHER-SHAPE CONTROL at BASE: sibling roadmap witness compiles+holds ==="
SIB=$(git ls-tree --name-only HEAD dag/test/claim/roadmap/ | grep -v hierarchy_view | head -2 | tr '\n' ' ')
echo "  siblings: $SIB"
for s in $SIB; do
  n=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $s | sed 's/^test fn //' | head -1)
  [ -z "$n" ] && { echo "  $s NO_TEST_FN skip"; continue; }
  rc=$(runw $s $CBB /tmp/c629_ctl_$(basename $s).out "$n" wet); echo "  $s first=$n rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c629_ctl_$(basename $s).out) FAIL=$(grep -ac '^FAIL ' /tmp/c629_ctl_$(basename $s).out)"
done
echo DONE_629_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w629h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w629b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
