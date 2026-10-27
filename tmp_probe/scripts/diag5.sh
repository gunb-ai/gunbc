cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A12 | head -40
echo "===== probe: ALL tprobe frames + bindings ====="
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p 2>&1 | grep -E 'DIAGARG|DIAGBIND' | grep -E 'mod="tprobe'
echo "===== emitted body ====="
find /tmp/out_p -name 'a_fold_accumulator.rs' -exec sed -n '/fn probe_case/,/^}/p' {} \;
