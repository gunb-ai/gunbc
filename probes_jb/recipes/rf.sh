echo SRC=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
echo BIN=$(sha256sum target/release/gunbc | cut -c1-16)
./target/release/gunbc compile --output-dir /tmp/pZr --source-root dag --source-root src/v2 --target dag --entry dag/gunbc/recurring_failure_mode/v1_accepts_a_container_at_a_coproduct_annotation_and_a_split_derived_element_untyped.dag > /tmp/zzr.out 2>&1; echo "rc=$?"
grep -E "^error" -A2 /tmp/zzr.out | cut -c1-200
echo ==END
