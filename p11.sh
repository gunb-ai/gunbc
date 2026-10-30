set -u
echo HEAD=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
run() { echo "=== $1"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $1 --functions $2 --hermetic --claim-run > /tmp/cb-$3.log 2>&1; rc=$?; echo "exit=$rc"; grep -E -A6 "^(FAIL|UNOBSERVED)" /tmp/cb-$3.log | head -60; echo "pass=$(grep -c ^PASS /tmp/cb-$3.log) fail=$(grep -c ^FAIL /tmp/cb-$3.log) unobs=$(grep -c ^UNOBSERVED /tmp/cb-$3.log)"; [ $rc -ne 0 ] && { echo "--- tail"; tail -15 /tmp/cb-$3.log; }; }
run dag/test/claim/roadmap/roadmap_serve_witness.dag $(grep -E "^test fn|^claim " dag/test/claim/roadmap/roadmap_serve_witness.dag | sed -E 's/^(test fn|claim) ([a-z_0-9]+).*/\2/' | paste -sd, -) rserv
run dag/test/claim/realization_census_admission_witness.dag $(grep -E "^test fn|^claim " dag/test/claim/realization_census_admission_witness.dag | sed -E 's/^(test fn|claim) ([a-z_0-9]+).*/\2/' | paste -sd, -) censadm
run dag/test/claim/realization_census_sheet_witness.dag $(grep -E "^test fn|^claim " dag/test/claim/realization_census_sheet_witness.dag | sed -E 's/^(test fn|claim) ([a-z_0-9]+).*/\2/' | paste -sd, -) censsheet
echo "=== DESIGN regen check: regenerate DESIGN.md in place and diff"
cp DESIGN.md /tmp/DESIGN.committed
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=DESIGN.md > /tmp/regen-check.log 2>&1
echo "regen_exit=$?"; tail -3 /tmp/regen-check.log
git diff --stat -- DESIGN.md
git diff -- DESIGN.md | head -20
echo DONE-P11
