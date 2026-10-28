set -uo pipefail
SHA="$(git rev-parse HEAD)"; echo "SUBJECT_SHA=$SHA"
export CARGO_TARGET_DIR="$PWD/target"
/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc --bin claim_batch || { echo "BUILD_FAILED"; exit 1; }
G=./target/release/gunbc; B=./target/release/claim_batch
ARGS=(); for m in non_text_cast_into_text_wall_witness host_text_crossing_entry_witness nominal_cast_wall_witness emitter_nat_int_cast_witness emitter_nested_refinement_cast_witness optional_cast_wall_witness refinement_cast_wall_witness integer_signed_grounding_witness; do
  f="dag/test/claim/${m}.dag"; [ -f "$f" ] || { echo "MISSING $f"; continue; }
  fnames=$(grep -hoE '^test fn [a-z_0-9]+' "$f" | awk '{print $3}' | paste -sd, -)
  [ -z "$fnames" ] && continue
  ARGS+=(--entry "$f" --functions "$fnames")
  echo "group $m: $(echo $fnames | tr ',' '\n' | wc -l) claims"
done
echo "=== CLAIMS (one process, claim_batch) ==="
$B --claim-run --hermetic --source-root dag --source-root src/v2 "${ARGS[@]}" 2>&1 | grep -E "HELD|did not hold|DID NOT HOLD|UNOBSERVED|claim_batch:|ledger|TOTAL|verdict" | tail -90
echo "batch_rc=${PIPESTATUS[0]}"
echo; echo "=== CENSUS (recorded command) ==="
rm -rf target/c1-census && mkdir -p target/c1-census
$G compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir target/c1-census > /tmp/census.txt 2>&1
echo "census_rc=$? bytes=$(wc -c < /tmp/census.txt)"
grep -iE "blocking error" /tmp/census.txt | tail -2
echo "invalid_cast_total=$(grep -c 'invalid cast' /tmp/census.txt || true)"
echo "invalid_cast_as_String=$(grep -c "invalid cast.*as String" /tmp/census.txt || true)"
grep 'invalid cast' /tmp/census.txt | head -8 | sed 's/^/  IC /'
echo; echo "=== EMIT: the two named modules (repo paths) ==="
for p in dag/gunbc/durable_cas_file_store.dag dag/extdeps/exec/command.dag; do
  out=target/c1-emit-$(basename $p .dag); rm -rf $out && mkdir -p $out
  echo "--- $p ---"
  $G compile --source-root dag --source-root src/v2 --entry $p --target rust --output-dir $out > /tmp/e.txt 2>&1
  echo "  rc=$?"
  tail -2 /tmp/e.txt | sed 's/^/  OUT /'
  for f in $(find $out -name '*.rs' 2>/dev/null); do
    n=$(grep -c 'unsupported cast' "$f" || true); [ "$n" != "0" ] && echo "  PANIC-ARM $f : $n"
  done
  for f in gunbc_durable_cas_file_store.rs exec_command.rs; do
    pth=$(find $out -name "$f" 2>/dev/null | head -1)
    [ -z "$pth" ] && { echo "  $f: not in this output"; continue; }
    echo "  $f lines=$(wc -l < "$pth") unsupported_cast=$(grep -c 'unsupported cast' "$pth" || true) int_to_decimal_string=$(grep -c 'int_to_decimal_string' "$pth" || true)"
  done
  echo "  total_rs_files=$(find $out -name '*.rs' | wc -l) panic_arms=$(grep -rc 'unsupported cast' $out 2>/dev/null | grep -v ':0' | wc -l)"
done
echo MARKER_C1_A_DONE
