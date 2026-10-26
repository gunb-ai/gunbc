set -e
cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
for c in f_carrier_param g_alias_param h_fold_returned_directly i_fold_nested_in_hof; do
  echo "########## CASE $c ##########"
  rm -rf /tmp/out_case
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe \
    --entry tmp_probe/cases/$c.dag --target rust --output-dir /tmp/out_case 2>&1 \
    | grep -E '^(error|fatal)|blocking' | sed 's/^/  DIAG /' || true
  if [ -d /tmp/out_case ]; then
    grep -rn 'panic!("unsupported cast' /tmp/out_case | sed 's|/tmp/out_case/|  FABRICATED-PANIC: |' || echo "  NO-FABRICATED-PANIC"
    find /tmp/out_case -name "$c.rs" -exec sed -n '/fn probe_case/,/^}/p' '{}' \; | sed 's/^/  EMIT /'
  fi
done
