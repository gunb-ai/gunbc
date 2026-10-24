set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
./target/release/gunbc test //gunbc/instruments:ambient-name-census > all.out 2>&1; echo exit=$?
echo ---LINES; grep -a "occurrence_binding_census\|instrument_targets" all.out | grep -avP "^(binding|binding_unanswered|module_subjects|rewritten|refused_rewritten)\t" | cut -c1-700 | head -30
echo ---KINDS; cut -f1 all.out | cut -c1-40 | sort | uniq -c | sort -rn | head -25
echo ---FIVE; grep -aP "^(binding_unbound_module|binding_unreached|binding_partial_module)\t" all.out | grep -aE "instrument_targets|ambient_operation_disposition_reading|namespace_wave_binding_diff|gunbc.target_binding|occurrence_binding_census" | cut -c1-300
echo ---UNBOUNDMODS; grep -ac "^binding_unbound_module" all.out
