cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A12 | head -40
for c in a_fold_accumulator c_fold_field_read d_fold_list_map r1_list_push_empty_init w_generic_wrapper w2_generic_wrapper_distinct; do
  echo "########## $c ##########"
  rm -rf /tmp/out_c
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/$c.dag --target rust --output-dir /tmp/out_c 2>&1 | grep -E '^(error|fatal)|blocking error' | head -5
  if [ -d /tmp/out_c ]; then
    grep -rn 'panic!("unsupported cast' /tmp/out_c | sed 's|/tmp/out_c/|FABRICATED: |' || echo "  NO FABRICATED PANIC"
    find /tmp/out_c -name "*$c.rs" -exec sed -n '/fn probe_case\|fn push_one\|fn distinct_ours/,/^}/p' {} \; | sed 's/^/  EMIT /'
  else
    echo "  NO OUTPUT (refused)"
  fi
done
echo "########## real store cas_slot_keys ##########"
rm -rf /tmp/out_s
./target/release/gunbc compile --source-root dag --entry dag/gunbc/durable_cas_file_store.dag --target rust --output-dir /tmp/out_s 2>&1 | grep -E '^(error|fatal)|blocking error' | head -8
find /tmp/out_s -name "*durable_cas_file_store.rs" | head -2
find /tmp/out_s -name "*durable_cas_file_store.rs" -exec grep -n 'panic!("unsupported cast' {} \; | head -3 || true
find /tmp/out_s -name "*durable_cas_file_store.rs" -exec grep -n -A3 "distinct_by_key" {} \; | head -12
