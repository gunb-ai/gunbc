#!/usr/bin/env bash
# Stage A only, lean and self-reporting. The previous dispatch died at the runner's 45-minute
# cap with "--- BEGIN-TESTS red" as its last line and 0 artifacts uploaded, which cannot
# distinguish a slow build from a hung test. So: ONE cargo invocation, a hard internal
# timeout below the runner cap, a heartbeat that distinguishes "compiling" from "hung", and
# the log is printed even when the run is cut short.
set -u
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
SHA="${1:?usage: repair_stage_a.sh <sha>}"
URL=https://github.com/gunb-ai/gunbc
D=/tmp/stage_a
LOG=/tmp/stage_a_test.log
CARGO_TIMEOUT="${CARGO_TIMEOUT:-1320}"   # 22 min, inside the 45 min runner cap
# The previous dispatch spent its entire 45-minute budget in a COLD cargo build. Debuginfo is
# the dominant term for a crate this size, so turn it off (and keep the profile trivial).
# These are build flags only: they cannot change whether a test passes.
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_DEV_OPT_LEVEL=0
export CARGO_INCREMENTAL=0

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

stamp "start"
rm -rf "$D" "$LOG"; mkdir -p "$D"
stamp "clone begin"
timeout 600 git clone -q "$URL" "$D/repo" || { echo "FATAL clone"; exit 1; }
cd "$D/repo" || { echo "FATAL cd"; exit 1; }
timeout 300 git fetch -q origin "$SHA" || { echo "FATAL fetch"; exit 1; }
timeout 120 git checkout -q "$SHA" || { echo "FATAL checkout"; exit 1; }
stamp "clone done MEASURED_AT $(git rev-parse HEAD)"
sha256sum src/v1/04_infer.dag src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/SEED /'

# One invocation, unbuffered, hard-bounded. --test-threads=1 so a hang names its test instead
# of burying it behind nine parallel rows.
stamp "cargo begin (timeout ${CARGO_TIMEOUT}s)"
timeout -k 30 "$CARGO_TIMEOUT" cargo test -j "$(nproc)" -p v1-compiler-tests --lib cross_coproduct -- --nocapture \
  > "$LOG" 2>&1 &
CARGO_PID=$!

# Heartbeat: growing log = compiling/running; flat log for many minutes = stuck. This is the
# only way to tell the two apart from the outside, and the last dispatch could not.
while kill -0 "$CARGO_PID" 2>/dev/null; do
  sleep 60
  sz=$(wc -c <"$LOG" 2>/dev/null || echo 0)
  age=$(find "$LOG" -newermt '-90 seconds' 2>/dev/null | wc -l)
  stamp "heartbeat log=$sz fresh=$age"
done
wait "$CARGO_PID"
rc=$?
stamp "cargo done rc=$rc  (124=timed out by my own bound, not by the runner)"

echo "--- BEGIN-TESTS red"
for r in $ROWS; do
  if   grep -qE "^test .*${r} \.\.\. ok$"     "$LOG"; then echo "ROW red $r PASS"
  elif grep -qE "^test .*${r} \.\.\. FAILED$" "$LOG"; then echo "ROW red $r FAIL"
  else echo "ROW red $r MISSING_FATAL"; fi
done | sort -u
grep -E "^test result:" "$LOG" | tail -1 | sed 's/^/SUMMARY red /'
grep -E "^error(\[|:)" "$LOG" | head -8 | sed 's/^/COMPILE_ERROR /'
# The failure text of the collision row is the measurement, not noise.
echo "BEGIN-COLLISION-DETAIL"
awk '/collision_two_zero_arg_generic_calls_both_type/,/^test [a-z_]+ \.\.\./' "$LOG" | head -60
echo "END-COLLISION-DETAIL"
echo "BEGIN-TESTLOG-TAIL"
tail -c 6000 "$LOG"
echo "END-TESTLOG-TAIL"
stamp "ALLDONE"
