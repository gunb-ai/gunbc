#!/usr/bin/env bash
# WHY THE v2 SELF-COMPILE REFUSES AFTER ADOPTION, AND WHICH MIRROR FLIPS IT.
# Facts so far (all measured here): a regenerator built from MAIN's mirrors translates the merged
# corpus cleanly (0 effect diagnostics, drift only on the PR's own surface); a regenerator built from
# HEAD's mirrors refuses with 1430; after adopting main's candidate, the rebuild refuses again.
# So one of the adopted mirrors, once compiled INTO the regenerator, makes the corpus's effect join
# lose registry rows. Bisect it, and print the refusal verbatim rather than aggregated.
set -uo pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cd "$(dirname "$0")"

report() { # $1 = label, $2 = out file
  local n=$(grep -cE 'effect summary incomplete' "$2")
  local verdict=$(grep -E "^required-regen" "$2" | tail -1)
  echo "RESULT[$1] effect_diags=$n verdict=[$verdict]"
}

regen() { # $1 = label
  rm -rf target/cand_$1
  ./target/release/claim_executor --required-regen --regen-candidate-dir target/cand_$1 \
    --regen-receipt target/receipt_$1.json --source-root dag --source-root src/v2 > /tmp/out_$1.out 2>&1
  report "$1" /tmp/out_$1.out
}

echo "########## BASELINE: main mirrors + scaffold, live corpus ##########"
git checkout ae563700cbe -- src/v1/stage0/src/
cargo build --release -p v1-compiler --bin claim_executor > /tmp/bb.log 2>&1
echo "build_errors=$(grep -cE '^error' /tmp/bb.log)"
regen baseline
echo "--- refusal verbatim, first 25 non-progress lines ---"
grep -vE "^✓|^◐|memory-cgroup|^\s*$" /tmp/out_baseline.out | head -25

echo "########## COPY THE BASELINE CANDIDATE AS THE ADOPTION SOURCE ##########"
rm -rf /tmp/adoptsrc; cp -r target/cand_baseline /tmp/adoptsrc

echo "########## BISECT: adopt ONE drifted mirror at a time ##########"
DRIFT="std_algebra.rs std_coercion.rs v1_compiler_emit_rust.rs v1_compiler_infer_method.rs v1_compiler_infer_resolve.rs v1_compiler_infer_types.rs"
for f in $DRIFT; do
  git checkout ae563700cbe -- src/v1/stage0/src/
  git checkout ae563700cbe -- src/v1/stage0/src/std_algebra.rs src/v1/stage0/src/std_types.rs src/v1/stage0/src/std_computation.rs 2>/dev/null
  cp /tmp/adoptsrc/src/$f src/v1/stage0/src/$f
  cargo build --release -p v1-compiler --bin claim_executor > /tmp/b_$f.log 2>&1
  berr=$(grep -cE '^error' /tmp/b_$f.log)
  if [ "$berr" != "0" ]; then echo "RESULT[$f] build_errors=$berr (regenerator cannot build with this mirror)"; grep -E '^error' /tmp/b_$f.log | head -3; continue; fi
  regen "$f"
done

echo "########## FULL ADOPTION (all six) ##########"
git checkout ae563700cbe -- src/v1/stage0/src/
for f in $DRIFT; do cp /tmp/adoptsrc/src/$f src/v1/stage0/src/$f; done
git checkout ae563700cbe -- src/v1/stage0/src/std_algebra.rs src/v1/stage0/src/std_types.rs src/v1/stage0/src/std_computation.rs
cargo build --release -p v1-compiler --bin claim_executor > /tmp/bfull.log 2>&1
echo "build_errors=$(grep -cE '^error' /tmp/bfull.log)"
regen full
echo "--- refusal verbatim, first 25 non-progress lines ---"
grep -vE "^✓|^◐|memory-cgroup|^\s*$" /tmp/out_full.out | head -25
echo "--- which MODULES lose rows (module_name is the prefix before the colon) ---"
grep -oE "^[a-z0-9_.]+: effect summary incomplete" /tmp/out_full.out | sed 's/: effect.*//' | sort | uniq -c | sort -rn | head -12
