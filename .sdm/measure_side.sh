set -e
# Runs inside one tree, already built with the out-of-tree instrument applied.
# One fresh `gunbc compile` process per depth: counters are thread-local and dumped at
# compile.reconcile completion, so a fresh process gives the EXCLUSIVE cost of exactly
# that program.
SIDE="$1"
export GUNBC_GENERIC_ARG_PASS_ACCOUNT=1
BIN=./target/release/gunbc
mkdir -p .sdm/run
fail=0
for d in 1 2 3 4 5 6; do
  printf "@@@ SIDE=%s DEPTH=%s\n" "$SIDE" "$d"
  set +e
  $BIN compile --source-root "$PWD" --entry "dag/test/gap_depth_${d}.dag" --target rust --output-dir ".sdm/run/out_${SIDE}_$d" \
      > .sdm/run/o_${SIDE}_$d.log 2> .sdm/run/e_${SIDE}_$d.log
  rc=$?
  set -e
  if [ $rc -ne 0 ]; then echo "COMPILE-FAIL rc=$rc"; tail -15 .sdm/run/e_${SIDE}_$d.log; fail=1; fi
  line=$(grep -h "generic-arg-pass-excl" .sdm/run/e_${SIDE}_$d.log || true)
  if [ -z "$line" ]; then echo "NO-INSTRUMENT"; fail=1; else echo "$line"; fi
  mark=$(grep -h "compile.reconcile" .sdm/run/e_${SIDE}_$d.log | tail -1 || true)
  if [ -z "$mark" ]; then echo "NO-RECONCILE-MARK"; fail=1; else echo "reconcile_mark: $mark"; fi
done

# ---- enrolled controls: the witness claim-run is the authority (§5 fail-closed) --
if [ ! -f .sdm/witness.dag ]; then echo "@@@ SIDE=$SIDE WITNESS-ABSENT (no controls on this tree)"; echo "@@@ SIDE=$SIDE DONE fail=$fail"; exit 0; fi
echo "@@@ SIDE=$SIDE WITNESS-CLAIM-RUN"
set +e
$BIN run --source-root "$PWD" --entry .sdm/witness.dag --claim-run > .sdm/run/w_${SIDE}.log 2>&1
wrc=$?
set -e
echo "witness_rc=$wrc"
[ $wrc -ne 0 ] && { echo "WITNESS-RUN-FAILED"; tail -25 .sdm/run/w_${SIDE}.log; fail=1; }
for t in an_inner_generic_bound_only_by_the_outer_settlement_is_re_inferred \
         a_nested_left_bound_bind_chain_is_accepted; do
  st=$(grep -o "$t[^,]*" .sdm/run/w_${SIDE}.log | head -1)
  printf "CONTROL %-70s %s\n" "$t" "${st:-NOT-REPORTED}"
  case "$st" in
    *PASS*|*pass*|*true*|*ok*) ;;
    *) echo "CONTROL-UNRESOLVED: $t"; fail=1 ;;
  esac
done
grep -ciE "TypeMismatch" .sdm/run/w_${SIDE}.log | sed 's/^/typemismatch_mentions=/'
echo "@@@ SIDE=$SIDE DONE fail=$fail"
