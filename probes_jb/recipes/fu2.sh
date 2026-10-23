echo SRC=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
echo BIN=$(sha256sum target/release/gunbc | cut -c1-16)
for k in fa fb fc; do echo "--$k"; ./target/release/gunbc compile --output-dir /tmp/pZ$k --source-root dag --source-root src/v2 --target dag --entry dag/gunbc/zz_$k.dag > /tmp/zz$k.out 2>&1; echo "rc=$?"; grep -E "^error" -A2 /tmp/zz$k.out | cut -c1-170; done
