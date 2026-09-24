set -x
export CARGO_TARGET_DIR=$PWD/target
cargo build --release -p v1-compiler --bin claim_executor --bin v1_src_dag_parse 2>&1 | tail -2
for f in src/v1/05_emit_rust.dag dag/std/operator_realization.dag; do ./target/release/v1_src_dag_parse $f 2>&1 | tail -2 || exit 1; done
for i in 1 2; do
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | grep -E "first_generation_equal|FAIL|error|panick" | head -20
cp target/stage0-regen-candidate/src/*.rs src/v1/stage0/src/
done
echo "=====BEGIN STAGE0 DIFF====="
git diff src/v1/stage0 | base64 -w0; echo
echo "=====END STAGE0 DIFF====="
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error" -A8 | head -40
G=./target/release/gunbc
echo "INTERP:"; $G run --source-root dag --source-root wh_probe --entry wh_probe/emit_probe.dag 2>&1 | tail -3
rm -rf /tmp/pout; $G compile --source-root dag --source-root wh_probe --entry wh_probe/emit_probe.dag --output-dir /tmp/pout --target rust 2>&1 | tail -5
grep -n "fn ts_before\|fn first_or_zero\|fn bytes_octets" -A7 /tmp/pout/src/probe_emit_probe.rs /tmp/pout/src/std_bytes.rs
(cd /tmp/pout && cargo build --release 2>&1 | grep -E "^error" -A10 | head -80; echo "EMITTED:"; ./target/release/$(grep -m1 '^name' Cargo.toml | cut -d'"' -f2) 2>&1 | tail -3; ls target/release | grep -v '\.d$' | head)
