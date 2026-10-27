#!/bin/bash
# One dispatch: seed build, witnesses green, realization/seal/declaration REDs, old->new readback.
set -u
F() { grep -o '^test fn [a-z_0-9]*' "dag/test/claim/$1.dag" | sed 's/test fn //' | paste -sd, -; }
CB() { FN="${ONLY:-$(F $1)}"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry "dag/test/claim/$1.dag" --functions "$FN" "${@:2}" > /tmp/cb.out 2>&1; rc=$?; grep -E '^(PASS|FAIL|UNOBSERVED)|error:|claim_batch:|passed|failed' /tmp/cb.out | grep -v '^\[pre-entry\]\|typed_cache\|floor-' | cut -c1-500 | head -60; echo "EXIT[$1 ${*:2}]=$rc PASS=$(grep -c '^PASS' /tmp/cb.out) FAIL=$(grep -c '^FAIL' /tmp/cb.out)"; }
STORE=dag/gunbc/deploy_transition_store.dag; DT=dag/gunbc/deploy_transition.dag
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"; git status --short | grep -v '^ D artifacts' | head -20
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
ls -la --time-style=full-iso target/release/claim_batch target/release/gunbc
cp $DT /tmp/dt.bak
echo "##### SEAL CONTROL: sole_constructor stripped from TransitionGuard"
sed -i "s/^type TransitionGuard sole_constructor {/type TransitionGuard {/" $DT; grep -n "^type TransitionGuard" $DT
CB transition_inhibition_store_seal_witness
cp /tmp/dt.bak $DT
echo "##### DONE"
