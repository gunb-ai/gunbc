cd /root/workspace/repo-root
export RUSTFLAGS="-Cdebuginfo=2"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -30
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p > /tmp/p.txt 2>&1
echo "--- DIAGCAST ---"; grep 'DIAGCAST' /tmp/p.txt | head -4
echo "--- DIAGLAMNONE ---"; grep 'DIAGLAMNONE' /tmp/p.txt | head -8
echo "--- DIAGLAM k ---"; grep 'DIAGLAM param=k' /tmp/p.txt | head -4
echo "--- DIAGBIND aspan list/probe ---"; grep 'DIAGBIND var=T' /tmp/p.txt | grep 'children=0' | cut -c1-230 | head -30
grep -rn 'panic!("unsupported cast' /tmp/out_p | sed 's|.*|FAB-STILL-RED|' | head -1
