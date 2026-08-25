set -u
echo "MARKER_HEAD=$(git rev-parse --short HEAD)"
echo "MARKER_SRC_DIGEST=$(md5sum < src/v1/05_emit_rust.dag)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
BIN=./target/release/gunbc
echo "MARKER_BIN_DIGEST=$(md5sum < $BIN)"
rm -rf /tmp/cand
set +e
$BIN compile --source-root dag --source-root src/v2 --entry src/v2/compiler/03_ingest.dag --output-dir /tmp/cand --target rust > /tmp/emit.log 2>&1
echo "MARKER_EMIT_RC=$?"
set -e
tail -2 /tmp/emit.log
echo "MARKER_FILES=$(find /tmp/cand -name '*.rs' | wc -l)  cargo_toml=$(ls /tmp/cand/Cargo.toml 2>/dev/null | wc -l)"
ls /tmp/cand | head
echo MARKER_SWEEP_DONE
