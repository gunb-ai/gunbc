CG=/sys/fs/cgroup/gunbc-probe; S=""; command -v sudo >/dev/null && S=sudo
echo "+memory" | $S tee /sys/fs/cgroup/cgroup.subtree_control >/dev/null; $S mkdir -p $CG; echo 17179869184 | $S tee $CG/memory.max >/dev/null; echo $$ | $S tee $CG/cgroup.procs >/dev/null
cargo build --release -p v1-compiler --bin gunbc > /dev/null 2>&1
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/docs_projection_gate.dag --function regen > /tmp/r.txt 2>&1; echo REGEN=$?
git diff --stat -- docs/
git diff -- docs/design-rung-drops.md | head -30 | cut -c1-200
echo ===PATCH-BEGIN===; git diff -- docs/ | gzip -9 | base64 -w0; echo; echo ===PATCH-END===
