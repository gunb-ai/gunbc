set -u
export CARGO_TARGET_DIR=$PWD/target/qb
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | grep -E "^error|Finished|could not compile" | head
G=$CARGO_TARGET_DIR/release/gunbc
echo "=== REFUSAL PROBE (genuinely unsupported cast)"
rm -rf /tmp/p1; $G compile --source-root dag --source-root src/v2 --entry dag/test/fixture/qb_unsupported_cast.dag --output-dir /tmp/p1 --target rust > /tmp/p1.log 2>&1; echo "rc=$?"
grep -iE "cast|blocking|error" /tmp/p1.log | head -10 | cut -c1-400; echo "rs files: $(find /tmp/p1 -name '*.rs' 2>/dev/null | wc -l)"; grep -rl "panic!(\"unsupported cast" /tmp/p1 2>/dev/null | head -3
echo "=== GITREMOTENAME PROBE"
rm -rf /tmp/p2; $G compile --source-root dag --source-root src/v2 --entry dag/test/fixture/qb_git_remote_name_cast.dag --output-dir /tmp/p2 --target rust > /tmp/p2.log 2>&1; echo "rc=$?"
grep -iE "cast|blocking" /tmp/p2.log | head -6 | cut -c1-400; grep -rn "fn remote_text" -A3 /tmp/p2 --include=*.rs | head -5
echo "=== ISO8601 EMIT"
rm -rf /tmp/p3; $G compile --source-root dag --source-root src/v2 --entry dag/extdeps/units/iso8601_calendar.dag --output-dir /tmp/p3 --target rust > /tmp/p3.log 2>&1; echo "rc=$?"; grep -E "blocking" /tmp/p3.log | tail -1
grep -rn "fn iso8601_seconds_per_hour" -A3 /tmp/p3 --include=*.rs | head -4; echo "panics: $(grep -rc 'unsupported cast' /tmp/p3 --include=*.rs | awk -F: '{s+=$2} END {print s}')"
echo "=== CGROUP EMIT"
rm -rf /tmp/p4; $G compile --source-root dag --source-root src/v2 --entry dag/extdeps/linux/cgroup_v2.dag --output-dir /tmp/p4 --target rust > /tmp/p4.log 2>&1; echo "rc=$?"; grep -E "blocking|cast cannot" /tmp/p4.log | tail -3 | cut -c1-300
grep -rn "init\|as List" /tmp/p4/src/v1/stage0/src/extdeps_linux_cgroup_v2.rs 2>/dev/null | grep -n "fold" | head -3 | cut -c1-300
echo "=== WITNESS"
./target/qb/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/emitter_nested_refinement_cast_witness.dag --functions single_refinement_carrier_emits_no_unsupported_cast,nested_refinement_carrier_emits_no_unsupported_cast,intermediate_refinement_casts_emit_no_unsupported_cast,brand_refinement_casts_emit_no_unsupported_cast,unrelated_carriers_refuse_the_cast_at_compile_time,empty_list_cast_emits_the_operand,nat_as_int_inclusion_emits_the_operand,numeric_conversion_still_uses_cast_syntax --hermetic --claim-run 2>&1 | grep -E "^(PASS|FAIL|UNOBSERVED)|error|cannot" | cut -c1-300
echo "=== NATIVE EMISSION CONTROLS"
timeout 1500 $G test //gunbc/instruments:native-emission-controls > /tmp/nec.log 2>&1; echo "rc=$?"
grep -E "^case |not held|REFUSED|error\[|nat_as_int|empty_list|roster" /tmp/nec.log | grep -vE "^roster" | grep -E "nat_as_int|empty_list|not_held|REFUSED|error" | head -20 | cut -c1-300
grep -cE "^case .* held observed" /tmp/nec.log; tail -5 /tmp/nec.log | cut -c1-300
