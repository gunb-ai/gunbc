set -x
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
for f in run_02_parse run_source_authority run_target_model run_grammar; do
  echo "===== $f"; timeout 1500 ./target/release/gunbc run --source-root dag --source-root src/v2 --source-root wrprobe --entry wrprobe/wr_probe.dag --function $f 2>&1 | head -150
done
