#!/bin/bash
# ALL SIX diagnostics, verbatim, for the ONLY valid merged pairing: my 04_infer.dag + main's
# mirrors. The previous dispatch filtered by line-prefix and so printed 1 of 6, and its arms A and C
# paired MAIN's 04_infer.dag with MY census claim file (which calls unify_generics with text: and
# module_name:, params main's 5-parameter version does not have) -- an arity artifact I created, not
# a fact about clean main. Nothing is filtered here: every line after the refusal header is printed.
# NO git checkout / git clean. Restored by cp at the end.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
cp keep/my_04_infer.dag src/v1/04_infer.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
echo "arm B: dagrelation=$(grep -c generic_types_unify src/v1/04_infer.dag) seedfold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs)"
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rrB.log 2>&1
echo "rc=$?  $(grep -aoE "produced [0-9]+ hard diagnostic" /tmp/rrB.log | head -1)"
echo "===== EVERYTHING after the refusal header, unfiltered ====="
awk '/refused: v2 self-compile/,0' /tmp/rrB.log | head -70
echo "===== (end) ====="
echo "===== also: any 'call shape' or 'no parameter' lines anywhere in the run ====="
grep -an "call shape\|no parameter\|missing required\|supplied more than once\|unknown function\|could not resolve" /tmp/rrB.log | head -20 | cut -c1-260
cp src/v1/04_infer.dag keep/my_04_infer.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
echo "restored: dag=$(sha256sum src/v1/04_infer.dag|cut -c1-12)"
echo ARM_B_DONE
