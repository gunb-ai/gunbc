set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
# #14575 @05a28f7d734 (GO msg_eaf7c7ea). D1: build + baseline wit 48 + wet REX 7 + post wit.
HEADSHA=05a28f7d7349707cf23e65c325322884dd0dc9e8
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag
[ ${#HEADSHA} -eq 40 ] || { echo "LEN STOP"; exit 9; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575n1
git fetch -q origin +refs/pull/14575/head:refs/verifier/f575n 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/f575n 2>/dev/null)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f575n 2>/dev/null) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575n1 $HEADSHA || { echo WORKTREE_ADD_FAILED; exit 9; }
WT=/tmp/wt575n1
sha(){ sha256sum "$1" | cut -d' ' -f1; }
cd $WT || exit 9
echo "BUILD_COMMIT=$(git rev-parse HEAD)"
for f in $WIT $REX $CEN; do [ "$(git cat-file blob $HEADSHA:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }; done
echo "HEAD_BLOBS_VERIFIED witness/real_execution/census"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t575n1 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t575n1/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
NAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $NAMES | tr ',' '\n' | wc -l) (expect 48)"
RNAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $REX | sed 's/^test fn //' | paste -sd, -)
echo "REX_COUNT=$(echo $RNAMES | tr ',' '\n' | wc -l) (expect 7)"
runwit () { # <tag> <entry> <fns> <wet0|wet1>
  local tag=$1 ent=$2 fns=$3 wet=$4 wf=""
  [ "$wet" = wet1 ] && wf=--wet
  echo "-------- [$tag] $(date -u +%H:%M:%S)Z"
  timeout 1400 $CB --claim-run $wf --source-root dag --source-root src/v2 --entry $ent --functions "$fns" > /tmp/n1_$tag.out 2>&1; local rc=$?
  local NP NF NU; NP=$(grep -ac '^PASS ' /tmp/n1_$tag.out); NF=$(grep -ac '^FAIL ' /tmp/n1_$tag.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/n1_$tag.out)
  echo "  ROW $tag rc=$rc PASS=$NP FAIL=$NF UNOBS=$NU"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/n1_$tag.out | head -8 | sed 's/^/  F| /' | cut -c1-200
  [ "$rc" = "0" ] && [ "$((NP+NF+NU))" = "0" ] && echo "  ROW $tag !! EXIT 0 HAVING OBSERVED NOTHING -- VOID"
}
runwit ctl_wit $WIT "$NAMES" wet0
runwit ctl_rex $REX "$RNAMES" wet1
runwit post_wit $WIT "$NAMES" wet0
git status --porcelain | head -3 | sed 's/^/  STALE| /'
echo DONE_575N1 $(date -u +%H:%M:%S)Z
