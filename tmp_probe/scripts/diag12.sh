cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A12 | head -40
echo "===== self-binds during probe compile ====="
./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/a_fold_accumulator.dag --target rust --output-dir /tmp/out_p 2>&1 | grep -E 'DIAGTMPL' | sort | uniq -c | sort -rn | head -25
echo "===== wrapper controls ====="
for w in w_generic_wrapper w2_generic_wrapper_distinct; do
  echo "--- $w"
  rm -rf /tmp/out_w
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/$w.dag --target rust --output-dir /tmp/out_w 2>&1 | grep -vE '^\[pre-entry\]|advisory|^\s*\||^\s*[0-9]+ \||^ ' | tail -4
  find /tmp/out_w -name '*.rs' | sed 's/^/  FILE /' | head -5
  find /tmp/out_w -name "*.rs" -exec grep -l "fn probe_case" {} \; | head -2 | xargs -r sed -n '/fn push_one\|fn distinct_ours\|fn probe_case/,/^}/p' 2>/dev/null | head -20
done
