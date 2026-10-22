#!/bin/bash
# Scratch (never lands): lane A brace-only regression receipt at e17b6a4a11e.
# The agreement claim's v1 route reads the .dag rows AT RUNTIME (take 1 je034bfb6 proved it: a
# .dag-only patch made the claim RED without a rebuild). Phase: { row 18 -> 13 ONLY, no other row
# moves. Expect RED on EXACTLY the two dotted unary record arms (v1 P(F(l)) vs want P(F(F(l)))),
# green on every other arm -- the discriminating input for the review 84349 cell at the declared
# prefix/postfix binding. No rebuild needed; the binaries from lane_prA_ab2_tmp.sh's second build
# are the declared-binding ones and runtime rows decide this phase.
set -uo pipefail
PIN=e17b6a4a11ec5cd143fdf566b36c43adb9e7beff
git fetch -q origin 2>/dev/null || true
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then echo "PIN: was $CUR; checkout $PIN"; git checkout -f -q "$PIN" || { echo PIN-FAILED; exit 3; }; fi
test "$(git rev-parse HEAD)" = "$PIN" || { echo PIN-MISMATCH; exit 3; }
echo "=== HEAD ==="; git rev-parse --short HEAD
echo STEP build; date -u +%H:%M:%S
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run_claim() {
  echo "STEP claim $1 @ $BIND"; date -u +%H:%M:%S
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/cC_$1_$BIND.log 2>&1
  echo "claim@$1/$BIND rc=$?"
  grep -a "MISMATCH\|DISAGREE\|agreement:" /tmp/cC_$1_$BIND.log | head -14 | cut -c1-200
}
BIND=brace13
python3 - <<'PY'
for p in ["dag/extdeps/languages/dag/syntax.dag", "dag/extdeps/languages/rust/syntax.dag"]:
    s = open(p).read()
    a, b = ('symbol: "{", binding_power: 18', 'symbol: "{", binding_power: 13')
    assert s.count(a) == 1, (p, a, s.count(a))
    open(p, "w").write(s.replace(a, b))
print("patched { row 18->13 in both .dag specs only")
PY
run_claim agreement_brace13 src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
git checkout -q -- dag/extdeps/languages/dag/syntax.dag dag/extdeps/languages/rust/syntax.dag
BIND=declared
run_claim agreement_declared src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
echo "dirty-after-restore=$(git status --porcelain -uno | wc -l)"
echo "=== DONE ==="
