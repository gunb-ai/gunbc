set -uo pipefail
SHA="$(git rev-parse HEAD)"; echo "SUBJECT_SHA=$SHA"
export CARGO_TARGET_DIR="$PWD/target"
/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc --bin claim_batch || { echo "BUILD_FAILED"; exit 1; }
G=./target/release/gunbc; B=./target/release/claim_batch
ARGS=(); for m in non_text_cast_into_text_wall_witness host_text_crossing_entry_witness nominal_cast_wall_witness emitter_nat_int_cast_witness emitter_nested_refinement_cast_witness optional_cast_wall_witness refinement_cast_wall_witness integer_signed_grounding_witness; do
  f="dag/test/claim/${m}.dag"; [ -f "$f" ] || { echo "MISSING $f"; continue; }
  ARGS+=(--entry "$f" --functions "$(grep -hoE '^test fn [a-z_0-9]+' "$f" | awk '{print $3}' | paste -sd, -)")
done
echo "=== CLAIMS ==="
$B --claim-run --hermetic --source-root dag --source-root src/v2 "${ARGS[@]}" > /tmp/claims.txt 2>&1; rc=$?
echo "batch_rc=$rc"
grep -cE '^PASS ' /tmp/claims.txt | sed 's/^/PASS_count=/'
grep -E '^FAIL ' /tmp/claims.txt | head -20 | sed 's/^/  /'
grep -iE "panic|internal error|unresolved|abort" /tmp/claims.txt | head -8 | sed 's/^/  ERR /'
tail -3 /tmp/claims.txt | sed 's/^/  TAIL /'
echo; echo "=== CENSUS (full log, it is small) ==="
rm -rf target/c1-census && mkdir -p target/c1-census
$G compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir target/c1-census > /tmp/census.txt 2>&1
echo "census_rc=$?"; sed 's/^/  C /' /tmp/census.txt
echo "invalid_cast_as_String=$(grep -c "invalid cast.*as String" /tmp/census.txt || true)"
echo; echo "=== EMIT both modules; list what appeared ==="
for p in dag/gunbc/durable_cas_file_store.dag dag/extdeps/exec/command.dag; do
  out=target/c1-emit-$(basename $p .dag); rm -rf $out && mkdir -p $out
  echo "--- $p ---"
  $G compile --source-root dag --source-root src/v2 --entry $p --target rust --output-dir $out > /tmp/e.txt 2>&1; echo "  rc=$?"
  grep -vE '^\[pre-entry\]|memory-cgroup|user_work_scope|floor-drain|gunbc test:' /tmp/e.txt | tail -6 | sed 's/^/  OUT /'
  find $out -name '*.rs' | sed 's#.*/##' | grep -iE "cas_file|command|exec" | sed 's/^/  RS /' | head -8
  tot=$(grep -rl 'unsupported cast' $out 2>/dev/null | wc -l); echo "  files_with_unsupported_cast_panic=$tot"
  for pat in gunbc_durable_cas_file_store.rs exec_command.rs; do
    pth=$(find $out -name "$pat" 2>/dev/null | head -1); [ -z "$pth" ] && continue
    echo "  $pat lines=$(wc -l <"$pth") unsupported_cast=$(grep -c 'unsupported cast' "$pth" || true) int_to_decimal=$(grep -c int_to_decimal_string "$pth" || true)"
  done
done
echo MARKER_B_DONE
