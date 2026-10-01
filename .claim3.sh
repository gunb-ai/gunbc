# usage: rt.sh MODE  (head | nostamp | nopred | base)
export GUNBC_MEMORY_BUDGET_BYTES=30064771072
MODE=$1
case $MODE in
  base) git fetch -q --depth=1 origin 038798b4960b88f73cdc5d03e9a3243349c684c7 && git checkout -q 038798b4960b88f73cdc5d03e9a3243349c684c7 -- src dag || echo BASEFAIL ;;
  nostamp) python3 - <<'PY'
p='src/v2/compiler/02_parse.dag'; s=open(p).read()
old=""") -> ParseStampResult {
  let minted = alloc_occurrence_id(alloc: prov.alloc)
  let from = parse_minted_id_list(nodes: [captured])"""
assert s.count(old)==1
s=s.replace(old,""") -> ParseStampResult {
  if production.name == ^dag_production_match_arm_stmt_body { ParseStampResult { node: captured, prov: prov } } else {
  let minted = alloc_occurrence_id(alloc: prov.alloc)
  let from = parse_minted_id_list(nodes: [captured])""")
i=s.index("fn parse_wrap_production_captured(")
j=s.index("\n}\n", i)
s=s[:j]+"\n  }"+s[j:]
open(p,'w').write(s)
PY
  sed -n '/^fn parse_wrap_production_captured(/,/^}/p' src/v2/compiler/02_parse.dag | head -40 ;;
  nopred) python3 - <<'PY'
p='src/v2/extdeps/languages/dag.dag'; s=open(p).read()
a=s.index("fn dag_grammar_match_arm_stmt_body_statement() -> GrammarExpr {")
b=s.index("fn dag_grammar_match_arm_stmt_body_expr()")
s=s[:a]+"fn dag_grammar_match_arm_stmt_body_statement() -> GrammarExpr {\n  dag_grammar_nonterminal(production: ^dag_production_stmt)\n}\n\n"+s[b:]
open(p,'w').write(s)
PY
  ;;
esac
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
timeout 2400 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry src/v2/test/claim/parse/match_arm_statement_body_parse_test.dag --functions a_match_arm_statement_body_is_stamped_and_stops_at_the_next_arm_holds > /tmp/o.txt 2>&1; echo "RC $MODE $?"
grep -E "^(PASS|FAIL)|error:|^\[witness\]" /tmp/o.txt | cut -c1-400 | sed "s/^/$MODE: /"
if [ "$MODE" = head ]; then
timeout 1200 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/parse_test_fn_decl_return_clause_test.dag --functions nfbcp_repeated_named_param_refuses_at_the_second_binding_holds > /tmp/o2.txt 2>&1; grep -E "^(PASS|FAIL)|^\[witness\]" /tmp/o2.txt | cut -c1-200 | sed "s/^/nfbcp: /"
fi
