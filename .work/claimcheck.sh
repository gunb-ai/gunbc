set -uo pipefail
echo "=== COMMIT: $(git rev-parse HEAD)"
echo "=== TOOLCHAIN: $(rustc --version) / $(cargo --version)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin v1_src_dag_parse 2>&1 | tail -3
echo "BUILD-EXIT=$?"
echo "=== BIN sha256 (before run)"
sha256sum target/release/claim_batch target/release/gunbc target/release/v1_src_dag_parse
FUNCS="an_end_of_input_refusal_after_a_comment_is_located_at_the_last_lexeme_in_line_col,an_end_of_input_refusal_without_a_comment_is_located_in_line_col_too,the_compile_lane_line_and_the_cli_line_render_the_same_position_for_the_same_diagnostic,a_file_with_no_lexemes_is_refused_as_unlocated_rather_than_at_a_fabricated_position,an_ordinary_code_span_is_located_in_line_col_with_the_caret_on_the_lexeme,the_sweep_refuses_the_no_lexemes_file_while_reporting_the_valid_modules_clean"
./target/release/claim_batch --wet --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry dag/test/claim/diagnostic_span_location_rendering_witness.dag --functions "$FUNCS" > /tmp/claim.out 2>&1
echo "CLAIM-EXIT=$?"
echo "<<<ROWS"
grep -nE '^(PASS|FAIL) ' /tmp/claim.out
echo "ROWS>>>"
echo "<<<FAILURE-DETAIL"
grep -nE '✗|failed:|assert|expected|got |witness ' /tmp/claim.out | head -40
echo "FAILURE-DETAIL>>>"
echo "=== BIN sha256 (after run)"
sha256sum target/release/claim_batch target/release/gunbc target/release/v1_src_dag_parse
