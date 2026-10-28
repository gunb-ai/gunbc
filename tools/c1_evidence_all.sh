set -uo pipefail
SHA="$(git rev-parse HEAD)"; echo "SUBJECT_SHA=$SHA"
export CARGO_TARGET_DIR="$PWD/target"
echo "=== BUILD ==="
/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc || { echo "BUILD_FAILED"; exit 1; }
G=./target/release/gunbc
echo; echo "=== CLAIMS: every test fn in the cast-wall witnesses ==="
total=0; held=0; fails=""
for m in non_text_cast_into_text_wall_witness host_text_crossing_entry_witness nominal_cast_wall_witness emitter_nat_int_cast_witness emitter_nested_refinement_cast_witness optional_cast_wall_witness refinement_cast_wall_witness; do
  f="dag/test/claim/${m}.dag"; [ -f "$f" ] || { echo "  MISSING $f"; continue; }
  for fn in $(grep -oE "test fn w_[a-z_0-9]+" "$f" | awk '{print $3}' | sort -u); do
    line=$($G test "//test/claim/${m}:${fn}" 2>&1 | grep "claim route:" | tail -1)
    total=$((total+1))
    if printf '%s' "$line" | grep -q "1 held"; then held=$((held+1)); else fails="$fails
    $m:$fn :: $line"; fi
  done
done
echo "CLAIM_TALLY total=$total held=$held did_not_hold=$((total-held))"
[ -n "$fails" ] && { echo "NOT_HELD:"; printf '%s\n' "$fails"; }
echo; echo "=== CONTROLS (named, verbatim verdict lines) ==="
for c in w_int_as_string_is_refused w_int_as_string_refusal_names_the_cast w_declared_int_to_text_route_is_accepted w_bool_as_int_is_still_refused w_bool_as_nat_is_refused w_nat_as_string_is_refused; do
  printf '  %-44s ' "$c"
  $G test "//test/claim/non_text_cast_into_text_wall_witness:${c}" 2>&1 | grep -E "HELD|did not hold|DID NOT" | tail -1
done
echo; echo "=== CENSUS (recorded command, --output-dir) ==="
rm -rf target/c1-census && mkdir -p target/c1-census
$G compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir target/c1-census > /tmp/census.txt 2>&1
echo "census_rc=$? bytes=$(wc -c < /tmp/census.txt)"
grep -iE "blocking error" /tmp/census.txt | tail -2
echo "invalid_cast_lines=$(grep -c 'invalid cast' /tmp/census.txt || true)"
grep 'invalid cast' /tmp/census.txt | head -8 | sed 's/^/  IC /'
echo; echo "=== EMIT ROUTE: the two named modules ==="
for m in gunbc.durable_cas_file_store extdeps.exec.command; do
  out=target/c1-emit-$(echo $m | tr . _); rm -rf $out && mkdir -p $out
  echo "--- entry $m ---"
  $G compile --source-root dag --source-root src/v2 --entry $m --target rust --output-dir $out > /tmp/e.txt 2>&1
  echo "  rc=$? log_bytes=$(wc -c < /tmp/e.txt)"
  find $out -name '*.rs' | head -3 | sed 's/^/  found /'
  for f in gunbc_durable_cas_file_store.rs exec_command.rs; do
    pth=$(find $out -name "$f" | head -1)
    [ -z "$pth" ] && { echo "  $f: NOT EMITTED"; continue; }
    echo "  $f lines=$(wc -l < "$pth") unsupported_cast=$(grep -c 'unsupported cast' "$pth" || true) int_to_decimal_string=$(grep -c 'int_to_decimal_string' "$pth" || true)"
    grep -n 'unsupported cast' "$pth" | head -4 | sed 's/^/    /'
  done
done
echo; echo MARKER_C1_ALL_DONE
