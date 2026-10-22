#!/bin/bash
# Scratch: D7 = final acceptance battery at the PR head (declared rows; no patches).
set -uo pipefail
echo "=== HEAD ==="; git rev-parse --short HEAD; git branch --show-current
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run_claim() { # $1 label $2 entry $3 fn
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/c7_$1.log 2>&1
  echo "claim@$1 rc=$?"
  grep -aE 'unary-prefix|resolve failed|error:|witness' /tmp/c7_$1.log | grep -av 'floor\|typecheck' | head -4 | cut -c1-240
}
echo "=== GREEN BATTERY ==="
run_claim values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_claim readers src/v2/test/claim/lex_token_class_membership_readers.dag lex_token_class_membership_readers_hold
run_claim roster dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag v1_interpreter_primitive_roster_acceptance_contract_holds
run_claim dispatch dag/test/claim/v1_interpreter_primitive_dispatch_authority_acceptance.dag v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds
run_claim surface dag/test/claim/v1_interpreter_primitive_surface_witness.dag derived_surface_excludes_deleted_bridge_arm
echo "=== DECODE TESTS (all) ==="
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|test result' | head -12 | cut -c1-220
echo "=== DONE ==="
