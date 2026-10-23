cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
./target/release/claim_batch --source-root dag --source-root src/v2 --entry $PWD/src/v2/test/claim/optional_payload_inhabitance_native.dag --functions  > /tmp/o.log 2>&1; echo "rc=$?"; grep -E "^(PASS|FAIL)" /tmp/o.log
echo ==END
