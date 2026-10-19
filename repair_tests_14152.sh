#!/usr/bin/env bash
# Run the #14152 fold/collision rows against a COMMITTED head -- no patching, no
# regeneration: the tree at $SHA is exactly what would ship, so what is measured is what
# ships. Fresh clone at a printed SHA; the shared /root/workspace/repo-root is never used
# (it is dirty with other sessions' .dag edits and produced a fake drift result in this lane).
#
# The cargo target dir lives under /root, which persists between dispatches (unlike /tmp,
# which is recreated -- a detached run there died with its container). So this dispatch
# reuses whatever the regeneration dispatch already compiled.
set -u
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
SHA="${1:?usage: repair_tests_14152.sh <sha> [label]}"
LABEL="${2:-head}"
URL=https://github.com/gunb-ai/gunbc
D=/tmp/repair_tests
LOG="$D/test_$LABEL.log"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_14152}"
mkdir -p "$CARGO_TARGET_DIR"
CARGO_TIMEOUT="${CARGO_TIMEOUT:-1500}"

ROWS='collision_two_zero_arg_generic_calls_both_type
collision_pair_still_refuses_when_instantiated_twice
fold_accepts_a_binding_whose_arguments_are_all_fresh_call_variables
fold_replaces_a_binding_that_is_the_generic_declarations_own_form
fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance
fold_replaces_a_binding_that_still_carries_a_type_variable_leaf
fold_keeps_a_settled_binding_against_an_actual_that_still_carries_a_type_variable
fold_refuses_two_distinct_settled_bindings_of_the_same_shape
fold_refuses_int_bound_then_bool'

stamp(){ echo "T=$(date -u +%H:%M:%S) $*"; }
seed(){ sha256sum "$1" | cut -c1-16; }

stamp "env: nproc=$(nproc) target=$(du -sh "$CARGO_TARGET_DIR" 2>/dev/null | cut -f1 || echo none)"
rm -rf "$D"; mkdir -p "$D"
stamp "clone begin"
timeout 600 git clone -q "$URL" "$D/repo" || { echo "FATAL clone"; exit 1; }
cd "$D/repo" || { echo "FATAL cd"; exit 1; }
timeout 300 git fetch -q origin "$SHA" || { echo "FATAL fetch"; exit 1; }
timeout 120 git checkout -q "$SHA" || { echo "FATAL checkout"; exit 1; }
stamp "clone done MEASURED_AT $(git rev-parse HEAD)"
git status --porcelain | grep -v '^??' | head -3 | sed 's/^/DIRTY /'
sha256sum src/v1/04_infer.dag src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/SEED /'
# The thing under test must actually be present, or the rows would pass vacuously.
grep -c "UnifyGenericsAcc" src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/FOLD_REFS /'
grep -c "fresh_call_variables" src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/REPAIR_MARKS /'
grep -c "^#\[test\]" src/v1/tests/src/cross_coproduct_generic_locate_test.rs | sed 's/^/TESTS_IN_FILE /'

stamp "compile gate (--no-run: catches a broken row in minutes, not after the whole run)"
timeout -k 30 "$CARGO_TIMEOUT" cargo test -p v1-compiler-tests --lib cross_coproduct --no-run >"$D/syntax.log" 2>&1
SYN_RC=$?
echo "SYNTAX_RC=$SYN_RC"
grep -E "^error(\[|:)" "$D/syntax.log" | head -12 | sed 's/^/SYNTAX_ERROR /'
if [ "$SYN_RC" != 0 ]; then
  echo "ABORT_SYNTAX"
  echo "BEGIN-LOG syntax.log $(wc -c <"$D/syntax.log")"; base64 -w0 "$D/syntax.log"; echo; echo "END-LOG syntax.log"
  exit 3
fi

stamp "rows begin"
timeout -k 30 "$CARGO_TIMEOUT" cargo test -j "$(nproc)" -p v1-compiler-tests --lib cross_coproduct -- --nocapture >"$LOG" 2>&1
rc=$?
stamp "rows done rc=$rc (124 = my own bound fired, not the runner's)"

for r in $ROWS; do
  if   grep -qE "^test .*${r} \.\.\. ok$"     "$LOG"; then echo "ROW $LABEL $r PASS"
  elif grep -qE "^test .*${r} \.\.\. FAILED$" "$LOG"; then echo "ROW $LABEL $r FAIL"
  else echo "ROW $LABEL $r MISSING_FATAL"; fi
done
grep -E "^test result:" "$LOG" | tail -1 | sed "s/^/SUMMARY $LABEL /"
grep -E "^error(\[|:)" "$LOG" | head -8 | sed 's/^/COMPILE_ERROR /'

# A FAILING row's message is the measurement (the collision row's expected pre-repair
# diagnostic, the accept row's binding dump). Print it rather than re-deriving it.
echo "BEGIN-FAILURE-DETAIL"
awk '/panicked at|assertion failed|must both type|must be replaced|must refuse|is unsettled/' "$LOG" | head -40 | cut -c1-320
echo "END-FAILURE-DETAIL"

echo "BEGIN-TESTLOG $(wc -c <"$LOG")"; base64 -w0 "$LOG"; echo; echo "END-TESTLOG"
stamp "ALLDONE"
