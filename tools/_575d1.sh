set -u
cd /root/workspace/repo-root 2>/dev/null || true
# DELTA RE-VERIFY #14575: 6bec9c50fe5 -> 69a41ee167c (10 review-driven commits, all touching this
# census: fixtures moved OUT of production into a new helpers module, authorities re-cited as
# DeclarationRefs, the population grounded on the emitted registry, 5 new witness claims, the wet
# entry rebuilt 3->7 claims). This dispatch = BASELINES ONLY (mutations are separate dispatches).
# Every claim runs inside a private worktree detached AT THE NEW HEAD, because the census reads the
# real index + real grep over the process cwd.
HEADSHA=69a41ee167cad3e5c0648b0b7484777817c55ad1  # expanded locally: git rev-parse 69a41ee167c. My first
# attempt here hand-filled the suffix from the 11-char prefix and the runner's origin correctly refused
# it ("upload-pack: not our ref") -- a fabricated SHA fails closed rather than fetching something else.
# Fetch through the PR ref and assert the resolved commit equals HEADSHA, so a moved head cannot be
# silently measured.
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD 2>/dev/null | cut -c1-12) (runner git is main; NEVER used for head gating) START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575d
git fetch -q origin refs/pull/14575/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP, re-resolve with gh pr view --json headRefOid"; exit 9; }
git cat-file -e $HEADSHA^{commit} || { echo "HEAD_COMMIT_UNAVAILABLE_ON_RUNNER"; exit 9; }
git worktree add -q --detach /tmp/wt575d $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt575d
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD)"
[ "$(git -C $WT rev-parse HEAD)" = "$HEADSHA" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }

CEN=src/v2/workflow/outside_discovery_roots_census.dag
LIV=src/v2/workflow/outside_discovery_roots_census_live.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag
HELP=src/v2/test/claim/outside_discovery_roots_census_helpers.dag
# head blob sha256s computed locally at 69a41ee167c (git show <head>:<path> | sha256sum)
declare -A SHA=(
 [$CEN]=373447efdc7af24cd3215c74df942d35bb23e50e99e9e074d02435f3852f6913
 [$LIV]=5e10233634d70918acb614b0e786686d7276e40c042bdb3292a84dcc3b42c802
 [$WIT]=bd9c33ad08c0c97eb49f96a46bbc9c2eceba663e15dfb29653acbc2e3373d472
 [$REX]=0bf641b5585ba3d5069c998269d6c7af1e7ae59e0894a0880698bd710a9ceb84
 [$HELP]=b978d675ffd51c38378e83f97e5520ddaa9f0d031858fe2a6ab11e5b1e891ec6 )
for f in $CEN $LIV $WIT $REX $HELP; do
  a=$(sha256sum $WT/$f | cut -d' ' -f1)
  [ "$a" = "${SHA[$f]}" ] && echo "SUBJECT_OK $(basename $f) ${a:0:16}" || { echo "SUBJECT_DRIFT $(basename $f) got=${a:0:16} want=${SHA[$f]:0:16}"; exit 9; }
done
# also assert the two wet-lane files the delta touched are at head bytes (a roster drift would silently
# change what the fleet runs without changing any claim)
for f in src/v2/workflow/floor_route_gap.dag src/v2/workflow/local_repo_wet_terminal.dag; do
  echo "LANE_FILE $(basename $f) $(sha256sum $WT/$f | cut -c1-16)"
done

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
echo "universe: wit=$(wc -l < /tmp/expwit.txt) rex=$(wc -l < /tmp/exprex.txt)"
# v1 source-root is needed only if an entry pulls `^import v1.`; asserted, not assumed
for e in $WIT $REX; do echo "v1_imports $(basename $e)=$(grep -c '^import v1\.' $e)"; done

run() { # tag entry fnsfile mode
  local tag=$1 entry=$2 exp=$3 mode=$4 fns
  fns=$( [ "$mode" = wet ] && echo "$REXFNS" || echo "$WITFNS" )
  local extra=(--claim-run)
  [ "$mode" = wet ] && extra+=(--wet)
  timeout 1500 $CB "${extra[@]}" --source-root dag --source-root src/v2 --source-root src/v1 \
    --entry "$entry" --functions "$fns" > /tmp/d_$tag.log 2>&1
  local rc=$?
  sed 's/\x1b\[[0-9;]*m//g' /tmp/d_$tag.log > /tmp/d_${tag}_c.log
  echo "--- [$tag] entry=$(basename $entry) mode=$mode rc=$rc bytes=$(wc -c < /tmp/d_${tag}_c.log) $(date -u +%H:%M:%S)Z"
  echo "  counts: PASS=$(grep -acE '^PASS ' /tmp/d_${tag}_c.log) FAIL=$(grep -acE '^FAIL ' /tmp/d_${tag}_c.log) UNOBSERVED=$(grep -acE '^UNOBSERVED ' /tmp/d_${tag}_c.log) SKIP=$(grep -acE '^SKIP ' /tmp/d_${tag}_c.log)"
  grep -aE '^(PASS|FAIL|UNOBSERVED|SKIP) ' /tmp/d_${tag}_c.log | awk '{print $2}' | sort > /tmp/got_$tag.txt
  echo "  printed_total=$(wc -l < /tmp/got_$tag.txt) expected=$(wc -l < $exp)"
  echo "  NEVER-PRINTED:"; comm -23 $exp /tmp/got_$tag.txt | sed 's/^/    NP /'
  echo "  PRINTED-NOT-EXPECTED:"; comm -13 $exp /tmp/got_$tag.txt | sed 's/^/    XN /'
  echo "  verdicts:"; grep -aE '^(PASS|FAIL|UNOBSERVED|SKIP) ' /tmp/d_${tag}_c.log | sed 's/^/    /' | cut -c1-170
  local nv=$(wc -l < /tmp/got_$tag.txt)
  if [ "$nv" = "0" ]; then echo "  RAW LOG, zero verdicts -- refusing to call this anything. first 16 lines:"; head -16 /tmp/d_${tag}_c.log | sed 's/^/    R| /' | cut -c1-220; fi
  echo "  panic=$(grep -ac panicked /tmp/d_${tag}_c.log)  eval_steps>0 claims=$(grep -ac 'eval_steps=[1-9]' /tmp/d_${tag}_c.log)"
  # FAIL arms print their cause (exit code never establishes cause)
  awk '/^FAIL /{f=10} f>0{print "    C| " substr($0,1,230); f--}' /tmp/d_${tag}_c.log | head -50
}

echo "=== ARM 1: witness entry, HERMETIC (34 old + 5 new = 39 expected)"
run wit $WIT /tmp/expwit.txt hermetic
echo "=== ARM 2: wet entry, WET (3 old + 4 new = 7 expected)"
run rex $REX /tmp/exprex.txt wet
echo "=== ARM 3: roster mirror check, no run needed -- chunk_19 identities vs the wet entry's claims"
awk '/fn floor_route_gap_expectation_chunk_19/,/^}$/' src/v2/workflow/floor_route_gap.dag \
  | grep -oE 'identity: "[^"]+"' | sed 's/identity: "//; s/"//' | sort > /tmp/chunk19.txt
sed 's/^/v2.test.claim.outside_discovery_roots_census_real_execution./' /tmp/exprex.txt | sort > /tmp/rexid.txt
echo "  chunk_19 rows=$(wc -l < /tmp/chunk19.txt) wet claims=$(wc -l < /tmp/rexid.txt)"
echo "  roster-only:"; comm -23 /tmp/chunk19.txt /tmp/rexid.txt | sed 's/^/    R-ONLY /'
echo "  claim-only:"; comm -13 /tmp/chunk19.txt /tmp/rexid.txt | sed 's/^/    C-ONLY /'
echo "  the stale identity the delta renamed must appear NOWHERE as data:"
echo "    old-name occurrences=$(git grep -c the_index_lists_exactly_the_population_the_note_declares_by_real_execution -- '*.dag' | wc -l) (non-comment listed below)"
git grep -n the_index_lists_exactly_the_population_the_note_declares_by_real_execution -- '*.dag' | grep -v ':\s*//' | sed 's/^/      /' | cut -c1-190
echo "DONE_14575D_BASELINE"
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
