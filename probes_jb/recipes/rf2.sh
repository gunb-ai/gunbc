echo SRC=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
./target/release/gunbc compile --output-dir /tmp/pZr --source-root dag --source-root src/v2 --target dag --entry dag/gunbc/recurring_failure_mode/v1_elaborates_bare_none_to_optional_unit_and_lets_unit_inhabit_bool.dag > /tmp/zzr.out 2>&1; echo "rc=$?"
grep -E "^error" -A2 /tmp/zzr.out | cut -c1-200
echo ==END
