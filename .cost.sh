export GUNBC_MEMORY_BUDGET_BYTES=30064771072
MODE=$1
if [ "$MODE" = base ]; then git fetch -q --depth=1 origin 224cdc4623f20201f28cdcc9f644e54f35700ca6 && git checkout -q 224cdc4623f20201f28cdcc9f644e54f35700ca6 -- src dag && git clean -fdq -- src dag || echo BASEFAIL; fi
if [ "$MODE" = nokw ]; then python3 - <<'PY'
p='src/v2/extdeps/languages/dag.dag'; s=open(p).read()
a=s.index("fn dag_grammar_declared_path_expr() -> GrammarExpr {")
b=s.index("\n}\n",a)
s=s[:a]+"fn dag_grammar_declared_path_expr() -> GrammarExpr {\n  dag_grammar_nonterminal(production: ^dag_production_qualified_name)"+s[b:]
open(p,'w').write(s)
PY
fi
mkdir -p src/v2/test/claim/scratch
cat > src/v2/test/claim/scratch/cost_scratch_test.dag <<'DAG'
module v2.test.scratch.cost_scratch

import v2.test.parse.expression_bodied_fn_decl_parse { g_tokenize_parse }
import v2.std.diagnostic { Accepted, Rejected }
import v2.std.live_tree { LiveTreeDisposition, SubstrateInputsOnly }
import v2.std.logic { Bool }

data live_tree_disposition: LiveTreeDisposition = SubstrateInputsOnly

test fn ret_body_holds() -> Bool {
  match g_tokenize_parse(text: "module m\ndata d: Int = match t {\n  A => return v\n  B => v\n}\n", file: ^cs_a) {
    Accepted { value: _, diagnostics: _ } => true
    Rejected { diagnostics: _ } => false
  }
}
test fn let_body_holds() -> Bool {
  match g_tokenize_parse(text: "module m\ndata d: Int = match t {\n  A =>\n    let w = v\n    w\n  B => v\n}\n", file: ^cs_b) {
    Accepted { value: _, diagnostics: _ } => true
    Rejected { diagnostics: _ } => false
  }
}
test fn no_match_holds() -> Bool {
  match g_tokenize_parse(text: "module m\ndata d: Int = v\n", file: ^cs_c) {
    Accepted { value: _, diagnostics: _ } => true
    Rejected { diagnostics: _ } => false
  }
}
DAG
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
./target/release/claim_batch --source-root dag --source-root src/v2 --entry src/v2/test/claim/scratch/cost_scratch_test.dag --functions ret_body_holds,let_body_holds,no_match_holds > /tmp/o.txt 2>&1
grep -E "^(PASS|FAIL)|^\[witness\]|error:" /tmp/o.txt | cut -c1-200 | sed "s/^/$MODE /"
./target/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/parse_test_fn_decl_return_clause_test.dag --functions nfbcp_repeated_named_param_refuses_at_the_second_binding_holds > /tmp/o2.txt 2>&1
grep -E "^(PASS|FAIL)|^\[witness\]" /tmp/o2.txt | cut -c1-200 | sed "s/^/$MODE /"
