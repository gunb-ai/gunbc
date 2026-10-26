cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
for c in a_fold_accumulator g_alias_param h_fold_returned_directly; do
  echo "########## $c ##########"
  ./target/release/gunbc compile --source-root dag --source-root tmp_probe --entry tmp_probe/cases/$c.dag --target rust --output-dir /tmp/out_d 2>&1 | grep DIAGFOLD | grep 'mod="tprobe'
done
