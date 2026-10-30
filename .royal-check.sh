set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
G=./target/release/gunbc
for e in "$@"; do
  echo "=================== $e"
  case "$e" in
    dag/test/*) timeout 1500 $G run --source-root dag --source-root src/v2 --entry "$e" --claim-run > /tmp/r.log 2>&1; rc=$?;;
    *) timeout 900 $G compile --source-root dag --source-root src/v2 --entry "$e" --output-dir /tmp/out --target dag > /tmp/r.log 2>&1; rc=$?;;
  esac
  echo "rc=$rc"
  grep -E 'error|FAIL|PASS|passed|failed|blocking|refused|expected|unparseable' /tmp/r.log | grep -vE '^\s*[0-9]+ \|' | sort | uniq -c | sort -rn | head -50
done
