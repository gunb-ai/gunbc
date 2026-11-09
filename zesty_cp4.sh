#!/bin/bash
# scratch (NOT for commit): the 7 unreached control-plane rows, batched alone; plus the two host gates.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgtF
mkdir -p /tmp/zlogF
if [ ! -x /tmp/tgtF/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheF ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc > /tmp/f-build.log 2>&1
  echo "BUILD_RC=$?"; tail -2 /tmp/f-build.log
fi
f=dag/test/claim/fabric/fabric_control_plane_witness.dag
fns=an_empty_roster_reports_no_admission_refusals_at_all,an_executor_refused_on_the_capacity_line_is_reported_not_dropped,an_offer_quoted_in_another_currency_is_refused_at_the_budget_seam,an_underfunded_budget_refuses_with_a_ledger_cause,the_encumbered_liability_is_the_full_selection_cost,w_the_broker_refuses_a_winner_admitted_for_another_demand,w_the_broker_refuses_a_winner_admitted_for_another_work
/tmp/tgtF/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /tmp/zlogF/cp7.log 2>&1
echo "SEVEN_RC=$? PASS=$(grep -c '^PASS' /tmp/zlogF/cp7.log) FAIL=$(grep -c '^FAIL' /tmp/zlogF/cp7.log)"
echo "--- all non-phase lines ---"
grep -vE '^\[|^\s*$' /tmp/zlogF/cp7.log | cut -c1-400
for h in srv3 srv4; do
  /tmp/tgtF/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/$h/gunbc-ghrunner.sudoers > /tmp/zlogF/gate_$h.log 2>&1
  echo "GATE_$h rc=$?"; tail -3 /tmp/zlogF/gate_$h.log | cut -c1-240
done
echo "CP4 DONE"
