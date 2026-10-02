CG=/sys/fs/cgroup/gunbc-probe; S=""; command -v sudo >/dev/null && S=sudo
echo "+memory" | $S tee /sys/fs/cgroup/cgroup.subtree_control >/dev/null; $S mkdir -p $CG; echo 23622320128 | $S tee $CG/memory.max >/dev/null; echo $$ | $S tee $CG/cgroup.procs >/dev/null
for t in cross_tree_bare_reference_tests strict_refusal_counts_blocking_diagnostics changed_witness_sublane_join_tests; do
cargo test --release -p v1-compiler --lib $t 2>&1 | grep -E "test result|error\[|FAILED|panicked" | head -4; done
