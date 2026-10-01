W=/home/briansrls/.worktrees/gunbc/royal-newt-820-mainbuild
cd $W && CARGO_TARGET_DIR=/tmp/claude-1000/main-target CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|Finished"
cd /home/briansrls/.worktrees/gunbc/royal-newt-820; bash /tmp/claude-1000/mregen.sh > /tmp/claude-1000/mregen.log 2>&1; grep -v Compiling /tmp/claude-1000/mregen.log
B=/tmp/claude-1000/royal-target/release
timeout 900 $B/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=docs/design-rung-drops.md 2>&1 | tail -1 | cut -c1-90
for F in text_boundary_identity_wall_witness_test kernel_refinement_at_structured_parameter_witness_test optional_at_required_position_witness_test; do f=$(find dag src/v2 -name "$F.dag" | head -1); echo "== $f"; timeout 1500 $B/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $(grep -oP '(?<=^test fn )\w+' $f | paste -sd,) 2>&1 | grep -E "^PASS|^FAIL|THREW|REFUS" | cut -c1-140; done
cargo fmt --all --check >/dev/null 2>&1; echo fmt=$?; echo M4DONE
