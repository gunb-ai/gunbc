set -e
cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
for c in a_fold_accumulator b_declared_alias_param; do
  echo "########## $c ##########"
  rm -rf /tmp/out_dag
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe \
    --entry tmp_probe/cases/$c.dag --target dag --output-dir /tmp/out_dag 2>&1 | grep -E '^(error|fatal)|blocking' || true
  find /tmp/out_dag -name "*$c*" | head -3
  find /tmp/out_dag -name "*$c*" -exec sed -n '1,80p' '{}' \;
done
