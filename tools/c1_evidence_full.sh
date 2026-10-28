set -uo pipefail
HEAD_SHA="$(git rev-parse HEAD)"; echo "SUBJECT_SHA=$HEAD_SHA"
PREWALL=dd53edf89a6
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_BIND_MEMORY_CGROUP_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES
export CARGO_TARGET_DIR="$PWD/target"; export GITHUB_ACTIONS=true
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
mkdir -p /sys/fs/cgroup/gunbc.bound 2>/dev/null || true
echo 51539607552 > /sys/fs/cgroup/gunbc.bound/memory.max 2>/dev/null || true
echo $$ > /sys/fs/cgroup/gunbc.bound/cgroup.procs 2>/dev/null || true

# Revert arm: export the pre-wall commit and build its compiler in parallel.
rm -rf /tmp/prewall && mkdir -p /tmp/prewall
git archive $PREWALL | tar -x -C /tmp/prewall && echo "prewall_exported=$PREWALL"
( cd /tmp/prewall && CARGO_TARGET_DIR=/tmp/prewall/target /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_batch > /tmp/prewall-build.log 2>&1; echo "prewall_build_rc=$?" >> /tmp/prewall-build.log ) &
PB=$!

/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc --bin claim_batch || { echo "BUILD_FAILED"; exit 1; }
G=./target/release/gunbc; B=./target/release/claim_batch

echo; echo "=== 1. CLAIMS (stack's cast-wall witnesses, one process) ==="
ARGS=(); for m in non_text_cast_into_text_wall_witness host_text_crossing_entry_witness nominal_cast_wall_witness emitter_nat_int_cast_witness emitter_nested_refinement_cast_witness optional_cast_wall_witness refinement_cast_wall_witness integer_signed_grounding_witness; do
  f="dag/test/claim/${m}.dag"; [ -f "$f" ] || { echo "  MISSING $f"; continue; }
  ARGS+=(--entry "$f" --functions "$(grep -hoE '^test fn [a-z_0-9]+' "$f" | awk '{print $3}' | paste -sd, -)")
done
$B --claim-run --hermetic --source-root dag --source-root src/v2 "${ARGS[@]}" > /tmp/claims.txt 2>&1; echo "batch_rc=$?"
echo "PASS=$(grep -cE '^PASS ' /tmp/claims.txt)  FAIL=$(grep -cE '^FAIL ' /tmp/claims.txt)"
grep -E '^FAIL ' /tmp/claims.txt | sed 's/^/  FAIL /'

echo; echo "=== 2. REVERT ARM at pre-wall $PREWALL (RED must NOT refuse there) ==="
wait $PB; tail -2 /tmp/prewall-build.log
if [ -x /tmp/prewall/target/release/claim_batch ]; then
  ( cd /tmp/prewall && ./target/release/claim_batch --claim-run --hermetic --source-root dag --source-root src/v2 \
      --entry dag/test/claim/non_text_cast_into_text_wall_witness.dag \
      --functions w_int_as_string_is_refused,w_declared_int_to_text_route_is_accepted 2>&1 | grep -E "^PASS |^FAIL |claim route|unobserved|REFUS" | sed 's/^/  PREWALL /' )
fi

echo; echo "=== 3. CENSUS whole corpus (bound cgroup) ==="
rm -rf target/c1-census && mkdir -p target/c1-census
$G compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir target/c1-census > /tmp/census.txt 2>&1
echo "census_rc=$? bytes=$(wc -c </tmp/census.txt)"
tail -12 /tmp/census.txt | sed 's/^/  C /'
echo "  invalid_cast_as_String=$(grep -c "invalid cast.*as String" /tmp/census.txt || true)"
echo "  invalid_cast_any=$(grep -c 'invalid cast' /tmp/census.txt || true)"

echo; echo "=== 4. EMIT the two named modules ==="
for e in dag/gunbc/durable_cas_file_store.dag dag/extdeps/exec/command.dag; do
  out=target/c1-emit-$(basename $e .dag); rm -rf $out && mkdir -p $out
  echo "--- $e ---"
  $G compile --source-root dag --source-root src/v2 --entry $e --target rust --output-dir $out > /tmp/e.txt 2>&1; echo "  rc=$?"
  grep -viE 'pre-entry|memory-cgroup|user_work_scope|floor-drain|gunbc test:' /tmp/e.txt | tail -3 | sed 's/^/  OUT /'
  for pat in gunbc_durable_cas_file_store.rs exec_command.rs; do
    pth=$(find $out -name "$pat" 2>/dev/null | head -1); [ -z "$pth" ] && { echo "  $pat: absent"; continue; }
    echo "  $pat lines=$(wc -l <"$pth") unsupported_cast=$(grep -c 'unsupported cast' "$pth" || true) int_to_decimal=$(grep -c int_to_decimal_string "$pth" || true)"
  done
done
echo "  emitted_rs_total=$(find target/c1-census target/c1-emit-durable_cas_file_store target/c1-emit-command -name '*.rs' 2>/dev/null | wc -l)"
echo "  files_with_unsupported_cast=$(grep -rl 'unsupported cast' target/c1-census target/c1-emit-durable_cas_file_store target/c1-emit-command 2>/dev/null | wc -l)"
grep -rl 'unsupported cast' target/c1-census target/c1-emit-durable_cas_file_store target/c1-emit-command 2>/dev/null | head -5 | sed 's/^/  PANICARM /'
echo MARKER_C1_FULL_DONE
