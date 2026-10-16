cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
for m in dag/gunbc/rung_drop/d0_store_walls_single_lifecycle.dag dag/gunbc/fabric/fabric_store_operation_admission.dag dag/gunbc/rung_drop.dag; do echo "== $m"; ./target/release/gunbc run --source-root dag --source-root src/v2 --entry $m --function __no_such_function__ 2>&1 | grep -E "cause" | head -1; done
echo "== WITNESS"; ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/rung_drop_superseded_standing_witness.dag --claim-run 2>&1 | grep -E "PASS|FAIL" | cut -c1-160
echo "== FABRIC"; ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/fabric/fabric_store_operation_admission_witness.dag --claim-run 2>&1 | grep -E "PASS|FAIL" | cut -c1-120
