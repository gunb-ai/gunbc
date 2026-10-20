#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): one-dispatch verification + regen.
#  1. RED: claims run against the pristine mirror (old binding 12 in force) -- must refuse.
#  2. digest A: canonical tree digests over the RE-SPELLED corpus, old binding.
#  3. REGEN to fixed point (heal sequence), round 2 writes 0.
#  4. digest B: same corpus, regenerated mirror (declared power) -- diff A B must be EMPTY
#     except this lane's own fixture items (the value arms are intentional RED fixtures).
#  5. GREEN: same claims, regenerated mirror -- must pass.
#  6. Emit the regenerated mirror as a base64 patch (runner has no push credentials).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

flt() { grep -viE "^\[|entries_resolve|Finished|Compiling|floor-teardown|git.Core|git.Plumbing|started |done in" | head -26; }

echo "=== 1a. BUILD (pristine mirror = old binding) ==="
cargo build --release -p v1-compiler --bin claim_batch --bin unary_prefix_digest --bin gunbc --bin claim_executor 2>&1 | tail -2

echo "=== 1b. RED values claim (old binding) ==="
./target/release/claim_batch --source-root src --source-root dag --entry src/v2/test/claim/manual/unary_prefix_values.dag --function unary_prefix_value_arms_hold > /tmp/red_values.log 2>&1; echo "exit=$?"
flt < /tmp/red_values.log

echo "=== 1c. RED agreement claim (old binding) ==="
./target/release/claim_batch --source-root src --source-root dag --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports > /tmp/red_agree.log 2>&1; echo "exit=$?"
grep -oE "ARM\[[^]]*\] v1=[^ ]* v2=[^ ]* want=[^ ]* [A-Z-]*" /tmp/red_agree.log | head -16

echo "=== 2. DIGEST A (old binding, re-spelled corpus) ==="
./target/release/unary_prefix_digest dag src > /tmp/digest_old.txt 2>/tmp/digest_a_err.txt; echo "items $(wc -l < /tmp/digest_old.txt)"

echo "=== 3. REGEN (heal sequence to fixed point) ==="
PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 2>&1 | grep -E "changed_paths|convergence_stages|RebuildScopeRefused|error:|✗" | head -8
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet 2>&1 | tail -2
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet 2>&1 | tail -2
  WRITTEN=$(git status --porcelain | wc -l | tr -d ' ')
  echo "round $ROUND dirty_files=$WRITTEN"
  if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
  PREV=$WRITTEN
done
echo "--- mirror diffstat ---"
git diff --stat -- src/v1/stage0/src | tail -8

echo "=== 4. REBUILD + DIGEST B (declared power) ==="
cargo build --release -p v1-compiler --bin claim_batch --bin unary_prefix_digest 2>&1 | tail -1
./target/release/unary_prefix_digest dag src > /tmp/digest_new.txt 2>/dev/null; echo "items $(wc -l < /tmp/digest_new.txt)"
echo "--- digest diff (want ONLY unary_prefix_values.dag fixture items) ---"
diff /tmp/digest_old.txt /tmp/digest_new.txt | grep -E "^[<>]" | head -30
echo "digest-diff-lines $(diff /tmp/digest_old.txt /tmp/digest_new.txt | grep -cE '^[<>]' || true)"
diff /tmp/digest_old.txt /tmp/digest_new.txt | grep -E "^[<>]" | grep -vc "unary_prefix_values.dag" || true

echo "=== 5. GREEN runs (regenerated mirror) ==="
for pair in \
 "src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports" \
 "src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold" \
 "src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree" \
 "src/v2/test/claim/manual/unary_prefix_v1_shape_probe.dag unary_prefix_v1_shape_probe_reports" \
 "src/v2/test/claim/manual/unary_prefix_v2_shape_probe.dag unary_prefix_v2_shape_probe_reports"; do
  set -- $pair
  echo "--- $1 :: $2 ---"
  ./target/release/claim_batch --source-root src --source-root dag --entry $1 --function $2 > /tmp/green.log 2>&1; echo "exit=$?"
  grep -oE "ARM\[[^]]*\][^\"]*|refused: [^\n]*" /tmp/green.log | head -16
  grep -iE "error:|✗" /tmp/green.log | head -6
done

echo "=== 6. PATCH BASE64 (regenerated mirror) ==="
git add -A src/v1/stage0/src
echo "BEGIN-PATCH"
git diff --cached | base64 -w0
echo
echo "END-PATCH"
