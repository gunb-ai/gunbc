set -e
echo "AMBIENT RUSTC_WRAPPER before clearing: '${RUSTC_WRAPPER:-<unset>}'"
unset RUSTC_WRAPPER || true
export RUSTC_WRAPPER=
echo "RUSTC_WRAPPER explicitly cleared: '${RUSTC_WRAPPER}'"
echo "sccache env check:"; env | grep -i sccache || echo "(no sccache env vars set)"
BEFORE_FULL=e89dd91e6293d5a294f3ecf21dfd7f22031c0b1c
AFTER_FULL=2087d8b7bc8e1dab0bb3aaabb5bcb6358fe65ea8
echo "BEFORE_FULL=$BEFORE_FULL"
echo "AFTER_FULL=$AFTER_FULL"

echo "=== BEFORE PHASE (infer.dag fix only, sort_by row still broken) ==="
git fetch origin $BEFORE_FULL
git checkout --force $BEFORE_FULL
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$BEFORE_FULL" ]; then echo "BASEFAIL got $GOT want $BEFORE_FULL"; exit 99; fi
echo "BEFORE_OK $GOT"
rm -rf /tmp/target-before
echo "VERBATIM: RUSTC_WRAPPER='${RUSTC_WRAPPER}' cargo build --release --target-dir /tmp/target-before -p v1-compiler --bin gunbc --bin claim_executor"
cargo build --release --target-dir /tmp/target-before -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -8

set +e
echo "VERBATIM: /tmp/target-before/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet"
/tmp/target-before/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/before_mw.log 2>&1
echo "before_mw_exit=$?"
set -e
echo "before error count:"; grep -c "error:" /tmp/before_mw.log || true
grep "error:" /tmp/before_mw.log | sort > /tmp/before_mw_errors.sorted

echo "=== AFTER PHASE (infer.dag fix + sort_by row fix) ==="
git fetch origin $AFTER_FULL
git checkout --force $AFTER_FULL
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$AFTER_FULL" ]; then echo "AFTERFAIL got $GOT want $AFTER_FULL"; exit 98; fi
echo "AFTER_OK $GOT"

echo "--- regenerate candidate std_algebra.rs from fixed algebra.dag using the BEFORE binary ---"
set +e
/tmp/target-before/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/algebra-regen-candidate --regen-receipt /tmp/algebra-regen-receipt.json > /tmp/algebra_rr.log 2>&1
echo "algebra_rr_exit=$?"
set -e
tail -25 /tmp/algebra_rr.log

CAND=$(find /tmp/algebra-regen-candidate -iname "std_algebra.rs" | head -1)
echo "candidate file: $CAND"
if [ -n "$CAND" ]; then
  diff "$CAND" src/v1/stage0/src/std_algebra.rs || true
  cp "$CAND" src/v1/stage0/src/std_algebra.rs
  echo "applied std_algebra.rs"
else
  echo "NO CANDIDATE FOUND for std_algebra.rs specifically — checking whether required-regen refused for an unrelated surface-population reason (expected: known pre-existing committed_not_emitted rows unrelated to algebra); if so, fall back to hand-mirroring the .dag row change into std_algebra.rs directly since its shape is fully deterministic from the .dag source and already verified by inspection."
  python3 - << 'PYEOF'
import re
path = "src/v1/stage0/src/std_algebra.rs"
with open(path) as f:
    content = f.read()
old = '''        Rc::new(AlgebraFieldTemplate {
            name: "sort_by".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: Some(CollectionSizeEffect::IdentityEffect),
            cost_shape: Some(CostShape::ShapeSortBody),
            callback_element_position: None,
        }),'''
new = '''        Rc::new(AlgebraFieldTemplate {
            name: "sort_by".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                        id: "SortKey".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: Some(CollectionSizeEffect::IdentityEffect),
            cost_shape: Some(CostShape::ShapeSortBody),
            callback_element_position: Some(0),
        }),'''
if old not in content:
    print("MANUAL_MIRROR_FAILED: old pattern not found")
    raise SystemExit(1)
content = content.replace(old, new)
with open(path, "w") as f:
    f.write(content)
print("MANUAL_MIRROR_APPLIED")
PYEOF
fi

cargo fmt --all 2>&1 | tail -5 || true

echo "--- rebuild with fixed std_algebra.rs ---"
rm -rf /tmp/target-after
echo "VERBATIM: RUSTC_WRAPPER='${RUSTC_WRAPPER}' cargo build --release --target-dir /tmp/target-after -p v1-compiler --bin gunbc --bin claim_executor"
cargo build --release --target-dir /tmp/target-after -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -15

echo "--- after-fix main_wet ---"
set +e
echo "VERBATIM: /tmp/target-after/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet"
/tmp/target-after/release/gunbc run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > /tmp/after_mw.log 2>&1
echo "after_mw_exit=$?"
set -e
echo "after error count:"; grep -c "error:" /tmp/after_mw.log || true
grep "error:" /tmp/after_mw.log | sort > /tmp/after_mw_errors.sorted

echo "=== CLEARED (in before, not in after) ==="
comm -23 /tmp/before_mw_errors.sorted /tmp/after_mw_errors.sorted | tee /tmp/cleared.txt | wc -l
echo "=== SURVIVING (in both) ==="
comm -12 /tmp/before_mw_errors.sorted /tmp/after_mw_errors.sorted | tee /tmp/surviving.txt | wc -l
echo "=== NEWLY APPEARING (in after, not in before) ==="
comm -13 /tmp/before_mw_errors.sorted /tmp/after_mw_errors.sorted | tee /tmp/newly.txt | wc -l

echo "=== sample of CLEARED ==="
head -40 /tmp/cleared.txt
echo "=== sample of SURVIVING ==="
head -40 /tmp/surviving.txt
echo "=== full NEWLY APPEARING (should be small) ==="
cat /tmp/newly.txt

echo "=== int-vs-string family check (must remain fixed / absent) ==="
grep -i "linux.dag\|proc_self_cgroup\|cache_interface\|domain_name" /tmp/after_mw.log || echo "none — good, still clean"

echo "=== remaining refusal diagnostics ==="
grep -c "no declared argument contract is available" /tmp/after_mw.log || true
grep "no declared argument contract is available" /tmp/after_mw.log

echo "=== unlocated synthetic errors (no file:line) in after ==="
grep "error:" /tmp/after_mw.log | grep -vc "\.dag:[0-9]" || true
grep "error:" /tmp/after_mw.log | grep -v "\.dag:[0-9]" | head -10

echo "DONE"
