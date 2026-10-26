cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -30
echo "===== probe a_fold_accumulator ====="
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_d 2>&1 | grep -E 'DIAGARG|DIAGFOLD' | grep -E 'tprobe|std_algebra|std\.algebra|list_push' | head -40
echo "===== real store fold (cas_slot_keys) ====="
./target/release/gunbc compile --source-root dag --entry dag/gunbc/durable_cas_file_store.dag --target rust --output-dir /tmp/out_s 2>&1 | grep -E 'DIAGARG|DIAGFOLD' | grep -E 'cas_slot_keys' | head -40
