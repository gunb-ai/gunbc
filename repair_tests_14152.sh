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
ROW_TIMEOUT="${ROW_TIMEOUT:-420}"

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
RELN=""
if [ "${ROW_PROFILE:-dev}" = release ]; then RELN="--release"; fi
timeout -k 30 "$CARGO_TIMEOUT" cargo test ${RELN} -p v1-compiler-tests --lib cross_coproduct --no-run >"$D/syntax.log" 2>&1
SYN_RC=$?
echo "SYNTAX_RC=$SYN_RC"
grep -E "^error(\[|:)" "$D/syntax.log" | head -12 | sed 's/^/SYNTAX_ERROR /'
if [ "$SYN_RC" != 0 ]; then
  echo "ABORT_SYNTAX"
  echo "BEGIN-LOG syntax.log $(wc -c <"$D/syntax.log")"; base64 -w0 "$D/syntax.log"; echo; echo "END-LOG syntax.log"
  exit 3
fi

# WHY ONE INVOCATION PER ROW, BOUNDED SEPARATELY. This crate pins RUST_TEST_THREADS=1
# (.cargo/config.toml:17) and EVERY row calls compile_to_resolved, which compiles the whole
# std corpus: ~2 minutes per row, so the 35-row crate filter cannot fit one dispatch -- the
# first attempt burned its whole bound and never returned a verdict on the rows after the
# third. One --exact invocation per row, each on its own timeout, so a slow or hung row
# costs its own budget and the other rows still answer. Extra positional args select a
# SUBSET, which is how the 9 rows are split across two dispatches. Positional args survive
# ctrl-build's arg join; env-prefix args do not (they become the -c command).
MOD=cross_coproduct_generic_locate_test::
# The SHA and LABEL are still in $3..; selecting from "$@" unshifted ran two bogus
# --exact invocations (the SHA and the label) which reported MISSING_FATAL rc=0 -- noise that
# also cost two cargo starts. Shift past them explicitly.
shift 2 || true
SELECT=("$@")
if [ "${#SELECT[@]}" -eq 0 ]; then SELECT=($ROWS); fi
: > "$LOG"
stamp "rows begin (${#SELECT[@]} rows, per-row bound ${ROW_TIMEOUT}s)"
for r in "${SELECT[@]}"; do
  [ -n "${SEED_DELAY:-}" ] && sleep "$SEED_DELAY"
  rowlog="$D/row.$r.log"
  t0=$(date +%s)
  # WHY --release IS WORTH ITS BUILD COST. The rows that call compile_to_resolved run the
  # COMPILER UNDER TEST inside an unoptimised binary: the collision row alone did not finish in
  # 1200s of dev profile, on a crate whose dependencies were already built (the --no-run gate
  # excludes compile time). The same work in a release binary is the same work the self-host
  # gate does with a release gunbc in ~10 minutes total. ROW_PROFILE=release costs one extra
  # dependency build once, into its own persisted target dir, and buys minutes per row.
  REL=""
  if [ "${ROW_PROFILE:-dev}" = release ]; then REL="--release"; fi
  timeout -k 15 "${ROW_TIMEOUT}" cargo test ${REL} -p v1-compiler-tests --lib cross_coproduct \
    -- --exact "${MOD}${r}" --nocapture --test-threads=1 >"$rowlog" 2>&1
  rrc=$?
  t1=$(date +%s)
  cat "$rowlog" >> "$LOG"
  # WHICH LINE DECIDES THE VERDICT. --nocapture interleaves the harness's own prints with the
  # test's, which splits `test NAME ... ok` away from its result on the next line, so matching
  # that single line reports MISSING_FATAL rc=0 on a run that passed. For a one-test --exact
  # invocation the summary line cannot be ambiguous: `1 passed; 0 failed` is the row passing,
  # any non-zero failed count is the row failing, and `0 passed; 0 failed` means the filter
  # matched nothing -- which is the only case that is genuinely MISSING.
  res=$(grep -E "^test result:" "$rowlog" | tail -1)
  passed=$(echo "$res" | sed -n "s/.*test result: [a-zA-Z]*\. \([0-9]*\) passed.*/\1/p")
  failed=$(echo "$res" | sed -n "s/.*; \([0-9]*\) failed.*/\1/p")
  [ -z "$passed" ] && passed=-1
  [ -z "$failed" ] && failed=-1
  if   [ "$rrc" = 124 ]; then echo "ROW $LABEL $r TIMEOUT_FATAL rc=124 after ${ROW_TIMEOUT}s elapsed=$((t1-t0))s"
  elif [ "$failed" != -1 ] && [ "$failed" -gt 0 ]; then echo "ROW $LABEL $r FAIL rc=$rrc elapsed=$((t1-t0))s"
  elif [ "$passed" != -1 ] && [ "$passed" -ge 1 ]; then echo "ROW $LABEL $r PASS rc=$rrc elapsed=$((t1-t0))s"
  elif [ "$passed" = 0 ] && [ "$failed" = 0 ]; then echo "ROW $LABEL $r MISSING_FATAL rc=$rrc filter matched nothing elapsed=$((t1-t0))s"
  else echo "ROW $LABEL $r NO_SUMMARY rc=$rrc elapsed=$((t1-t0))s"; fi
done
grep -E "^test result:" "$LOG" | sed "s/^/SUMMARY $LABEL /"
grep -E "^error(\[|:)" "$LOG" | head -8 | sed 's/^/COMPILE_ERROR /'

# A FAILING row's message is the measurement (the collision row's expected pre-repair
# diagnostic, the accept row's binding dump). Print it rather than re-deriving it.
echo "BEGIN-FAILURE-DETAIL"
awk '/panicked at|assertion failed|must both type|must be replaced|must refuse|is unsettled/' "$LOG" | head -40 | cut -c1-320
echo "END-FAILURE-DETAIL"

echo "BEGIN-TESTLOG $(wc -c <"$LOG")"; base64 -w0 "$LOG"; echo; echo "END-TESTLOG"
stamp "ALLDONE"
