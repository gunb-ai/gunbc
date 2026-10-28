set -uo pipefail
SHA="$(git rev-parse HEAD)"; echo "SUBJECT_SHA=$SHA"
export CARGO_TARGET_DIR="$PWD/target"
/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc || { echo "BUILD_FAILED"; exit 1; }
G=./target/release/gunbc
echo "=== CENSUS (recorded command, --output-dir) ==="
rm -rf target/c1-census && mkdir -p target/c1-census
$G compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir target/c1-census > /tmp/census.txt 2>&1
echo "census_rc=$? bytes=$(wc -c < /tmp/census.txt)"
grep -iE "blocking error" /tmp/census.txt | tail -2
echo "invalid_cast_lines=$(grep -c 'invalid cast' /tmp/census.txt || true)"
grep 'invalid cast' /tmp/census.txt | head -6 | sed 's/^/  IC /'
echo "=== EMIT: the two named modules ==="
for m in gunbc.durable_cas_file_store extdeps.exec.command; do
  out=target/c1-emit-$(echo $m | tr . _); rm -rf $out && mkdir -p $out
  echo "--- entry $m ---"
  $G compile --source-root dag --source-root src/v2 --entry $m --target rust --output-dir $out > /tmp/e.txt 2>&1
  echo "  rc=$? log=$(wc -c < /tmp/e.txt)"
  tail -3 /tmp/e.txt | sed 's/^/  OUT /'
  for f in gunbc_durable_cas_file_store.rs exec_command.rs; do
    pth=$(find $out -name "$f" 2>/dev/null | head -1)
    [ -z "$pth" ] && { echo "  $f: NOT EMITTED"; continue; }
    echo "  $f lines=$(wc -l < "$pth") unsupported_cast=$(grep -c 'unsupported cast' "$pth" || true) int_to_decimal_string=$(grep -c 'int_to_decimal_string' "$pth" || true)"
    grep -n 'unsupported cast' "$pth" | head -4 | sed 's/^/    /'
  done
done
echo MARKER_C1_LEAN_DONE
