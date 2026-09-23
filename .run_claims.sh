set -e
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
CB=$(find target -path '*release/claim_batch' -type f | head -1)
T=dag/test/claim/machine_intake/mtcollins1_boot_diagnostic_bundle_witness_test.dag
FN=a_stat_size_text_becomes_a_byte_size,a_post_boot_sel_timeout_is_recorded_and_leaves_the_verdict_alone,a_retained_capture_reports_its_firmware_statement,only_definitive_no_answers_say_the_bmc_did_not_answer
echo "== fixed"; $CB --source-root dag --source-root src/v2 --entry $T --functions $FN 2>&1 | tail -8 || true
python3 - <<'P'
f="dag/gunbc/machine_intake/mtcollins1_boot_diagnostic_bundle.dag"
s=open(f).read()
s=s.replace("CheckedNatReady { value: m } => Present { value: byte_size(count: m) }","CheckedNatReady { value: m } => Present { value: byte_size(count: n as Nat) }")
s=s.replace("import std.checked_arithmetic","import std.nat { Nat }\nimport std.checked_arithmetic")
open(f,"w").write(s)
P
echo "== mutated"; $CB --source-root dag --source-root src/v2 --entry $T --functions $FN 2>&1 | tail -8 || true
