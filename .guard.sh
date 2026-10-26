#!/usr/bin/env bash
set -u
B=target/release/claim_batch
fns() { grep -oP '^test fn \K[a-z0-9_]+' "$1" | paste -sd, -; }
run() { "$B" $1 --source-root dag --source-root src/v2 --entry "dag/test/claim/$2" --functions "${3:-$(fns dag/test/claim/$2)}" > .out 2>&1; e=$?
  echo "== $2: PASS=$(grep -c '^PASS ' .out) FAIL=$(grep -c '^FAIL ' .out) exit=$e"; grep -E '^FAIL |panicked|error\[|compile error|error:' .out | cut -c1-320 | head -14; }
echo "##### A dry deploy_transition_witness"
run "" deploy_transition_witness.dag
echo "##### B seal witness"
run "" transition_lease_store_seal_witness.dag
echo "##### C inhibition seal"
run "" transition_inhibition_store_seal_witness.dag
echo "##### D non-holder release"
run "" durable_exclusive_hold_nonholder_release_witness.dag
