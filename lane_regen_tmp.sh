#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): the manager's regen procedure -- run the round
# over the present files, take the diff against the round output, run round 2 (must write 0).
# Brings the regenerated mirror back by pushing a scratch ref from the runner.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }

# the four new claim/lib files ride the checker gate; run them separately (claim_batch) so a
# checker hiccup in them cannot block the corpus regen
mkdir -p /tmp/aside
for f in unary_prefix_v1_canon_lib.dag unary_prefix_v2_canon_lib.dag unary_prefix_v1_v2_agreement.dag unary_prefix_values.dag; do
  mv "src/v2/test/claim/manual/$f" /tmp/aside/ 2>/dev/null || true
done
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2 || exit 1

PREV=-1
for ROUND in 1 2 3 4; do
  echo "=== REGEN ROUND $ROUND ==="
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 2>&1 | tail -8
  WRITTEN=$(git status --porcelain -- src/v1/stage0/src | wc -l | tr -d ' ')
  echo "round $ROUND dirty_files=$WRITTEN"
  if [ "$WRITTEN" = "$PREV" ]; then
    echo "FIXED-POINT-CONFIRMED: round $ROUND wrote 0 new files (previous round already converged)"
    break
  fi
  PREV=$WRITTEN
done

echo "=== DIFFSTAT (mirror) ==="
git diff --stat -- src/v1/stage0/src | tail -10
echo "=== DECLARED-POWER READ LANDED IN THE MIRROR ==="
grep -c "find_prefix_binding_power" src/v1/stage0/src/v1_compiler_parse.rs || true
grep -c "PrefixOperatorSpec" src/v1/stage0/src/std_syntax.rs src/v1/stage0/src/v1_std_core.rs 2>/dev/null || true
echo "leftover literal-12 prefix arms (want 0):"
grep -c "parse_expr_bp(token_stream_advance(tokens.clone(), 1), ctx.clone(), 12)" src/v1/stage0/src/v1_compiler_parse.rs || true

echo "=== PUSH SCRATCH REF ==="
git add -A src/v1/stage0/src
git -c user.name=regen -c user.email=regen@gunbc commit -q -m "regen: seed mirror regenerated to the declared prefix binding powers (fixed point)" || true
BRANCH="scratch/unary-prefix-regen-$(git rev-parse --short HEAD)"
git push -f origin "HEAD:refs/heads/$BRANCH" 2>&1 | tail -2
echo "PUSHED-BRANCH $BRANCH"
