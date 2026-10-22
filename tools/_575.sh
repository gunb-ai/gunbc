set -u
cd /root/workspace/repo-root 2>/dev/null || true
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD 2>/dev/null | cut -c1-12) (runner git is main; never used for head gating)"
HEADSHA=6bec9c50fe5393d3691095b3643c98d4b2a89225

# The census reads the REAL index and the REAL grep over the process cwd, so every arm runs inside a
# private worktree detached AT THE PR HEAD. That (a) makes the subject bytes the authored blobs rather
# than installs, and (b) keeps RED-A's index mutation off the shared runner checkout entirely.
# (A `git clone -s` would not do: it only transfers branch-reachable objects, so the PR head commit --
# fetched but unreferenced -- is not copied, and the checkout silently stays on main. Learned the hard
# way: attempt 1 died at CLONE_NOT_AT_HEAD with CLONE_HEAD = the base.)
WT=/tmp/wt575
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
git fetch -q origin $HEADSHA 2>&1 | tail -1
git cat-file -e $HEADSHA^{commit} || { echo "HEAD_COMMIT_UNAVAILABLE_ON_RUNNER"; exit 9; }
git worktree add -q --detach $WT $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD)"
[ "$(git -C $WT rev-parse HEAD)" = "$HEADSHA" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }

CEN=src/v2/workflow/outside_discovery_roots_census.dag
LIV=src/v2/workflow/outside_discovery_roots_census_live.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag

declare -A H
H[$CEN]=90f3af355d8e70e2d2985d4b886ae1cb3e20a9bb1f35f3c222b2a8f486ef056b
H[$LIV]=d6f1239b4356b39bc6ec27e73a0a263e049a813d9fb605e026e11a1c78295aff
H[$WIT]=6880a88b51747a64790e16357f6a6286465f6e8c0c4830b13e0073566a8d38f5
H[$REX]=55763b1ed1675c2cffd05dbc308dc3992d61628dbdb75d4d383f5308a2ffaa15
echo "=== subject bytes as checked out (asserted against MY local clone of the gh-printed head):"
bad=0
for f in $CEN $LIV $WIT $REX; do
  a=$(sha256sum $WT/$f | cut -d' ' -f1)
  if [ "$a" = "${H[$f]}" ]; then printf "  OK   %-58s %s\n" "$(basename $f)" "${a:0:16}"
  else printf "  !!   %-58s got=%s want=%s\n" "$(basename $f)" "${a:0:16}" "${H[$f]:0:16}"; bad=$((bad+1)); fi
done
echo "SUBJECT_DRIFT=$bad"; [ $bad -eq 0 ] || exit 9

echo "=== GROUND TRUTH in the head clone: git ls-files under the note's subject, filtered to files that declare a test fn"
cd $WT
git ls-files 'src/v1/tests/claim/*' | grep '\.dag$' | sort > /tmp/tracked_all.txt
: > /tmp/tcb.txt
while read f; do [ -f "$f" ] && grep -qE '^[[:space:]]*test fn [A-Za-z_][A-Za-z_0-9]*' "$f" && echo "$f" >> /tmp/tcb.txt; done < /tmp/tracked_all.txt
sort -u /tmp/tcb.txt -o /tmp/tcb.txt
echo "tracked_dag_under_subject=$(wc -l < /tmp/tracked_all.txt) declaring_test_fn=$(wc -l < /tmp/tcb.txt)"
cat > /tmp/note13.txt <<'EOF'
src/v1/tests/claim/annotation_gap_round_trip_test.dag
src/v1/tests/claim/bare_variant_reference_occurrence_control_test.dag
src/v1/tests/claim/carrier_realization_census.dag
src/v1/tests/claim/checkpoint_identity_keying_witness_test.dag
src/v1/tests/claim/escape_json_string_differential.dag
src/v1/tests/claim/item_kind_dissolves_shape_predicates_control_test.dag
src/v1/tests/claim/keyword_pattern_binder_refusal_test.dag
src/v1/tests/claim/parse_source_extent_control_test.dag
src/v1/tests/claim/production_fed_exposure_discrimination_control_test.dag
src/v1/tests/claim/reference_derived_disposition_census_witness_test.dag
src/v1/tests/claim/tco_tail_call_assigns_every_loop_slot_witness_test.dag
src/v1/tests/claim/transport_roster_closure_witness_test.dag
src/v1/tests/claim/type_declaration_occurrence_control_test.dag
EOF
sort -u /tmp/note13.txt -o /tmp/note13.txt
echo "  my-13 vs git-listed-claim-bearers  NEVER-IN-GIT:"; comm -23 /tmp/note13.txt /tmp/tcb.txt | sed 's/^/    NP /'
echo "  EXTRA-IN-GIT:"; comm -13 /tmp/note13.txt /tmp/tcb.txt | sed 's/^/    EX /'
tot=0; while read f; do n=$(grep -cE '^[[:space:]]*test fn [A-Za-z_][A-Za-z_0-9]*' "$f"); tot=$((tot+n)); printf "    %3d %s\n" "$n" "$f"; done < /tmp/tcb.txt
echo "  SUM_TEST_FN_OVER_THE_13=$tot"
grep -oE '"src/v1/tests/claim/[^"]+"' $CEN | tr -d '"' | sort > /tmp/census_list.txt
echo "  census note_declared_paths vs ground truth: NEVER:"; comm -23 /tmp/census_list.txt /tmp/tcb.txt | sed 's/^/    NP /'
echo "  EXTRA:"; comm -13 /tmp/census_list.txt /tmp/tcb.txt | sed 's/^/    EX /'

export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
CB=/root/workspace/repo-root/.t14575/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha256sum $CB | cut -c1-16)"
cd $WT

WITFNS=$(grep "^test fn" $WIT | sed 's/^test fn //; s/() -> Bool {//' | paste -sd, -)
REXFNS=$(grep "^test fn" $REX | sed 's/^test fn //; s/() -> Bool {//' | paste -sd, -)
grep "^test fn" $WIT | sed 's/^test fn //; s/() -> Bool {//' | sort > /tmp/exp_wit.txt
grep "^test fn" $REX | sed 's/^test fn //; s/() -> Bool {//' | sort > /tmp/exp_rex.txt
echo "WITNESS_CLAIMS_IN_FILE=$(wc -l < /tmp/exp_wit.txt)  PAIRING_CLAIMS_IN_FILE=$(wc -l < /tmp/exp_rex.txt)"

run() { # tag entry fns mode expectfile
  tag=$1; ent=$2; fns=$3; mode=$4; exp=$5
  extra=""; [ "$mode" = wet ] && extra="--wet"
  timeout 1200 $CB $extra --source-root dag --source-root src/v2 --source-root src/v1 \
    --entry "$ent" --functions "$fns" > /tmp/r_$tag.log 2>&1
  rc=$?
  sed 's/\x1b\[[0-9;]*m//g' /tmp/r_$tag.log > /tmp/r_${tag}_c.log
  echo "--- [$tag] mode=$mode entry=$(basename $ent) rc=$rc bytes=$(wc -c < /tmp/r_${tag}_c.log)"
  echo "  counts: PASS=$(grep -acE '^PASS ' /tmp/r_${tag}_c.log) FAIL=$(grep -acE '^FAIL ' /tmp/r_${tag}_c.log) UNOBSERVED=$(grep -acE '^UNOBSERVED ' /tmp/r_${tag}_c.log) SKIP=$(grep -acE '^SKIP ' /tmp/r_${tag}_c.log)"
  grep -aE '^(PASS|FAIL|UNOBSERVED|SKIP) ' /tmp/r_${tag}_c.log | sed 's/^\([A-Za-z]*\) \([^ ]*\).*/\2/' | sort > /tmp/got_$tag.txt
  echo "  printed_total=$(wc -l < /tmp/got_$tag.txt) expected=$(wc -l < $exp)"
  echo "  NEVER-PRINTED:"; comm -23 $exp /tmp/got_$tag.txt | sed 's/^/    NP /'
  echo "  EXTRA:"; comm -13 $exp /tmp/got_$tag.txt | sed 's/^/    EX /'
  echo "  every non-PASS verdict line:"; grep -aE '^(FAIL|UNOBSERVED|SKIP) ' /tmp/r_${tag}_c.log | sed 's/^/    /' | cut -c1-165
  echo "  panic=$(grep -ac panicked /tmp/r_${tag}_c.log)"
}

echo "=== ARM 1 hermetic witness (expect 34 PASS rc=0)"
run wit $WIT "$WITFNS" hermetic /tmp/exp_wit.txt

echo "=== ARM 2 wet pairing (expect 3 PASS rc=0)"
run rex $REX "$REXFNS" wet /tmp/exp_rex.txt

echo "=== ARM 3 RED-A: drop one path from the INDEX of the PRIVATE clone (the listing's real input)"
DROP=src/v1/tests/claim/type_declaration_occurrence_control_test.dag
echo "  PRE_LISTED=$(git ls-files -- $DROP | wc -l)  file_exists_on_disk=$([ -f $DROP ] && echo yes || echo no)"
git update-index --remove -- $DROP
echo "  MID_LISTED=$(git ls-files -- $DROP | wc -l)  ls_files_glob=$(git ls-files '*.dag' | grep -c "^$DROP$")  file_still_on_disk=$([ -f $DROP ] && echo yes || echo no)"
run redA $REX "$REXFNS" wet /tmp/exp_rex.txt
git update-index --add -- $DROP
echo "  POST_LISTED=$(git ls-files -- $DROP | wc -l) (must be 1)"
[ "$(git ls-files -- $DROP | wc -l)" = "1" ] && echo "  RED_A_INDEX_RESTORED" || echo "  RED_A_INDEX_NOT_RESTORED"

restore_census() { git -C $WT checkout -q $HEADSHA -- $CEN; a=$(sha256sum $CEN | cut -d' ' -f1); [ "$a" = "${H[$CEN]}" ] && echo "  census_restored=$(sha256sum $CEN | cut -c1-16) MATCH" || echo "  census_restore_MISMATCH ${a:0:16}"; }

echo "=== ARM 4 RED-B: the lane predicate excludes nothing (path_is_in_a_scan_lane => true)"
cp -f /root/workspace/repo-root/tools/stage/mutB_scanlane_true.dag $CEN
echo "  MUTB_INSTALLED=$(sha256sum $CEN | cut -c1-16)"
run redB $WIT "$WITFNS" hermetic /tmp/exp_wit.txt
restore_census

echo "=== ARM 5 RED-C: census visibility always true (populations stop being disjoint)"
cp -f /root/workspace/repo-root/tools/stage/mutC_visible_true.dag $CEN
echo "  MUTC_INSTALLED=$(sha256sum $CEN | cut -c1-16)"
run redC $WIT "$WITFNS" hermetic /tmp/exp_wit.txt
restore_census

echo "=== ARM 6 RED-D: grain suffix removed from the NOT-declared receipt branch only"
echo "  (the manager's requirement is pattern-grain on EVERY line; a claim that checks only some lines"
echo "   would stay green while a non-vouched row silently loses its label)"
python3 - <<'PY'
src=open("/root/workspace/repo-root/tools/stage/head_outside_discovery_roots_census.dag").read()
old='''  } else {
    concat(concat(head, concat(" | NOT declared by ", attribution)), grain)
  }'''
new='''  } else {
    concat(head, concat(" | NOT declared by ", attribution))
  }'''
assert src.count(old)==1
open('/tmp/mutD_grain.dag','w').write(src.replace(old,new))
print("  MUT_D_BUILT")
PY
cp -f /tmp/mutD_grain.dag $CEN
echo "  MUTD_INSTALLED=$(sha256sum $CEN | cut -c1-16)"
run redD $WIT "$WITFNS" hermetic /tmp/exp_wit.txt
restore_census

echo "=== ARM 7 control: committed census back, re-run hermetic (a restore is re-run, not trusted)"
run wit_final $WIT "$WITFNS" hermetic /tmp/exp_wit.txt
echo "  final census sha=$(sha256sum $CEN | cut -c1-16) (author ORIG_CENSUS_SHA=90f3af355d8e70e2)"
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
echo "worktree_cleaned=$(ls -d $WT 2>/dev/null || echo gone)"
echo DONE_14575
