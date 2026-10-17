set -e
# Runs inside ONE variant of the A/B dispatch, after its binary is built. Emits the raw
# counter lines verbatim; the analyzer, not this script, decides pass/fail.
V="$1"
export GUNBC_GENERIC_ARG_PASS_ACCOUNT=1
# The build uses a SHARED CARGO_TARGET_DIR across variants (so ~200 dependency crates
# compile once instead of three times), so the binary is NOT under ./target -- reading the
# wrong path gave rc=127 and a silently empty A/B. Variants run strictly in sequence, so
# each measures the binary it just built.
BIN="${CARGO_TARGET_DIR:?CARGO_TARGET_DIR must be exported by ab.sh}/release/gunbc"
CB="${CARGO_TARGET_DIR}/release/claim_batch"
[ -x "$BIN" ] || { echo "NO-BINARY at $BIN -- refusing to emit an empty measurement"; exit 12; }
echo "binary=$BIN sha256=$(sha256sum "$BIN" | cut -d' ' -f1) bytes=$(stat -c %s "$BIN")"
[ -x "$CB" ] || echo "WARN no claim_batch at $CB (witness verdicts will be skipped)"
mkdir -p .sdm/run
fail=0

# ---- preflight ---------------------------------------------------------------------
# Earlier dispatches burned time on rc=1/rc=127 with nothing shown, because the failure
# branch tailed a file that was empty and because the --entry / --source-root spelling was
# assumed. So: print the real CLI surface, then try a uniquely-named probe fixture under
# each candidate (entry-spelling x source-root) pair and adopt the first that compiles,
# printing every attempt's FULL output. The winning pair is reused verbatim by the series.
# The probe has its own module name so it can never collide with a fixture or a tracked
# file: source-discovery refuses the WHOLE compile when a module path is declared twice.
echo "@@@ VARIANT=$V PREFLIGHT"
set +e
timeout 300 $BIN compile --help > .sdm/run/help_$V.log 2>&1
echo "help_rc=$?"
grep -iE "source-root|entry|target|output-dir|claim" .sdm/run/help_$V.log | head -25
set -e

# Each attempt is (name, where the probe file is placed, how --entry is spelled, roots).
# Placed probe files are uniquely named so they can never collide with a tracked module.
FIXDIR=""; FIXENTRY=""; SOURCE_ROOT_ARGS=""
try_attempt() { # name probe_dir entry roots
  local name="$1" dir="$2" entry="$3" roots="$4"
  rm -rf .sdm/run/pf_out; mkdir -p "$dir"
  cp .sdm/fixtures_pristine/pf_probe.dagx "$dir/sd503_pf_probe_unique_9f3c.dag"
  echo "--- attempt $name: probe=$dir entry=$entry roots=$roots"
  set +e
  timeout 600 $BIN compile $roots --entry "$entry" --target rust \
    --output-dir .sdm/run/pf_out > .sdm/run/pf.log 2>&1
  local rc=$?
  set -e
  echo "attempt $name rc=$rc"
  head -40 .sdm/run/pf.log
  if [ $rc -eq 0 ]; then FIXDIR="$dir"; FIXENTRY="$entry"; SOURCE_ROOT_ARGS="$roots"; echo "PREFLIGHT-OK $name"; return 0; fi
  rm -f "$dir/sd503_pf_probe_unique_9f3c.dag"
  return 1
}
if ! { try_attempt repo-root-dagtest  "dag/test" "dag/test/sd503_pf_probe_unique_9f3c.dag" "--source-root $PWD" \
    || try_attempt repo-root-dagtest-2roots "dag/test" "dag/test/sd503_pf_probe_unique_9f3c.dag" "--source-root $PWD --source-root dag" \
    || try_attempt repo-root-toplevel  "."        "sd503_pf_probe_unique_9f3c.dag"         "--source-root $PWD" \
    || try_attempt dag-root            "dag"      "sd503_pf_probe_unique_9f3c.dag"         "--source-root dag"; }; then
  rm -f dag/test/sd503_pf_probe_unique_9f3c.dag sd503_pf_probe_unique_9f3c.dag dag/sd503_pf_probe_unique_9f3c.dag 2>/dev/null
  echo "PREFLIGHT-FAILED: no entry/source-root spelling compiled the probe; aborting rather than emitting an empty A/B"
  exit 14
fi
# Fixtures live in EXACTLY the directory the preflight proved, under their real .dag name,
# and nowhere else (one module path, one file). They are re-staged from the pristine .dagx
# copies before every variant so a re-run cannot leave a stale or duplicated copy behind.
PRE=.sdm/fixtures_pristine
rm -f "$FIXDIR"/gap_depth_*.dag "$FIXDIR"/garr_*.dag
for f in "$PRE"/gap_depth_*.dagx "$PRE"/garr_*.dagx; do
  base=$(basename "$f" .dagx)
  cp "$f" "$FIXDIR/$base" || { echo "FIXTURE-STAGE-FAILED $f"; exit 15; }
done
echo "fixtures staged into $FIXDIR: $(ls "$FIXDIR"/gap_depth_*.dag "$FIXDIR"/garr_*.dag | wc -l) files; elsewhere under the root: $(find . -name 'gap_depth_*.dag' -o -name 'garr_*.dag' | grep -v "^./$FIXDIR" | wc -l)"

echo "PREFLIGHT SUMMARY fixdir=$FIXDIR entry_prefix=$FIX_PREFIX roots=$SOURCE_ROOT_ARGS witness=$WITNESS_ENTRY"

# ---- depth series ------------------------------------------------------------------
for SERIES in gap_depth garr; do
  for d in 1 2 3 4 5 6; do
    f="${FIX_PREFIX}${SERIES}_${d}.dag"
    [ -f "$f" ] || { echo "MISSING-FIXTURE $f"; fail=1; continue; }
    printf "@@@ VARIANT=%s SERIES=%s DEPTH=%s entry=%s\n" "$V" "$SERIES" "$d" "$f"
    set +e
    t0=$(date +%s)
    $BIN compile $SOURCE_ROOT_ARGS --entry "$f" --target rust \
      --output-dir ".sdm/run/out_${V}_${SERIES}_${d}" \
      > .sdm/run/o_${V}_${SERIES}_${d}.log 2> .sdm/run/e_${V}_${SERIES}_${d}.log
    rc=$?
    echo "compile_rc=$rc elapsed=$(( $(date +%s) - t0 ))s"
    set -e
    grep -h "generic-arg-pass-excl" .sdm/run/e_${V}_${SERIES}_${d}.log || echo "NO-INSTRUMENT"
    grep -hiE "compile.reconcile done in" .sdm/run/e_${V}_${SERIES}_${d}.log || echo "NO-RECONCILE-MARK"
    # duplicate diagnostics are the fingerprint of a replayed first pass
    sort .sdm/run/e_${V}_${SERIES}_${d}.log | uniq -d | grep -c "type mismatch\|TypeMismatch" | sed 's/^/duplicate_diagnostics=/'
    if [ $rc -ne 0 ]; then
      echo "COMPILE-FAIL rc=$rc"; echo "--- stdout ---"; head -30 .sdm/run/o_${V}_${SERIES}_${d}.log
      echo "--- stderr ---"; head -30 .sdm/run/e_${V}_${SERIES}_${d}.log; fail=1
    fi
  done
done

# ---- the specimen whose 2618 threshold the bar names -------------------------------
# Counter provenance for the "<=2618 top r2_n" acceptance line: this is the counter
# (r1_pass_n_top + recheck_n_top on the top-level pass), the specimen
# (src/v2/compiler/compile.dag), and whether instrumentation overhead is included (it is
# not: every guard returns early unless GUNBC_GENERIC_ARG_PASS_ACCOUNT is set, so the
# counts are free; only the *_excl_ms columns pay for an Instant).
echo "@@@ VARIANT=$V SPECIMEN entry=src/v2/compiler/compile.dag"
set +e
t0=$(date +%s)
$BIN compile $SOURCE_ROOT_ARGS --entry src/v2/compiler/compile.dag --target rust \
  --output-dir ".sdm/run/out_${V}_specimen" > .sdm/run/o_${V}_spec.log 2> .sdm/run/e_${V}_spec.log
echo "specimen_rc=$? elapsed=$(( $(date +%s) - t0 ))s"
set -e
grep -h "generic-arg-pass-excl" .sdm/run/e_${V}_spec.log || echo "NO-INSTRUMENT"
grep -hiE "compile.reconcile done in" .sdm/run/e_${V}_spec.log || echo "NO-RECONCILE-MARK"

# ---- emitted v2 closure, for file-by-file attribution ------------------------------
# The two rows of gunbc.emitted_subject_build_gate: the self-host entry and the
# v2-native-cli entry. Emitted with the SAME binary that produced the counters above, into
# a per-variant directory, and reduced to a manifest of path + sha256 so the two variants'
# closures can be diffed offline and EVERY differing file attributed (identical count alone
# is not the deliverable). Gated by CLOSURES=yes so only the two A/B variants pay for it.
if [ "${CLOSURES:-no}" = yes ]; then
  echo "@@@ VARIANT=$V CLOSURES"
  for ROW in "self-host:src/v2/compiler/source_root_eval.dag" "v2-native-cli:src/v2/cli/compile_cli.dag"; do
    LABEL="${ROW%%:*}"; ENTRY="${ROW#*:}"
    OUT=".sdm/run/closure_${V}_${LABEL}"
    rm -rf "$OUT"
    set +e
    t0=$(date +%s)
    $BIN compile $SOURCE_ROOT_ARGS --entry "$ENTRY" --target rust --output-dir "$OUT" \
      > .sdm/run/cl_${V}_${LABEL}.log 2>&1
    crc=$?
    set -e
    echo "CLOSURE $V $LABEL entry=$ENTRY rc=$crc elapsed=$(( $(date +%s) - t0 ))s"
    if [ $crc -ne 0 ]; then echo "CLOSURE-OUTPUT-BEGINS $LABEL"; head -25 .sdm/run/cl_${V}_${LABEL}.log; echo "CLOSURE-OUTPUT-ENDS"; else
      (cd "$OUT" && find . -type f | sort | while read -r f; do
         echo "MANIFEST $V $LABEL ${f#./} $(sha256sum "$f" | cut -d' ' -f1)"; done)
      echo "CLOSURE $V $LABEL files=$(find "$OUT" -type f | wc -l)"
    fi
  done
fi

# ---- the witness, run the way claims are ACTUALLY run ------------------------------
# A `test fn` under dag/test/claim is executed by `claim_batch --entry E --function F`,
# which prints one PASS/FAIL verdict per function. This witness is cited by no roster and
# no workflow (the v1_required_lanes_withdrawn drop), so nothing in CI runs it; per parent,
# no new lane is added -- the seven verdicts are instead pasted verbatim into the PR body.
W="$WITNESS_ENTRY"
if [ -f "$W" ] && [ -x "$CB" ]; then
  echo "@@@ VARIANT=$V WITNESS"
  for FN in a_lambda_after_the_argument_that_binds_its_parameter_is_accepted \
            a_lambda_before_the_argument_that_binds_its_parameter_is_accepted_under_the_final_context \
            a_nested_left_bound_bind_chain_is_accepted \
            an_earlier_literal_whose_generic_is_bound_later_is_accepted \
            an_earlier_record_literal_whose_generic_is_bound_later_is_accepted \
            a_lambda_return_that_disagrees_with_the_substituted_callable_refuses \
            an_inner_generic_bound_only_by_the_outer_settlement_is_re_inferred ; do
    set +e
    t0=$(date +%s)
    timeout 900 "$CB" $SOURCE_ROOT_ARGS --entry "$W" --function "$FN" \
      > .sdm/run/fn_${V}_${FN}.log 2>&1
    rc=$?
    set -e
    verdict=$(grep -hoE "^(PASS|FAIL|PANIC)[^ ]*( .*)?" .sdm/run/fn_${V}_${FN}.log | head -1)
    echo "CLAIMFN $V ${FN} rc=$rc elapsed=$(( $(date +%s) - t0 ))s verdict=${verdict:-NONE}"
    if [ "$rc" -ne 0 ]; then
      echo "CLAIMFN-OUTPUT-BEGINS ${FN}"; head -40 .sdm/run/fn_${V}_${FN}.log; echo "CLAIMFN-OUTPUT-ENDS"
    fi
  done
else
  echo "@@@ VARIANT=$V WITNESS-SKIPPED file=$([ -f "$W" ] && echo yes || echo no) claim_batch=$([ -x "$CB" ] && echo yes || echo no)"
fi

echo "@@@ VARIANT=$V DONE fail=$fail"
