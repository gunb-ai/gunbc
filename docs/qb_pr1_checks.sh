set -u
export CARGO_TARGET_DIR=$PWD/target/qb
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | grep -E "^error|Finished|could not compile" | head
G=$CARGO_TARGET_DIR/release/gunbc
echo "=== WITNESS"
./target/qb/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/emitter_nested_refinement_cast_witness.dag --functions single_refinement_carrier_emits_no_unsupported_cast,nested_refinement_carrier_emits_no_unsupported_cast,intermediate_refinement_casts_emit_no_unsupported_cast,brand_refinement_casts_emit_no_unsupported_cast,unrelated_carriers_still_refuse_the_cast,alias_of_refinement_cast_emits_the_operand,generic_call_cast_emits_the_operand,empty_list_cast_emits_the_operand,nat_as_int_inclusion_emits_the_operand,numeric_conversion_still_uses_cast_syntax --hermetic --claim-run 2>&1 | grep -E "^(PASS|FAIL|UNOBSERVED)" | cut -c1-300
./target/qb/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/refinement_cast_wall_witness.dag --hermetic --claim-run 2>&1 | grep -E "^(PASS|FAIL|UNOBSERVED)" | cut -c1-200
echo "=== NATIVE EMISSION CONTROLS"
timeout 1500 $G test //gunbc/instruments:native-emission-controls > /tmp/nec.log 2>&1; echo "rc=$?"
grep -E "nat_as_int|empty_list_cast|not_held|REFUSED|^error" /tmp/nec.log | head -12 | cut -c1-200
