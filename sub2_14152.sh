#!/bin/bash
# Unsettled-generic-as-Unit hunt, round 2. The patch is prepared and rustfmt-PARSED
# LOCALLY (sub_carrier_infer.rs) instead of being written by python on the runner, which
# is what broke round 1 (duplicated 'Some(c) => {' -> mismatched closing delimiter, and a
# bare-leaf anchor that missed because the emitted form is 'Some(c) => crate::...,' with
# 24-space indent). Instruments substitute_generics_apply's two binding arms; prints only
# when the inserted node's printed shape contains 'Unit'.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
cp sub_carrier_infer.rs src/v1/stage0/src/v1_compiler_infer.rs
echo "patched: seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) SUBTVprobe=$(grep -c 'SUBTV var=' src/v1/stage0/src/v1_compiler_infer.rs) SUBLEAFprobe=$(grep -c 'SUBLEAF leaf=' src/v1/stage0/src/v1_compiler_infer.rs) (want 36 / 1 / 1)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error" | head -6
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sub2.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/sub2.log) SUBTV=$(grep -ac '^SUBTV ' /tmp/sub2.log) SUBLEAF=$(grep -ac '^SUBLEAF ' /tmp/sub2.log)"
echo "--- distinct SUBTV (var / inserted / ins_span):"
grep "^SUBTV " /tmp/sub2.log | sed 's/ ins_children.*//' | sort | uniq -c | sort -rn | head -10
echo "--- distinct SUBLEAF:"
grep "^SUBLEAF " /tmp/sub2.log | sed 's/ ins_expr.*//' | sort | uniq -c | sort -rn | head -10
echo "--- sample with inferred/expr_data (first 6 lines of each kind):"
grep -m3 "^SUBTV " /tmp/sub2.log
grep -m3 "^SUBLEAF " /tmp/sub2.log
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo SUB2_DONE
