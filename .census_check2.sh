set -u
cargo build --release -p v1-compiler --bin gunbc > build.log 2>&1; rc=$?
grep -E "^(warning|error)" -A6 build.log | grep -E "never used|never constructed|never read|unused|^error|-->" | head -300
echo "BUILD_RC=$rc"
[ $rc -eq 0 ] || { tail -80 build.log; exit 3; }
B=./target/release/gunbc
fail=0
for e in "$@"; do
  out=$(mktemp -d)
  if $B compile --source-root dag --source-root src/v2 --entry "$e" --output-dir "$out" --target dag >"$out.log" 2>&1; then echo "OK   $e"; else echo "FAIL $e"; tail -15 "$out.log"; fail=1; fi
done
exit $fail
