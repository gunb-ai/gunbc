#!/bin/bash
# Dispatch after the side-chat fix: build, then the two behavioural witnesses.
# Claim lists are derived from the .dag files, so the claims added at this head run too.
set -u
F() { grep -o '^test fn [a-z_0-9]*' "dag/test/claim/$1.dag" | sed 's/test fn //' | paste -sd, -; }
CB() { FN="${ONLY:-$(F $1)}"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry "dag/test/claim/$1.dag" --functions "$FN" "${@:2}" > /tmp/cb.out 2>&1; rc=$?; grep -E '^(PASS|FAIL|UNOBSERVED)|error:|claim_batch:|passed|failed' /tmp/cb.out | grep -v '^\[pre-entry\]\|typed_cache\|floor-' | cut -c1-500 | head -60; echo "EXIT[$1 ${*:2}]=$rc PASS=$(grep -c '^PASS' /tmp/cb.out) FAIL=$(grep -c '^FAIL' /tmp/cb.out)"; }
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"; git status --short | grep -v '^ D artifacts' | head -20
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -3
echo "##### SEALFIX GREEN: wet witness (17 claims incl. the independently planted main bytes)"
CB deploy_transition_wet_witness --wet
echo "##### SEALFIX GREEN: seal witness (14 claims incl. the removed-route RED)"
CB transition_inhibition_store_seal_witness
echo "##### SEALFIX GREEN: hermetic witness (unchanged code path)"
CB deploy_transition_witness --hermetic
echo "##### DONE"
