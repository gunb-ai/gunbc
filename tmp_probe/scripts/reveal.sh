cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -30
for c in r1_list_push_empty_init r2_fold_accumulator r3_list_push_typed_init; do
  echo "########## $c ##########"
  rm -rf /tmp/out_r
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/$c.dag --target rust --output-dir /tmp/out_r 2>&1 | grep -E '^error' -A4 | grep -E "expected|expected:|got|type|reveal" | head -12
done
