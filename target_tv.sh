CG=/sys/fs/cgroup/gunbc-probe; S=""; command -v sudo >/dev/null && S=sudo
echo "+memory" | $S tee /sys/fs/cgroup/cgroup.subtree_control >/dev/null; $S mkdir -p $CG; echo 17179869184 | $S tee $CG/memory.max >/dev/null; echo $$ | $S tee $CG/cgroup.procs >/dev/null
git rev-parse --short HEAD
cargo build --release -p v1-compiler --bin gunbc > /dev/null 2>&1; echo BUILD=$?
D=src/v2/test/claim/manual
printf 'module v2.test.manual.ctl_trivial\n\nimport v2.std.logic { Bool }\n\ntest fn ctl_t() -> Bool { true }\n' > $D/ctl_trivial_test.dag
./target/release/gunbc run --claim-run --source-root dag --source-root src/v2 --entry $D/ctl_trivial_test.dag 2>&1 | grep -E "phase=|population|process_cpu|PASS|FAIL" | cut -c1-200
