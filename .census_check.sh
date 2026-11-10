set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
B=./target/release/gunbc
ls -la $B || exit 3
fail=0
for e in "$@"; do
  out=$(mktemp -d)
  if $B compile --source-root dag --source-root src/v2 --entry "$e" --output-dir "$out" --target dag >"$out.log" 2>&1; then echo "OK   $e"; else echo "FAIL $e"; tail -15 "$out.log"; fail=1; fi
done
exit $fail
