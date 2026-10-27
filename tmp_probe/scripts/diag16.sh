cd /root/workspace/repo-root
export RUSTFLAGS="-Cdebuginfo=2"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -40
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p > /tmp/p.txt 2>&1
echo "--- DIAGCAST ---"; grep 'DIAGCAST' /tmp/p.txt | head -6
echo "--- DIAGBIND (T-ish) ---"; grep 'DIAGBIND' /tmp/p.txt | grep -i 'list_push\|var=T\|distinct' | head -30
echo "--- DIAGBIND all sample ---"; grep -c 'DIAGBIND' /tmp/p.txt
grep 'DIAGBIND' /tmp/p.txt | head -25
grep -rn 'panic!("unsupported cast' /tmp/out_p | sed 's|.*|FAB-STILL-RED|' | head -1
