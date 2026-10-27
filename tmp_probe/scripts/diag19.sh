cd /root/workspace/repo-root
export RUSTFLAGS="-Cdebuginfo=2"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -30
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p > /tmp/p.txt 2>&1
echo "--- DIAGLAM ---"; grep 'DIAGLAM' /tmp/p.txt | head -12
echo "--- DIAGSUBST ---"; grep 'DIAGSUBST' /tmp/p.txt | head -12
echo "--- MINT ---"; grep 'MINT-' /tmp/p.txt | head -14
grep -rn 'panic!("unsupported cast' /tmp/out_p | sed 's|.*|FAB-STILL-RED|' | head -1
