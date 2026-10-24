set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 @b7854d98d76 regen FINISH arms (g2 died at runner wall before these):
# G1 drift-repair (route WRITES) for heal.yml + DESIGN.md; G2 unrostered path refuses typed.
# G0 fixed-point for all three + G1 instrument-dispatch.yml were already observed in the g2 dispatch.
OID=b7854d98d76584c65b04df7ce1a971b1ab7322fc
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613m
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613g 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613g 2>/dev/null)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613g 2>/dev/null) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613m $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613m
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
echo "BUILD_COMMIT=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613m CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t14613m/release/gunbc
[ -x $GB ] || { echo "NO gunbc -- nothing below is a result"; exit 9; }
echo "GB_SHA=$(sha $GB)"
for f in .github/workflows/heal.yml DESIGN.md; do
  b=$(basename "$f")
  echo "-------- [$b] G1 drift-repair control $(date -u +%H:%M:%S)Z"
  ORIG=$(sha $f)
  printf '\n# verifier-stray-line (must be removed by the regen route)\n' >> $f
  COR=$(sha $f)
  echo "  pre-repair: intact_sha=$ORIG corrupted_sha=$COR"
  timeout 1200 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$f > /tmp/c613m_g1_$b.out 2>&1; rc=$?
  echo "  ROW G1-$b rc=$rc"
  grep -aiE 'refus|error|not rostered|unauthorized|panic' /tmp/c613m_g1_$b.out | head -4 | sed 's/^/  E| /' | cut -c1-190
  if git cat-file blob HEAD:$f | cmp -s - $f; then
    echo "  REPAIRED_BYTE_EXACT (route restored to HEAD bytes)"
    [ "$ORIG" = "$(sha $f)" ] && echo "  RESTORED_TO_ORIGINAL sha=$ORIG" || echo "  !! restored but sha!=intact"
  else
    echo "  !! NOT repaired:"; diff <(git cat-file blob HEAD:$f) $f | head -6 | sed 's/^/  D| /' | cut -c1-190
  fi
  git cat-file blob HEAD:$f > $f
done
echo "-------- G2 unrostered path refuses typed $(date -u +%H:%M:%S)Z"
timeout 900 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/definitely_not_rostered_613.yml > /tmp/c613m_g2.out 2>&1; rc=$?
echo "  ROW G2 rc=$rc unrostered_hits=$(grep -ac 'GeneratedArtifactPathUnrostered' /tmp/c613m_g2.out)"
grep -aiE 'GeneratedArtifactPathUnrostered|refus|unauthorized|not rostered' /tmp/c613m_g2.out | head -5 | sed 's/^/  O| /' | cut -c1-190
[ "$rc" = "0" ] && [ "$(grep -ac 'GeneratedArtifactPathUnrostered' /tmp/c613m_g2.out)" = "0" ] && echo "  ROW G2 !! rc=0 WITHOUT the typed refusal -- investigate (silent acceptance would be a finding)"
git status --porcelain | head -3 | sed 's/^/  STALE| /'
echo DONE_613_M
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
