set -u
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
B=target/release/claim_batch
echo "== RED-CLAIMS"
F=$(grep -o "^test fn [a-z_]*" dag/test/claim/nominal_cast_wall_witness.dag | sed 's/test fn //' | paste -sd,)
timeout 900 $B --source-root dag --source-root src/v2 --entry dag/test/claim/nominal_cast_wall_witness.dag --functions $F --eval-budget-ms 800000 2>&1 | grep -E "PASS|FAIL|subject|error" | cut -c1-300 | head -40
echo "== DIFF"
timeout 1500 target/release/gunbc compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json 2>&1 | grep -E "^error\[" | sort > diff_after.txt
wc -l diff_after.txt; echo "== DIFF-BEGIN"; gzip -c diff_after.txt | base64 -w0; echo; echo "== DIFF-END"
echo "== V2"
F=$(grep -o "^test fn cns_[a-z_]*" src/v2/test/claim/cast_nominal_to_string_probe.dag | sed 's/test fn //' | paste -sd,)
timeout 1500 $B --source-root dag --source-root src/v2 --entry src/v2/test/claim/cast_nominal_to_string_probe.dag --functions $F --eval-budget-ms 1200000 2>&1 | grep -E "PASS|FAIL" | cut -c1-200
