set -u
python3 fixtures/trace.py
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|Finished" -A5 | head -30
for e in kci_kernel kci_foreign; do
  echo "== $e"
  timeout 900 ./target/release/gunbc compile --source-root dag --entry dag/test/probe/$e.dag --output-dir /tmp/out_$e --target rust 2>&1 | grep -E "TRACE|error|blocking|refus" | cut -c1-900 | head -40
done
