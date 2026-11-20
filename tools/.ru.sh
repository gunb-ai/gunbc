set -o pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
for e in "$@"; do
  echo "=== $e"
  GUNBC_MEMORY_BUDGET_BYTES=20000000000 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry "$e" --claim-run > /tmp/out.txt 2>&1
  rc=$?
  grep -E "^(PASS|FAIL)|error|Error|refus" /tmp/out.txt | grep -v "^\[pre-entry\]" | head -60
  echo "rc=$rc pass=$(grep -c '^PASS' /tmp/out.txt) fail=$(grep -c '^FAIL' /tmp/out.txt)"
done
