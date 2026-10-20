#!/usr/bin/env bash
# Run PR-1's freshness row(s) against a COMMITTED head. Fresh clone at a printed SHA.
#
# WHY NOT reuse repair_tests_14152.sh. That script's cargo filter is `--lib cross_coproduct` and its
# module prefix is `cross_coproduct_generic_locate_test::`. PR-1's row lives in a NEW module
# (generic_call_variable_freshness_test), so that filter selects a different test binary and the row
# would be reported MISSING -- and MISSING is exactly the shape a vacuous pass takes. So this is a
# separate runner, and it keeps every safeguard the earlier one earned:
#   * fresh clone at a ref-resolved SHA, never the shared /root/workspace/repo-root (dirty, produced a
#     fake drift result in this lane);
#   * the target dir under /root, which persists between dispatches (/tmp is recreated);
#   * --release, because the row calls compile_to_resolved and did not finish in 1200s of dev;
#   * a --no-run compile gate, so a row that does not build costs minutes not the whole budget;
#   * verdict read from the `test result:` SUMMARY line, not from a per-test line (--nocapture
#     interleaves the harness's prints and splits the per-test line, which reported MISSING on passes);
#   * `0 passed; 0 failed` is MISSING_FATAL, never a pass -- a filter matching nothing is the failure
#     mode that looks like success.
#
# WHAT THIS RUNNER ADDS. For PR-1 the PASS/FAIL verdict is NOT the measurement: the row's own prints
# (how many ids, how many distinct, how many site-tagged, how many bare) are. A row could pass while
# asserting nothing, so the inventory lines are echoed plainly. Both trees also carry a guard on the
# shape under test, because measuring the mutant while believing you are measuring the repair is the
# failure this lane keeps hitting.
set -u
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export GIT_PAGER=cat PAGER=cat GIT_TERMINAL_PROMPT=0
SHA="${1:?usage: pr1_row.sh <sha> <label> <pin|repair|name-keyed> [row ...]}"
LABEL="${2:?label required}"
KIND="${3:?pin|repair|name-keyed required}"
URL=https://github.com/gunb-ai/gunbc
D=/tmp/pr1_row_$(echo "$LABEL" | tr -c 'a-zA-Z0-9' '_')
LOG="$D/test_$LABEL.log"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_pr1_rows}"
mkdir -p "$CARGO_TARGET_DIR"
CARGO_TIMEOUT="${CARGO_TIMEOUT:-1500}"
ROW_TIMEOUT="${ROW_TIMEOUT:-900}"
ROW="${ROW:-a_call_that_passes_actuals_gets_no_call_site_mint}"
MODULE="${MODULE:-call_site_mint_boundary_test}"
stamp(){ echo "T=$(date -u +%H:%M:%S) $*"; }
seed(){ sha256sum "$1" | cut -c1-16; }

stamp "env: nproc=$(nproc) target=$(du -sh "$CARGO_TARGET_DIR" 2>/dev/null | cut -f1 || echo none) kind=$KIND"
rm -rf "$D"; mkdir -p "$D"
stamp "clone begin"
# Retried: a bare `git clone` on the runner failed transiently twice at ~07:33/07:42 UTC, both times
# while another dispatch was cloning at once. A gate that dies at clone must SAY so with the stderr,
# not print a bare FATAL -- a reader cannot otherwise tell an infra failure from a real verdict.
CLONED=0
for attempt in 1 2 3; do
  rm -rf "$D/repo"
  if timeout 900 git clone -q --no-checkout "$URL" "$D/repo" 2> "$D/clone_err.$attempt"; then
    CLONED=1; stamp "clone ok on attempt $attempt"; break
  fi
  stamp "clone attempt $attempt failed: $(tr '\n' ' ' < "$D/clone_err.$attempt" | cut -c1-200)"
  sleep 20
done
[ "$CLONED" = 1 ] || { echo "FATAL clone (3 attempts; stderr echoed above)"; exit 1; }
cd "$D/repo" || { echo "FATAL cd clone"; exit 1; }
timeout 300 git fetch -q origin "$SHA" || { echo "FATAL fetch $SHA"; exit 1; }
timeout 120 git checkout -q "$SHA" || { echo "FATAL checkout $SHA"; exit 1; }
HEAD_RESOLVED=$(git rev-parse HEAD)
stamp "clone done MEASURED_AT $HEAD_RESOLVED"
[ "$HEAD_RESOLVED" = "$SHA" ] || echo "WARNING requested $SHA got $HEAD_RESOLVED"
git status --porcelain | grep -v '^??' | head -3 | sed 's/^/DIRTY /'
sha256sum src/v1/04_infer.dag src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/SEED /'

# --- GUARDS on the shape under test, both directions -------------------------------------------
DAG=src/v1/04_infer.dag
MIRROR=src/v1/stage0/src/v1_compiler_infer.rs
ACC=$(grep -c UnifyGenericsAcc "$DAG" || true)
FRESH=$(grep -c fresh_call_variables "$MIRROR" || true)
# NOTE: `map_insert(acc, g, type_variable_node(id: g))` exists on MAIN (04_infer.dag:11557,
# unbound_owner_param_variables), so it cannot distinguish a minting tree from main. Mint presence is
# the mint's own markers, none of which main carries (verified: each is 0 at origin/main).
NK_DAG=$(grep -c 'uninstantiated_call_generics\|fresh_call_variables' "$DAG" || true)
SPAN_DAG=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' "$DAG" || true)
NK_MIRROR=$(grep -c 'type_variable_node("g"' "$MIRROR" 2>/dev/null || true)
echo "GUARD acc=$ACC mirror_fresh_marks=$FRESH dag_namekeyed=$NK_DAG dag_span_keyed=$SPAN_DAG"
[ "$ACC" -eq 0 ] || { echo "VOID: the accumulator is present; PR-1 must not carry it"; exit 5; }
# The row and its registration must exist, or the filter matches nothing and I would be reporting a
# PASS produced by an empty selection.
grep -q "fn $ROW" "src/v1/tests/src/$MODULE.rs" \
  || { echo "VOID: the row is not in src/v1/tests/src/$MODULE.rs"; exit 5; }
grep -q "^mod $MODULE;" src/v1/tests/src/lib.rs \
  || { echo "VOID: the module is not registered in lib.rs -- the row would never run"; exit 5; }
if [ "$KIND" = probe ]; then
  # A PROBE tree (probe/* refs only, never shipped): it carries its own extra module and no mint. Keep
  # the guards that stop a no-op measurement (row must exist, module must be registered, it must print
  # an inventory) but skip the mint-shape guards, which are about the shipped pin tree.
  grep -qE "println!|print!" "src/v1/tests/src/$MODULE.rs" \
    || { echo "VOID: the probe prints nothing, so it produces no measurement"; exit 5; }
elif [ "$KIND" = pin ]; then
  # THE SHIPPED PR-1 TREE: no mint at all, only the boundary pin. Guards must fail if a mint is
  # secretly present (then this is not the tree the PR carries) or if the pin row is a no-op.
  [ "$FRESH" -eq 0 ] || { echo "VOID: the mirror still mints (fresh_call_variables=$FRESH); this is not the shipped pin tree"; exit 5; }
  [ "$NK_DAG" -eq 0 ] && [ "$SPAN_DAG" -eq 0 ] || { echo "VOID: the .dag still carries a mint (mint_markers=$NK_DAG span=$SPAN_DAG)"; exit 5; }
  grep -q "MINT-BOUNDARY-INVENTORY" "src/v1/tests/src/$MODULE.rs" \
    || { echo "VOID: the pin row prints no inventory, so it cannot show it looked at anything"; exit 5; }
  # Each guard name must be BOTH set and asserted (>=2 mentions: assignment + assert). A row that
  # lost an assert could pass while measuring nothing; matching only `let NAME` was too strict
  # (the row writes `let mut saw_fn`) and made a good tree read as VOID.
  for G in saw_fn call_nodes; do
    [ "$(grep -c "$G" "src/v1/tests/src/$MODULE.rs")" -ge 2 ] \
      || { echo "VOID: the pin row does not both set and assert $G -- it could pass by finding nothing"; exit 5; }
  done
  grep -q "assert" "src/v1/tests/src/$MODULE.rs" \
    || { echo "VOID: the pin row contains no assert at all"; exit 5; }
elif [ "$KIND" = repair ]; then
  # The SPAN-KEYED mint tree (probe/pr1-producer-repair). The mint's presence is mint_markers; the
  # id FORM is what separates repair from mutant, so span markers must be present here.
  [ "$NK_DAG" -ge 1 ] || { echo "VOID: the .dag carries no mint at all -- this is the pin tree, measure it as pin"; exit 5; }
  [ "$FRESH" -ge 2 ] || { echo "VOID: the regenerated mirror does not carry the repair"; exit 5; }
  [ "$SPAN_DAG" -ge 1 ] || { echo "VOID: the .dag does not mint span-keyed ids"; exit 5; }
else
  # The NAME-KEYED mutant (probe/pr1-mut-namekeyed): it mints, but not with a span in the id.
  [ "$NK_DAG" -ge 1 ] || { echo "VOID: name-keyed mutation absent at the call site (mint_markers=$NK_DAG)"; exit 5; }
  [ "$SPAN_DAG" -eq 0 ] || { echo "VOID: the span-keyed id survived; this is not the mutant"; exit 5; }
  # THE MUTANT MUST ACTUALLY REACH THE MIRROR. Tests import from the checked-in mirror, so a .dag-only
  # mutation would make the row fail for ABSENCE of minting rather than for the sharing -- a red that
  # looks like the control while proving nothing. A name-keyed mirror mints with the bare parameter
  # name, so look for that, and for the absence of the span concatenation.
  if [ "$FRESH" -eq 0 ]; then
    echo "VOID: the mutant's mirror carries no minting fold at all -- regenerate it, do not measure it"
    exit 5
  fi
  if grep -q 'v1_rt::concat' "$MIRROR" && grep -q 'span.start' "$MIRROR" && grep -c "type_variable_node(v1_rt::concat" "$MIRROR" | grep -qv '^0$'; then
    echo "SUSPECT_MUTANT_MIRROR: the mirror still concatenates a span into a minted id; compare against the repair's mirror before believing any verdict"
  fi
fi

stamp "compile gate (--no-run)"
RELN=""; [ "${ROW_PROFILE:-release}" = release ] && RELN="--release"
timeout -k 30 "$CARGO_TIMEOUT" cargo test $RELN -p v1-compiler-tests --lib "$MODULE" --no-run >"$D/syntax.log" 2>&1
SYN_RC=$?
echo "SYNTAX_RC=$SYN_RC"
grep -E "^error(\[|:)" "$D/syntax.log" | head -10 | sed 's/^/SYNTAX_ERROR /'
if [ "$SYN_RC" != 0 ]; then
  echo "ABORT_SYNTAX"
  echo "SYNTAX_TAIL"; tail -30 "$D/syntax.log"; echo "END_SYNTAX_TAIL"
  exit 3
fi

shift 3 || true
SELECT=("$@"); [ ${#SELECT[@]} -eq 0 ] && SELECT=("$ROW")
: > "$LOG"
stamp "rows begin (${#SELECT[@]} rows, bound ${ROW_TIMEOUT}s each)"
for r in "${SELECT[@]}"; do
  rowlog="$D/row.$r.log"; t0=$(date +%s)
  timeout -k 15 "$ROW_TIMEOUT" cargo test $RELN -p v1-compiler-tests --lib "$MODULE" \
    -- --exact "${MODULE}::${r}" --nocapture --test-threads=1 >"$rowlog" 2>&1
  rrc=$?; t1=$(date +%s)
  cat "$rowlog" >> "$LOG"
  res=$(grep -E "^test result:" "$rowlog" | tail -1)
  passed=$(echo "$res" | sed -n "s/.*test result: [a-zA-Z]*\. \([0-9]*\) passed.*/\1/p")
  failed=$(echo "$res" | sed -n "s/.*; \([0-9]*\) failed.*/\1/p")
  [ -z "$passed" ] && passed=-1; [ -z "$failed" ] && failed=-1
  if   [ "$rrc" = 124 ]; then echo "ROW $LABEL $r TIMEOUT_FATAL rc=124 elapsed=$((t1-t0))s"
  elif [ "$failed" -gt 0 ] 2>/dev/null; then echo "ROW $LABEL $r FAIL rc=$rrc elapsed=$((t1-t0))s"
  elif [ "$passed" -ge 1 ] 2>/dev/null; then echo "ROW $LABEL $r PASS rc=$rrc elapsed=$((t1-t0))s"
  elif [ "$passed" = 0 ] && [ "$failed" = 0 ]; then echo "ROW $LABEL $r MISSING_FATAL rc=$rrc filter matched nothing elapsed=$((t1-t0))s"
  else echo "ROW $LABEL $r NO_SUMMARY rc=$rrc elapsed=$((t1-t0))s"; fi
done

# THE ROW'S OWN PRINTS ARE THE MEASUREMENT, not just the verdict. A passing row that asserted nothing
# would look identical to a passing row that measured 24 site-tagged ids, so print the inventory.
echo "ROW_PRINTS_BEGIN"
grep -aE "positions|distinct|site-tagged|bare|ids=|collide|minted|expected|assertion|panicked" "$LOG" | head -40 | cut -c1-300
echo "ROW_PRINTS_END"
grep -E "^test result:" "$LOG" | sed "s/^/SUMMARY $LABEL /"
grep -E "^error(\[|:)" "$LOG" | head -8 | sed 's/^/COMPILE_ERROR /'
echo "BEGIN-TESTLOG_GZ $(wc -c <"$LOG")"; gzip -9 < "$LOG" | base64 -w0; echo; echo "END-TESTLOG_GZ"
stamp "ALLDONE"
