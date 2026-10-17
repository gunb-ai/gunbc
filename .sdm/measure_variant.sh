set -e
# Runs inside one variant of the A/B dispatch, after its binary is built.
# Both depth series are measured with the same fresh-process protocol: the counters are
# thread_local and dumped at compile.reconcile completion, so one process == exactly one
# program's exclusive cost.
V="$1"
export GUNBC_GENERIC_ARG_PASS_ACCOUNT=1
# The build uses a SHARED CARGO_TARGET_DIR across variants (so the ~200 dependency
# crates compile once, not three times). With a shared target dir the binary is NOT
# under ./target -- reading the wrong path gives rc=127 and a silently empty A/B.
# Variants are built+measured strictly in sequence, so each measures its own binary.
BIN="${CARGO_TARGET_DIR:?CARGO_TARGET_DIR must be exported by ab.sh}/release/gunbc"
if [ ! -x "$BIN" ]; then echo "NO-BINARY at $BIN -- refusing to emit an empty measurement"; exit 12; fi
echo "binary=$BIN sha256=$(sha256sum "$BIN" | cut -d' ' -f1)"
mkdir -p .sdm/run
fail=0
# ---- preflight ---------------------------------------------------------------------
# Several dispatches were burned on rc=1/rc=127 with no stderr shown: the failure branch
# printed a tail of a file that was empty (stderr went to a different file) and the
# entry/source-root spelling was assumed rather than learned. So print the real CLI
# surface, then try the fixture under every (entry-spelling x source-root) combination and
# adopt the first that compiles, showing each attempt's FULL output. Measurement only.
echo "@@@ VARIANT=$V PREFLIGHT"
set +e
timeout 300 $BIN compile --help > .sdm/run/help_$V.log 2>&1
echo "help_rc=$?"
grep -iE "source-root|entry|target|output-dir|claim" .sdm/run/help_$V.log | head -25
set -e
cp .sdm/fixtures/gap_depth_2.dag ./gap_depth_2.dag
ENTRY_KIND=""; SOURCE_ROOT_ARGS=""
for combo in "root|one" "root|two" "abs|one" "abs|two"; do
  KIND=${combo%%|*}; SR=${combo##*|}
  if [ "$KIND" = root ]; then E=gap_depth_2.dag; else E="$PWD/.sdm/fixtures/gap_depth_2.dag"; fi
  if [ "$SR" = one ]; then set -- --source-root "$PWD"; else set -- --source-root "$PWD" --source-root dag; fi
  echo "--- attempt entry=$KIND source_roots=$SR ---"
  set +e
  timeout 900 $BIN compile "$@" --entry "$E" --target rust \
    --output-dir .sdm/run/preflight_${KIND}_${SR} > .sdm/run/preflight_${KIND}_${SR}.log 2>&1
  prc=$?
  set -e
  echo "preflight_${KIND}_${SR} rc=$prc"
  head -40 .sdm/run/preflight_${KIND}_${SR}.log
  if [ $prc -eq 0 ]; then ENTRY_KIND="$KIND"; SOURCE_ROOT_ARGS="$*"; echo "PREFLIGHT-OK entry=$KIND source_roots=$SR"; break; fi
done
rm -f ./gap_depth_2.dag
if [ -z "$ENTRY_KIND" ]; then
  echo "PREFLIGHT-FAILED: no entry/source-root spelling compiled the fixture; aborting rather than emitting an empty A/B"
  exit 14
fi

for SERIES in gap_depth garr; do
  for d in 1 2 3 4 5 6; do
    f="dag/test/${SERIES}_${d}.dag"
    [ -f "$f" ] || { echo "MISSING-FIXTURE $f"; fail=1; continue; }
    printf "@@@ VARIANT=%s SERIES=%s DEPTH=%s\n" "$V" "$SERIES" "$d"
    set +e
    t0=$(date +%s)
    eval $BIN compile ${SOURCE_ROOT_ARGS} --entry "$f" --target rust \
        --output-dir ".sdm/run/out_${V}_${SERIES}_$d" \
        > .sdm/run/o_${V}_${SERIES}_$d.log 2> .sdm/run/e_${V}_${SERIES}_$d.log
    rc=$?
    echo "fixture_elapsed=$(( $(date +%s) - t0 ))s"
    set -e
    if [ $rc -ne 0 ]; then
      echo "COMPILE-FAIL rc=$rc"; echo "--- stdout ---"; head -30 .sdm/run/o_${V}_${SERIES}_$d.log
      echo "--- stderr ---"; head -30 .sdm/run/e_${V}_${SERIES}_$d.log; fail=1
    fi
    line=$(grep -h "generic-arg-pass-excl" .sdm/run/e_${V}_${SERIES}_$d.log || true)
    if [ -z "$line" ]; then echo "NO-INSTRUMENT"; fail=1; else echo "$line"; fi
    mark=$(grep -h "compile.reconcile" .sdm/run/e_${V}_${SERIES}_$d.log | tail -1 || true)
    if [ -z "$mark" ]; then echo "NO-RECONCILE-MARK"; fail=1; else echo "reconcile: $mark"; fi
    # duplicate-diagnostic check (parent item 6): the same diagnostic reported twice is
    # the signature of a replayed first pass.
    dup=$(grep -oE "error\[?[A-Za-z]*\]?: .*" .sdm/run/e_${V}_${SERIES}_$d.log | sort | uniq -c | awk '$1>1' | wc -l)
    echo "duplicate_diagnostics=$dup"
  done
done

# ---- the full-program specimen (where the 2618-class counter is meaningful) --------
for SPEC in src/v2/compiler/compile.dag; do
  if [ -f "$SPEC" ]; then
    printf "@@@ VARIANT=%s SPECIMEN=%s\n" "$V" "$SPEC"
    set +e
    timeout 1500 $BIN compile --source-root dag --source-root src/v2 --entry "$SPEC" \
      --target rust --output-dir ".sdm/run/out_${V}_spec" \
      > .sdm/run/o_${V}_spec.log 2> .sdm/run/e_${V}_spec.log
    rc=$?
    set -e
    echo "specimen_rc=$rc"
    grep -h "generic-arg-pass-excl" .sdm/run/e_${V}_spec.log || echo "NO-INSTRUMENT-SPECIMEN"
    grep -h "compile.reconcile" .sdm/run/e_${V}_spec.log | tail -1 || echo "NO-RECONCILE-SPECIMEN"
  else
    echo "NO-SPECIMEN $SPEC"
  fi
done

# ---- the witness, run the way claims are ACTUALLY run --------------------------------
# A `test fn` under dag/test/claim is executed by `claim_batch --entry E --function F`,
# which prints one PASS/FAIL verdict per function (report_outcome in
# src/v1/stage0/src/bin/claim_batch.rs). Grep says this witness is cited by NO roster and
# no workflow, so nothing in CI runs it -- asserting a bare `gunbc run --claim-run` exit
# code therefore proved nothing about the seven verdicts. Run each test fn by name so a
# regression names itself, and keep the instrument env armed so the counts and the verdicts
# come from the same process.
W=dag/test/claim/generic_argument_recheck_witness.dag
CB="$CARGO_TARGET_DIR/release/claim_batch"
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
    timeout 900 "$CB" --source-root "$PWD" --entry "$PWD/$W" --function "$FN" \
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
