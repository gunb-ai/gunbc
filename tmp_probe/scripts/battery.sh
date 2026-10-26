cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
for c in e_fold_nonempty_seed n_annotated_accumulator o_annotated_lambda_param a_fold_accumulator; do
  echo "########## $c ##########"
  rm -rf /tmp/out_case
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/$c.dag --target rust --output-dir /tmp/out_case 2>&1 | grep -E '^(error|fatal)|blocking' | sed 's/^/  DIAG /'
  if [ -d /tmp/out_case ]; then
    find /tmp/out_case -name "*.rs" | grep -v pre_entry | sed 's/^/  FILE /'
    grep -rn 'panic!("unsupported cast' /tmp/out_case | sed 's|/tmp/out_case/|  FABRICATED: |'
    find /tmp/out_case -name "*probe*.rs" -exec cat '{}' \; | grep -E "fn probe_case|distinct_by_key|list_map|=> " | sed 's/^/  EMIT /'
  else
    echo "  NO OUTPUT (compile refused)"
  fi
done
