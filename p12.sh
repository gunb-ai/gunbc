set -u
echo HEAD=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "=== authorization_pattern fresh-live"
./target/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/authorization_pattern_selection_witness.dag --functions $(grep -E "^test fn|^claim " dag/test/claim/authorization_pattern_selection_witness.dag | sed -E "s/^(test fn|claim) ([a-z_0-9]+).*/\2/" | paste -sd, -) --hermetic --claim-run > /tmp/cb-authpat-f.log 2>&1
rc=$?
echo "exit=$rc"
grep -E -A6 "^(FAIL|UNOBSERVED)" /tmp/cb-authpat-f.log | head -40
echo "pass=$(grep -c ^PASS /tmp/cb-authpat-f.log) fail=$(grep -c ^FAIL /tmp/cb-authpat-f.log) unobs=$(grep -c ^UNOBSERVED /tmp/cb-authpat-f.log)"
[ $rc -ne 0 ] && tail -20 /tmp/cb-authpat-f.log
echo DONE-P12
