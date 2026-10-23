cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
timeout 2400 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry $PWD/src/v2/workflow/roster_import_sweep.dag --function roster_controls_report > /tmp/full.log 2>&1; echo "rc=$?"; echo "bytes=$(wc -c < /tmp/full.log)"; grep -E "CONTROL-FAIL|held|error" /tmp/full.log | cut -c1-400 | head -20; tail -c 600 /tmp/full.log
echo ==END
