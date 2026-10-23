#!/usr/bin/env bash
# Post-merge regeneration of the four generated projections left UNMERGED by the
# generated-artifact merge driver (its own message: take neither side's bytes; regenerate
# from the MERGED authorities). Mid-merge, so --regen-affected-scope is unavailable (a
# scoped round refuses an uncommitted .dag edit) -- this is a whole-population round.
# Convergence per declared step 3: the step-2 binary predates the mirrors it installs, so
# rebuild from the installed seed and re-run until the emitter installs nothing.
set -u
MIR=src/v1/stage0/src
CAND=target/stage0-regen-candidate/src/v1/stage0/src
TARGETS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"

echo "HEAD=$(git rev-parse HEAD)"
echo "UNMERGED: $(git diff --name-only --diff-filter=U | tr '\n' ' ')"
echo "unmerged .dag (must be 0): $(git diff --name-only --diff-filter=U | grep -c '\.dag')"
echo "fold in authority: 04_infer.dag UnifyGenericsAcc=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag)"

for round in 1 2 3 4 5; do
  echo "########## ROUND $round ##########"
  cargo build --release --bin claim_executor 2>&1 | grep -E "^error|Finished" | tail -3
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen_r$round.log 2>&1
  rc=$?
  echo "REGEN_EXIT=$rc"
  grep -E "generated_stage0_seed_emitted|hard diagnostic|refus|install|Regenerat|candidate" /tmp/regen_r$round.log | head -12
  if [ ! -d "$CAND" ]; then
    echo "NO CANDIDATE TREE — refusing to continue"; tail -25 /tmp/regen_r$round.log; exit 2
  fi
  # how far apart are the mirrors from the emitter's own output?
  pend=0
  for f in $TARGETS; do
    if ! cmp -s "$MIR/$f" "$CAND/$f"; then echo "PENDING $f"; pend=$((pend+1)); fi
  done
  allpend=$( (cd "$MIR" && for g in *.rs; do cmp -s "$g" "../../../../../$CAND/$g" || echo x; done) | wc -l)
  echo "PENDING_TARGETS=$pend PENDING_ALL_MIRRORS=$allpend"
  echo "DRIFTED_MIRROR_NAMES:"
  (cd "$MIR" && for g in *.rs; do cmp -s "$g" "../../../../../$CAND/$g" || echo "  $g"; done) | head -40
  if [ "$allpend" = "0" ]; then echo "CONVERGED at round $round"; break; fi
  cp -f $CAND/*.rs $MIR/
  echo "installed candidate mirrors for next round"
done

echo "########## FINAL: the four targets, gzip+base64 ##########"
for f in $TARGETS; do
  echo "@@@BEGIN $f sha=$(sha256sum $MIR/$f | cut -c1-12) lines=$(wc -l < $MIR/$f)"
  gzip -c < "$MIR/$f" | base64 -w 100
  echo "@@@END $f"
done
echo "########## FINAL VERDICT ##########"
cargo build --release --bin claim_executor 2>&1 | grep -E "^error|Finished" | tail -2
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen_final.log 2>&1
echo "FINAL_REGEN_EXIT=$?"
grep -E "generated_stage0_seed_emitted|hard diagnostic|install|Regenerat" /tmp/regen_final.log | head -8
echo "########## DONE ##########"
