set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 @ e36e8124af18 (GO msg_76ebec15). Dispatch 2: regen fixed point via the DECLARED route
# (gunbc run generated_artifact_gate main_wet_one --arg path=...) for DESIGN.md, instrument-dispatch.yml, heal.yml.
# Arms: G0 route rc=0 + no drift on each file; G1 drift-repair control on each (corrupt, route must repair byte-exact);
# G2 unrostered path refuses typed (GeneratedArtifactPathUnrostered).
OID=e36e8124af184e8243ddac366d71e92d5ba68fb2
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613h
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613g 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613g 2>/dev/null)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613g 2>/dev/null) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613h $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613h
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613h CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t14613h/release/gunbc
[ -x $GB ] || { echo "NO gunbc -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha $GB)"
for f in .github/workflows/instrument-dispatch.yml .github/workflows/heal.yml DESIGN.md; do
  echo "-------- [$f] G0 no-drift route $(date -u +%H:%M:%S)Z"
  ORIG=$(sha $f)
  timeout 900 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$f > /tmp/c613h_g0.out 2>&1; rc=$?
  NP=$(grep -ac '^PASS ' /tmp/c613h_g0.out); NF=$(grep -ac '^FAIL ' /tmp/c613h_g0.out)
  echo "  ROW G0-$(basename $f) rc=$rc PASS=$NP FAIL=$NF sha_before=$ORIG sha_after=$(sha $f)"
  head -4 /tmp/c613h_g0.out | sed 's/^/  O| /' | cut -c1-180
  [ "$ORIG" = "$(sha $f)" ] && echo "  FIXED_POINT byte-identical (route wrote nothing new)" || echo "  !! DRIFTED (rc=$rc): route changed $f"
  [ "$rc" = "0" ] && [ "$((NP+NF))" = "0" ] && echo "  !! rc=0 zero-verdict VOID: $(head -3 /tmp/c613h_g0.out | tr '\n' ' ' | cut -c1-180)"
done
for f in .github/workflows/instrument-dispatch.yml .github/workflows/heal.yml DESIGN.md; do
  echo "-------- [$f] G1 drift-repair control $(date -u +%H:%M:%S)Z"
  cp $f /tmp/613h_$(basename $f).keep
  printf '\n# verifier-stray-line (must be removed by the regen route)\n' >> $f
  COR=$(sha $f)
  timeout 900 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$f > /tmp/c613h_g1.out 2>&1; rc=$?
  echo "  ROW G1-$(basename $f) rc=$rc corrupted_sha=$COR"
  git cat-file blob HEAD:$f | cmp -s - $f && echo "  REPAIRED_BYTE_EXACT (route actually writes)" || { echo "  !! NOT repaired:"; diff <(git cat-file blob HEAD:$f) $f | head -4 | sed 's/^/  D| /'; }
  git cat-file blob HEAD:$f > $f
done
echo "-------- G2 unrostered path refuses typed"
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/definitely_not_rostered_613.yml > /tmp/c613h_g2.out 2>&1; rc=$?
echo "  ROW G2 rc=$rc unrostered=$(grep -ac 'GeneratedArtifactPathUnrostered' /tmp/c613h_g2.out)"
head -5 /tmp/c613h_g2.out | sed 's/^/  O| /' | cut -c1-180
git status --porcelain | head -3 | sed 's/^/  STALE| /'
echo DONE_613_G2
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
