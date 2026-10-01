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
mkdir -p src/v2/test/claim/scratch
cat > src/v2/test/claim/scratch/rt_scratch_test.dag <<'DAG'
module v2.test.scratch.rt_scratch

import gunbc.instruments.dag_emit_real_grammar_round_trips { dag_round_trip_observed }
import v2.test.execution.dag_grammar_backward_round_trip { rt_parse, rt_first_stamped, rt_stamp_of }
import v2.std.node { Node }
import v2.std.integer { Int }
import v2.std.diagnostic { Accepted, Rejected }
import v2.std.optional { Absent, Present }
import std.process { ProcessExit, exit_failure }
import v2.std.text { String }

data src: String = "module rt.match_stmt_body\n\ntype T = A | B\n\nfn f(t: T, v: Int) -> Int {\n  match t {\n    A =>\n      let w = v\n      w + 1\n    B => v\n  }\n}\n"

fn stamped() -> String {
  match rt_parse(text: src) {
    Rejected { diagnostics: _ } => "parse refused"
    Accepted { value: tree, diagnostics: _ } =>
      match rt_first_stamped(node: tree, emitted: ^dag_surface_match_arm_stmt_body) {
        Present { value: _ } => concat("stmt_body stamp present; match_arm stamps: ", arms_text(n: arm_count(node: tree)))
        Absent => concat("stmt_body stamp ABSENT; match_arm stamps: ", arms_text(n: arm_count(node: tree)))
      }
  }
}

fn arms_text(n: Int) -> String { if n == 2 { "2" } else if n == 1 { "1" } else if n == 0 { "0" } else { "more than 2" } }

fn arm_count(node: Node) -> Int {
  let here = match rt_stamp_of(node: node) {
    Present { value: s } => if s == ^dag_surface_match_arm { 1 } else { 0 }
    Absent => 0
  }
  fold(node.children, init: here, f: fn(acc, e) { acc + arm_count(node: e.target) })
}

fn stmt_body() -> ProcessExit {
  let o = concat(stamped(), concat(" | round trip: ", dag_round_trip_observed(text: src)))
  exit_failure(reason: o)
}
DAG
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
timeout 2400 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/scratch/rt_scratch_test.dag --function stmt_body > /tmp/o.txt 2>&1; echo "RC $MODE $?"
grep -vE "floor-|memory-cgroup|^\s*$" /tmp/o.txt | cut -c1-600 | sed "s/^/$MODE: /" | tail -8
