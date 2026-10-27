cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A12 | head -40
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p 2>&1 | grep -vE '^\[pre-entry\]|advisory|^\s*\||^\s*[0-9]+ \||^ ' | tail -6
echo "----- emitted -----"
find /tmp/out_p -name 'a_fold_accumulator.rs' -exec sed -n '/fn probe_case/,/^}/p' {} \;
grep -rn 'panic!("unsupported cast' /tmp/out_p | sed 's|/tmp/out_p/|FABRICATED: |' || echo "NO FABRICATED PANIC"
