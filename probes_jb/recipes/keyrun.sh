cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run(){ f=$1; fns=$(grep -o '^test fn [a-z_0-9]*' $f | awk '{print $3}' | paste -sd,); echo "== $f"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $PWD/$f --functions $fns 2>&1 | grep -E "^(PASS|FAIL)"; echo "rc=${PIPESTATUS[0]}"; }
for f in $FILES; do run $f; done
echo ==END
