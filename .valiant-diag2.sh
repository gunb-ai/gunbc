#!/usr/bin/env bash
# DIAGNOSTIC: reproduce the gen-2 condition and dump the refusal in full.
# Tree at start: mirrors = HEAD. We replay gen1b (main mirrors + live corpus emit + adopt), then
# rebuild, regen again, and print the refusal's distinct diagnostics.
set -uo pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cd "$(dirname "$0")"

echo "=== STEP 1: main mirror set + link scaffold, build, emit (the clean gen1b) ==="
git checkout ae563700cbe -- src/v1/stage0/src/
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b1.log 2>&1
echo "build_errors=$(grep -cE '^error' /tmp/b1.log)"
rm -rf target/candA
./target/release/claim_executor --required-regen --regen-candidate-dir target/candA \
  --regen-receipt target/receiptA.json --source-root dag --source-root src/v2 > /tmp/gA.out 2>&1
echo "regenA_effect=$(grep -cE 'effect summary incomplete' /tmp/gA.out)"
grep -E "^required-regen" /tmp/gA.out | tail -2

echo "=== STEP 2: adopt A, scaffold the 3 std-core bodies, rebuild ==="
diff -rq target/candA/src src/v1/stage0/src 2>/dev/null | grep '^Files' | sed 's|^Files target/candA/src/\(.*\) and .*|\1|' | while read -r f; do cp "target/candA/src/$f" "src/v1/stage0/src/$f"; done
git checkout ae563700cbe -- src/v1/stage0/src/std_algebra.rs src/v1/stage0/src/std_types.rs src/v1/stage0/src/std_computation.rs
git status --porcelain | grep stage0/src | wc -l | sed 's/^/mirrors_changed=/'
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b2.log 2>&1
echo "build_errors=$(grep -cE '^error' /tmp/b2.log)"

echo "=== STEP 3: regen B, dump the refusal ==="
rm -rf target/candB
./target/release/claim_executor --required-regen --regen-candidate-dir target/candB \
  --regen-receipt target/receiptB.json --source-root dag --source-root src/v2 > /tmp/gB.out 2>&1
echo "regenB_exit=$?"
grep -cE "effect summary incomplete" /tmp/gB.out | sed 's/^/regenB_effect=/'
echo "--- distinct diagnostic SHAPES (module stripped) ---"
grep -E "^[a-z0-9_.]+: " /tmp/gB.out | sed -E 's/^[a-z0-9_.]+: //; s/\(.*//' | sort | uniq -c | sort -rn | head -12
echo "--- distinct CALLEES named in the refusals ---"
grep -oE "calls [a-zA-Z0-9_.]+" /tmp/gB.out | sort | uniq -c | sort -rn | head -25
echo "--- first 8 refusal lines verbatim ---"
grep -E "^[a-z0-9_.]+: " /tmp/gB.out | head -8
echo "--- where does the effect registry come from? search emitted mirror ---"
grep -c "effect summary incomplete" src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/v1_compiler_infer_mentions=/'
grep -c "effect summary incomplete" src/v1/stage0/src/v1_std_core.rs | sed 's/^/v1_std_core_mentions=/'
echo "--- diff adopted tree vs main on the registry-bearing mirrors ---"
for f in v1_std_core.rs v1_compiler_infer.rs v1_compiler_emit_rust.rs std_coercion.rs; do
  cmp -s src/v1/stage0/src/$f <(git show ae563700cbe:src/v1/stage0/src/$f) && echo "$f SAME-AS-MAIN" || echo "$f DIFFERS-FROM-MAIN"
done
