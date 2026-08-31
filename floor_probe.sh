set -o pipefail
cargo build --release -p v1-compiler --bins 2>&1 | tail -2
./target/release/claim_executor --required-ci --source-root dag --source-root src/v2 --required-lane witnesses > /tmp/floor.log 2>&1
echo "EXIT=$?"
echo "===SUMMARY"
grep -E "planned=|phase |ROUTED to lane" /tmp/floor.log | tail -30
echo "===COUNTS-COMPUTED-REMOTELY"
echo "FAILED_LINES=$(grep -cE '^FAILED|FAILED in' /tmp/floor.log)"
echo "LOG_LINES=$(wc -l < /tmp/floor.log)"
echo "===FAILED-ROSTER"
grep -E '^FAILED|FAILED in' /tmp/floor.log | sed -E 's/ *\(.*//' | sort | uniq -c | sort -rn
echo "===ROSTER-END"
