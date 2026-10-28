set -u
export CARGO_TARGET_DIR=$PWD/target/qb
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | grep -E "^error|Finished|could not compile" | head
G=$CARGO_TARGET_DIR/release/gunbc
for e in qb_cons_head_direct qb_cons_head_payload qb_git_remote_name_cast; do
  echo "=== PROBE $e"; rm -rf /tmp/$e; $G compile --source-root dag --source-root src/v2 --entry dag/test/fixture/$e.dag --output-dir /tmp/$e --target rust > /tmp/$e.log 2>&1; echo "rc=$?"; grep -E "^error\[|blocking error" /tmp/$e.log | cut -c1-260
done
echo "=== ISO8601"; rm -rf /tmp/p3; $G compile --source-root dag --source-root src/v2 --entry dag/extdeps/units/iso8601_calendar.dag --output-dir /tmp/p3 --target rust > /tmp/p3.log 2>&1; echo "rc=$?"; grep -E "^error\[|blocking error" /tmp/p3.log | cut -c1-260
grep -rn "fn iso8601_seconds_per_hour" -A3 /tmp/p3 --include=*.rs | head -4
echo "=== WITNESS"
./target/qb/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/emitter_nested_refinement_cast_witness.dag --functions single_refinement_carrier_emits_no_unsupported_cast,nested_refinement_carrier_emits_no_unsupported_cast,intermediate_refinement_casts_emit_no_unsupported_cast,brand_refinement_casts_emit_no_unsupported_cast,unrelated_carriers_refuse_the_cast_at_compile_time,empty_list_cast_emits_the_operand,nat_as_int_inclusion_emits_the_operand,numeric_conversion_still_uses_cast_syntax --hermetic --claim-run 2>&1 | grep -E "^(PASS|FAIL|UNOBSERVED)" | cut -c1-300
echo "=== NATIVE EMISSION CONTROLS"
timeout 1500 $G test //gunbc/instruments:native-emission-controls > /tmp/nec.log 2>&1; echo "rc=$?"
grep -E "nat_as_int|empty_list|not_held|REFUSED|^error|cast cannot" /tmp/nec.log | head -20 | cut -c1-300; tail -4 /tmp/nec.log | cut -c1-300
