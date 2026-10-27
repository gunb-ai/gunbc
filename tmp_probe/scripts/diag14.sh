cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -30
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p > /tmp/p.txt 2>&1
echo "--- DIAGMINT (fabrication) ---"; grep DIAGMINT /tmp/p.txt | sort | uniq -c | sort -rn | head -15
echo "--- DIAGMINT2 (unbound var) ---"; grep DIAGMINT2 /tmp/p.txt | sort | uniq -c | sort -rn | head -10
echo "--- DIAGELEM ---"; grep DIAGELEM /tmp/p.txt | sort | uniq -c | sort -rn | head -12
grep -rn 'panic!("unsupported cast' /tmp/out_p | sed 's|.*|FAB STILL|' | head -1
