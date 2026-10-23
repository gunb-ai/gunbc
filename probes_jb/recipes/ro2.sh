cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
timeout 1500 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $PWD/src/v2/test/claim/manual/zz_scratch_roster.dag --functions roster_kind_controls_hold > /tmp/cb.log 2>&1; echo "rc=$?"; grep -E "^(PASS|FAIL)|CONTROL-FAIL|expected keyword" /tmp/cb.log | cut -c1-600
echo ==END
