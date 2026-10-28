#!/usr/bin/env bash
# SANCTIONED REGEN (#14876 handoff --required-regen recipe), iterated to a TRUE fixed point.
#
# HOW SELF-HOSTING WORKS HERE, and so why a bootstrap is needed. src/v1/stage0/src/v1_compiler_*.rs
# and the std-surface bodies are the DAG->Rust TRANSLATION of the corpus, produced by the compiler's
# compiled-in translator. So a change to src/v1/05_emit_rust.dag reaches its mirror when ANY working
# regenerator translates the corpus -- the translation rules are unchanged, the translated source is
# new. But claim_executor LINKS v1-stage0-std-core, whose std.algebra/std.types/std.computation
# bodies are the 26-error target: with them red the regenerator will not link at all. Two measured
# facts on this runner set the bootstrap:
#   - control (everything main): self-compile clean, 0 effect diagnostics.
#   - TEST A (head's compiler mirrors + merged corpus): 1430 'effect summary incomplete' refusals --
#     head's committed mirrors are stale against the merged corpus and cannot serve as a regenerator.
#   - TEST B (all mirrors main + merged corpus): clean, and required-regen names the drifting surface.
# So generation 1 may bootstrap to main's mirror set (and, only to LINK, to main's three std-core
# bodies), translate the live corpus, and we adopt. Each later generation builds from what was
# adopted. A scaffold is never the source of a committed byte, and the FINAL pass runs unscaffolded:
# fixed point == (regenerator built from the committed tree) reproduces the committed tree byte-for-
# byte, the v2 self-compile is clean, and v1-stage0-std-core builds with 0 errors as committed.
set -uo pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cd "$(dirname "$0")"

TOTAL_SRC=$(ls src/v1/stage0/src/*.rs | wc -l)
HALT=""
for gen in 1 2 3 4 5; do
  echo "############ GENERATION $gen ############"
  SCAFFOLDED=0
  cargo build --release -p v1-compiler --bin claim_executor > /tmp/build$gen.log 2>&1
  BERR=$(grep -cE '^error' /tmp/build$gen.log)
  echo "BUILD_ERRORS_AS_COMMITTED=$BERR"
  if [ "$BERR" != "0" ]; then
    echo "--- link scaffold: main's three std-core bodies (not translation logic; never committed) ---"
    grep -E '^error' /tmp/build$gen.log | grep -oE 'std_[a-z_]+\.rs' | sort -u | head -6 | sed 's/^/  broken: /'
    git checkout ae563700cbe -- src/v1/stage0/src/std_algebra.rs src/v1/stage0/src/std_types.rs src/v1/stage0/src/std_computation.rs
    SCAFFOLDED=1
    cargo build --release -p v1-compiler --bin claim_executor > /tmp/build$gen.log 2>&1
    BERR=$(grep -cE '^error' /tmp/build$gen.log)
    echo "BUILD_ERRORS_AFTER_SCAFFOLD=$BERR"
    if [ "$BERR" != "0" ]; then
      grep -E '^error' /tmp/build$gen.log | sort | uniq -c | sort -rn | head -8
      HALT="generation $gen: regenerator cannot link even with the std-core scaffold"; break
    fi
  fi

  do_regen() { # $1 = candidate dir tag
    rm -rf target/cand$1
    ./target/release/claim_executor --required-regen \
      --regen-candidate-dir target/cand$1 --regen-receipt target/receipt$1.json \
      --source-root dag --source-root src/v2 > /tmp/gen$1.out 2>&1
    echo "REGEN_EXIT[$1]=$?"
    echo "EFFECT_DIAGS[$1]=$(grep -cE 'effect summary incomplete' /tmp/gen$1.out)"
    grep -E "^required-regen" /tmp/gen$1.out | tail -2
  }

  do_regen $gen
  EFFECT_DIAGS=$(grep -cE 'effect summary incomplete' /tmp/gen$gen.out)

  if [ "$EFFECT_DIAGS" != "0" ] && [ "$gen" = "1" ]; then
    echo "--- TEST A condition (head mirrors stale): FULL BOOTSTRAP to main's coherent mirror set ---"
    git checkout ae563700cbe -- src/v1/stage0/src/
    SCAFFOLDED=1
    cargo build --release -p v1-compiler --bin claim_executor > /tmp/build1b.log 2>&1
    BERR=$(grep -cE '^error' /tmp/build1b.log)
    echo "BOOTSTRAP_BUILD_ERRORS=$BERR"
    if [ "$BERR" != "0" ]; then grep -E '^error' /tmp/build1b.log | head -8; HALT="generation 1: main's mirror set will not build"; break; fi
    do_regen 1b
    EFFECT_DIAGS=$(grep -cE 'effect summary incomplete' /tmp/gen1b.out)
    if [ "$EFFECT_DIAGS" != "0" ]; then
      grep -vE "^✓|^◐|memory-cgroup" /tmp/gen1b.out | head -14
      HALT="generation 1: refused even under the full main bootstrap ($EFFECT_DIAGS effect diagnostics)"; break
    fi
    rm -rf target/cand$gen; cp -r target/cand1b target/cand$gen
  elif [ "$EFFECT_DIAGS" != "0" ]; then
    grep -vE "^✓|^◐|memory-cgroup" /tmp/gen$gen.out | head -14
    HALT="generation $gen: v2 self-compile refused with $EFFECT_DIAGS effect-summary diagnostics"; break
  fi

  CAND=$(ls target/cand$gen/src/*.rs 2>/dev/null | wc -l)
  echo "CANDIDATE_FILES=$CAND TOTAL_SRC=$TOTAL_SRC"
  if [ "$CAND" -lt "$TOTAL_SRC" ]; then
    HALT="generation $gen: candidate incomplete ($CAND of $TOTAL_SRC); refusing to adopt"; break
  fi

  echo "ARM PROPAGATION CHECK: emitted mirror carries the keyed return-type arm? $(grep -c 'THE APPLIED-TYPE ARMS THAT SPELL' target/cand$gen/src/v1_compiler_emit_rust.rs) site(s), node_is_keyed_collection calls=$(grep -c 'node_is_keyed_collection' target/cand$gen/src/v1_compiler_emit_rust.rs)"

  DIFFS=$(diff -rq target/cand$gen/src src/v1/stage0/src 2>/dev/null | grep -v '^Only in' | sed 's|^Files target/cand[0-9b]*/src/\(.*\) and .*|\1|')
  if [ -z "$DIFFS" ]; then
    if [ "$SCAFFOLDED" = "1" ]; then
      echo "candidate == committed but under a scaffold; continuing so a later generation re-verifies unscaffolded"
    else
      echo "TRUE FIXED POINT at generation $gen (unscaffolded candidate == committed)"; break
    fi
  else
    echo "--- adopting $(echo "$DIFFS" | wc -l) drifted mirror(s):"; echo "$DIFFS" | head -12
    echo "$DIFFS" | while read -r f; do cp "target/cand$gen/src/$f" "src/v1/stage0/src/$f"; done
  fi
done
if [ -n "$HALT" ]; then echo "HALT: $HALT"; fi

echo "############ FINAL VERIFICATION (unscaffolded, exactly as it would be committed) ############"
git status --porcelain | awk '{print $2}' | grep '^src/v1/stage0/src/.*\.rs$' | wc -l | sed 's/^/MIRRORS_CHANGED_VS_HEAD=/'
cargo build --release -p v1-compiler --bin claim_executor > /tmp/final-build.log 2>&1
echo "FINAL_COMPILER_BUILD_ERRORS=$(grep -cE '^error' /tmp/final-build.log)"
grep -E '^error' /tmp/final-build.log | head -6
cargo build --release -p v1-stage0-std-core > /tmp/final-stdcore.log 2>&1
echo "STDCORE_ERRORS=$(grep -cE '^error' /tmp/final-stdcore.log)"
grep -nE "^error(\[|:)|^ *-->" /tmp/final-stdcore.log | sed 's/^[[0-9;]*//g' | head -60
if [ "$(grep -cE '^error' /tmp/final-stdcore.log)" = "0" ]; then
  ./target/release/claim_executor --required-regen \
    --regen-candidate-dir target/candfinal --regen-receipt target/receiptfinal.json \
    --source-root dag --source-root src/v2 > /tmp/genfinal.out 2>&1
  echo "FINAL_REGEN_EXIT=$?"; grep -E "^required-regen" /tmp/genfinal.out | tail -2
  echo "FINAL_DRIFT:"; diff -rq target/candfinal/src src/v1/stage0/src 2>/dev/null | grep -v '^Only in' | head -20
fi

echo "=== VALIANT-MIRROR-BEGIN ==="
git status --porcelain | awk '{print $2}' | grep '^src/v1/stage0/src/.*\.rs$' | while read -r f; do
  echo "VALIANT-FILE $f $(wc -c < "$f")"
  base64 -w 120 "$f"
done
echo "=== VALIANT-MIRROR-END ==="
