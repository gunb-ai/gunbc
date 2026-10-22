#!/bin/bash
# Scratch: lane A battery + decode ONLY (no regen) at pinned head, to bank receipts.
set -uo pipefail
PIN=dfd432b29865af9d11311c98efa49c19e1ced161
git fetch -q origin 2>/dev/null || true
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then echo "PIN: was $CUR; checkout $PIN"; git checkout -f -q "$PIN" || { echo PIN-FAILED; exit 3; }; fi
test "$(git rev-parse HEAD)" = "$PIN" || { echo PIN-MISMATCH; exit 3; }
echo "=== HEAD ==="; git rev-parse --short HEAD
echo "STEP build claim_batch"; date -u +%H:%M:%S
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run_claim() {
  echo "STEP claim $1"; date -u +%H:%M:%S
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/cA_$1.log 2>&1
  echo "claim@$1 rc=$?"
  grep -am1 'FAIL\|resolve failed' /tmp/cA_$1.log | cut -c1-200
}
run_claim values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_claim readers src/v2/test/claim/lex_token_class_membership_readers.dag lex_token_class_membership_readers_hold
run_claim roster dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag v1_interpreter_primitive_roster_acceptance_contract_holds
run_claim dispatch dag/test/claim/v1_interpreter_primitive_dispatch_authority_acceptance.dag v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds
run_claim surface dag/test/claim/v1_interpreter_primitive_surface_witness.dag derived_surface_excludes_deleted_bridge_arm
echo "STEP decode tests"; date -u +%H:%M:%S
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|test result' | head -8 | cut -c1-200
echo "=== DONE ==="
