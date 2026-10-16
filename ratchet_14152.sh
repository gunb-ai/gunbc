#!/usr/bin/env bash
# RATCHET TEST for #14152. Observation to explain: `claim_executor --required-regen`
# returned first_generation_equal=true on clean main (measured at 84655a00232), and refuses
# on THIS tree with 2 hard type mismatches inside src/v1/02_parse.dag -- a file this PR never
# touches and which is byte-identical to main's. The printed pair is
#   expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'
# i.e. structurally identical strings, which is the shape of a de Bruijn / index-depth artifact
# rather than a wrong program.
#
# HYPOTHESIS (the only variable between the two trees that could do this is the fold executing
# in the interpreter that performs the self-compile): the ci auto-heal regenerated the mirrors
# FROM the fold using the PRE-fold seed as interpreter, which bound generic actuals first-wins
# and therefore accepted those two sites. Now that the shipped seed carries the fold, the
# interpreter refuses them, so a regen can never run again: the fold can be emitted once but
# cannot itself emit. If true, this is a blocker for the constructor-tag climb, not a merge
# nuisance, and it must be fixed at the fold's leaf comparison rather than by hand-porting bytes.
set -u
SEED=src/v1/stage0/src
FILES="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
sig() { grep -c 'UnifyGenericsAcc' "$SEED/v1_compiler_infer.rs"; }
shax() { sha256sum "$SEED/v1_compiler_infer.rs" | cut -c1-12; }
put() { for f in $FILES; do cp "$1/$f" "$SEED/$f"; done; }

echo "HEAD=$(git rev-parse HEAD)"
echo "unmerged mirrors: $(git diff --name-only --diff-filter=U | tr '\n' ' ')"

for side in seedgreen seedred; do
  echo "########## $side (expect sig $( [ $side = seedgreen ] && echo 36 || echo 0 )) ##########"
  put $side
  echo "APPLIED sig=$(sig) sha=$(shax)"
  cargo build --release --bin claim_executor 2>&1 | grep -E "^error|Finished" | tail -2
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr_$side.log 2>&1
  echo "REGEN_EXIT[$side]=$?"
  grep -E "first_generation_equal|refused|hard diagnostic|type mismatch|install|candidate" /tmp/rr_$side.log | head -8
done

echo "########## SAME TEST AT THE TWO SPANS, NARROWER: does the fold-carrying interpreter refuse the FILE or just those sites? ##########"
put seedgreen
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr_g2.log 2>&1
echo "green again, count of hard diagnostics: $(grep -c 'type mismatch' /tmp/rr_g2.log)"
grep -E "type mismatch" /tmp/rr_g2.log | head -4
echo "spans resolved to source lines:"
python3 - <<'PY'
d=open('src/v1/02_parse.dag',encoding='utf-8').read()
for a in (83460,84172):
    ls=d[:a].rfind('\n')+1; le=d.find('\n',a)
    print('  byte',a,'line',d[:a].count('\n')+1,'->',d[ls:le].strip()[:120])
PY

echo "########## RESTORE ##########"
put seedgreen; echo "RESTORED sig=$(sig) sha=$(shax)"
echo "########## DONE ##########"
