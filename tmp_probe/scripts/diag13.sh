cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A8 | head -30
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p > /tmp/p.txt 2>&1
grep DIAGTMPL /tmp/p.txt | sort | uniq -c | sort -rn | head -20
echo "--- emitted ---"
grep -rn 'panic!("unsupported cast' /tmp/out_p | sed 's|/tmp/out_p/|FAB: |' || echo NOFAB
echo "===== wrapper (control) ====="
rm -rf /tmp/out_w; ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/w_generic_wrapper.dag --target rust --output-dir /tmp/out_w > /tmp/w.txt 2>&1
grep DIAGTMPL /tmp/w.txt | sort | uniq -c | sort -rn | head -12
