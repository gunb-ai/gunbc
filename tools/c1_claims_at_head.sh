set -uo pipefail
WANT=6246a3bde24c982e016678ba118cd1d62f85fdaa
echo "GITHUB_SHA=$GITHUB_SHA"
echo "SUBJECT_SHA=$(git rev-parse HEAD)"
[ "$(git rev-parse HEAD)" = "$WANT" ] || { echo "WRONG_HEAD refusing to run"; exit 1; }
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_BIND_MEMORY_CGROUP_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES
export CARGO_TARGET_DIR="$PWD/target"; export GITHUB_ACTIONS=true
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
mkdir -p /sys/fs/cgroup/gunbc.bound 2>/dev/null || true
echo 51539607552 > /sys/fs/cgroup/gunbc.bound/memory.max 2>/dev/null || true
echo $$ > /sys/fs/cgroup/gunbc.bound/cgroup.procs 2>/dev/null || true
/opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
B=./target/release/claim_batch
ARGS=(); for m in non_text_cast_into_text_wall_witness host_text_crossing_entry_witness nominal_cast_wall_witness emitter_nat_int_cast_witness emitter_nested_refinement_cast_witness optional_cast_wall_witness refinement_cast_wall_witness integer_signed_grounding_witness; do
  f="dag/test/claim/${m}.dag"; [ -f "$f" ] || { echo "  MISSING $f"; continue; }
  ARGS+=(--entry "$f" --functions "$(grep -hoE '^test fn [a-z_0-9]+' "$f" | awk '{print $3}' | paste -sd, -)")
done
$B --claim-run --hermetic --source-root dag --source-root src/v2 "${ARGS[@]}" > /tmp/claims.txt 2>&1; echo "batch_rc=$?"
echo "CLAIM_TALLY PASS=$(grep -cE '^PASS ' /tmp/claims.txt) FAIL=$(grep -cE '^FAIL ' /tmp/claims.txt)"
grep -E '^FAIL ' /tmp/claims.txt | sed 's/^/  FAIL /'
echo MARKER_CLAIMS_HEAD_DONE
