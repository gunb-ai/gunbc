cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A12 | head -40
echo "===== chain dump at distinct_by_key ====="
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p 2>&1 | grep DIAGARG | grep caller="probe_case"
echo "===== wrapper control w_generic_wrapper ====="
rm -rf /tmp/out_w; ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/w_generic_wrapper.dag --target rust --output-dir /tmp/out_w 2>&1 | grep -E '^error|blocking' | head -4
find /tmp/out_w -name 'w_generic_wrapper.rs' -exec sed -n '/fn push_one/,/^}/p;/fn probe_case/,/^}/p' {} \; 2>/dev/null
echo "===== wrapper control w2 ====="
rm -rf /tmp/out_w2; ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/w2_generic_wrapper_distinct.dag --target rust --output-dir /tmp/out_w2 2>&1 | grep -E '^error|blocking' | head -4
find /tmp/out_w2 -name 'w2_generic_wrapper_distinct.rs' -exec sed -n '/fn distinct_ours/,/^}/p;/fn probe_case/,/^}/p' {} \; 2>/dev/null
