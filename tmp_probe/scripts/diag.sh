cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/h_fold_returned_directly.dag --target rust --output-dir /tmp/out_d > /tmp/all.txt 2>&1
grep DIAGFOLD /tmp/all.txt > /tmp/d.txt
echo "TOTAL=$(wc -l < /tmp/d.txt)"
echo "---- frames with a lambda arg ----"
grep 'is_lambda=true' /tmp/d.txt | sort -u | head -40
echo "---- frames mentioning the probe module's fn ----"
grep 'tprobe\|probe_case' /tmp/d.txt | sort -u | head -20
