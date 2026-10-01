set -u
cd /home/briansrls/.worktrees/gunbc/royal-newt-820
export CARGO_TARGET_DIR=/tmp/claude-1000/royal-target; B=$CARGO_TARGET_DIR/release
bld(){ CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_batch --bin claim_executor --bin gunbc --bin infer_semantics_witness 2>&1 | grep -E "^error|Finished"; }
echo "== regen0 with main binary"; rm -rf target/stage0-regen-candidate; /tmp/claude-1000/main-target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/claude-1000/mregen_0.txt 2>&1; grep -E "first_generation|FAIL" /tmp/claude-1000/mregen_0.txt | head -3; [ -d target/stage0-regen-candidate/src ] || { echo NO_CAND0; grep -iE "error|refus" /tmp/claude-1000/mregen_0.txt | head -8 | cut -c1-300; exit 1; }; cp target/stage0-regen-candidate/src/*.rs src/v1/stage0/src/; python3 /tmp/claude-1000/patch_witness.py; echo "== build0"; bld
for i in 1 2; do
 echo "== regen $i"; rm -rf target/stage0-regen-candidate; $B/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/claude-1000/mregen_$i.txt 2>&1
 grep -E "first_generation|FAIL" /tmp/claude-1000/mregen_$i.txt | head -3 | cut -c1-200
 [ -d target/stage0-regen-candidate/src ] || { echo NO_CAND; grep -iE "error|refus" /tmp/claude-1000/mregen_$i.txt | head -5 | cut -c1-300; exit 1; }
 cp target/stage0-regen-candidate/src/*.rs src/v1/stage0/src/; python3 /tmp/claude-1000/patch_witness.py; git diff --stat -- src/v1/stage0/src | tail -1
 echo "== build $i"; bld
done
echo MDONE
