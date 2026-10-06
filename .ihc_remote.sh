set -uo pipefail
echo "HEAD $(git rev-parse HEAD)"
export CARGO_TARGET_DIR=$PWD/target
mkdir -p /tmp/hold && cp src/v1/stage0/src/target_invocation_host.rs src/v1/stage0/src/cli_run.rs /tmp/hold/
git checkout -- src/v1/stage0/src/target_invocation_host.rs src/v1/stage0/src/cli_run.rs; git status --short src/v1/stage0
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error" -A5 | head -40
CG=/sys/fs/cgroup/gunbcrun; mkdir -p $CG; echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null; echo 25769803776 > $CG/memory.max; echo $$ > $CG/cgroup.procs; echo "cgroup: $(cat /proc/self/cgroup) max=$(cat $CG/memory.max)"
for round in 1 2 3; do
  ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/cand$round --regen-receipt /tmp/receipt$round.txt > /tmp/regen$round.log 2>&1
  echo "round $round exit=$?"; grep -E "changed_paths|FAIL|refused" /tmp/regen$round.log | head -10
  grep -v "^regen-round-cost: phase\|unattributed" /tmp/regen$round.log | tail -40 | cut -c1-600
  grep -q "changed_paths=0" /tmp/regen$round.log && break
done
cp /tmp/hold/*.rs src/v1/stage0/src/
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error" -A12 | head -60
echo "=== TEST"
./target/release/gunbc test //gunbc/instruments:interpolation-hole-census > /tmp/ihc.out 2>&1; echo "exit=$?"
head -80 /tmp/ihc.out
echo "=== WITNESS"
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/target_invocation_witness_test.dag --function the_interpolation_hole_census_operand_routes_to_its_own_producer 2>&1 | tail -5
git add -A src/v1/stage0 >/dev/null 2>&1
echo "=== STATUS"; git status --short | grep -v '^?? [0-9a-f]\{64\}$'
mkdir -p /tmp/out && cp /tmp/ihc.out /tmp/out/ && git diff --cached --binary -- src/v1/stage0 > /tmp/out/stage0.patch
echo "=== BUNDLE"; tar czf - -C /tmp out | base64 -w0; echo; echo "=== END"
