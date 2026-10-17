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
for SERIES in gap_depth garr; do
  for d in 1 2 3 4 5 6; do
    f="dag/test/${SERIES}_${d}.dag"
    [ -f "$f" ] || { echo "MISSING-FIXTURE $f"; fail=1; continue; }
    printf "@@@ VARIANT=%s SERIES=%s DEPTH=%s\n" "$V" "$SERIES" "$d"
    set +e
    t0=$(date +%s)
    $BIN compile --source-root "$PWD" --entry "$f" --target rust \
        --output-dir ".sdm/run/out_${V}_${SERIES}_$d" \
        > .sdm/run/o_${V}_${SERIES}_$d.log 2> .sdm/run/e_${V}_${SERIES}_$d.log
    rc=$?
    echo "fixture_elapsed=$(( $(date +%s) - t0 ))s"
    set -e
    if [ $rc -ne 0 ]; then echo "COMPILE-FAIL rc=$rc"; tail -12 .sdm/run/e_${V}_${SERIES}_$d.log; fail=1; fi
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

# ---- the witness claim-run ---------------------------------------------------------
# Resolved from the TRACKED path (how the floor resolves an entry). A materialised copy
# under .sdm made source-discovery see the module twice and refuse the compile outright.
W=dag/test/claim/generic_argument_recheck_witness.dag
if [ -f "$W" ]; then
  echo "@@@ VARIANT=$V WITNESS"
  set +e
  t0=$(date +%s)
  timeout 1500 $BIN run --source-root "$PWD" --entry "$W" --claim-run > .sdm/run/w_$V.log 2>&1
  echo "witness_rc=$? claim_elapsed=$(( $(date +%s) - t0 ))s"
  set -e
  grep -ciE "TypeMismatch" .sdm/run/w_$V.log | sed 's/^/typemismatch_mentions=/'
  tail -6 .sdm/run/w_$V.log
  # The strengthened witness must assert the EXACT diagnostic location, and the
  # location is render_type_to_string(got) -- which is derived, never guessed. So ask
  # the real compiler for the real diagnostics of the witness body, per variant, and
  # print them verbatim. This is measurement only; no assertion changes here.
  set +e
  timeout 900 $BIN compile --source-root "$PWD" --source-root dag --entry dag/test/claim/generic_argument_recheck_witness.dag \
    --target rust --output-dir ".sdm/run/out_${V}_witnesscompile" \
    > .sdm/run/cw_${V}.log 2> .sdm/run/cwe_${V}.log
  echo "witness_compile_rc=$?"
  set -e
  grep -oE "type mismatch: expected [^ ]+ got [^ ]+ at [^ ]+ in [^ ]+" .sdm/run/cwe_${V}.log \
    | sort | uniq -c | sed "s/^/WITNESS-DIAG $V /" || echo "WITNESS-DIAG $V none"
  grep -cE "^gunbc: " .sdm/run/cwe_${V}.log | sed "s/^/WITNESS-BLOCKING-LINES $V /"
else
  echo "@@@ VARIANT=$V WITNESS-ABSENT"
fi
echo "@@@ VARIANT=$V DONE fail=$fail"
