cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
for c in m_reveal_fold_type n_annotated_accumulator o_annotated_lambda_param; do
  echo "########## $c ##########"
  rm -rf /tmp/out_d
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/$c.dag --target rust --output-dir /tmp/out_d 2>&1 | grep -E '^(error|advisory\[[^]]*\]: (type|cast|unsupported))|blocking' | head -8
  if [ -d /tmp/out_d ]; then
    grep -rn 'panic!("unsupported cast' /tmp/out_d | sed 's|/tmp/out_d/|  FABRICATED: |' | head -3 || echo "  NO-FABRICATED-PANIC"
    find /tmp/out_d -name "$c.rs" -exec sed -n '/fn probe_case/,/^}/p' '{}' \; | sed 's/^/  EMIT /' | head -12
  fi
done
