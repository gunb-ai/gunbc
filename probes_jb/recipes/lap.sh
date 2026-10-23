cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
echo SRC=$(git rev-parse HEAD); echo BIN=$(sha256sum target/release/gunbc | cut -c1-16)
timeout 1500 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry $PWD/src/v2/test/claim/zz_scratch_listat_probe.dag --function lap_report > /tmp/lap.log 2>&1; echo "rc=$?"; grep -E "int_as_bool|error|rror:" /tmp/lap.log | cut -c1-1500
echo ==END
