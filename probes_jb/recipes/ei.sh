echo SRC=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
echo BIN=$(sha256sum target/release/gunbc | cut -c1-16)
./target/release/gunbc compile --output-dir /tmp/pZei --source-root dag --source-root src/v2 --target dag --entry dag/gunbc/zz_ei.dag > /tmp/zzei.out 2>&1; echo "rc=$?"
grep -E "^error" -A2 /tmp/zzei.out | cut -c1-160
echo ==END
