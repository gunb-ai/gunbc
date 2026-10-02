CG=/sys/fs/cgroup/gunbc-probe; S=""; command -v sudo >/dev/null && S=sudo
echo "+memory" | $S tee /sys/fs/cgroup/cgroup.subtree_control >/dev/null; $S mkdir -p $CG; echo 23622320128 | $S tee $CG/memory.max >/dev/null; echo $$ | $S tee $CG/cgroup.procs >/dev/null
cargo clippy -p v1-compiler --lib --tests -- -D warnings 2>&1 | grep -E "^(error|warning)|-->" | head -20; echo CLIPPY_DONE
cargo test --release -p v1-compiler --lib changed_witness_sublane_join_tests 2>&1 | grep -E "test result|error\[" | head -3
