set +e
cargo build --release -p v1-compiler --bin gunbc >/dev/null 2>&1 || { echo BUILD_FAILED; exit 1; }
G=./target/release/gunbc; S="--source-root dag --source-root src/v2"
echo "== regen"; rm -rf /tmp/regen_out; mkdir -p /tmp/regen_out/.github/workflows
$G run $S --entry dag/test/scratch_regen_probe.dag --function run 2>&1 | grep -v "^\[pre-entry\]" | tail -4
f=.github/workflows/fleet-converge.yml; cmp -s /tmp/regen_out/$f $f && echo "SAME $f" || { echo "DIFF $f"; diff /tmp/regen_out/$f $f | head -20; }
for w in workflow_dispatch_input_witness; do echo "== $w"; $G run $S --entry dag/test/claim/$w.dag --claim-run 2>&1 | grep -vE "^\s*(✓|◐)|user_work_scope|workspace-root|^\[pre-entry\]" | tail -40; done
