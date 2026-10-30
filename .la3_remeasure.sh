#!/bin/bash
# LOOK-AHEAD-3 one-dispatch remeasure. Usage: remeasure.sh <successor-sha>
# Run from worktree root via: ctrl-build --remote --timeout 5h -- bash /tmp/la3/remeasure.sh <sha>
set -x
SHA=${1:?usage: remeasure.sh <successor-sha>}
export GITHUB_SHA=$SHA
echo "=== VALIDITY: mirrors must carry the fused rule ==="
HITS=$(grep -c "declaration_resolved_container\|unbound_container_diagnostics\|map_builtin_spelling\|unimported_map_diagnostics" src/v1/stage0/src/v1_compiler_infer_resolve.rs || true); echo "FUSED_RULE_MIRROR_HITS=$HITS"; if [ "${HITS:-0}" -lt 1 ]; then echo "MIRROR_LACKS_FUSED_RULE_ABORT"; exit 9; fi
SEEDHITS=$(grep -c "declaration_resolved_container\|unbound_container_diagnostics\|map_builtin_spelling\|unimported_map_diagnostics" src/v1/04_resolve.dag || true); echo "FUSED_RULE_SEED_HITS=$SEEDHITS"; if [ "${SEEDHITS:-0}" -lt 1 ]; then echo "SEED_LACKS_FUSED_RULE_ABORT"; exit 9; fi
grep -c "map_support_keys\|map_lookup" src/v1/stage0/src/v1_compiler_infer_method.rs 2>/dev/null || true
echo "=== GATE 1: parse ==="
cargo build --release -p v1-compiler --bin v1_src_dag_parse --bin gunbc 2>&1 | tail -3
./target/release/v1_src_dag_parse > /tmp/parse_la3.txt 2>&1; echo "PARSE_EXIT=$?"
echo "PARSE_REFUSED=$(grep -c PARSE-REFUSED /tmp/parse_la3.txt)"
echo "CITED_ABSENT=$(grep -c CITED-DECLARATION-ABSENT /tmp/parse_la3.txt)"
echo "STALE=$(grep -c CITATION-DEBT-ROW-STALE /tmp/parse_la3.txt)"
echo "=== CENSUS scoped ==="
./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/census_la3.txt 2>&1; echo "CENSUS_EXIT=$?"
tail -60 /tmp/census_la3.txt
echo "=== CLAIM package 8 members ==="
./target/release/gunbc test //v2/test/claim/parse/expression_bodied_fn_decl_parse:all > /tmp/claim_la3.txt 2>&1; echo "CLAIM_EXIT=$?"
tail -40 /tmp/claim_la3.txt
echo "=== MAP SITES BY MODULE (from binding_unbound lines) ==="
grep -a "binding_unbound .* Map " /tmp/census_la3.txt | awk '{print $2}' | sort | uniq -c | sort -rn | head -40
echo "MAP_TOTAL=$(grep -ac 'binding_unbound .* Map ' /tmp/census_la3.txt)"
echo "=== totals lines ==="
grep -aE "modules [0-9]+|resolve_refused|infer_refused|inferred|roots" /tmp/census_la3.txt | tail -20
