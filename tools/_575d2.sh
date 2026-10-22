set -u
cd /root/workspace/repo-root 2>/dev/null || true
# DELTA RE-VERIFY #14575, dispatch D2: THREE mutations of the census the delta rewrote, each aimed at the
# authority machinery the delta ADDED (the join's emitted-registry conjunct, the citation list, the receipt's
# grain-phrased credit). Plants are staged bytes derived locally from the head blob; each replaces ONE
# whole balanced expression (never a fragment), and each was pre-mapped to the claims that consume it by a
# claim-body scan of the witness and wet entries. Restores are byte-compared against head.
HEADSHA=8f7af1271081476ec2c2019db4d2d448c481ea52  # gh pr view 14575 --json headRefOid (re-verify sweep)
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag
HEAD_CEN_SHA=726e9c1462a9830a1dc139c84f77679f7ddf1e3c996a5b7839c1086ac2e35a45
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) (runner git is main; head gating comes ONLY from the fetch assertion below) START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575m
git fetch -q origin refs/pull/14575/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575m $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt575m
[ "$(git -C $WT rev-parse HEAD)" = "$HEADSHA" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ "$(sha $WT/$CEN)" = "$HEAD_CEN_SHA" ] && echo "SUBJECT_OK census ${HEAD_CEN_SHA:0:16}" || { echo "SUBJECT_DRIFT census"; exit 9; }
for p in census_new_head census_p1 census_p2 census_p3; do
  [ -f tools/stage/$p.dag ] || { echo "MISSING_PLANT tools/stage/$p.dag"; exit 9; }
  echo "plant $p $(sha tools/stage/$p.dag | cut -c1-16)"
done
[ "$(sha tools/stage/census_new_head.dag)" = "$HEAD_CEN_SHA" ] || { echo "PLANT_SOURCE_NOT_HEAD_BYTES"; exit 9; }

[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575d 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575d
echo 51539607552 > /sys/fs/cgroup/ver575d/memory.max
RB=$(cat /sys/fs/cgroup/ver575d/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575d memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver575d/cgroup.procs; echo $BASHPID >&11
cd $WT
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575d/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha256sum $CB | cut -c1-12) BUILT_AT $(date -u +%H:%M:%S)Z"
cd $WT
WITFNS=$(grep "^test fn" $WIT | sed 's/^test fn //; s/() -> Bool {//' | paste -sd, -)
grep "^test fn" $WIT | sed 's/^test fn //; s/() -> Bool {//' | sort > /tmp/expwit.txt
REXFNS=$(grep "^test fn" $REX | sed 's/^test fn //; s/() -> Bool {//' | paste -sd, -)
grep "^test fn" $REX | sed 's/^test fn //; s/() -> Bool {//' | sort > /tmp/exprex.txt

run() { # tag entry fnsfile mode
  local tag=$1 entry=$2 exp=$3 mode=$4 fns
  fns=$( [ "$mode" = wet ] && echo "$REXFNS" || echo "$WITFNS" )
  local extra=(--claim-run)
  [ "$mode" = wet ] && extra+=(--wet)
  timeout 1500 $CB "${extra[@]}" --source-root dag --source-root src/v2 --source-root src/v1 \
    --entry "$entry" --functions "$fns" > /tmp/m_$tag.log 2>&1
  local rc=$?
  sed 's/\x1b\[[0-9;]*m//g' /tmp/m_$tag.log > /tmp/m_${tag}_c.log
  echo "--- [$tag] entry=$(basename $entry) mode=$mode rc=$rc bytes=$(wc -c < /tmp/m_${tag}_c.log) census=$(sha $CEN | cut -c1-16) $(date -u +%H:%M:%S)Z"
  echo "  counts: PASS=$(grep -acE '^PASS ' /tmp/m_${tag}_c.log) FAIL=$(grep -acE '^FAIL ' /tmp/m_${tag}_c.log) UNOBSERVED=$(grep -acE '^UNOBSERVED ' /tmp/m_${tag}_c.log) SKIP=$(grep -acE '^SKIP ' /tmp/m_${tag}_c.log)"
  grep -aE '^(PASS|FAIL|UNOBSERVED|SKIP) ' /tmp/m_${tag}_c.log | awk '{print $2}' | sort > /tmp/mgot_$tag.txt
  echo "  printed_total=$(wc -l < /tmp/mgot_$tag.txt) expected=$(wc -l < $exp)"
  echo "  NEVER-PRINTED:"; comm -23 $exp /tmp/mgot_$tag.txt | sed 's/^/    NP /'
  echo "  PRINTED-NOT-EXPECTED:"; comm -13 $exp /tmp/mgot_$tag.txt | sed 's/^/    XN /'
  echo "  FAILING CLAIMS:"; grep -aE '^FAIL ' /tmp/m_${tag}_c.log | awk '{print "    F| "$2}'
  echo "  PASSING CLAIMS:"; grep -aE '^PASS ' /tmp/m_${tag}_c.log | awk '{print "    P| "$2}' | cut -c1-150
  local nv=$(wc -l < /tmp/mgot_$tag.txt)
  if [ "$nv" = "0" ]; then echo "  ZERO-VERDICT ARM: this is a refusal, NOT a red. raw log first 16:"; head -16 /tmp/m_${tag}_c.log | sed 's/^/    R| /' | cut -c1-220; fi
  echo "  panic=$(grep -ac panicked /tmp/m_${tag}_c.log)"
}
restore() {
  cp -f /root/workspace/repo-root/tools/stage/census_new_head.dag $CEN
  [ "$(sha $CEN)" = "$HEAD_CEN_SHA" ] && echo "  RESTORED_BYTE_IDENTICAL ${HEAD_CEN_SHA:0:16}" || { echo "  RESTORE_FAILED got=$(sha $CEN)"; exit 9; }
}

echo "=== P1: the join's emitted-registry conjunct deleted (fact.path == path && any(emitted == emitted_entry_for_module(..)) => path == path)"
echo "  predicted: FAIL only the_join_requires_the_fact_to_name_the_path_and_the_module_to_be_emitted (witness); wet positive-direction claim unaffected"
cp -f /root/workspace/repo-root/tools/stage/census_p1.dag $CEN
[ "$(sha $CEN | cut -c1-16)" = "$(sha /root/workspace/repo-root/tools/stage/census_p1.dag | cut -c1-16)" ] || { echo "PLANT_APPLY_FAILED"; exit 9; }
run p1_wit $WIT /tmp/expwit.txt hermetic
restore
echo "=== P2: one element removed from outside_roots_citations (scan_dir_authority)"
echo "  predicted: FAIL the_citation_list_covers_every_authority_the_cause_carries (witness, length==5); and in the wet entry the citation-subset claim"
cp -f /root/workspace/repo-root/tools/stage/census_p2.dag $CEN
run p2_wit $WIT /tmp/expwit.txt hermetic
run p2_rex $REX /tmp/exprex.txt wet
restore
echo "=== P3: the receipt's vouched branch reworded back to the pre-delta phrasing ('| declared by <note> also declares')"
echo "  predicted: FAIL only the_receipt_credits_the_note_only_at_the_grain_it_holds"
cp -f /root/workspace/repo-root/tools/stage/census_p3.dag $CEN
run p3_wit $WIT /tmp/expwit.txt hermetic
restore
echo "=== CONTROL after restores: clean census, both entries (expect 39/39 and 7/7)"
run ctl_wit $WIT /tmp/expwit.txt hermetic
run ctl_rex $REX /tmp/exprex.txt wet
echo DONE_14575D_MUTATIONS
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
