cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A12 | head -40
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p 2>&1 | grep -E 'DIAGBIND bind="T"' | sort | uniq -c | head -25
