#!/bin/bash
# ALL SIX diagnostics, verbatim, for the ONLY valid merged pairing: my 04_infer.dag + main's
# mirrors. The previous dispatch filtered by line-prefix and so printed 1 of 6, and its arms A and C
# paired MAIN's 04_infer.dag with MY census claim file (which calls unify_generics with text: and
# module_name:, params main's 5-parameter version does not have) -- an arity artifact I created, not
# a fact about clean main. Nothing is filtered here: every line after the refusal header is printed.
# TWO independent phases in ONE dispatch. NO git checkout / git clean; each phase restores
# by cp. set -uo pipefail without -e so phase 2 still runs if phase 1 fails.
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
echo "############ PHASE 2: PRODUCER PROBE (pre-fold shipped seed + backtrace patch) ############"

# PRODUCER HUNT, parent-approved rung (msg_0306d56f then msg_fca596f5): print the CALLER and a
# backtrace at the substitute_generics_apply bare-leaf arm, filtered to the fabricated Unit node
# (name=="Unit", span <kernel:Unit>:0-4), so we learn WHO hands a Unit node as an 'actual'.
# Carrier = seedred/v1_compiler_infer.rs (pre-fold shipped seed, sha 33d65c851a015068) patched to
# sub_carrier_infer2.rs; patch was rustfmt-parsed locally and its anchor verified count==1.
# Build AFTER the swap (gunbc cause=StaleBinary otherwise). NO git checkout/git clean.
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src/v1_compiler_infer.rs
cp seedred/v1_compiler_infer.rs.bak_probe $SEED 2>/dev/null || true
cp sub_carrier_infer2.rs $SEED
echo "swapped: $(sha256sum $SEED|cut -c1-12) probe=$(grep -c PROD-PROBE $SEED) backtrace=$(grep -c force_capture $SEED)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -3
RUST_BACKTRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/prod_selfhost.log 2>&1
echo "self-host rc=$?"
echo "=== PROD-UNIT rows (the fabrications) ==="
grep -a "PROD-UNIT" /tmp/prod_selfhost.log | head -12
echo "count=$(grep -ac 'PROD-UNIT' /tmp/prod_selfhost.log)"
echo "=== the frames that produced them (own-crate frames, first 12 per row group) ==="
grep -a -A40 "PROD-UNIT" /tmp/prod_selfhost.log | grep -aE "v1_compiler|infer|substitute|04_infer" | sort | uniq -c | sort -rn | head -25
cp /tmp/prod_selfhost.log prod_selfhost.log 2>/dev/null
echo "=== BEGIN-TAR ==="; tar -c prod_selfhost.log 2>/dev/null | base64 -w0 | head -c 600000; echo; echo "=== END-TAR ==="
cp seedred/v1_compiler_infer.rs.bak_probe $SEED
echo "restored: $(sha256sum $SEED|cut -c1-12) (expect 33d65c851a015068 pre-fold, or 5e9c37585fe088ac fold)"
echo PROD_DONE

echo ALL_DONE
