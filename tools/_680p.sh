set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14680 culprit pair (manager scope part 1): the 7 service_spine claims must PASS at dae01be7434
# and FAIL at 226bb079ecc (#13832, the culprit merge). Both OIDs predate the main break -> buildable.
# Head arms (7/7 + mutation) are HELD until #14681 heals main; head c23a0f1 now includes broken main.
PASS_SHA=dae01be7434
FAIL_SHA=226bb079ecc1ac1bc0883145783442c8808f500a
F=src/v2/test/claim/normalize/service_spine.dag
echo "START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w680p /tmp/w680f
git cat-file -e "$PASS_SHA^{commit}" 2>/dev/null || git fetch -q origin $PASS_SHA || true
git cat-file -e "$FAIL_SHA^{commit}" 2>/dev/null || git fetch -q origin $FAIL_SHA || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
for R in "p $PASS_SHA" "f $FAIL_SHA"; do set -- $R; T=$1; S=$2
  echo "=== rev=$T $S ==="
  git worktree add -q --detach /tmp/w680$T $S || { git fetch -q origin $S && git worktree add -q --detach /tmp/w680$T $S; } || { echo "WT_$T SKIP"; continue; }
  cd /tmp/w680$T || exit 9
  HEADNOW=$(git rev-parse HEAD); echo "  checkout=$HEADNOW"
  ( CARGO_TARGET_DIR=/root/workspace/repo-root/.t680$T cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
  CB=/root/workspace/repo-root/.t680$T/release/claim_batch
  if [ ! -x $CB ]; then echo "  NO_CB_$T"; cd /root/workspace/repo-root; continue; fi
  echo "  BUILD_COMMIT_$T=$HEADNOW CB_SHA_$T=$(sha $CB)"
  FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $F | sed 's/^test fn //' | sort | paste -sd,)
  echo "  names_from_file=$(echo $FNS | tr ',' '\n' | wc -l)"
  timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $F --functions "$FNS" > /tmp/c680_$T.out 2>&1; rc=$?
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c680_$T.out) FAIL=$(grep -ac '^FAIL ' /tmp/c680_$T.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c680_$T.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c680_$T.out | sed 's/^/    V| /' | cut -c1-140
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c680_$T.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
  cd /root/workspace/repo-root
done
echo DONE_680_PAIR
cd /root/workspace/repo-root
git worktree remove --force /tmp/w680p >/dev/null 2>&1 || true
git worktree remove --force /tmp/w680f >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
