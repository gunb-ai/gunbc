#!/bin/bash
# PRODUCER HUNT, parent-approved rung (msg_0306d56f then msg_fca596f5): print the CALLER and a
# backtrace at the substitute_generics_apply bare-leaf arm, filtered to the fabricated Unit node
# (name=="Unit", span <kernel:Unit>:0-4), so we learn WHO hands a Unit node as an 'actual'.
# Carrier = seedred/v1_compiler_infer.rs (pre-fold shipped seed, sha 33d65c851a015068) patched to
# sub_carrier_infer2.rs; patch was rustfmt-parsed locally and its anchor verified count==1.
# Build AFTER the swap (gunbc cause=StaleBinary otherwise). NO git checkout/git clean.
set -uo pipefail
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
