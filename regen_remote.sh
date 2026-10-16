cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | grep -E "^error|Finished" | head
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1; grep -n -m3 "refus" /tmp/regen.out | cut -c1-250; tail -4 /tmp/regen.out | cut -c1-200
echo "=== CAND"
grep -rhn "compile_error!(\"shared layer refused" target/stage0-regen-candidate/src 2>/dev/null | sed 's/.*refused for/refused for/' | cut -c1-90 | sort | uniq -c | sort -rn | head -15
grep -rc "compile_error!(\"shared layer refused" target/stage0-regen-candidate/src | grep -v ":0" | head -10
grep -n -B3 "refused for .Hash" target/stage0-regen-candidate/src/v1_compiler_dag_collect_support.rs | cut -c1-220 | head -20
grep -n -B2 -A1 "refused for .Nat" target/stage0-regen-candidate/src/std_measure.rs | cut -c1-200 | head -24
echo "=== GREP"
grep -n "record_field_declaring_generic_names" src/v1/stage0/src/v1_compiler_emit_rust.rs | head -3
git diff --stat | tail -12
echo "=== SELFHOST"
cargo build --release --bin gunbc 2>&1 | tail -2; target/release/gunbc test //gunbc/instruments:self-host 2>&1 | tail -15
echo "=== PATCH"
git diff -- src/v1/stage0 | gzip -9 | base64 -w0
echo
echo "=== END"
