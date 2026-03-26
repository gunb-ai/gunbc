//! Source-level audit tests for v2 .dag files.
//!
//! These tests read .dag source files and assert on textual content.
//! No compilation needed — just file reads and string matching.

use crate::helpers::read_v2_file;

fn live_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_live_contains(source: &str, needle: &str, message: &str) {
    assert!(live_source(source).contains(needle), "{message}");
}

fn assert_live_not_contains(source: &str, needle: &str, message: &str) {
    assert!(!live_source(source).contains(needle), "{message}");
}

#[test]
fn pipe_arrow_token_exists() {
    let source = read_v2_file("src/v2/00_core.dag");
    assert!(source.contains("PipeArrow"), "00_core.dag should contain PipeArrow");
}

#[test]
fn null_coalesce_in_binop_kind() {
    let source = read_v2_file("src/v2/00_core.dag");
    assert!(
        source.contains("NullCoalesce"),
        "00_core.dag should contain NullCoalesce"
    );
}

#[test]
fn parse_supports_pipe_arrow() {
    let source = read_v2_file("src/v2/02_parse.dag");
    assert!(source.contains("PipeArrow"), "02_parse.dag should contain PipeArrow");
    assert!(
        source.contains("parse_pipe_rhs"),
        "02_parse.dag should contain parse_pipe_rhs"
    );
}

#[test]
fn parse_supports_null_coalesce() {
    let source = read_v2_file("src/v2/02_parse.dag");
    assert!(
        source.contains("NullCoalesce"),
        "02_parse.dag should contain NullCoalesce"
    );
}

#[test]
fn parser_uses_expected_token_api_for_control_flow() {
    let source = read_v2_file("src/v2/02_parse.dag");
    assert_live_contains(
        &source,
        "type ExpectedToken",
        "02_parse.dag should define a typed ExpectedToken API",
    );
    assert_live_contains(
        &source,
        "fn shape_matches_expected(",
        "02_parse.dag should match parser control flow on ExpectedToken",
    );
    assert_live_not_contains(
        &source,
        "fn kind_matches_tag(",
        "02_parse.dag should no longer route parser control flow through string tag dispatch",
    );
}

#[test]
fn emit_handles_null_coalesce() {
    let source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        source.contains("unwrap_or_else"),
        "05_emit_rust.dag should contain unwrap_or_else"
    );
}

#[test]
fn emit_handles_for_loop() {
    let source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        source.contains("emit_typed_for_each"),
        "05_emit_rust.dag should contain emit_typed_for_each"
    );
    assert!(
        source.contains("iter().cloned()"),
        "05_emit_rust.dag should contain iter().cloned()"
    );
}

#[test]
fn emit_generates_cargo_toml() {
    let source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        source.contains("emit_cargo_toml"),
        "05_emit_rust.dag should contain emit_cargo_toml"
    );
}

#[test]
fn emit_has_tco_support() {
    let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        rust_source.contains("emit_typed_tco_body"),
        "05_emit_rust.dag should contain emit_typed_tco_body"
    );
    assert!(
        rust_source.contains("emit_typed_tco_expr"),
        "05_emit_rust.dag should contain emit_typed_tco_expr"
    );
    assert!(
        rust_source.contains("emit_tco_params"),
        "05_emit_rust.dag should contain emit_tco_params"
    );
    assert!(
        rust_source.contains("loop {"),
        "05_emit_rust.dag should contain 'loop {{'"
    );
    assert!(
        rust_source.contains("continue;"),
        "05_emit_rust.dag should contain 'continue;'"
    );
    assert!(
        rust_source.contains("break "),
        "05_emit_rust.dag should contain 'break '"
    );

    let python_source = read_v2_file("src/v2/05_emit_python.dag");
    assert!(
        python_source.contains("emit_py_typed_tco_body"),
        "05_emit_python.dag should contain emit_py_typed_tco_body"
    );
    assert!(
        python_source.contains("while True:"),
        "05_emit_python.dag should contain 'while True:'"
    );
    assert!(
        python_source.contains("continue"),
        "05_emit_python.dag should contain 'continue'"
    );

    let core_source = read_v2_file("src/v2/00_core.dag");
    assert!(
        core_source.contains("fn expr_has_self_call"),
        "00_core.dag should contain fn expr_has_self_call"
    );
    assert!(
        core_source.contains("fn expr_has_non_tail_self_call"),
        "00_core.dag should contain fn expr_has_non_tail_self_call"
    );
}

#[test]
fn parse_supports_where_clause() {
    let source = read_v2_file("src/v2/02_parse.dag");
    assert!(
        source.contains("try_where_clause"),
        "02_parse.dag should contain try_where_clause"
    );
    assert!(
        source.contains("parse_predicates"),
        "02_parse.dag should contain parse_predicates"
    );
}

#[test]
fn parse_supports_response_blocks() {
    let source = read_v2_file("src/v2/02_parse.dag");
    assert!(
        source.contains("parse_optional_response_block"),
        "02_parse.dag should contain parse_optional_response_block"
    );
    assert!(
        source.contains("parse_optional_mock_response_block"),
        "02_parse.dag should contain parse_optional_mock_response_block"
    );
}

#[test]
fn typecheck_has_cycle_detection() {
    let source = read_v2_file("src/v2/04_infer.dag");
    assert!(
        source.contains("detect_type_cycles"),
        "04_infer.dag should contain detect_type_cycles"
    );
    assert!(
        source.contains("recursive_types"),
        "04_infer.dag should contain recursive_types"
    );
}

#[test]
fn pattern_lookup_uses_explicit_subject_status() {
    let source = read_v2_file("src/v2/04_patterns.dag");
    assert_live_contains(
        &source,
        "type PatternSubject",
        "04_patterns.dag should define an explicit PatternSubject status channel",
    );
    assert_live_contains(
        &source,
        "PatternLookupBlocked",
        "04_patterns.dag should represent blocked lookup explicitly",
    );
    assert_live_not_contains(
        &source,
        "scrut.name == \"Error\"",
        "04_patterns.dag should not branch on raw scrutinee Error names inline",
    );
    assert_live_not_contains(
        &source,
        "variant.name == \"Dynamic\"",
        "04_patterns.dag should not branch on raw variant Dynamic names inline",
    );
}

#[test]
fn resolve_filters_failed_imports_and_cycles() {
    let source = read_v2_file("src/v2/03_resolve.dag");
    assert!(
        source.contains("acyclic_resolved"),
        "03_resolve.dag should contain acyclic_resolved"
    );
    assert!(
        source.contains("r.resolved.target_module != none"),
        "03_resolve.dag should contain 'r.resolved.target_module != none'"
    );
    assert!(
        source.contains("r.diagnostics |> count == 0"),
        "03_resolve.dag should contain 'r.diagnostics |> count == 0'"
    );
}

#[test]
fn typecheck_gates_inference_on_env_errors_and_resolves_expr_types() {
    let resolve_source = read_v2_file("src/v2/04_resolve.dag");
    assert_live_contains(
        &resolve_source,
        "fn resolve_expr_types(",
        "04_resolve.dag should define fn resolve_expr_types",
    );

    let infer_source = read_v2_file("src/v2/04_infer.dag");
    assert_live_not_contains(
        &infer_source,
        "fn resolve_expr_types(",
        "04_infer.dag should not define fn resolve_expr_types",
    );
    assert_live_contains(
        &infer_source,
        "if env_errors |> count > 0 {",
        "04_infer.dag should gate inference on env_errors before infer_items",
    );
}

#[test]
fn emit_preserves_field_provenance_and_named_arg_ordering() {
    let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        rust_source.contains("serde_rename_template"),
        "05_emit_rust.dag should reference serde_rename_template from LanguageSpec"
    );

    let emit_source = read_v2_file("src/v2/05_emit.dag");
    assert!(
        emit_source.contains("order_typed_call_args"),
        "05_emit.dag should contain order_typed_call_args"
    );
}

#[test]
fn emit_deletes_intrinsic_string_classifier_and_lambda_scope_fallback() {
    let emit_source = read_v2_file("src/v2/05_emit.dag");
    assert!(
        !emit_source.contains("classify_intrinsic_method"),
        "05_emit.dag should NOT contain classify_intrinsic_method"
    );

    let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        !rust_source.contains("extend_scope_for_lambda"),
        "05_emit_rust.dag should NOT contain extend_scope_for_lambda"
    );
    assert!(
        !rust_source.contains("let needs_wrap = false"),
        "05_emit_rust.dag should NOT contain 'let needs_wrap = false'"
    );
}

#[test]
fn final_cleanup_removes_parser_and_cli_fabrication_fallbacks() {
    let parse_source = read_v2_file("src/v2/02_parse.dag");
    assert!(
        parse_source.contains("parse_recovery_placeholder()"),
        "02_parse.dag should contain parse_recovery_placeholder()"
    );
    assert!(
        !parse_source.contains(
            "make_expr_node(expr_data: ExprLiteral { value: LitNull }, inferred: none, span: SourceSpan { start: 0, end: 0 })"
        ),
        "02_parse.dag should NOT contain fabricated null literal fallback"
    );

    let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        !rust_source.contains("LitNull => \"\""),
        "05_emit_rust.dag should NOT contain 'LitNull => \"\"'"
    );
    assert!(
        !rust_source.contains("None => Node { name: \"\""),
        "05_emit_rust.dag should NOT contain 'None => Node {{ name: \"\"'"
    );
}

#[test]
fn unannotated_function_reports_signature_resolution_error() {
    let source = read_v2_file("src/v2/04_sigs.dag");
    assert_live_contains(
        &source,
        "fn collect_func_call_edges(",
        "04_sigs.dag should define fn collect_func_call_edges",
    );
    assert_live_contains(
        &source,
        "fn topo_resolve_loop(",
        "04_sigs.dag should define fn topo_resolve_loop",
    );
}

#[test]
fn service_calls_under_return_inject_service_params() {
    let source = read_v2_file("src/v2/00_core.dag");
    assert!(source.contains("ExprReturn"), "00_core.dag should contain ExprReturn");
    assert!(
        source.contains("ExprForEach"),
        "00_core.dag should contain ExprForEach"
    );
    assert!(source.contains("ExprIndex"), "00_core.dag should contain ExprIndex");
    assert!(source.contains("ExprSlice"), "00_core.dag should contain ExprSlice");
    assert!(
        source.contains("fn map_children"),
        "00_core.dag should contain fn map_children"
    );
}

#[test]
fn testgen_emits_valid_rust() {
    let source = read_v2_file("src/v2/05_emit_rust.dag");
    let shared_source = read_v2_file("src/v2/05_emit.dag");
    assert!(
        !source.contains("todo!(\"unsupported simple expr"),
        "05_emit_rust.dag should NOT contain todo!(\"unsupported simple expr"
    );
    assert!(
        source.contains("compile_error!(\"unsupported simple expr")
            || source.contains("compile_error!(\\\"unsupported simple expr")
            || source.contains("emit_error_expr(message: \"unsupported simple expr")
            || shared_source.contains("emit_error_expr(message: \"unsupported simple expr"),
        "simple expr fail-loud path should exist in Rust emit or shared emit"
    );
    assert!(
        !source.contains("Ok(Default::default())"),
        "05_emit_rust.dag should NOT contain Ok(Default::default())"
    );
    assert!(
        source.contains("has_mock_prefix"),
        "05_emit_rust.dag should contain has_mock_prefix"
    );
    assert!(
        !source.contains("starts_with_prefix"),
        "05_emit_rust.dag should NOT contain starts_with_prefix"
    );
    let emit_source = read_v2_file("src/v2/05_emit.dag");
    assert!(
        emit_source.contains("extract_test_projections"),
        "05_emit.dag should contain extract_test_projections"
    );
    assert!(
        emit_source.contains("TestProjection"),
        "05_emit.dag should contain TestProjection"
    );
    assert!(
        emit_source.contains("Rust => concat(\"Rc<dyn Fn("),
        "05_emit.dag should render Rust callable types as Rc<dyn Fn(...)> in shared emit"
    );
    assert!(
        !emit_source.contains("Rust => concat(\"impl Fn("),
        "05_emit.dag should not render Rust callable types as impl Fn in shared emit"
    );
}

#[test]
fn testgen_service_mock_source_gate() {
    let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
    assert!(
        rust_source.contains("emit_test_file"),
        "05_emit_rust.dag should contain emit_test_file"
    );

    let emit_source = read_v2_file("src/v2/05_emit.dag");
    assert!(
        emit_source.contains("extract_test_projections"),
        "05_emit.dag should contain extract_test_projections"
    );
    assert!(
        emit_source.contains("has_mock_prefix"),
        "05_emit.dag should contain has_mock_prefix"
    );
    assert!(
        !emit_source.contains("Ok(Default::default())"),
        "05_emit.dag should NOT contain Ok(Default::default())"
    );
    assert!(
        !emit_source.contains("Value::Object(Default::default())"),
        "05_emit.dag should NOT contain Value::Object(Default::default())"
    );
    assert!(
        !emit_source.contains("_ => \"null\""),
        "05_emit.dag should NOT contain '_ => \"null\"'"
    );
    assert!(
        !emit_source.contains("extract_mock_props"),
        "05_emit.dag should NOT contain extract_mock_props"
    );
    assert!(
        !emit_source.contains("starts_with_prefix"),
        "05_emit.dag should NOT contain starts_with_prefix"
    );
}
