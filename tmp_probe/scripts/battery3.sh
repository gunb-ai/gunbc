cd /root/workspace/repo-root
export RUSTFLAGS="-Cdebuginfo=2"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E '^error' -A10 | head -30
for c in a_fold_accumulator r1_list_push_empty_init r2_fold_accumulator r3_list_push_typed_init w_generic_wrapper w2_generic_wrapper_distinct g_alias_param f_carrier_param c_fold_field_read d_fold_list_map i_fold_nested_in_hof; do
  rm -rf /tmp/o_$c
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/$c.dag --target rust --output-dir /tmp/o_$c > /tmp/b_$c.txt 2>&1
  P=$(grep -rl 'panic!("unsupported cast' /tmp/o_$c 2>/dev/null | wc -l)
  E=$(grep -cE '^(error|  x )' /tmp/b_$c.txt)
  echo "CASE $c panicfiles=$P errlines=$E"
done
echo "--- VAC refusals ---"; grep -h 'DIAGVAC' /tmp/b_a_fold_accumulator.txt | head -6
echo "--- mint traces on red probe ---"; grep -h 'MINT-RESOLVE-OPT\|MINT-ENV-TPBIND' /tmp/b_a_fold_accumulator.txt | sort -u | head -6
echo "--- emitted keys line (red probe) ---"; grep -rn 'distinct_by_key' /tmp/o_a_fold_accumulator 2>/dev/null | head -4
echo "--- store site ---"; rm -rf /tmp/o_store; ./target/release/gunbc compile --source-root dag --entry dag/gunbc/durable_cas_file_store.dag --target rust --output-dir /tmp/o_store > /tmp/b_store.txt 2>&1; grep -rl 'panic!("unsupported cast' /tmp/o_store | wc -l; grep -rn 'distinct_by_key' /tmp/o_store 2>/dev/null | head -3
