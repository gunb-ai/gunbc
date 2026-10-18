#!/bin/bash
# EFFECT OF THE 'binding IS the generic declaration's own form' ARMS, measured by identity.
#   usage: bash ab_extension_14152.sh A|B
#
# A = the fold WITHOUT the new arms  (04_infer.dag at a8e23ad9db5, the durability tip)
# B = the fold WITH them             (04_infer.dag at 706f059d3d6, the extension commit)
# Both use the SAME 04_types carrier (my 4 helpers, no peer arm) so the ONLY difference between the two
# runs is the two arms. That is what makes 'which rows did the extension admit' a measurement rather than
# an impression: the split is computed locally by identity from the two census JSONs
# (instruments/ab_split.py).
#
# ONE CONFIG PER DISPATCH, and each run ships its own artifacts before it can be truncated: a previous
# run did both configs in one dispatch, took 69 minutes, and died with 'rpc DeadlineExceeded' during the
# second self-host, losing that config AND the single end-of-run tar.
#
# THE TEST FILE IS IDENTICAL IN BOTH CONFIGS (taken from the extension commit either way). If each config
# took its own, 'the accept control is green only in B' would be an artefact of two heads rather than a
# measurement of the fold.
#
# WHAT MUST COME BACK, and what each number is for:
#  - rows/distinct per config: the parent's 208-by-identity re-measure, and the size of the admission.
#  - the accepted-row split: rows refusing in A and absent from B's refusals are what the arms admitted.
#    It must be a SUBSET of the 141 rows probe v4 says the predicate answers true for; rows outside that
#    set mean the arm fired somewhere the probe did not predict, and get reported, not explained away.
#  - suite passed/failed: the accept control must go GREEN in B and RED in A, and its refusal twin must
#    stay RED in BOTH. A twin that goes green is the arms over-admitting.
#  - 02_parse rows: re-checked on every head, because X2 was dropped on a census that did not survive a
#    clean clone; if the extension ever makes a parse row appear, that is a regression, not a curiosity.
#  - regen convergence: the arms are new DAG code, so the mirrors must still converge.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
MODE=${1:-}
case "$MODE" in A|B) ;; (*) echo "usage: $0 A|B"; exit 2;; esac
export MODE            # the census python heredoc below reads it; unexported it read '' and produced a
                       # census for a file named census_.json -- a missing value that looks like a zero.
URL=https://github.com/gunb-ai/gunbc.git
BASE_SHA=$(timeout 300 git ls-remote "$URL" refs/heads/main | cut -f1)
# DERIVED, not typed: three dispatches in this lane have already been lost to a mistyped 40-character
# literal, and a typo inside one is invisible to the eye and to grep. Pinning to the tip is also what
# makes 'main' mean the S0 base the parent asked for, rather than a stale SHA I happened to write down.
[ -n "$BASE_SHA" ] || { echo "could not resolve main"; exit 20; }
# Every SHA is resolved from a REF NAME by ls-remote. My first draft of this block hardcoded A_SHA from
# memory and it was simply wrong (8f43f87779... where the real head is a63bfbf5a5...), which the fetch
# would have reported as 'not our ref' after several minutes of build. A 40-character literal cannot be
# proofread by eye, so none are authored here.
resolve() { timeout 300 git ls-remote "$URL" "refs/heads/$1" | cut -f1; }
A_SHA=$(resolve scratch/rebind-census)     # carriers/A_infer_merged.dag + my_04_types_backup + census tool
B_SHA=$(resolve session/gentle-carp-16)   # the fold arms + tests, merged with main at S0
for v in A_SHA B_SHA; do [ -n "${!v}" ] || { echo "could not resolve $v"; exit 20; }; echo "  $v=${!v}"; done
EXT_SHA=$B_SHA                                            # the TESTS are B's, in both configs
CARRIER_SHA=$A_SHA                                        # same commit carries tools/rebind_census.sh
# The old NOEXT_SHA line is gone: it referenced a variable this block no longer defines, and under
# `set -u` a stale reference aborts the script at once rather than silently using an empty value.
if [ "$MODE" = "A" ]; then SRC_SHA=$A_SHA; else SRC_SHA=$B_SHA; fi
echo "CONFIG $MODE: 04_infer.dag from $SRC_SHA ; 04_types/tests/census from $B_SHA (IDENTICAL in both configs, so the ONLY difference between A and B is the two arms in 04_infer.dag)"
D=/tmp/ab_$MODE
rm -rf $D; mkdir -p $D/logs; LOGS=$D/logs; export LOGS   # the census heredoc below reads os.environ['LOGS']
cd $D
timeout 900 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { echo "CLONE FAILED"; tail -5 $LOGS/clone.log; exit 20; }
cd repo
echo "main HEAD at clone=$(git rev-parse HEAD)"
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fb.log 2>&1 || { echo "BASE FETCH FAILED"; exit 25; }
git checkout -q -f FETCH_HEAD || { echo "CHECKOUT FAILED"; exit 26; }
SHA=$(git rev-parse HEAD)
echo "PINNED main=$SHA"
[ "$SHA" = "$BASE_SHA" ] || { echo "WRONG TREE"; exit 27; }
echo "dirty before anything=$(git status --porcelain | wc -l) (want 0)"

# Sources are fetched BY SHA and ASSERTED non-empty: a failed `git show` leaves a zero-byte file, which
# merge-file and cp both read as DATA. This exact trap turned a whole config into a silent wrong answer.
fetch_at() { # $1 sha, $2 log
  timeout 600 git fetch --depth 1 origin "$1" > "$2" 2>&1 || { echo "FETCH FAILED for $1"; tail -3 "$2"; exit 30; }
  git rev-parse FETCH_HEAD
}
S1=$(fetch_at "$SRC_SHA" $LOGS/fs.log)
echo "fetched source $S1 (want $SRC_SHA)"; [ "$S1" = "$SRC_SHA" ] || { echo "WRONG SOURCE"; exit 30; }
if [ "$MODE" = "A" ]; then
  # A's fold source is the carrier file, produced by the SAME 3-way merge as B's (same base, same theirs
  # side) with the pre-extension source on my side, then diffed against B: 74 lines added, 0 removed, all
  # of them my arms/helpers/comments. So A is a strict subset of B and the delta is the arms, not my
  # editing. It is read from the repo, never from a shared /tmp, so the run is reproducible.
  git show "$S1":carriers/A_infer_merged.dag > $LOGS/src_04_infer.dag || { echo "no A carrier"; exit 31; }
else
  git show "$S1":src/v1/04_infer.dag > $LOGS/src_04_infer.dag || { echo "no 04_infer"; exit 31; }
fi
git show "$S1":src/v1/tests/claim/generic_identity_census.dag > src/v1/tests/claim/generic_identity_census.dag || { echo "no census claim"; exit 31; }
S2=$(fetch_at "$EXT_SHA" $LOGS/fe.log)
echo "fetched tests $S2 (want $EXT_SHA)"; [ "$S2" = "$EXT_SHA" ] || { echo "WRONG TESTS SHA"; exit 30; }
git show "$S2":src/v1/tests/src/cross_coproduct_generic_locate_test.rs > src/v1/tests/src/cross_coproduct_generic_locate_test.rs || { echo "no fold test"; exit 31; }
# THE MODULE DECLARATION IS PART OF THE TEST. main's src/v1/tests/src/lib.rs has NO
# 'mod cross_coproduct_generic_locate_test' line, so installing only the .rs file yields a test that does
# not exist: cargo compiled 52 other tests, filtered them all, and reported 'ok' -- zero tests run.
git show "$S2":src/v1/tests/src/lib.rs > src/v1/tests/src/lib.rs || { echo "no tests lib.rs"; exit 31; }
[ "$(grep -ac 'mod cross_coproduct_generic_locate_test' src/v1/tests/src/lib.rs)" = "1" ] \
  || { echo "lib.rs does not declare the fold test module -- the suite would silently run zero tests"; exit 31; }
S3=$(fetch_at "$CARRIER_SHA" $LOGS/fc.log)
echo "fetched carriers $S3 (want $CARRIER_SHA)"; [ "$S3" = "$CARRIER_SHA" ] || { echo "WRONG CARRIER"; exit 32; }
git show "$S2":src/v1/04_types.dag > $LOGS/my_04_types.dag || { echo "no 04_types from B"; exit 32; }
git show "$S3":tools/rebind_census.sh > $LOGS/rebind_census.sh || { echo "no census tool"; exit 32; }
for f in $LOGS/src_04_infer.dag $LOGS/my_04_types.dag $LOGS/rebind_census.sh \
         src/v1/tests/claim/generic_identity_census.dag \
         src/v1/tests/src/cross_coproduct_generic_locate_test.rs; do
  [ -s "$f" ] || { echo "EMPTY FILE from a failed git show: $f -- abort (this is the silent-wrong-answer trap)"; exit 33; }
done
echo "sizes: infer=$(wc -c < $LOGS/src_04_infer.dag) types=$(wc -c < $LOGS/my_04_types.dag)"

# THE DIFFERENCE BETWEEN THE CONFIGS, asserted rather than assumed. Counted as ARMS ('if binding_...'),
# not as the bare name: the fn definition also matches '(n: ', so a bare-name count gave 3 in B and would
# have failed B's own label check at 3 vs 2.
ARMS=$(grep -ac 'if binding_is_generic_declaration_own_form(n: ' $LOGS/src_04_infer.dag)
if [ "$MODE" = "A" ]; then WANT=0; else WANT=2; fi
echo "arms in the installed source=$ARMS (want $WANT)"
[ "$ARMS" = "$WANT" ] || { echo "CONFIG MISLABELLED: expected $WANT arms, got $ARMS"; exit 34; }
echo "tests: fold_decl=$(grep -ac 'fn fold_decl' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) acceptControl=$(grep -ac 'fn fold_replaces_a_binding_that_is_the_generic_declarations_own_form' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) twin=$(grep -ac 'fn fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) deletedRow=$(grep -ac 'fn fold_accepts_disj_and_applied_spellings' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) (want 1/1/1/0)"
echo "claim census: calls=$(grep -ac 'unify_generics(' src/v1/tests/claim/generic_identity_census.dag) withText=$(grep -ac 'text: ' src/v1/tests/claim/generic_identity_census.dag) (want 2 and 2; 0 withText = main's copy = the regen refusal I already paid 9 minutes for)"

# PAIRING CHECK, before any build. For every function main's 04_types.dag defines, if the 04_infer.dag we
# are about to install CALLS it, the 04_types.dag we are about to install must DEFINE it. This is the class
# of error that just cost 9 minutes: a main-merged 04_infer under a pre-S0 04_types, which the regen only
# reports as 81 'not found in scope' diagnostics after claim_executor has been built. Generic over S0's
# additions rather than naming them, so the next function main adds is caught the same way.
git show "$BASE_SHA":src/v1/04_types.dag > $LOGS/base_04_types.dag 2>/dev/null || git show HEAD:src/v1/04_types.dag > $LOGS/base_04_types.dag
ORPHANS=""
for fn in $(grep -oE '^fn [a-z0-9_]+' $LOGS/base_04_types.dag | awk '{print $2}' | sort -u); do
  if grep -qE "(^|[^A-Za-z0-9_])$fn\\(" $LOGS/src_04_infer.dag; then
    grep -qE "^fn $fn\\(" $LOGS/my_04_types.dag || ORPHANS="$ORPHANS $fn"
  fi
done
if [ -n "$ORPHANS" ]; then
  echo "PAIRED FILES ARE FROM DIFFERENT GENERATIONS -- 04_infer.dag calls functions 04_types.dag does not define:"
  for fn in $ORPHANS; do echo "   MISSING: $fn"; done
  echo "   (this is the pre-S0-carrier bug: A must use the SAME 04_types as B; fix the source pair, do not proceed)"
  echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
  exit 6
fi
echo "pairing check: every function called by the installed 04_infer.dag is defined by the installed 04_types.dag"
cp $LOGS/src_04_infer.dag src/v1/04_infer.dag
cp $LOGS/my_04_types.dag src/v1/04_types.dag
git checkout -q -f -- src/v1/stage0 2>/dev/null || true

cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -3
ROUND=1
INST=1
while [ $ROUND -le 3 ] && [ "$INST" != "0" ]; do
  timeout 2400 target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr_${MODE}_${ROUND}.log 2>&1
  echo "=== regen round $ROUND rc=$?"
  grep -a "^required-regen" $LOGS/rr_${MODE}_${ROUND}.log | head -2 | cut -c1-300
  if [ ! -d target/stage0-regen-candidate/src ]; then
    echo "NO CANDIDATE: the receipt is the answer. Hard diagnostics verbatim:"
    grep -aE "^type mismatch|^call shape|arm decides|not found in|no field|effect summary" $LOGS/rr_${MODE}_${ROUND}.log | head -14 | cut -c1-300
    echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
    exit 7
  fi
  INST=0
  for f in $(ls target/stage0-regen-candidate/src 2>/dev/null); do
    if [ -f src/v1/stage0/src/$f ]; then
      if ! cmp -s target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f; then
        cp target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f
        INST=$((INST+1))
        echo "   installed $f"
      fi
    else
      echo "   CANDIDATE NOT IN SEED (reported, never committed): $f"
    fi
  done
  echo "   round $ROUND installed=$INST"
  ROUND=$((ROUND+1))
done
echo "CONVERGED at round=$ROUND SEED sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"

# THE ARMS MUST BE IN THE EMITTED SEED, or the census measures the wrong build. Counted as occurrences
# minus definitions, because the emitter's argument-passing syntax is its own business and guessing it
# would make a good build look mislabelled.
FN=$(grep -ac 'fn binding_is_generic_declaration_own_form' src/v1/stage0/src/v1_compiler_infer.rs)
ALL=$(grep -ac 'binding_is_generic_declaration_own_form' src/v1/stage0/src/v1_compiler_infer.rs)
CALLS=$((ALL - FN))
echo "emitted seed: fold=$(grep -ac UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) refusal=$(grep -ac '(first argument)' src/v1/stage0/src/v1_compiler_infer.rs) definitions=$FN calls=$CALLS"
if [ "$MODE" = "A" ]; then
  if [ "$FN" != "0" ] || [ "$CALLS" != "0" ]; then echo "CONFIG A HAS THE ARMS IN THE SEED -- the comparison would be meaningless"; exit 12; fi
else
  if [ "$FN" -lt 1 ] || [ "$CALLS" -lt 2 ]; then echo "THE ARMS ARE NOT IN THE EMITTED SEED (want >=1 def, >=2 calls) -- a census here measures the wrong build"; exit 12; fi
fi

# THE SUITE LIVES IN v1-compiler-tests AS A LIB MODULE (src/v1/tests/src/lib.rs:30), not as a --test
# target of v1-compiler. My first attempt said `cargo test -p v1-compiler --test
# cross_coproduct_generic_locate_test`, failed with 'no test target named ...', and my own
# `grep -E "^error|Finished"` filter HID the error -- the run reported rc=101 with an empty suite
# section and looked like a suite that passed nothing. Invocation copied verbatim from cleanab2, the run
# that produced 24/3 at this tree.
echo "--- SUITE (compiled against THIS config's regenerated mirror, so it tests this seed) ---"
cargo build --release -p v1-compiler-tests --tests > $LOGS/tb_${MODE}.log 2>&1
echo "test build rc=$?"
grep -E "^error|error\[" $LOGS/tb_${MODE}.log | head -6 | cut -c1-200
timeout 1200 cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 \
  | grep -E "^test |test result|^error\[|panicked|assertion" \
  | sed -E 's/^test cross_coproduct_generic_locate_test:://' > $LOGS/suite_${MODE}.txt
if [ ! -s $LOGS/suite_${MODE}.txt ]; then
  echo "SUITE PRODUCED NO OUTPUT: FAILURE, not a pass"
  tail -15 $LOGS/tb_${MODE}.log
  echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
  exit 8
fi
OKN=$(grep -c 'ok$' $LOGS/suite_${MODE}.txt)
BADN=$(grep -c FAILED $LOGS/suite_${MODE}.txt)
echo "suite passed=$OKN failed=$BADN"
# A suite that runs ZERO tests is not a pass. Config A produced '0 passed; 0 failed; 52 filtered out'
# because cargo compiles the whole test lib and my filter selects one module -- and if the module is not
# DECLARED in src/v1/tests/src/lib.rs it simply does not exist, so everything is filtered out and the run
# reports ok. Installing lib.rs is therefore part of installing the tests, and N>0 is a gate, not a log line.
if [ "$((OKN + BADN))" -eq 0 ]; then
  echo "SUITE RAN ZERO TESTS -- VOID, not a pass. Is 'mod cross_coproduct_generic_locate_test' declared in src/v1/tests/src/lib.rs? (in this tree: $(grep -ac 'mod cross_coproduct_generic_locate_test' src/v1/tests/src/lib.rs))"
  tail -8 $LOGS/tb_${MODE}.log
  echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
  exit 8
fi
grep FAILED $LOGS/suite_${MODE}.txt | cut -c1-100 | sed 's/^/   FAIL /'
grep -aE '^test result' $LOGS/suite_${MODE}.txt | head -3 | cut -c1-170
# The two rows that must differ between configs, printed explicitly so a mislabel is visible rather than
# buried in a pass count: the accept control must be GREEN only in B; the twin must be RED in BOTH.
# Three controls, each with a STATED expectation per config, so a wrong outcome is visible at a glance
# rather than being read out of a pass count afterwards:
#   accept control  -- MUST be ok in B and FAILED in A (without the arms the pair still refuses, which is
#                      the RED-before that makes the arm's value checkable at all)
#   fold twin        -- MUST be FAILED in BOTH: two distinct settled instances still refuse, whatever the
#                      arms do. Green anywhere means the arms over-admit.
#   S0 relation row  -- MUST be ok in BOTH: since S0 deleted the mixed application-vs-non-application
#                      acceptance, Box<Int> vs a Disj Box<Int> refuses at relation grain in both configs,
#                      because neither of them owns the relation.
for t in fold_replaces_a_binding_that_is_the_generic_declarations_own_form \
         fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance \
         fold_refuses_an_applied_spelling_against_a_non_application_spelling_of_one_name; do
  L=$(grep -a "^test $t" $LOGS/suite_${MODE}.txt | head -1)
  echo "  CONTROL $t -> ${L:-ROW ABSENT FROM THE SUITE OUTPUT}"
  case "$t" in
    fold_replaces_a_binding_that_is_the_generic_declarations_own_form)
      if [ "$MODE" = "B" ]; then WANTP=" ok"; else WANTP="FAILED"; fi ;;
    *) WANTP="" ;;
  esac
  # A MISMATCH IS REPORTED LOUDLY, NOT FATAL. The hard gates above (zero tests, missing row) are 'no data'
  # gates and must abort. This one is a 'the data surprised me' gate: if the accept control is already ok
  # in A, some EARLIER arm in unify_generics admits that pair and my control is VACUOUS -- a finding to
  # report, not a reason to destroy the self-host census that takes 20+ minutes and is the measurement the
  # parent is actually waiting on. Killing a 35-minute run over an expectation I might have got wrong
  # would be the worse error.
  if [ -n "$WANTP" ]; then
    case "$L" in
      *"$WANTP") echo "    (as expected: '${WANTP}')" ;;
      *) echo "    *** CONTROL EXPECTATION MISFIRED: expected '${WANTP}' in config $MODE, got '${L}' ***";
         echo "    (not fatal: an earlier arm in unify_generics may already admit this pair, which would make";
         echo "     this control vacuous. REPORTED, and reported onward, rather than silently accepted.)";;
    esac
  fi
  # Absent is a FAILURE. A control that never ran cannot support 'the accept control is green only in B',
  # and the first version of this script printed the note and carried on, which is how a config with ZERO
  # controls compiled would have produced a clean-looking A/B.
  case "$L" in
    *" ok") ;;
    *FAILED) ;;
    *) echo "CONTROL ROW MISSING: $t -- the suite did not run this test; refusing to report a delta";
       echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="; exit 8;;
  esac
done

echo "=== BEGIN-TAR ==="
( cd $LOGS && tar -c suite_${MODE}.txt tb_${MODE}.log rr_${MODE}_1.log rr_${MODE}_2.log src_04_infer.dag my_04_types.dag 2>/dev/null | base64 -w0 )
echo
echo "=== END-TAR ==="
echo "PARTIAL SHIP DONE (suite + regen survive a timeout in the self-host below)"

# THE BINARY MUST BE BUILT BEFORE IT IS RUN. The first run of this script went straight to
# `target/release/gunbc test` and got rc=127 'No such file or directory': cargo test does not produce
# that binary, and a 127 with an empty census is the same 'looks like data, is nothing' trap as an empty
# suite. cleanab2 builds it explicitly; I dropped that line when deriving this script.
echo "--- SELF-HOST ---"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -2
if [ ! -x target/release/gunbc ]; then
  echo "gunbc MISSING after build -- self-host cannot run; shipping logs"
  echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
  exit 9
fi
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh_${MODE}.log 2>&1
RC=$?
echo "self-host rc=$RC rebind=$(grep -ac 'bound as' $LOGS/sh_${MODE}.log) files=$(grep -aoE 'src/[a-z0-9/_.]+\.dag' $LOGS/sh_${MODE}.log | sort -u | wc -l)"
if [ ! -s $LOGS/sh_${MODE}.log ]; then
  echo "SELF-HOST LOG EMPTY at rc=$RC -- nothing to census; shipping logs"
  echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
  exit 9
fi
bash $LOGS/rebind_census.sh $LOGS/sh_${MODE}.log --json $LOGS/census_${MODE}.json > $LOGS/census_${MODE}.txt 2>&1
echo "census rc=$?"
grep -a "total rows=\|rows parsed\|classes:" $LOGS/census_${MODE}.txt | head -3 | cut -c1-300
python3 - <<'PY'
import json, os, collections
rows = json.load(open(os.path.join(os.environ['LOGS'], 'census_%s.json' % os.environ['MODE'])))
ident = lambda r: (r.get('formal', ''), r.get('actual', ''), r.get('span', ''))
print("  rows=%d distinct=%d" % (len(rows), len({ident(r) for r in rows})))
byfile = collections.Counter((r.get('span', '') or '<no span>').split(':')[0] for r in rows)
print("  rows by file:", dict(byfile.most_common(12)))
print("  02_parse rows=%d (any nonzero is a REGRESSION of this change, report it)"
      % len([r for r in rows if '02_parse' in (r.get('span', '') or '')]))
PY
echo "=== BEGIN-TAR ==="
( cd $LOGS && tar -c sh_${MODE}.log census_${MODE}.json census_${MODE}.txt suite_${MODE}.txt tb_${MODE}.log rr_${MODE}_1.log rr_${MODE}_2.log src_04_infer.dag my_04_types.dag 2>/dev/null | base64 -w0 )
echo
echo "=== END-TAR ==="
echo "AB_${MODE}_DONE"
