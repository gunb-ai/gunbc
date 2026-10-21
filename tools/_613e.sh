set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 tail: the two arms lost when dispatch _613a hit the 55m cap (D2 unrostered refusal, final
# witness after restores). Same head, same cached target dir; builds land in seconds if unchanged.
OID=34ad3659dbd0c9eda30a14604797f453e5bd7d5c  # gh pr view 14613 --json headRefOid
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613e
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613e 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613e)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613e) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613e $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613e
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
WIT=dag/test/claim/instruments/instrument_dispatch_workflow_witness.dag
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14613/release/claim_batch
GB=/root/workspace/repo-root/.t14613/release/gunbc
[ -x $CB ] && [ -x $GB ] || { echo NO_BINARIES; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12) GB_SHA=$(sha $GB|cut -c1-12) (expect 74cb9cf0168d / 5329ccff34b5 from dispatch A)"
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $WIT | sed 's/^test fn //' | sort | paste -sd,)
echo "claim_count=$(echo $FNS | tr ',' '\n' | wc -l)"
echo "=== D2 (re-run): unrostered path must refuse, naming GeneratedArtifactPathUnrostered ==="
timeout 900 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/definitely_not_rostered_613.yml > /tmp/c613e_d2.out 2>&1; rc=$?
echo "  rc=$rc unrostered_hits=$(grep -ac 'GeneratedArtifactPathUnrostered' /tmp/c613e_d2.out)"
grep -aE 'GeneratedArtifactPathUnrostered|refus|Refused|unrostered' /tmp/c613e_d2.out | head -4 | sed 's/^/    D| /' | cut -c1-190
echo "=== F. final witness after restores (must be 17/17 PASS rc=0) ==="
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c613e_final.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c613e_final.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613e_final.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c613e_final.out)"
grep -aE '^FAIL ' /tmp/c613e_final.out | sed 's/^/    V| /' | cut -c1-170
echo "=== F. the delta's own surface: documentary_refs_witness over design_document (names read from the FILE) ==="
DOC=dag/test/claim/documentary_refs_witness.dag
DFNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $DOC | sed 's/^test fn //' | sort | paste -sd,)
echo "  doc claims from file: $DFNS"
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $DOC --functions "$DFNS" > /tmp/c613e_doc.out 2>&1; drc=$?
echo "  rc=$drc PASS=$(grep -ac '^PASS ' /tmp/c613e_doc.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613e_doc.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c613e_doc.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c613e_doc.out | sed 's/^/    V| /' | cut -c1-160
if grep -qa 'hermetic route has no arm' /tmp/c613e_doc.out; then
  echo "  !! hermetic refused -> wet re-run:"
  timeout 900 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $DOC --functions "$DFNS" > /tmp/c613e_docwet.out 2>&1
  echo "     wet rc=$? PASS=$(grep -ac '^PASS ' /tmp/c613e_docwet.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613e_docwet.out)"
fi
[ "$drc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c613e_doc.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
echo DONE_613_E
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
