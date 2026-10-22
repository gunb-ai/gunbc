set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 tail: the two arms lost when dispatch _613a hit the 55m cap (D2 unrostered refusal, final
# witness after restores). Same head, same cached target dir; builds land in seconds if unchanged.
OID=d7bb6af37d92d0d2b6df838d0509d17d57f78216  # gh pr view 14613 --json headRefOid
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613c
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613c 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613c)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613c) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613c $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613c
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
timeout 900 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/definitely_not_rostered_613.yml > /tmp/c613c_d2.out 2>&1; rc=$?
echo "  rc=$rc unrostered_hits=$(grep -ac 'GeneratedArtifactPathUnrostered' /tmp/c613c_d2.out)"
grep -aE 'GeneratedArtifactPathUnrostered|refus|Refused|unrostered' /tmp/c613c_d2.out | head -4 | sed 's/^/    D| /' | cut -c1-190
echo "=== F. final witness after restores (must be 17/17 PASS rc=0) ==="
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c613c_final.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c613c_final.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613c_final.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c613c_final.out)"
grep -aE '^FAIL ' /tmp/c613c_final.out | sed 's/^/    V| /' | cut -c1-170
echo DONE_613_C
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
