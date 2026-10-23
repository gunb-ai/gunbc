echo SRC=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
echo BIN=$(sha256sum target/release/gunbc | cut -c1-16)
for k in fa fb fc; do echo "--$k"; ./target/release/gunbc compile --output-dir /tmp/pZ$k --source-root dag --source-root src/v2 --target dag --entry dag/gunbc/zz_$k.dag > /tmp/zz$k.out 2>&1; echo "rc=$?"; grep -E "^error" -A2 /tmp/zz$k.out | cut -c1-170; done
echo "--native"
timeout 1500 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry $PWD/src/v2/test/claim/zz_scratch_listat_probe.dag --function lap_report > /tmp/lap.log 2>&1; echo "rc=$?"; grep -E "int_as_bool|rror:" /tmp/lap.log | cut -c1-1500
echo ==END
