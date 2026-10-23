#!/bin/bash
# PRODUCER HUNT (parent's approved rung, msg_0306d56f -> msg_fca596f5): name WHO hands a Unit node
# as the 'actual' at the substitute_generics_apply bare-leaf arm, where 7 bindings were fabricated
# (leaf=T at <kernel:T>:0-1 x5, leaf=element at dag/std/types.dag:5232-5239 x2; inserted node
# name="Unit" span <kernel:Unit>:0-4, expr_data NoExprData, inferred None).
#
# WHY THE LAST ATTEMPT PRODUCED NOTHING, three ways, all mine:
#  1. the patch prints only when GUNBC_EMIT_TRACE is set, and the dispatch never set it -> 0 rows;
#  2. the probe carrier is the FOLD family (its unify_generics returns Rc<UnifyGenericsAcc>) but the
#     dispatch restored main's PRE-FOLD siblings (Rc<HashMap<..>>) -> 6 type errors, lib uncompilable;
#  3. my grep filtered 'PROD-UNIT' while the label emitted is 'PROD leaf=' -> 0 matches even with 1+2 fixed.
# So: whole FOLD carrier family (seedgreen/*) with only v1_compiler_infer.rs replaced by the probe,
# the env var set, and the filter taken from the printer. NO git checkout / git clean.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
for f in v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs; do
  cp "seedgreen/$f" "$SEED/$f"
done
cp sub_carrier_infer2.rs "$SEED/v1_compiler_infer.rs"
echo "swapped: infer=$(sha256sum $SEED/v1_compiler_infer.rs|cut -c1-12) probe=$(grep -c 'PROD leaf=' $SEED/v1_compiler_infer.rs) backtrace=$(grep -c force_capture $SEED/v1_compiler_infer.rs) typesfam=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|warning: unused variable: .__sh|Finished" | tail -4
export GUNBC_EMIT_TRACE=1
RUST_BACKTRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/prod2.log 2>&1
echo "self-host rc=$? rebind=$(grep -ac 'bound as' /tmp/prod2.log)"
echo "=== PRODUCER ROWS (filter from the printer, not from memory) ==="
grep -a "PROD leaf=" /tmp/prod2.log | head -8 | cut -c1-260
echo "count=$(grep -ac 'PROD leaf=' /tmp/prod2.log)"
echo "=== the frames behind them: own-crate frames, most frequent first ==="
grep -a -A60 "PROD leaf=" /tmp/prod2.log | grep -aoE "v1_compiler[a-z_]*::[a-zA-Z0-9_:]+" | sort | uniq -c | sort -rn | head -20
cp /tmp/prod2.log prod_selfhost2.log
echo "=== BEGIN-TAR ==="; tar -c prod_selfhost2.log 2>/dev/null | base64 -w0; echo; echo "=== END-TAR ==="
cp seedgreen/v1_compiler_infer.rs "$SEED/v1_compiler_infer.rs"
echo "restored: $(sha256sum $SEED/v1_compiler_infer.rs|cut -c1-12) (expect 5e9c37585fe0 fold family)"
echo PROD2_DONE
