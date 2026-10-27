cd /root/workspace/repo-root
git apply tmp_probe/scripts/patch17.py 2>/dev/null || true
export RUSTFLAGS="-Cdebuginfo=2"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -40
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p > /tmp/p.txt 2>&1
echo "--- DIAGMINTRL ---"; grep 'DIAGMINTRL' /tmp/p.txt | head -20
echo "--- DIAGFIELD (T) ---"; grep 'DIAGFIELD' /tmp/p.txt | grep 'T' | head -20
echo "--- DIAGBIND (T) ---"; grep 'DIAGBIND' /tmp/p.txt | grep 'var=T' | head -20
echo "--- DIAGCAST ---"; grep 'DIAGCAST' /tmp/p.txt | head -4
grep -rn 'panic!("unsupported cast' /tmp/out_p | sed 's|.*|FAB-STILL-RED|' | head -1
