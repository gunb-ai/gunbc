#!/bin/bash
# Sixth dispatch: UNMUTATED baseline for the two roadmap readers (no declaration deleted).
set -u
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
for spec in "dag/gunbc/roadmap/roadmap_belt_actuate.dag:belt_transition_admission_for_instance" "dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag:rlm_launch_deployment_receipt_bound"; do
  e=${spec%%:*}; f=${spec##*:}; echo "-- BASELINE $e ($f)"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $e --functions $f > /tmp/base.out 2>&1; echo "rc=$?"
  grep -E "^(PASS|FAIL)" /tmp/base.out | cut -c1-260 | head -3
  grep -E "error" /tmp/base.out | sed 's/ (imported.*//' | sort | uniq -c | head -8
done
echo "##### DONE"
