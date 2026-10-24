set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
./target/release/gunbc test //gunbc/instruments:ambient-name-census > all.out 2>&1; echo exit=$?
echo ---UNBOUND; grep -aP "^binding_unbound\t" all.out | grep -aE "instrument_targets|ambient_operation_disposition_reading|namespace_wave_binding_diff|gunbc.target_binding|occurrence_binding_census" | awk -F'\t' '{print $2"\t"$3}' | sort | uniq -c | sort -rn | head -60
echo ---OTHERMODS; grep -aP "^binding_unbound_module\t" all.out | awk -F'\t' '$3>0' | head -5
echo ---INFER; grep -a "native-census-infer" all.out | grep -aE "instrument_targets|ambient_operation_disposition|occurrence_binding_census|target_binding|namespace_wave" | head -6 | cut -c1-900
