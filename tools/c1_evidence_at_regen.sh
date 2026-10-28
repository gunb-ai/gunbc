set -uo pipefail
echo "SUBJECT_SHA=$(git rev-parse HEAD)"
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_BIND_MEMORY_CGROUP_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES
export CARGO_TARGET_DIR="$PWD/target"; export GITHUB_ACTIONS=true
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
mkdir -p /sys/fs/cgroup/gunbc.bound 2>/dev/null || true
echo 51539607552 > /sys/fs/cgroup/gunbc.bound/memory.max 2>/dev/null || true
echo $$ > /sys/fs/cgroup/gunbc.bound/cgroup.procs 2>/dev/null || true
/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc --bin claim_batch || { echo "BUILD_FAILED"; exit 1; }
G=./target/release/gunbc; B=./target/release/claim_batch
echo; echo "=== 1. CLAIMS (cast-wall witnesses, one process) ==="
ARGS=(); for m in non_text_cast_into_text_wall_witness host_text_crossing_entry_witness nominal_cast_wall_witness emitter_nat_int_cast_witness emitter_nested_refinement_cast_witness optional_cast_wall_witness refinement_cast_wall_witness integer_signed_grounding_witness; do
  f="dag/test/claim/${m}.dag"; [ -f "$f" ] || { echo "  MISSING $f"; continue; }
  ARGS+=(--entry "$f" --functions "$(grep -hoE '^test fn [a-z_0-9]+' "$f" | awk '{print $3}' | paste -sd, -)")
done
$B --claim-run --hermetic --source-root dag --source-root src/v2 "${ARGS[@]}" > /tmp/claims.txt 2>&1; echo "batch_rc=$?"
echo "PASS=$(grep -cE '^PASS ' /tmp/claims.txt)  FAIL=$(grep -cE '^FAIL ' /tmp/claims.txt)"
grep -E '^FAIL ' /tmp/claims.txt | sed 's/^/  FAIL /'
echo; echo "=== 2. CENSUS whole corpus ==="
rm -rf target/c1-census && mkdir -p target/c1-census
$G compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir target/c1-census > /tmp/census.txt 2>&1
echo "census_rc=$? bytes=$(wc -c </tmp/census.txt)"
tail -8 /tmp/census.txt | sed 's/^/  C /'
echo "  invalid_cast_as_String=$(grep -c "invalid cast.*as String" /tmp/census.txt || true)"
echo "  invalid_cast_any=$(grep -c 'invalid cast' /tmp/census.txt || true)"
echo; echo "=== 3. EMIT the two named modules ==="
for e in dag/gunbc/durable_cas_file_store.dag dag/extdeps/exec/command.dag; do
  out=target/c1-emit-$(basename $e .dag); rm -rf $out && mkdir -p $out
  echo "--- $e ---"
  $G compile --source-root dag --source-root src/v2 --entry $e --target rust --output-dir $out > /tmp/e.txt 2>&1; echo "  rc=$?"
  for pat in gunbc_durable_cas_file_store.rs exec_command.rs; do
    pth=$(find $out -name "$pat" 2>/dev/null | head -1); [ -z "$pth" ] && { echo "  $pat: absent"; continue; }
    echo "  $pat lines=$(wc -l <"$pth") unsupported_cast=$(grep -c 'unsupported cast' "$pth" || true) int_to_decimal=$(grep -c int_to_decimal_string "$pth" || true)"
    grep -n 'unsupported cast' "$pth" | head -3 | sed 's/^/    /'
  done
done
echo "  emitted_rs_total=$(find target/c1-census target/c1-emit-durable_cas_file_store target/c1-emit-command -name '*.rs' 2>/dev/null | wc -l)"
echo "  files_with_unsupported_cast=$(grep -rl 'unsupported cast' target/c1-census target/c1-emit-durable_cas_file_store target/c1-emit-command 2>/dev/null | wc -l)"
grep -rl 'unsupported cast' target/c1-census target/c1-emit-durable_cas_file_store target/c1-emit-command 2>/dev/null | head -5 | sed 's/^/  PANICARM /'
echo MARKER_DONE
