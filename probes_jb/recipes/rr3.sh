cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
f=src/v2/test/claim/manual/scoped_type_reflection_conformance.dag; fns=$(grep -o '^test fn [A-Za-z_0-9]*' $f | awk '{print $3}' | paste -sd,); ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $PWD/$f --functions $fns 2>&1 | grep -E "^(PASS|FAIL)"; echo "rc=${PIPESTATUS[0]}"; echo ==END
