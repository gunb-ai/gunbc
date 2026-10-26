set -e
cd /root/workspace/repo-root
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
echo '===== full diagnostics ====='
./target/release/gunbc compile \
  --source-root dag --source-root tmp_probe \
  --entry tmp_probe/tprobe/lambda_hof_param_red_probe.dag \
  --target rust --output-dir /tmp/out_red 2>&1 | grep -v '^\[pre-entry\]' | grep -v '^\s*|' | grep -v '^\s*[0-9]* |' | grep -v '^ ' || true
echo '----- FABRICATED RUNTIME PANICS (the red) -----'
grep -rn 'panic!("unsupported cast' /tmp/out_red | sed 's|/tmp/out_red/||' || echo '(none - green)'
echo '----- emitted probe bodies -----'
find /tmp/out_red -name 'lambda_hof_param_red_probe.rs' -exec sed -n '1,140p' '{}' \;
