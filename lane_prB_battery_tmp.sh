#!/bin/bash
# Scratch (never lands): lane B affected battery. The fix touches v1's data initializer route;
# consumers of that route: the facts tools (the #14559 main-break class), data-refusal parse
# claims, data-initializer emit claims, and the split-brace fixture's extractor test.
set -uo pipefail
PIN=aaa3ae108c87c76485f1d97fe2e039dc480881ba
git fetch -q origin 2>/dev/null || true
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then
  echo "PIN: runner was at $CUR; checking out $PIN"
  git checkout -f -q "$PIN" || { echo "PIN-FAILED"; exit 3; }
fi
git rev-parse HEAD | cut -c1-11
test "$(git rev-parse HEAD)" = "$PIN" || { echo "PIN-MISMATCH"; exit 3; }

echo "=== HEAD ==="; git rev-parse --short HEAD; git branch --show-current
echo "STEP BUILD claim_batch"; date -u +%H:%M:%S
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run_claim() {
  echo "STEP claim $2"; date -u +%H:%M:%S
  local entry="$1" fn="$2"
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$entry" --function "$fn" > /tmp/bat_$(basename "$entry").log 2>&1
  echo "claim@${fn} rc=$? $(grep -acm1 'FAIL' /tmp/bat_$(basename "$entry").log >/dev/null 2>&1 && echo '')"
  grep -am1 'FAIL\|resolve failed' /tmp/bat_$(basename "$entry").log | cut -c1-200
}
run_claim src/v2/test/claim/parse/data_initializer_layout_parse.dag a_continuation_line_data_initializer_parses_on_both_routes
run_claim src/v2/test/claim/parse/expression_bodied_fn_decl_parse.dag expression_bodied_fn_decl_parses_holds
run_claim src/v2/test/claim/parse/data_keyword_as_value.dag the_data_keyword_as_a_value_parses_holds
run_claim src/v2/test/claim/parse/parse_test_grammar_expect.dag expect_wrapping_every_row_changes_no_accepted_tree_holds
run_claim src/v2/test/claim/emit/swift_produced_decl_emit.dag swift_produced_decl_emits_the_door_line_holds
run_claim src/v2/test/claim/execution/dag_emit_interface.dag a_string_literal_node_is_spelled_quoted_and_escaped_holds
run_claim src/v2/test/claim/concept_index_enumeration/concept_index_enumeration.dag witness_enumeration_nonempty
run_claim src/v2/test/claim/long/decl_index_enumeration.dag decl_facts_nonempty_over_witness_roots
echo "SKIP rust lib tests (separate dispatch)"
if false; then echo "=== rust tests (split-brace fixture extractor + parse env decode) ==="
cargo test --release -p v1-compiler --lib cli_run 2>&1 | grep -aE 'test result|FAILED|panicked' | head -4 | cut -c1-160
fi
echo "=== DONE ==="
