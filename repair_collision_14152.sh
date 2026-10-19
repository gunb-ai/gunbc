#!/usr/bin/env bash
# Measured run for the #14152 producer repair. FRESH CLONE at a printed SHA, remote runner.
#
# Stages, all in ONE dispatch so every number comes from the same base tree:
#   S1 RED      -- collision rows + existing controls on the UNPATCHED tree
#   S2 WITHDRAWN-- same rows with the seed dropped (--mutate withdrawn). This IS the
#                  pre-repair behaviour expressed in the new code: a row that PASSES here
#                  does not test the seed, whatever it did at S1.
#   S3 NAMEKEY  -- fresh ids keyed on the bare parameter name (--mutate name-keyed). The
#                  collision row MUST fail here; that is the parent's discriminator
#                  (msg_78d4eb21) for call-site-keyed vs name-keyed. If it PASSES, the
#                  collision is not observable in this harness and that is a reported limit.
#   S4 REPAIR   -- the real patch: every row must pass.
#   S5 REGEN    -- regenerate the mirrors from the patched .dag (never hand-edited).
#   S6 SHIPPED  -- the rows again against the REGENERATED mirrors, which is what ships.
#   S7 SUITE    -- the whole test crate on the patched tree.
#
# Per-row results are printed by name and an absent row is reported MISSING, never read
# as a pass. rc comes from the command, never from a pipe.
set -u
# The runner's cargo is not on the default PATH (probe v18's proven form).
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
SHA="${1:?usage: repair_collision_14152.sh <sha> [sha-of-test-commit]}"
TESTSHA="${2:-$1}"
D="${REPO_DIR:-/tmp/repair_14152}"
URL=https://github.com/gunb-ai/gunbc
INSTR_REF="scratch/rebind-census"
LOGS=/tmp/repair_14152_logs
rm -rf "$D" "$LOGS"; mkdir -p "$LOGS"

say "cargo location"
command -v cargo || { echo "FATAL no cargo on PATH"; exit 1; }

say(){ echo "=== $* ==="; }

say "clone"
timeout 900 git clone -q "$URL" "$D/repo" || { echo "FATAL clone"; exit 1; }
cd "$D/repo" || { echo "FATAL cd"; exit 1; }
git fetch -q origin "$SHA" || { echo "FATAL fetch $SHA"; exit 1; }
git fetch -q origin "$INSTR_REF" || { echo "FATAL fetch instr"; exit 1; }
git checkout -q "$SHA" || { echo "FATAL checkout $SHA"; exit 1; }
echo "MEASURED_AT $(git rev-parse HEAD)"
git status --porcelain | head -3 | sed 's/^/DIRTY /'

# The collision rows live on their own commit; apply them by path so the measured .dag is
# still $SHA's, and so no probe machinery lands on the tree we measure.
if [ "$TESTSHA" != "$SHA" ]; then
  git checkout "$TESTSHA" -- src/v1/tests/src/cross_coproduct_generic_locate_test.rs \
    || { echo "FATAL test checkout $TESTSHA"; exit 1; }
  echo "TESTFILE_FROM $TESTSHA"
fi

mkdir -p "$D/instr"
git archive "origin/$INSTR_REF" | tar -x -C "$D/instr" || { echo "FATAL instr extract"; exit 1; }
echo "INSTR_AT $(git rev-parse origin/$INSTR_REF)"
patcher(){ python3 "$D/instr/patch_producer_instantiate.py" "$@"; }

INFER=src/v1/04_infer.dag
cp "$INFER" "$LOGS/infer.orig"

ROWS='collision_two_zero_arg_generic_calls_both_type
collision_pair_still_refuses_when_instantiated_twice
fold_replaces_a_binding_that_is_the_generic_declarations_own_form
fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance
fold_replaces_a_binding_that_still_carries_a_type_variable_leaf
fold_keeps_a_settled_binding_against_an_actual_that_still_carries_a_type_variable
fold_refuses_two_distinct_settled_bindings_of_the_same_shape
fold_refuses_int_bound_then_bool'

testrows(){ # testrows <label>  -- one cargo invocation, per-row verdicts by name
  local label="$1" log="$LOGS/test_$label.log"
  echo "--- BEGIN-TESTS $label"
  cargo test -p v1-compiler-tests --lib cross_coproduct >"$log" 2>&1
  local rc=$?
  echo "--- TESTS $label rc=$rc"
  local r
  for r in $ROWS; do
    if   grep -qE "^test .*${r} \.\.\. ok"     "$log"; then echo "ROW $label $r PASS"
    elif grep -qE "^test .*${r} \.\.\. FAILED" "$log"; then echo "ROW $label $r FAIL"
    else echo "ROW $label $r MISSING_FATAL"; fi
  done
  grep -E "^test result:" "$log" | tail -1 | sed "s/^/SUMMARY $label /"
  grep -E "^error(\[|:)" "$log" | head -5 | sed "s/^/COMPILE_ERROR $label /"
  return $rc
}

say "S0 SYNTAX CHECK of the appended rows (cheap abort: ~8 min, not 30)"
# The rows are new Rust against an API I read but did not compile; a type error here would
# otherwise be discovered after five stages. --no-run builds without running.
cargo test -p v1-compiler-tests --lib cross_coproduct --no-run >"$LOGS/syntax.log" 2>&1
SYNTAX_RC=$?
echo "SYNTAX_RC=$SYNTAX_RC"
grep -E "^error(\[|:)" "$LOGS/syntax.log" | head -12 | sed 's/^/SYNTAX_ERROR /'
if [ "$SYNTAX_RC" != 0 ]; then
  echo "ABORT_SYNTAX -- the appended rows do not compile; nothing below is measured"
  for f in "$LOGS/syntax.log"; do echo "BEGIN-LOG $(basename "$f") $(wc -c <"$f")"; base64 -w0 "$f"; echo; echo "END-LOG syntax.log"; done
  exit 3
fi

say "S1 RED (unpatched .dag)"
sha256sum "$INFER" | sed 's/^/S1 /'
testrows red; echo "S1_RC=$?"

say "S2 WITHDRAWN mutation (must behave like S1)"
patcher --mutate withdrawn "$D/repo" || { echo "FATAL patch withdrawn"; exit 1; }
sha256sum "$INFER" | sed 's/^/S2 /'
testrows withdrawn; echo "S2_RC=$?"
cp "$LOGS/infer.orig" "$INFER"

say "S3 NAME-KEYED mutation (collision row must FAIL)"
patcher --mutate name-keyed "$D/repo" || { echo "FATAL patch name-keyed"; exit 1; }
sha256sum "$INFER" | sed 's/^/S3 /'
testrows namekeyed; echo "S3_RC=$?"
cp "$LOGS/infer.orig" "$INFER"

say "S4 REPAIR (real patch)"
patcher "$D/repo" || { echo "FATAL patch"; exit 1; }
sha256sum "$INFER" | sed 's/^/S4 /'
testrows repair; echo "S4_RC=$?"

say "S5 regenerate mirrors from the patched .dag"
bash "$D/instr/instruments/regen_full.sh" >"$LOGS/regen.log" 2>&1
echo "REGEN_RC=$?"
grep -E "HEAD=|RR_EXIT|RC_EXIT|SEED|installed|round|CONVERGED|REGEN_FULL_DONE|candidates:" "$LOGS/regen.log" | tail -14 | sed 's/^/REGEN /'
sha256sum src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/REGEN_MIRROR /'
grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/REGEN_FOLD_REFS /'

say "S6 SHIPPED (rows against the regenerated mirrors)"
testrows shipped; echo "S6_RC=$?"

say "S7 full test crate on the patched+regenerated tree"
cargo test -p v1-compiler-tests >"$LOGS/suite.log" 2>&1
echo "SUITE_RC=$?"
grep -E "^test result:|^error" "$LOGS/suite.log" | tail -20 | sed 's/^/SUITE /'

say "ARTIFACTS"
for f in "$LOGS"/*; do
  [ -f "$f" ] || continue
  echo "BEGIN-LOG $(basename "$f") $(wc -c <"$f")"; base64 -w0 "$f"; echo; echo "END-LOG $(basename "$f")"
done
echo "ALLDONE"
