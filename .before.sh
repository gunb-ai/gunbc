set -u
git checkout -q ea035ecb0ea17f26f1e38694ebc4d6dc200eb772 -- src/v1/04_infer.dag src/v1/stage0/src/v1_interpreter.rs src/v1/stage0/src/v1_compiler_infer.rs; git checkout -q HEAD -- dag/test/claim/nominal_cast_wall_witness.dag; cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
echo "== RED-CLAIMS"
F=$(grep -o "^test fn [a-z_]*" dag/test/claim/nominal_cast_wall_witness.dag | sed 's/test fn //' | paste -sd,)
timeout 900 $B --source-root dag --source-root src/v2 --entry dag/test/claim/nominal_cast_wall_witness.dag --functions $F --eval-budget-ms 800000 2>&1 | grep -E "PASS|FAIL|subject|error" | cut -c1-300 | head -40
echo "== DIFF"
timeout 1500 target/release/gunbc compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json 2>&1 | grep -E "^error\[" | sort > diff_before.txt
wc -l diff_before.txt; echo "== DIFF-BEGIN"; gzip -c diff_before.txt | base64 -w0; echo; echo "== DIFF-END"
