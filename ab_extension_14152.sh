#!/bin/bash
# EFFECT OF THE 'binding IS the generic declaration's own form' ARMS, measured by identity.
#   usage: bash ab_extension_14152.sh A|B
#
# A = the fold WITHOUT the new arms  (04_infer.dag at a8e23ad9db5, the durability tip)
# B = the fold WITH them             (04_infer.dag at 706f059d3d6, the extension commit)
# Both use the SAME 04_types carrier (my 4 helpers, no peer arm) so the ONLY difference between the
# two runs is the two arms. That is what makes 'which rows did the extension admit' a measurement
# rather than an impression: the split is computed locally by identity from the two census JSONs.
#
# ONE CONFIG PER DISPATCH, and each run tars its own artifacts before it can be truncated: a previous
# run did both configs in one dispatch, took 69 minutes, and died with 'rpc DeadlineExceeded' during
# the second self-host, losing that config AND the single end-of-run tar.
#
# WHAT MUST COME BACK, and what each number is for:
#  - rows/distinct per config: the parent's 208-by-identity re-measure, and the size of the admission.
#  - the accepted-row split: rows refusing in A and NOT in B are what the arms admitted. It must be a
#    SUBSET of the 142 predicate rows (122 conn-class + 20 out-of-class); rows outside that set mean
#    the arm fired somewhere the probe did not predict, and gets reported, not explained away.
#  - suite passed/failed: the accept control must go GREEN in B and RED in A, and its refusal twin must
#    stay RED in BOTH. A twin that goes green is the arms over-admitting.
#  - 02_parse rows: re-checked every head, because X2 was dropped on a census that did not survive a
#    clean clone; if the extension ever makes a parse row appear, that is a regression, not a curiosity.
#  - regen convergence: the arms are new DAG code, so the mirrors must still converge at round 2.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
MODE=${1:-}; case "$MODE" in A|B) ;; (*) echo "usage: $0 A|B"; exit 2;; esac
BASE_SHA=59606870e8ba3001a845627696509e8aa16bb0f0        # main
NOEXT_SHA=$(cd /home/briansrls/.worktrees/gunbc/gentle-carp-16 2>/dev/null && git rev-parse a8e23ad9db5 || echo a8e23ad9db56c29781789004102e57f91806a3f4)
EXT_SHA=706f059d3d606652171fea45940a64322c37c00b         # the extension commit, pushed
CARRIER_SHA=9531e67c97a0ff4477a6db251322053d864dcae6     # scratch: carriers + tools
[ "$MODE" = "A" ] && SRC_SHA=$NOEXT_SHA || SRC_SHA=$EXT_SHA
echo "CONFIG $MODE: 04_infer.dag from $SRC_SHA"
URL=https://github.com/gunb-ai/gunbc.git
D=/tmp/ab_$MODE; rm -rf $D; mkdir -p $D/logs; LOGS=$D/logs
cd $D
timeout 900 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { echo "CLONE FAILED"; tail -5 $LOGS/clone.log; exit 20; }
cd repo
echo "main HEAD at clone=$(git rev-parse HEAD)"
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fb.log 2>&1 || { echo "BASE FETCH FAILED"; exit 25; }
git checkout -q -f FETCH_HEAD || { echo "CHECKOUT FAILED"; exit 26; }
SHA=$(git rev-parse HEAD); echo "PINNED main=$SHA"; [ "$SHA" = "$BASE_SHA" ] || { echo "WRONG TREE"; exit 27; }
echo "dirty before anything=$(git status --porcelain | wc -l) (want 0)"

# Sources are fetched BY SHA and ASSERTED non-empty: a failed `git show` leaves a zero-byte file, which
# merge-file and cp both read as data. This exact trap turned a whole config into a silent wrong answer.
timeout 600 git fetch --depth 1 origin $SRC_SHA > $LOGS/fs.log 2>&1 || { echo "SRC FETCH FAILED for $SRC_SHA"; tail -3 $LOGS/fs.log; exit 30; }
git show FETCH_HEAD:src/v1/04_infer.dag > $LOGS/src_04_infer.dag || { echo "no 04_infer"; exit 31; }
git show FETCH_HEAD:src/v1/tests/claim/generic_identity_census.dag > src/v1/tests/claim/generic_identity_census.dag || { echo "no census claim"; exit 31; }
# THE TEST FILE COMES FROM THE EXTENSION COMMIT IN BOTH CONFIGS. If each config took its own, config A
# would simply not contain the new rows and 'the accept control is green only in B' would be an artefact
# of two different heads rather than a measurement of the fold. One test file, one difference: the arms.
if [ "$MODE" = "B" ]; then
  git show FETCH_HEAD:src/v1/tests/src/cross_coproduct_generic_locate_test.rs > src/v1/tests/src/cross_coproduct_generic_locate_test.rs || { echo "no fold test"; exit 31; }
else
  timeout 600 git fetch --depth 1 origin $EXT_SHA > $LOGS/fe.log 2>&1 || { echo "EXT FETCH FAILED"; tail -3 $LOGS/fe.log; exit 30; }
  git show FETCH_HEAD:src/v1/tests/src/cross_coproduct_generic_locate_test.rs > src/v1/tests/src/cross_coproduct_generic_locate_test.rs || { echo "no fold test"; exit 31; }
fi
timeout 600 git fetch --depth 1 origin $CARRIER_SHA > $LOGS/fc.log 2>&1 || { echo "CARRIER FETCH FAILED"; exit 32; }
CAR=$(git rev-parse FETCH_HEAD); [ "$CAR" = "$CARRIER_SHA" ] || { echo "WRONG CARRIER $CAR"; exit 32; }
git show $CAR:carriers/my_04_types_backup.dag > $LOGS/my_04_types.dag || { echo "no 04_types carrier"; exit 32; }
git show $CAR:tools/rebind_census.sh > $LOGS/rebind_census.sh || { echo "no census tool"; exit 32; }
for f in $LOGS/src_04_infer.dag $LOGS/my_04_types.dag $LOGS/rebind_census.sh src/v1/tests/claim/generic_identity_census.dag src/v1/tests/src/cross_coproduct_generic_locate_test.rs; do
  [ -s "$f" ] || { echo "EMPTY FILE from a failed git show: $f -- abort (this is the silent-wrong-answer trap)"; exit 33; }
done
echo "sizes: infer=$(wc -c < $LOGS/src_04_infer.dag) types=$(wc -c < $LOGS/my_04_types.dag)"
# THE DIFFERENCE BETWEEN THE CONFIGS, asserted rather than assumed.
echo "arms in the installed source: ownFormArms=$(grep -ac 'if binding_is_generic_declaration_own_form(n: ' $LOGS/src_04_infer.dag) (want 0 in A, 2 in B)"
[ "$MODE" = "A" ] && WANT=0 || WANT=2
[ "$(grep -ac 'if binding_is_generic_declaration_own_form(n: ' $LOGS/src_04_infer.dag)" = "$WANT" ] || { echo "CONFIG MISLABELLED: expected $WANT arms"; exit 34; }
# The test file must be the one that goes with the source, or 'control goes green' measures the wrong head.
echo "tests: fold_decl=$(grep -ac 'fn fold_decl' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) acceptControl=$(grep -ac 'fn fold_replaces_a_binding_that_is_the_generic_declarations_own_form' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) twin=$(grep -ac 'fn fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) deletedRow=$(grep -ac 'fn fold_accepts_disj_and_applied_spellings' src/v1/tests/src/cross_coproduct_generic_locate_test.rs) (want 1/1/1/0)"
echo "claim census: calls=$(grep -ac 'unify_generics(' src/v1/tests/claim/generic_identity_census.dag) withText=$(grep -ac 'text: ' src/v1/tests/claim/generic_identity_census.dag) (want 2 and 2)"

cp $LOGS/src_04_infer.dag src/v1/04_infer.dag
cp $LOGS/my_04_types.dag src/v1/04_types.dag
git checkout -q -f -- src/v1/stage0 2>/dev/null || true

cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -3
r=1; inst=1
while [ $r -le 3 ] && [ "$inst" != "0" ]; do
  timeout 2400 target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr_${MODE}_${r}.log 2>&1
  echo "=== regen round $r rc=$?"
  grep -a "^required-regen" $LOGS/rr_${MODE}_${r}.log | head -2 | cut -c1-300
  if [ ! -d target/stage0-regen-candidate/src ]; then
    echo "NO CANDIDATE: the receipt is the answer. Hard diagnostics verbatim:"
    grep -aE "^type mismatch|^call shape|arm decides|not found in|no field|effect summary" $LOGS/rr_${MODE}_${r}.log | head -14 | cut -c1-300
    tar -cC $LOGS . 2>/dev/null | base64 -w0; echo; echo "=== END-TAR ==="
    exit 7
  fi
  inst=0
  for f in $(ls target/stage0-regen-candidate/src 2>/dev/null); do
    if [ -f src/v1/stage0/src/$f ]; then
      cmp -s target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f || { cp target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f; inst=$((inst+1)); echo "   installed $f"; }
    else echo "   CANDIDATE NOT IN SEED (reported, never committed): $f"; fi
  done
  echo "   round $r installed=$inst"; r=$((r+1))
done
echo "CONVERGED at round=$r SEED sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"
echo "SEED PROOF: fold=$(grep -ac UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) refusal=$(grep -ac '(first argument)' src/v1/stage0/src/v1_compiler_infer.rs) ownFormFn=$(grep -ac 'fn binding_is_generic_declaration_own_form' src/v1/stage0/src/v1_compiler_infer.rs) ownFormCall=$(grep -ac 'binding_is_generic_declaration_own_form' src/v1/stage0/src/v1_compiler_infer.rs)"
# Count CALLS as occurrences minus definitions, rather than asserting on an emitted call syntax I have
# not seen: the emitter's argument passing (&&, &, clone) is its own business and guessing it would make
# a good build look mislabelled.
FN=$(grep -ac 'fn binding_is_generic_declaration_own_form' src/v1/stage0/src/v1_compiler_infer.rs)
CALLS=$(( $(grep -ac 'binding_is_generic_declaration_own_form' src/v1/stage0/src/v1_compiler_infer.rs) - FN ))
echo "emitted seed: definitions=$FN calls=$CALLS"
if [ "$MODE" = "A" ]; then
  [ "$FN" = "0" ] && [ "$CALLS" = "0" ] || { echo "CONFIG A HAS THE ARMS IN THE SEED -- the comparison would be meaningless"; exit 12; }
else
  [ "$FN" -ge 1 ] && [ "$CALLS" -ge 2 ] || { echo "THE ARMS ARE NOT IN THE EMITTED SEED (want >=1 def, >=2 calls) -- a census here measures the wrong build"; exit 12; }
fi

timeout 1800 cargo build --release -p v1-compiler --test cross_coproduct_generic_locate_test > $LOGS/suite_${MODE}.txt 2>&1
echo "suite build rc=$?"
timeout 1200 cargo test --release -p v1-compiler --test cross_coproduct_generic_locate_test >> $LOGS/suite_${MODE}.txt 2>&1
echo "suite rc=$?"
grep -aE '^test result|^test fold|FAILED' $LOGS/suite_${MODE}.txt | head -30 | sed 's/^/  /' | cut -c1-170
# The two rows that must differ between configs, printed explicitly so a mislabel is visible rather than
# buried in a pass count: the accept control must be GREEN only in B; the twin must be RED in BOTH.
for t in fold_replaces_a_binding_that_is_the_generic_declarations_own_form fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance; do
  printf "  CONTROL %-72s %s\n" "$t" "$(grep -a "^test $t" $LOGS/suite_${MODE}.txt | head -1 | cut -c1-60)"
done

timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh_${MODE}.log 2>&1
echo "self-host rc=$? rebind=$(grep -ac 'bound as' $LOGS/sh_${MODE}.log) files=$(grep -aoE 'src/[a-z0-9/_.]+\.dag' $LOGS/sh_${MODE}.log | sort -u | wc -l)"
bash $LOGS/rebind_census.sh $LOGS/sh_${MODE}.log --json $LOGS/census_${MODE}.json > $LOGS/census_${MODE}.txt 2>&1
echo "census rc=$?"; grep -a "total rows=\|classes:" $LOGS/census_${MODE}.txt | head -2 | cut -c1-300
python3 - <<'PY'
import json,os,collections
logs=os.environ.get('LOGS','')
rows=json.load(open(os.path.join(logs,'census_%s.json'%os.environ['MODE'])))
ident=lambda r:(r.get('formal',''),r.get('actual',''),r.get('span',''))
print("  rows=%d distinct=%d"%(len(rows),len({ident(r) for r in rows})))
byfile=collections.Counter((r.get('span','') or '<no span>').split(':')[0] for r in rows)
print("  rows by file:",dict(byfile.most_common(12)))
print("  02_parse rows=%d (any nonzero is a REGRESSION of this change, report it)"%len([r for r in rows if '02_parse' in (r.get('span','') or '')]))
PY
echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c sh_${MODE}.log census_${MODE}.json census_${MODE}.txt suite_${MODE}.txt rr_${MODE}_1.log rr_${MODE}_2.log src_04_infer.dag my_04_types.dag 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
echo "AB_${MODE}_DONE"
