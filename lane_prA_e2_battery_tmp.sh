#!/bin/bash
# Scratch (never lands; mirrors untracked): prA lane E2 = FULL BATTERY at the rebased head
# with the regenerated mirror applied (the bin battery from f11cefc6 predates two edits:
# the emitted-text arm was rewritten to the compile_dag_rust_emit_check builtin, and the
# branch was rebased onto 647d770c78c). claim_batch is built AFTER regen state exists.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
git clean -x -d --force >/dev/null

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current
echo "mirror witnesses: find_prefix_binding_power=$(grep -c find_prefix_binding_power src/v1/stage0/src/v1_compiler_parse.rs) literal-min-bp=$(grep -c 'min_bp: 1[0-9]' src/v1/stage0/src/v1_compiler_parse.rs)"

echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1

run_claim() { # $1 label $2 entry $3 fn  (extra --source-root src/v1 passed by caller)
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/claim_$1.log 2>&1
  local rc=$?
  echo "claim@$1 rc=$rc"
  grep -aE "✗|refus|DISAGREE|error|panic" /tmp/claim_$1.log | head -4
  tail -1 /tmp/claim_$1.log | sed 's/^/  last: /'
  return 0
}

run_claim values_arms src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim values_emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_claim readers src/v2/test/claim/lex_token_class_membership_readers.dag lex_token_class_membership_readers_hold
run_claim roster dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag v1_interpreter_primitive_roster_acceptance_contract_holds
run_claim dispatch dag/test/claim/v1_interpreter_primitive_dispatch_authority_acceptance.dag v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds
run_claim surface dag/test/claim/v1_interpreter_primitive_surface_witness.dag derived_surface_excludes_deleted_bridge_arm
echo "=== DONE ==="
