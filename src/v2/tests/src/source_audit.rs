//! Source-level audit tests for v2 .dag files.
//!
//! These tests read .dag source files and assert on textual content.
//! No compilation needed — just file reads and string matching.

use std::path::{Path, PathBuf};

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

fn collect_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, files);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("dag" | "rs")
        ) {
            files.push(path);
        }
    }
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn pipe_arrow_token_exists() {
    let source = read_v2_file("src/v2/00_core.dag");
    assert!(
        source.contains("PipeArrow"),
        "00_core.dag should contain PipeArrow"
    );
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
    assert!(
        source.contains("PipeArrow"),
        "02_parse.dag should contain PipeArrow"
    );
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
        "fn token_matches_expected(",
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

    // Phase 3: Python and Go call emit_tco_unified (fully parameterized).
    // Per-language tco_match/tco_match_arm are unified into emit_unified_tco_match
    // in the shared emitter.
    let python_source = read_v2_file("src/v2/05_emit_python.dag");
    assert!(
        python_source.contains("emit_tco_unified"),
        "05_emit_python.dag should call emit_tco_unified"
    );

    // Unified TCO dispatcher lives in the shared emitter.
    let shared_source = read_v2_file("src/v2/05_emit.dag");
    assert!(
        shared_source.contains("fn emit_unified_tco_expr"),
        "05_emit.dag should contain emit_unified_tco_expr"
    );
    assert!(
        shared_source.contains("fn emit_unified_tco_body"),
        "05_emit.dag should contain emit_unified_tco_body"
    );
    assert!(
        shared_source.contains("fn emit_unified_tco_match"),
        "05_emit.dag should contain emit_unified_tco_match (Phase 3)"
    );
    assert!(
        shared_source.contains("fn emit_tco_unified"),
        "05_emit.dag should contain emit_tco_unified (Phase 3 entry point)"
    );

    // TCO syntax tokens live in LanguageSpec spec values, not backend emitters.
    // Match spec field assignments to avoid false positives from comments.
    let languages_source = read_v2_file("src/v2/languages.dag");
    assert!(
        languages_source.contains("loop_keyword: \"while True\""),
        "languages.dag python_spec should set loop_keyword to 'while True'"
    );
    assert!(
        languages_source.contains("continue_str: \"continue\\n\""),
        "languages.dag python_spec should set continue_str to 'continue\\n' (TCO loop newline)"
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
    assert!(
        source.contains("ExprReturn"),
        "00_core.dag should contain ExprReturn"
    );
    assert!(
        source.contains("ExprForEach"),
        "00_core.dag should contain ExprForEach"
    );
    assert!(
        source.contains("ExprIndex"),
        "00_core.dag should contain ExprIndex"
    );
    assert!(
        source.contains("ExprSlice"),
        "00_core.dag should contain ExprSlice"
    );
    assert!(
        source.contains("fn map_children"),
        "00_core.dag should contain fn map_children"
    );
}

#[test]
fn complexity_source_and_stage0_stay_in_parity_on_classifier_hooks() {
    let source = read_v2_file("src/v2/complexity.dag");
    let stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_complexity.rs");

    // Graph types/functions moved to dsl/std/graph.dag (PR #336 follow-up).
    // Check they exist in the graph module's stage0 mirror instead.
    let graph_stage0 = read_v2_file("src/v2/stage0/src/std_graph.rs");
    for needle in [
        "pub struct CallGraph",
        "pub struct DfsFinishAcc",
        "pub struct SccComponentAcc",
        "pub fn dfs_finish_order(",
        "pub fn dfs_collect_component(",
    ] {
        assert_live_contains(
            &graph_stage0,
            needle,
            &format!("stage0 std_graph should contain {needle}"),
        );
    }

    for needle in [
        "fn max_path_self_calls(",
        "fn max_path_self_calls_with_cont(",
        "fn max_path_self_calls_block(",
        "fn build_scc_measure_params(",
        "fn normalize_asymptotic(",
        // normalize_constants merged into normalize_asymptotic (single-pass)
        "fn format_cost_class(",
        "fn format_cost_inner(",
        "fn parenthesize_additive_cost(",
        "fn classify_complexity(",
        "ExprReturn",
    ] {
        assert_live_contains(
            &source,
            needle,
            &format!("src/v2/complexity.dag should contain {needle}"),
        );
    }

    for needle in [
        "pub fn max_path_self_calls(",
        "pub fn max_path_self_calls_with_cont(",
        "pub fn max_path_self_calls_block(",
        "pub fn build_scc_measure_params(",
        "pub fn normalize_asymptotic(",
        // normalize_constants merged into normalize_asymptotic (single-pass)
        "pub fn format_cost_class(",
        "pub fn format_cost_inner(",
        "pub fn parenthesize_additive_cost(",
        "pub fn classify_complexity(",
        "ExprData::ExprReturn",
    ] {
        assert_live_contains(
            &stage0,
            needle,
            &format!("stage0 complexity mirror should contain {needle}"),
        );
    }

    // classify_complexity returns String (formatted output).
    assert_live_contains(
        &source,
        "fn classify_complexity(expr: CostExpr) -> String",
        "classify_complexity should return String",
    );
    assert_live_contains(
        &stage0,
        "pub fn classify_complexity(expr: Rc<CostExpr>) -> String",
        "stage0 classify_complexity should return String",
    );

    assert_live_not_contains(
        &source,
        "type ComplexityClass",
        "ComplexityClass should not exist — CostExpr is the single authority",
    );
    assert_live_not_contains(
        &source,
        "formatted: String",
        "ComplexityReport should not carry a formatted string field",
    );
    assert_live_not_contains(
        &source,
        "fn recursive_variant_field_key(",
        "complexity should consume infer_env recursive field keys rather than redefining them",
    );
    assert_live_not_contains(
        &source,
        "fn scc_members_for(",
        "complexity should not rebuild SCCs with repeated reachability passes",
    );
    assert_live_not_contains(
        &source,
        "split(delimiter: \"::\")",
        "complexity should not recover recursive fields from concatenated string keys",
    );
    assert_live_not_contains(
        &stage0,
        "pub formatted: String",
        "stage0 ComplexityReport should not carry a formatted string field",
    );
    assert_live_not_contains(
        &stage0,
        "pub fn recursive_variant_field_key(",
        "stage0 complexity mirror should consume infer_env recursive field keys rather than redefining them"
    );
    assert_live_not_contains(
        &stage0,
        "pub fn scc_members_for(",
        "stage0 complexity mirror should not rebuild SCCs with repeated reachability passes",
    );
    assert_live_not_contains(
        &stage0,
        ".split(&\"::\".to_string())",
        "stage0 complexity mirror should not recover recursive fields from concatenated string keys"
    );
}

#[test]
fn parser_progress_witness_hooks_live_in_parse_layer() {
    let source = read_v2_file("src/v2/02_parse.dag");
    let stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_parse.rs");
    let complexity = read_v2_file("src/v2/complexity.dag");

    for needle in [
        "type ParserHelperIdentity",
        "type ParserResultWitness",
        "fn parser_progress_flag_var(",
        "fn parser_helper_identity(",
        "fn parser_passthrough_state_expr(",
        "fn parser_result_witness(",
    ] {
        assert_live_contains(
            &source,
            needle,
            &format!("src/v2/02_parse.dag should contain {needle}"),
        );
    }

    for needle in [
        "pub enum ParserHelperIdentity",
        "pub enum ParserResultWitness",
        "pub fn parser_progress_flag_var(",
        "pub fn parser_helper_identity(",
        "pub fn parser_passthrough_state_expr(",
        "pub fn parser_result_witness(",
    ] {
        assert_live_contains(
            &stage0,
            needle,
            &format!("stage0 parser mirror should contain {needle}"),
        );
    }

    assert_live_contains(
        &complexity,
        "import v2.compiler.parse {",
        "src/v2/complexity.dag should consume parser-owned progress witnesses",
    );
    assert_live_not_contains(
        &complexity,
        "\"skip_newlines\"",
        "complexity should not hardcode parser helper names inline",
    );
    assert_live_not_contains(
        &complexity,
        "\"skip_continuation_newlines\"",
        "complexity should not hardcode parser helper names inline",
    );
}

#[test]
fn recursive_variant_witnesses_are_structural() {
    let env_source = read_v2_file("src/v2/04_env.dag");
    let env_stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_infer_env.rs");
    let infer_source = read_v2_file("src/v2/04_infer.dag");

    // InductiveField replaces RecursiveVariantFieldWitness (CX-L1)
    for needle in ["fn put_inductive_field(", "fn merge_inductive_fields("] {
        assert_live_contains(
            &env_source,
            needle,
            &format!("src/v2/04_env.dag should contain {needle}"),
        );
    }

    for needle in [
        "pub fn put_inductive_field(",
        "pub fn merge_inductive_fields(",
    ] {
        assert_live_contains(
            &env_stage0,
            needle,
            &format!("stage0 infer env mirror should contain {needle}"),
        );
    }

    // CX-NEXT Phase 2: inductive fields are now declared in std/node.dag
    // and loaded via inductive_fields_list_to_map(compiler_inductive_fields).
    // The put_inductive_field calls moved from inline kernel seeding to the
    // std/node.dag data table. Inference loads from std.node, not inline calls.
    assert_live_contains(
        &infer_source,
        "import std.node { compiler_inductive_fields",
        "inference should import compiler inductive fields from std.node",
    );
    assert_live_contains(
        &infer_source,
        "inductive_fields_list_to_map(fields: compiler_inductive_fields)",
        "inference should build fields from std.node data",
    );
}

#[test]
fn emit_backends_do_not_consume_inductive_fields() {
    let emit_source = read_v2_file("src/v2/05_emit.dag");
    let emit_stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_emit.rs");
    let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
    let go_source = read_v2_file("src/v2/05_emit_go.dag");
    let py_source = read_v2_file("src/v2/05_emit_python.dag");
    let rust_stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_emit_rust.rs");
    let go_stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_emit_go.rs");
    let py_stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_emit_python.rs");

    assert_live_not_contains(
        &emit_source,
        "scope.type_env.inductive_fields",
        "shared emit helpers should not read inductive fields from TypeEnv",
    );
    assert_live_not_contains(
        &emit_stage0,
        "scope.type_env.inductive_fields",
        "stage0 shared emit helpers should not read inductive fields from TypeEnv",
    );

    for (label, source) in [
        ("src/v2/05_emit_rust.dag", rust_source.as_str()),
        ("src/v2/05_emit_go.dag", go_source.as_str()),
        ("src/v2/05_emit_python.dag", py_source.as_str()),
        (
            "src/v2/stage0/src/v2_compiler_emit_rust.rs",
            rust_stage0.as_str(),
        ),
        (
            "src/v2/stage0/src/v2_compiler_emit_go.rs",
            go_stage0.as_str(),
        ),
        (
            "src/v2/stage0/src/v2_compiler_emit_python.rs",
            py_stage0.as_str(),
        ),
    ] {
        assert_live_not_contains(
            source,
            "inductive_fields",
            &format!("{label} should not consume inductive fields directly"),
        );
        assert_live_not_contains(
            source,
            "InductiveField",
            &format!("{label} should not import inductive field types directly"),
        );
    }
}

#[test]
fn compile_gate_keeps_infer_errors_blocking_in_stage0() {
    let source = read_v2_file("src/v2/compile.dag");
    let stage0 = read_v2_file("src/v2/stage0/src/v2_compiler_compile.rs");

    // Emission gate fires on typed_diags (infer errors), not complexity
    // diagnostics. Complexity analysis is opt-in (analyze_complexity:
    // false by default); when enabled, ComplexityUnknown is still
    // non-blocking at emission/CLI (cx-design.md C6). Re-enable blocking
    // when CX-5 lands and violations reach 0.
    // Type errors block emission. Complexity violations are surfaced but
    // non-blocking (analyzer limitations ratchet down over time).
    assert_live_contains(
        &source,
        "analyze_complexity: false",
        "src/v2/compile.dag should default complexity analysis off",
    );
    assert_live_contains(
        &source,
        "let type_errors = typed_diags |> filter(d => is_error_diagnostic(d: d.diagnostic))",
        "src/v2/compile.dag should gate emission on type errors",
    );
    assert_live_not_contains(
        &source,
        "bootstrap_mode",
        "src/v2/compile.dag should not have bootstrap escape hatch",
    );
    assert_live_contains(
        &stage0,
        "typed_diags",
        "stage0 compile mirror should reference typed_diags for fail-closed path",
    );
    assert_live_not_contains(
        &stage0,
        "BOOTSTRAP_MODE",
        "stage0 compile mirror should not have BOOTSTRAP_MODE escape hatch",
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
    // Callable wrapping is data-driven: template lives in languages.dag,
    // emitter reads callable_type_template from LanguageSpec.
    let lang_source = read_v2_file("src/v2/languages.dag");
    assert_live_contains(
        &lang_source,
        "Rc<dyn Fn(",
        "languages.dag should declare Rust callable_type_template with Rc<dyn Fn(...)>",
    );
    assert_live_contains(
        &lang_source,
        "callable_type_template",
        "languages.dag should contain callable_type_template field",
    );
    assert_live_contains(
        &emit_source,
        "callable_type_template",
        "05_emit.dag should read callable_type_template from LanguageSpec",
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

// ── Canonical accessor and fallback elimination audits ────────────────
//
// After the P5.11 child-layout centralization, ad-hoc `None => texpr`
// fallbacks must not exist in consumer files. Child layout knowledge
// lives in 00_core.dag accessors only.

#[test]
fn canonical_accessors_exist_in_core() {
    let source = read_v2_file("src/v2/00_core.dag");
    let accessors = [
        "fn expr_child_at",
        "fn if_condition",
        "fn if_then_branch",
        "fn if_else_branch",
        "fn match_scrutinee",
        "fn match_arm_nodes",
        "fn binop_left",
        "fn binop_right",
        "fn unaryop_operand",
        "fn field_access_base",
        "fn method_receiver",
        "fn method_arg_nodes",
        "fn lambda_body",
        "fn let_value",
        "fn let_body",
        "fn cast_expr",
        "fn cast_target",
        "fn foreach_collection",
        "fn foreach_body",
        "fn index_base",
        "fn index_expr",
        "fn slice_base",
        "fn slice_start",
        "fn slice_end",
        "fn return_value",
    ];
    for acc in &accessors {
        assert!(source.contains(acc), "00_core.dag must define {}", acc);
    }
}

#[test]
fn no_self_fallback_in_consumer_files() {
    // After accessor migration, no consumer file should contain the
    // fail-open "None => texpr" or "None => expr" child-access pattern.
    let files = [
        "src/v2/04_infer.dag",
        "src/v2/04_service.dag",
        "src/v2/05_emit.dag",
        "src/v2/05_emit_rust.dag",
        "src/v2/05_emit_go.dag",
        "src/v2/05_emit_python.dag",
        "src/v2/complexity.dag",
        "src/v2/ownership.dag",
    ];
    for path in &files {
        let source = read_v2_file(path);
        assert!(
            !source.contains("None => texpr"),
            "{} must not contain 'None => texpr' self-fallback (use canonical accessors)",
            path
        );
        assert!(
            !source.contains("None => expr }"),
            "{} must not contain 'None => expr' self-fallback (use canonical accessors)",
            path
        );
    }
}

#[test]
fn serializer_has_no_expr_other_fallback() {
    let source = read_v2_file("src/v2/compile.dag");
    assert!(
        !source.contains("ExprOther"),
        "compile.dag serializer must not collapse any variant to ExprOther"
    );
}

// ── Ratchet audits ────────────────────────────────────────────────────
//
// These tests make invisible breakage visible by counting structural
// properties and asserting they stay within known bounds.

const PARSE_ITEM_KEYWORD_ARM_RATCHET: usize = 0;

#[test]
fn parse_item_keyword_arm_count() {
    let source = read_v2_file("src/v2/02_parse.dag");
    let func_start = source
        .find("fn parse_item(")
        .expect("parse_item must exist in 02_parse.dag");
    let rest = &source[func_start..];
    let func_end = rest[1..].find("\nfn ").map(|i| i + 1).unwrap_or(rest.len());
    let func_body = &rest[..func_end];
    // Count on live source (comments stripped) to avoid false positives
    // from historical notes or TODOs mentioning ShKw patterns.
    let live = live_source(func_body);
    let arm_count = live.matches("Some { value: ShKw").count();
    assert_eq!(
        arm_count, PARSE_ITEM_KEYWORD_ARM_RATCHET,
        "parse_item has {} keyword arms, expected {} — \
         update PARSE_ITEM_KEYWORD_ARM_RATCHET if this is intentional",
        arm_count, PARSE_ITEM_KEYWORD_ARM_RATCHET
    );
}

fn unescape_dag_string_literal(mut rest: &str) -> String {
    let mut out = String::new();
    while let Some(ch) = rest.chars().next() {
        rest = &rest[ch.len_utf8()..];
        if ch == '"' {
            break;
        }
        if ch == '\\' {
            let (esc, tail) = rest
                .chars()
                .next()
                .map(|c| (c, &rest[c.len_utf8()..]))
                .expect("trailing backslash in ratchet.dag pattern");
            rest = tail;
            out.push(match esc {
                'n' => '\n',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                other => other,
            });
        } else {
            out.push(ch);
        }
    }
    out
}

#[test]
fn second_syntax_spec_exists() {
    // Stream 0 exit criterion: SyntaxSpec abstraction is real.
    // A second SyntaxSpec (Rust) must exist alongside the .dag spec,
    // proving the parser can be driven by non-.dag spec data.
    let ws = crate::helpers::workspace_root();
    let rust_spec = ws.join("dsl/extdeps/languages/rust/syntax.dag");
    assert!(
        rust_spec.exists(),
        "Rust SyntaxSpec must exist at dsl/extdeps/languages/rust/syntax.dag"
    );
    let content = std::fs::read_to_string(&rust_spec).expect("should read Rust syntax spec");
    // Verify it declares a SyntaxSpec instance
    assert!(
        content.contains("rust_syntax_spec: SyntaxSpec"),
        "Rust spec should declare rust_syntax_spec: SyntaxSpec"
    );
    // Verify it has item forms, operators, keyword literals
    assert!(
        content.contains("rust_item_forms"),
        "should define item forms"
    );
    assert!(
        content.contains("rust_operators"),
        "should define operators"
    );
    assert!(
        content.contains("rust_keyword_literals"),
        "should define keyword literals"
    );
    // Verify it has Rust-specific keywords
    assert!(
        content.contains("\"struct\""),
        "should include struct keyword"
    );
    assert!(content.contains("\"enum\""), "should include enum keyword");
    assert!(content.contains("\"impl\""), "should include impl keyword");
}

#[test]
fn no_expr_data_before_catch_all_in_core() {
    // The NoExprData arm must appear BEFORE any catch-all `_` in
    // expr_has_non_tail_self_call. Verify by checking that NoExprData
    // appears in the function and is not shadowed.
    let source = read_v2_file("src/v2/00_core.dag");
    let func_start = source
        .find("fn expr_has_non_tail_self_call")
        .expect("function must exist");
    let func_body = &source[func_start..];
    let no_expr_pos = func_body
        .find("NoExprData =>")
        .expect("NoExprData arm must exist");
    let wildcard_pos = func_body.find("_ =>\n").expect("catch-all arm must exist");
    assert!(
        no_expr_pos < wildcard_pos,
        "NoExprData must appear BEFORE catch-all _ in expr_has_non_tail_self_call"
    );
}

#[test]
fn rt_functions_derived_from_registry() {
    // rt_functions() is now derived from rt_function_registry via fold.
    // Verify exact parity: every registry entry appears, and no extras.
    use v2_compiler::extdeps_languages_rust_emit::{rt_function_registry, rt_functions};

    let registry = rt_function_registry();
    let funcs = rt_functions();

    assert_eq!(
        registry.len(),
        funcs.len(),
        "registry count ({}) != rt_functions count ({})",
        registry.len(),
        funcs.len()
    );

    for f in registry.iter() {
        assert!(
            funcs.contains_key(&f.name),
            "registry entry '{}' missing from rt_functions()",
            f.name
        );
    }
}

#[test]
fn rt_bridge_function_names_derived_from_registry() {
    // rt_bridge_function_names() is derived from rt_function_registry,
    // filtering entries where name != bridge_name.
    use v2_compiler::extdeps_languages_rust_emit::{
        rt_bridge_function_names, rt_function_registry,
    };

    let registry = rt_function_registry();
    let bridge = rt_bridge_function_names();

    let expected: Vec<_> = registry
        .iter()
        .filter(|f| f.name != f.bridge_name)
        .collect();

    assert!(
        !expected.is_empty(),
        "registry should have bridge_name overrides"
    );
    assert_eq!(
        expected.len(),
        bridge.len(),
        "expected {} bridge entries, got {}",
        expected.len(),
        bridge.len()
    );

    for f in &expected {
        match bridge.get(&f.name) {
            Some(bn) => assert_eq!(
                *bn, f.bridge_name,
                "bridge mismatch for '{}': expected '{}', got '{}'",
                f.name, f.bridge_name, bn
            ),
            None => panic!(
                "registry entry '{}' (bridge '{}') missing from rt_bridge_function_names()",
                f.name, f.bridge_name
            ),
        }
    }
}

#[test]
fn rt_wraps_result_derived_from_registry() {
    // rt_wraps_result() is derived from RuntimeFunction.wraps_result via fold.
    // Verify exact parity: every registry entry with wraps_result: true appears,
    // and no extra entries exist.
    use v2_compiler::extdeps_languages_rust_emit::{rt_function_registry, rt_wraps_result};

    let registry = rt_function_registry();
    let wraps_map = rt_wraps_result();

    let registry_wraps: Vec<String> = registry
        .iter()
        .filter(|f| f.wraps_result)
        .map(|f| f.name.clone())
        .collect();

    assert!(
        !registry_wraps.is_empty(),
        "registry should have wraps_result entries"
    );

    // Every registry wraps_result: true must appear in derived map
    for name in &registry_wraps {
        assert!(
            wraps_map.contains_key(name),
            "RuntimeFunction '{}' has wraps_result: true but missing from rt_wraps_result()",
            name
        );
    }

    // No extra entries in derived map
    for (name, _) in wraps_map.iter() {
        assert!(registry_wraps.contains(name),
            "rt_wraps_result() contains '{}' but no matching registry entry with wraps_result: true", name);
    }
}

#[test]
fn method_wraps_result_derived_from_specs() {
    // rust_method_wraps_result() is derived from SimpleMethodSpec.wraps_result via fold.
    // Verify exact parity: every spec entry with wraps_result: true appears,
    // and no extra entries exist.
    use v2_compiler::extdeps_languages_rust_emit::{
        rust_method_wraps_result, rust_simple_method_specs,
    };

    let specs = rust_simple_method_specs();
    let wraps_map = rust_method_wraps_result();

    let spec_wraps: Vec<String> = specs
        .iter()
        .filter(|s| s.wraps_result)
        .map(|s| s.method_name.clone())
        .collect();

    assert!(
        !spec_wraps.is_empty(),
        "specs should have wraps_result entries"
    );

    // Every spec wraps_result: true must appear in derived map
    for name in &spec_wraps {
        assert!(wraps_map.contains_key(name),
            "SimpleMethodSpec '{}' has wraps_result: true but missing from rust_method_wraps_result()", name);
    }

    // No extra entries in derived map
    for (name, _) in wraps_map.iter() {
        assert!(spec_wraps.contains(name),
            "rust_method_wraps_result() contains '{}' but no matching spec entry with wraps_result: true", name);
    }
}

#[test]
fn method_templates_derived_from_specs() {
    // rust_method_templates() is derived from SimpleMethodSpec list.
    // Verify exact parity: every spec entry appears in template map.
    use v2_compiler::extdeps_languages_rust_emit::{
        rust_method_templates, rust_simple_method_specs,
    };

    let specs = rust_simple_method_specs();
    let templates = rust_method_templates();

    assert_eq!(
        specs.len(),
        templates.len(),
        "spec count ({}) != template map count ({})",
        specs.len(),
        templates.len()
    );

    for spec in specs.iter() {
        match templates.get(&spec.method_name) {
            Some(tmpl) => assert_eq!(
                *tmpl, spec.template,
                "template mismatch for '{}': spec='{}', map='{}'",
                spec.method_name, spec.template, tmpl
            ),
            None => panic!(
                "SimpleMethodSpec '{}' missing from rust_method_templates()",
                spec.method_name
            ),
        }
    }
}

#[test]
fn is_copy_checkpoint_parity() {
    // Derive all assertions from rust_type_checkpoints — single authority.
    // Every checkpoint must return Some(is_copy), never None.
    // User-defined types (no checkpoint) correctly return None.
    use v2_compiler::extdeps_languages_rust_types::rust_type_checkpoints;
    use v2_compiler::v2_compiler_coercion::is_copy;
    use v2_compiler::v2_compiler_emit::RenderTarget;

    let checkpoints = rust_type_checkpoints();
    assert!(
        !checkpoints.is_empty(),
        "rust_type_checkpoints should have entries"
    );

    // Every declared checkpoint must return Some(_), not None
    for cp in checkpoints.iter() {
        let result = is_copy(RenderTarget::Rust, cp.dag_name.clone());
        assert!(
            result.is_some(),
            "TypeCheckpoint '{}' should return Some from is_copy, got None — \
             checkpoint exists but is_copy can't find it",
            cp.dag_name
        );
        assert_eq!(
            result, cp.is_copy,
            "TypeCheckpoint '{}' is_copy mismatch: checkpoint says {:?}, is_copy returns {:?}",
            cp.dag_name, cp.is_copy, result
        );
    }

    // User-defined types must return None (no checkpoint exists)
    let result = is_copy(RenderTarget::Rust, "MyStruct".to_string());
    assert_eq!(
        result, None,
        "user-defined type should return None from is_copy"
    );
}
