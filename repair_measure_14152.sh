#!/usr/bin/env bash
# Measured run for the #14152 producer repair -- part 1: RED baseline + the shipped
# (regenerated) repair. FRESH CLONE at a printed SHA, remote runner.
#
# WHY THE MIRROR AND NOT THE .dag IS WHAT GETS MEASURED:
#   src/v1/stage0/build.rs does NOT regenerate from .dag -- it only panics when the mirror
#   is missing. The tests import v1_compiler::v1_compiler_infer::unify_generics, which is
#   the CHECKED-IN MIRROR src/v1/stage0/src/v1_compiler_infer.rs. So patching 04_infer.dag
#   alone changes nothing a cargo test can see: that is the StaleBinary trap. Therefore:
#     * the repair is measured on the mirror REGENERATED from the patched .dag (stage B2/B3)
#       -- regeneration is the only sanctioned way a mirror changes;
#     * the pristine pre-repair baseline is measured on the mirror as committed (stage A);
#     * the mutations (name-keyed, withdrawn) are applied to that SAME generated mirror, so
#       they are true mutations of the shipped code, and are never committed.
#   Stage B1 prints the emitted Rust text of the seeded region so the mutation patcher can
#   be written against what the emitter really produces instead of guessed at.
#
# Per-row verdicts are printed by name; an absent row is MISSING_FATAL, never a pass. rc is
# taken from the command, never from a pipe.
set -u
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
SHA="${1:?usage: repair_measure_14152.sh <sha>}"
D="${REPO_DIR:-/tmp/repair_14152}"
URL=https://github.com/gunb-ai/gunbc
INSTR_REF="scratch/rebind-census"
LOGS=/tmp/repair_14152_logs
INFER_DAG=src/v1/04_infer.dag
MIRROR=src/v1/stage0/src/v1_compiler_infer.rs

ROWS='collision_two_zero_arg_generic_calls_both_type
collision_pair_still_refuses_when_instantiated_twice
fold_accepts_a_binding_whose_arguments_are_all_fresh_call_variables
fold_replaces_a_binding_that_is_the_generic_declarations_own_form
fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance
fold_replaces_a_binding_that_still_carries_a_type_variable_leaf
fold_keeps_a_settled_binding_against_an_actual_that_still_carries_a_type_variable
fold_refuses_two_distinct_settled_bindings_of_the_same_shape
fold_refuses_int_bound_then_bool'

if [ "${1:-}" = "--selfcheck" ]; then
  echo "row-count=$(printf '%s\n' $ROWS | wc -l)"
  test -f "$(dirname "$0")/patch_producer_instantiate.py" || { echo "SELFCHECK FAIL patcher"; exit 1; }
  test -f "$(dirname "$0")/instruments/regen_full.sh" || { echo "SELFCHECK FAIL regen"; exit 1; }
  echo "SELFCHECK OK"; exit 0
fi

say(){ echo "=== $* ==="; }
seed(){ sha256sum "$1" | cut -c1-16; }

testrows(){ # testrows <label>
  local label="$1" log="$LOGS/test_$label.log"
  echo "--- BEGIN-TESTS $label"
  cargo test -p v1-compiler-tests --lib cross_coproduct >"$log" 2>&1
  local rc=$?
  echo "--- TESTS $label rc=$rc"
  local r
  for r in $ROWS; do
    if   grep -qE "^test .*${r} \.\.\. ok$"     "$log"; then echo "ROW $label $r PASS"
    elif grep -qE "^test .*${r} \.\.\. FAILED$" "$log"; then echo "ROW $label $r FAIL"
    else echo "ROW $label $r MISSING_FATAL"; fi
  done
  grep -E "^test result:" "$log" | tail -1 | sed "s/^/SUMMARY $label /"
  grep -E "^error(\[|:)" "$log" | head -5 | sed "s/^/COMPILE_ERROR $label /"
  return $rc
}

say "cargo location"
command -v cargo || { echo "FATAL no cargo on PATH"; exit 1; }

say "clone"
rm -rf "$D" "$LOGS"; mkdir -p "$LOGS"
timeout 900 git clone -q "$URL" "$D/repo" || { echo "FATAL clone"; exit 1; }
cd "$D/repo" || { echo "FATAL cd"; exit 1; }
git fetch -q origin "$SHA" || { echo "FATAL fetch $SHA"; exit 1; }
git checkout -q "$SHA" || { echo "FATAL checkout"; exit 1; }
git fetch -q origin "$INSTR_REF" || { echo "FATAL fetch instr"; exit 1; }
echo "MEASURED_AT $(git rev-parse HEAD)"
git status --porcelain | grep -v '^??' | head -3 | sed 's/^/DIRTY /'
mkdir -p "$D/instr"
git archive "origin/$INSTR_REF" | tar -x -C "$D/instr" || { echo "FATAL instr extract"; exit 1; }
echo "INSTR_AT $(git rev-parse origin/$INSTR_REF)"

say "A. PRISTINE BASELINE (mirror as committed; the pre-repair head)"
echo "SEED dag=$(seed "$INFER_DAG") mirror=$(seed "$MIRROR")"
cargo test -p v1-compiler-tests --lib cross_coproduct --no-run >"$LOGS/syntax.log" 2>&1
echo "SYNTAX_RC=$?"
grep -E "^error(\[|:)" "$LOGS/syntax.log" | head -12 | sed 's/^/SYNTAX_ERROR /'
if grep -qE "^error(\[|:)" "$LOGS/syntax.log"; then
  echo "ABORT_SYNTAX -- appended rows do not compile; nothing below is measured"
  echo "BEGIN-LOG syntax.log $(wc -c <"$LOGS/syntax.log")"; base64 -w0 "$LOGS/syntax.log"; echo; echo "END-LOG syntax.log"
  exit 3
fi
testrows red; echo "A_RC=$?"

say "B1. patch the .dag (repair) and REGENERATE the mirror from it"
cp "$INFER_DAG" "$LOGS/infer.dag.orig"
cp "$MIRROR"   "$LOGS/infer.mirror.orig"
python3 "$D/instr/patch_producer_instantiate.py" "$D/repo" || { echo "FATAL patch"; exit 1; }
echo "SEED after-dag-patch dag=$(seed "$INFER_DAG") mirror=$(seed "$MIRROR") (mirror must not have moved yet)"
bash "$D/instr/instruments/regen_full.sh" >"$LOGS/regen.log" 2>&1
echo "REGEN_RC=$?"
grep -E "HEAD=|RR_EXIT|RC_EXIT|SEED|installed|round|CONVERGED|REGEN_FULL_DONE|candidates:|unmerged" "$LOGS/regen.log" | tail -16 | sed 's/^/REGEN /'
echo "SEED after-regen dag=$(seed "$INFER_DAG") mirror=$(seed "$MIRROR")"
if [ "$(seed "$MIRROR")" = "$(seed "$LOGS/infer.mirror.orig")" ]; then
  echo "REGEN_NOOP FATAL -- the regenerated mirror is byte-identical to the pre-repair mirror;"
  echo "  either the emitter did not see the .dag change or the .dag change is inert. Do not"
  echo "  read the next stage as a pass."
fi

say "B1b. EMITTED TEXT of the seeded region (so the mutation patcher matches real output)"
echo "BEGIN-EMITTED"
awk '/PRODUCER REPAIR|fresh_call_variables|uninstantiated_call_generics|inferred: s\.inferred|substitute_generics\(\{/' "$MIRROR" \
  | head -60 | sed 's/^ */  /'
echo "END-EMITTED"
echo "MIRROR_CONTEXT_BEGIN"
grep -n "fresh_call_variables\|uninstantiated_call_generics" "$MIRROR" | head -8
MIRROR_CONTEXT_END=""; echo "MIRROR_CONTEXT_END"
# Print a window around the first hit so the emitted shape (Rc, borrows) is visible verbatim.
FIRST=$(grep -n "uninstantiated_call_generics" "$MIRROR" | head -1 | cut -d: -f1)
if [ -n "${FIRST:-}" ]; then
  echo "BEGIN-WINDOW"
  sed -n "$((FIRST-30)),$((FIRST+40))p" "$MIRROR"
  echo "END-WINDOW"
else
  echo "WINDOW_MISSING_FATAL -- the seeded code is not in the regenerated mirror"
fi

say "B2. rows against the REGENERATED mirror (this is what would ship)"
testrows repair; echo "B2_RC=$?"

say "B3. whole test crate on the regenerated tree"
cargo test -p v1-compiler-tests >"$LOGS/suite.log" 2>&1
echo "SUITE_RC=$?"
grep -E "^test result:|^error" "$LOGS/suite.log" | tail -20 | sed 's/^/SUITE /'

say "ARTIFACTS"
for f in "$LOGS"/*; do
  [ -f "$f" ] || continue
  echo "BEGIN-LOG $(basename "$f") $(wc -c <"$f")"; base64 -w0 "$f"; echo; echo "END-LOG $(basename "$f")"
done
echo "ALLDONE"
