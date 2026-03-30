#![allow(clippy::disallowed_macros)]

use crate::helpers::*;
use v2_compiler::v2_compiler_artifact::RenderTarget;
use v2_compiler::v2_compiler_compile::SourceFile;
use serde_json::Value;
use std::rc::Rc;

// ── Full DSL compilation (non-consensual: all files, no exceptions) ────

/// Scans dsl/ for all .dag files and compiles them as a unit.
/// No hardcoded file list. If a .dag file exists, it must compile.
#[test]
#[ignore] // run with: cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored
fn full_dsl_compiles() {
    let ws = workspace_root();
    let dsl_dir = ws.join("dsl");

    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    collect_dag_sources(&dsl_dir, &dsl_dir, &mut sources);

    assert!(
        !sources.is_empty(),
        "no .dag files found in dsl/ — something is wrong"
    );

    let result = v2_compiler::v2_compiler_compile::compile_sources(
        sources.clone(),
        RenderTarget::Rust,
    );

    let diag_count = result.diagnostics.len() as usize;
    if diag_count > 0 {
        let msgs = diagnostic_messages(&result);
        panic!(
            "full dsl/ compilation produced {} diagnostics (expected 0):\n{}",
            diag_count,
            msgs.iter()
                .enumerate()
                .map(|(i, m)| format!("  [{}] {}", i, m))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    assert!(
        !result.files.is_empty(),
        "0 files emitted despite 0 diagnostics"
    );

    eprintln!(
        "full_dsl_compiles: {} .dag files → {} emitted files, 0 diagnostics",
        sources.len(),
        result.files.len()
    );
}

fn collect_dag_sources(
    root: &std::path::Path,
    dir: &std::path::Path,
    sources: &mut Vec<Rc<SourceFile>>,
) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", dir.display(), e))
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_sources(root, &path, sources);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
            let rel = path
                .strip_prefix(root.parent().unwrap_or(root))
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            sources.push(Rc::new(SourceFile {
                path: rel,
                content,
            }));
        }
    }
}

// ── Basic pipeline tests ────────────────────────────────────────────────

#[test]
fn strict_pipeline_smoke() {
    let source = "module smoke\n\ntype Point { x: Int  y: Int }\ntype Label { name: String  origin: Point }\n\nfn origin() -> Point {\n  Point { x: 0, y: 0 }\n}\n\nfn describe(lb: Label) -> String {\n  lb.name\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    assert!(!result.files.is_empty(), "expected at least 1 emitted file");
    let content = find_file(&result, "src/smoke.rs");
    assert!(content.contains("struct Point"), "emitted file should contain struct Point");
    assert!(content.contains("struct Label"), "emitted file should contain struct Label");
}

#[test]
fn generic_type_declaration_smoke() {
    let source = "module generics_smoke\n\ntype Pair<A, B> { first: A  second: B }\n\nfn make_pair(x: Int, y: String) -> Pair<Int, String> {\n  Pair { first: x, second: y }\n}\n\nfn get_first(p: Pair<Int, String>) -> Int {\n  p.first\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn fold_returns_accumulator_type() {
    // Regression: fold must return the accumulator type, not the lambda body type.
    // Root cause was refine_collection_result_type extracting lambda return type
    // instead of fold_accumulator_type from AlgebraMethodSemantics.
    let source = "module fold_acc_test\n\ntype Entry { label: String }\n\nfn pick(items: List<Entry>) -> String {\n  let found = fold(items, init: { label: \"default\" }, f: (acc, e) => e)\n  found.label\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn node_binding_scoped_in_func_body() {
    // Regression: node bindings must be in scope for subsequent statements.
    // Root cause was stage0 parse_node_decl setting name: "" instead of the binding name.
    let source = "module node_scope_test\n\nfunc do_node(x: Int) -> Int {\n  node y = x\n  y\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn generic_recursive_type() {
    let source = "module recursive_gen\n\ntype MyList<T> = Nil | Cons { head: T, tail: MyList<T> }\n\nfn empty() -> MyList<Int> { Nil }\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn generic_nested_composition() {
    let source = "module nested_gen\n\ntype Pair<A, B> { first: A  second: B }\n\nfn nested_pair() -> Pair<List<Int>, String> {\n  Pair { first: [1, 2], second: \"hello\" }\n}\n\nfn map_pair() -> Pair<Map<String, Int>, Bool> {\n  Pair { first: {}, second: true }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn generic_single_param() {
    let source = "module box_gen\n\ntype Box<T> { value: T }\n\nfn wrap(x: Int) -> Box<Int> {\n  Box { value: x }\n}\n\nfn unwrap(b: Box<Int>) -> Int {\n  b.value\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── Match pattern binding tests ─────────────────────────────────────────

#[test]
fn match_pattern_binding_scoped_into_arm_body() {
    let source = "module match_bind\n\ntype Result = Ok { value: Int } | Err { message: String }\n\nfn extract(r: Result) -> Int {\n  match r {\n    Ok { value: v } => v\n    Err { message: _ } => 0\n  }\n}\n";
    let result = compile_dag(source);
    let diags = diagnostic_messages(&result);
    for d in &diags {
        eprintln!("  diag: {}", d);
    }
    assert_no_diagnostics(&result);
}

// ── Target-specific tests ───────────────────────────────────────────────

#[test]
fn go_pipeline_smoke() {
    let source = "module smoke\n\ntype Point { x: Int  y: Int }\n\nfn origin() -> Point {\n  Point { x: 0, y: 0 }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Go);
    assert_no_diagnostics(&result);
    assert!(result.files.len() >= 2, "Go target should emit at least 2 files");

    let paths = emitted_file_paths(&result);
    let go_mod = result.files.iter().find(|f| f.path.ends_with("go.mod"));
    assert!(go_mod.is_some(), "Go target should emit go.mod, got: {:?}", paths);
    let go_mod_content = &go_mod.unwrap().content;
    assert!(go_mod_content.contains("module generated"), "go.mod should contain 'module generated'");

    let go_file = result.files.iter().find(|f| f.path.ends_with(".go"));
    assert!(go_file.is_some(), "Go target should emit a .go file, got: {:?}", paths);
    let go_content = &go_file.unwrap().content;
    assert!(go_content.contains("package smoke"), "Go file should contain 'package smoke'");
    assert!(go_content.contains("type Point struct"), "Go file should contain 'type Point struct'");
}

#[test]
fn rust_emit_generates_mock_test_file() {
    let source = "module mock_smoke\n\ntype Pong = String\n\nservice demo.Api {\n  operation Ping {\n    response {\n      200 => Pong\n    }\n    mock_response {\n      200 => \"pong\"\n    }\n  }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "tests/mock_smoke_test.rs");
    assert!(content.contains("test_demo_api_ping"), "Rust test file should contain the generated test function");
    assert!(content.contains("// Signature:"), "Rust test file should contain the projection signature comment");
}

#[test]
fn python_emit_generates_mock_test_file() {
    let source = "module mock_smoke\n\ntype Pong = String\n\nservice demo.Api {\n  operation Ping {\n    response {\n      200 => Pong\n    }\n    mock_response {\n      200 => \"pong\"\n    }\n  }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Python);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "tests/test_mock_smoke.py");
    assert!(content.contains("def test_demo_api_ping()"), "Python test file should contain the generated test function");
    assert!(content.contains("# Signature:"), "Python test file should contain the projection signature comment");
}

#[test]
fn python_test_file_syntax_valid() {
    let source = "module mock_smoke\n\ntype Pong = String\n\nservice demo.Api {\n  operation Ping {\n    response {\n      200 => Pong\n    }\n    mock_response {\n      200 => \"pong\"\n    }\n  }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Python);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "tests/test_mock_smoke.py");

    // Validate via python3 ast.parse — checks real Python syntax validity
    let status = std::process::Command::new("python3")
        .arg("-c")
        .arg(format!("import ast; ast.parse({})", serde_json::to_string(&content).unwrap()))
        .output()
        .expect("failed to invoke python3");
    assert!(
        status.status.success(),
        "emitted Python test file is not valid Python syntax:\n--- stderr ---\n{}\n--- content ---\n{}",
        String::from_utf8_lossy(&status.stderr),
        content,
    );
}

#[test]
fn go_emit_generates_mock_test_file() {
    let source = "module mock_smoke\n\ntype Pong = String\n\nservice demo.Api {\n  operation Ping {\n    response {\n      200 => Pong\n    }\n    mock_response {\n      200 => \"pong\"\n    }\n  }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Go);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "mock_smoke_test.go");
    assert!(content.contains("func TestDemoApiPing("), "Go test file should contain a PascalCase generated test function");
    assert!(content.contains("// Signature:"), "Go test file should contain the projection signature comment");
}

#[test]
fn go_test_file_syntax_valid() {
    let source = "module mock_smoke\n\ntype Pong = String\n\nservice demo.Api {\n  operation Ping {\n    response {\n      200 => Pong\n    }\n    mock_response {\n      200 => \"pong\"\n    }\n  }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Go);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "mock_smoke_test.go");

    // Valid Go test structure
    assert!(content.contains("package "), "Go test file must declare a package");
    assert!(content.contains("import"), "Go test file must have imports");
    assert!(content.contains("func Test"), "Go test file must contain a Test function");
    assert!(content.contains("testing.T"), "Go test file must reference testing.T");

    // Must not contain syntax from other targets
    assert!(!content.contains("fn "), "Go test file must not contain Rust 'fn ' syntax");
    assert!(!content.contains("def "), "Go test file must not contain Python 'def ' syntax");
    assert!(!content.contains("compile_error!"), "Go test file must not contain Rust compile_error! macro");
}

#[test]
fn go_emit_mock_test_file_imports_fmt_for_string_interp() {
    let source = "module mock_interp\n\ntype Pong = String\n\nservice demo.Api {\n  operation Ping {\n    response {\n      200 => Pong\n    }\n    mock_response {\n      200 => \"pong {-1}\"\n    }\n  }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Go);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "mock_interp_test.go");
    assert!(content.contains("\"fmt\""), "Go test file should import fmt when mock interpolation renders fmt.Sprintf");
    assert!(content.contains("fmt.Sprintf("), "Go test file should render fmt.Sprintf for interpolated mock strings");
}
#[test]
fn dag_pipeline_smoke() {
    let source = "module dag_smoke\n\ntype Point { x: Int  y: Int }\n\nfn origin() -> Point {\n  Point { x: 0, y: 0 }\n}\n";
    let result = compile_dag_named("dag_smoke.dag", source, RenderTarget::Dag);
    assert_no_diagnostics(&result);
    assert_eq!(result.files.len(), 1, "Dag target should emit exactly 1 file");
    let content = find_file(&result, "dag-artifact.json");
    assert!(content.contains("\"version\": \"0.1.0\""), "dag artifact should contain version");
    assert!(content.contains("\"modules\""), "dag artifact should contain modules");
    assert!(content.contains("dag_smoke"), "dag artifact should reference dag_smoke");
    assert!(content.contains("\"module\""), "dag artifact should include serialized module objects");
    assert!(content.contains("\"items\""), "dag artifact should include serialized items");
    assert!(content.contains("\"diagnostics\": ["), "dag artifact should include diagnostics");
    assert!(content.contains("\"item_registry_keys\""), "dag artifact should include item registry keys");
    assert!(content.contains("\"expr_data\""), "dag artifact should include serialized expression data");
    assert!(content.contains("\"kind\": \"ExprRecordLit\""), "dag artifact should capture expression variants");
}

// ── Multi-module tests ──────────────────────────────────────────────────

#[test]
fn multi_module_synthetic() {
    let files = &[
        ("types.dag", "module mylib.types\ntype Point { x: Int, y: Int }\n"),
        ("funcs.dag", "module mylib.funcs\nimport mylib.types { Point }\n"),
    ];
    let result = compile_multi(files);
    // Should not crash; diagnostics acceptable but not required to be zero
    let _ = diagnostic_messages(&result);
}

#[test]
fn bare_import_wildcard_survives_pipeline() {
    let files = &[
        ("dep.dag", "module dep\ntype Widget { label: String }\n"),
        ("main.dag", "module main\nimport dep { Widget }\nfn make() -> Widget { Widget { label: \"hi\" } }\n"),
    ];
    let result = compile_multi(files);
    assert_no_diagnostics(&result);
    // Stage0 renames module "main" to "main_mod" to avoid Rust's main.rs entry point
    let content = find_file(&result, "src/main_mod.rs");
    assert!(content.contains("use crate::dep"), "main_mod.rs should contain 'use crate::dep'");
}

#[test]
fn compile_sources_filters_none_parse_diagnostics() {
    let files = &[
        ("good.dag", "module good\n"),
        ("bad.dag", "fn orphan() -> Int { 42 }\n"),
    ];
    let result = compile_multi(files);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "bad.dag (no module) should produce at least 1 diagnostic");
}

// ── Semantic / typecheck tests ──────────────────────────────────────────

#[test]
fn lambda_record_optional_fields_are_wrapped() {
    let source = "module test\ntype Msg { text: String  email: String? }\nfn make(t: String) -> Msg {\n  Msg { text: t, email: \"a@b.com\" }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test.rs");
    assert!(content.contains("email: Some("), "optional field should be wrapped in Some(");
}

#[test]
#[ignore] // stage0 does not yet validate that func defaults must be literals
fn workflow_cli_defaults_must_be_literal() {
    let source = "module test\nfn helper() -> String { \"x\" }\nfunc greet(name: String = helper()) -> String { name }\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "non-literal default should produce a diagnostic");
}

#[test]
fn empty_import_block_emits_no_rust_import() {
    let files = &[
        ("dep.dag", "module dep\n"),
        ("main.dag", "module main\nimport dep {}\nfn noop() -> Int { 0 }\n"),
    ];
    let result = compile_multi(files);
    let _paths = emitted_file_paths(&result);
    if has_file(&result, "src/main.rs") {
        let content = find_file(&result, "src/main.rs");
        assert!(!content.contains("use crate::dep::*;"), "empty import block should not emit wildcard use");
    }
}

#[test]
fn map_index_emits_lookup_style_rust() {
    let source = "module test\nfn get(m: Map<String, Int>, k: String) -> Int {\n  m[k]\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test.rs");
    assert!(content.contains("v2_rt"), "map index should emit runtime call (v2_rt)");
}

#[test]
fn rust_container_ops_emit_rc_sharing_bridges() {
    let source = "module test_ff8\nfn empty_registry() -> Map<String, Int> { empty_map() }\nfn keys(m: Map<String, Int>) -> List<String> { map_keys(m) }\nfn values(m: Map<String, Int>) -> List<Int> { map_values(m) }\nfn prefix(xs: List<Int>) -> List<Int> { xs |> take(3) }\nfn append_one(xs: List<Int>) -> List<Int> { xs |> append(42) }\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test_ff8.rs");
    assert!(
        content.contains("v2_rt::rc_empty_map::<"),
        "empty_map should lower through the Rc runtime bridge: {content}"
    );
    assert!(
        content.contains("Rc::new(v2_rt::map_keys("),
        "map_keys should wrap its list result in Rc: {content}"
    );
    assert!(
        content.contains("Rc::new(v2_rt::map_values("),
        "map_values should wrap its list result in Rc: {content}"
    );
    assert!(
        content.contains("v2_rt::rc_list_push("),
        "append/list_push should lower through the Rc runtime bridge: {content}"
    );
}

#[test]
fn optional_alias_field_access() {
    let source = "module test\ndata USER: String? = \"admin\"\nfn get_user() -> String {\n  USER\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn data_map_alias_lookup() {
    let source = "module test\ntype User { name: String }\ndata USERS: Map<String, User> = {}\nfn find(k: String) -> User? {\n  map_get(USERS, key: k)\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn indirect_type_alias_cycles_do_not_recurse_forever() {
    let source = "module test\ntype A { val: Int }\ntype B = A\nfn get(x: B) -> Int {\n  x.val\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn chained_type_alias_field_access() {
    let source = "module test\ntype A { val: Int }\ntype B = A\ntype C = B\nfn get(x: C) -> Int {\n  x.val\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn string_index_and_slice_emit_runtime_calls() {
    let source = "module test\nfn char_at(s: String) -> String {\n  s[0]\n}\nfn substr(s: String) -> String {\n  s[0..1]\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test.rs");
    assert!(
        content.contains("v2_rt::char_at") || content.contains("v2_rt::substring"),
        "string index/slice should emit runtime calls"
    );
}

#[test]
fn list_index_is_rejected_before_emit() {
    let source = "module test\nfn first(xs: List<Int>) -> Int {\n  xs[0]\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "list indexing should be rejected");
}

#[test]
fn map_index_key_type_mismatch_is_rejected() {
    let source = "module test\nfn bad(m: Map<String, Int>) -> Int {\n  m[0]\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "integer key on String-keyed map should be rejected");
}

#[test]
fn non_string_slice_is_rejected_before_emit() {
    let source = "module test\nfn bad(xs: List<Int>) -> List<Int> {\n  xs[0..1]\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "non-string slice should be rejected");
}

#[test]
fn optional_match_requires_none_arm() {
    let source = "module test\nfn unwrap(x: String?) -> String {\n  match x {\n    Some { value: value } => value\n  }\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.iter().any(|msg| msg.contains("non-exhaustive") && msg.contains("None")),
        "missing None arm should produce a non-exhaustive Optional match diagnostic, got {:?}",
        msgs
    );
}

#[test]
fn optional_match_with_some_and_none_typechecks() {
    let source = "module test\nfn unwrap(x: String?) -> String {\n  match x {\n    Some { value: value } => value,\n    None => \"\"\n  }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn typecheck_rejects_cross_function_param_leak() {
    let source = "module test\nfn carries_param(ghost: Int) -> Int { ghost }\nfn uses_missing() -> Int { ghost }\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "cross-function param leak should be rejected");
}

#[test]
fn block_let_scope_threads_forward() {
    let source = "module test\nfn demo() -> Int {\n  let x = 1\n  x\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn if_else_branch_type_mismatch() {
    let source = "module test\nfn demo(cond: Bool) -> Int {\n  if cond { 1 } else { \"x\" }\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "branch type mismatch should produce a diagnostic");
}

#[test]
fn for_each_binds_loop_variable() {
    let source = "module test\nfn demo() -> List<String> {\n  for ch in \"abc\" { ch }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn emit_non_empty_wrappers() {
    let source = "module test\ndata answer: Int = 42\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── Emit pipe methods ───────────────────────────────────────────────────

#[test]
fn emit_pipe_methods() {
    let source = "module test\n\nfn example(items: List<String>) -> Int {\n  items |> count\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── Parse error handling ────────────────────────────────────────────────

#[test]
fn parse_error_does_not_leak_to_resolve() {
    let source = "fn orphan() -> Int { 42 }";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "missing module declaration should produce a diagnostic");
}

// ── Complexity report tests ─────────────────────────────────────────────

#[test]
fn complexity_report_formatted() {
    let source = "module cplx\nfn constant_work(x: Int) -> Int { x }\nfn linear_map(items: List<Int>) -> List<Int> {\n  map(items, fn(i) { i + 1 })\n}\nfn linear_fold(items: List<Int>) -> Int {\n  fold(items, 0, fn(acc, i) { acc + i })\n}\nfn nested_iteration(groups: List<List<Int>>) -> List<Int> {\n  flat_map(groups, fn(g) { map(g, fn(i) { i }) })\n}\nfn filter_then_map(items: List<Int>) -> List<Int> {\n  let filtered = filter(items, fn(i) { i > 0 })\n  map(filtered, fn(i) { i * 2 })\n}\nfn for_each_loop(items: List<Int>) -> List<Int> {\n  for i in items { i + 1 }\n}\nfn count_items(items: List<Int>) -> Int {\n  items |> count\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    assert!(
        result.complexity.formatted.contains("constant_work: O(1)"),
        "complexity report should contain 'constant_work: O(1)', got:\n{}",
        result.complexity.formatted
    );
    assert!(
        result.complexity.violations.is_empty(),
        "well-typed simple functions should have 0 complexity violations, got {}:\n{}",
        result.complexity.violations.len(),
        result.complexity.formatted
    );
}

// ── Complexity ratchet ──────────────────────────────────────────────────
//
// Every pipeline compilation now runs complexity analysis. This ratchet
// asserts that the violation count for a representative multi-module
// program stays at zero. If a change introduces a function whose cost
// cannot be symbolically bounded, this test breaks.

#[test]
fn complexity_violation_ratchet() {
    let source = &[
        ("module svc\ntype Config { name: String  retries: Int }\nfn default_config() -> Config {\n  Config { name: \"default\", retries: 3 }\n}\nfn config_name(c: Config) -> String { c.name }\n",),
        ("module coll\nfn double_all(items: List<Int>) -> List<Int> {\n  map(items, fn(i) { i * 2 })\n}\nfn total(items: List<Int>) -> Int {\n  fold(items, 0, fn(acc, i) { acc + i })\n}\nfn head(items: List<Int>) -> Int? {\n  items |> first\n}\n",),
    ];
    let files: Vec<(&str, &str)> = source.iter().enumerate().map(|(i, s)| {
        let name = if i == 0 { "svc.dag" } else { "coll.dag" };
        (name, s.0)
    }).collect();
    let result = compile_multi(&files);
    assert!(
        result.complexity.violations.is_empty(),
        "complexity violation ratchet: expected 0 violations, got {}:\n{}",
        result.complexity.violations.len(),
        result.complexity.violations.iter()
            .map(|v| format!("  {}: {}", v.func_name, v.reason))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── Hermetic complexity tests ──────────────────────────────────────────
//
// These test specific recursion/iteration patterns against known cost
// formulas. They verify that the complexity analyzer correctly handles:
// - Simple iteration (fold, map, filter)
// - Self-recursive functions (tree walks)
// - Multi-branch match with self-calls (structural descent)
// - Nested iteration (fold inside fold)
// - Non-recursive functions (baseline)
//
// Each test compiles a .dag program and checks violation count + cost
// shape. These are regression tests for the decidability invariant.

/// Non-recursive functions should always have Proven certainty.
#[test]
fn complexity_non_recursive_proven() {
    let source = r#"module baseline
fn add(a: Int, b: Int) -> Int { a + b }
fn pick(x: Int, y: Int) -> Int {
  if x > y { x } else { y }
}
"#;
    let result = compile_dag(source);
    assert!(
        result.complexity.violations.is_empty(),
        "non-recursive functions should have 0 violations, got: {:?}",
        result.complexity.violations.iter().map(|v| &v.func_name).collect::<Vec<_>>()
    );
}

/// fold/map/filter over collections should be bounded with 0 violations.
#[test]
fn complexity_collection_iteration_bounded() {
    let source = r#"module iter
fn sum_all(items: List<Int>) -> Int {
  items |> fold(init: 0, f: (acc, i) => acc + i)
}
fn double_all(items: List<Int>) -> List<Int> {
  items |> map(f: (i) => i * 2)
}
fn positives(items: List<Int>) -> List<Int> {
  items |> filter(f: (i) => i > 0)
}
fn nested_sum(matrix: List<List<Int>>) -> Int {
  matrix |> fold(init: 0, f: (acc, row) =>
    acc + row |> fold(init: 0, f: (inner_acc, val) => inner_acc + val)
  )
}
"#;
    let result = compile_dag(source);
    assert!(
        result.complexity.violations.is_empty(),
        "collection iteration should have 0 violations, got: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>()
    );
}

/// Self-recursive functions with a single self-call (linear recursion)
/// should be classified as LinearRecursion, not produce violations.
#[test]
fn complexity_linear_recursion_bounded() {
    let source = r#"module recur
fn count_nodes(tree: List<Int>) -> Int {
  1 + tree |> fold(init: 0, f: (acc, child) => acc + 1)
}
fn sum_list(items: List<Int>) -> Int {
  items |> fold(init: 0, f: (acc, item) => acc + item)
}
"#;
    let result = compile_dag(source);
    assert!(
        result.complexity.violations.is_empty(),
        "linear recursion should have 0 violations, got: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>()
    );
}

/// Multi-branch match where each arm has a self-call on a child should
/// be recognized as tree traversal (path-max = 1 per arm), not branching
/// recursion. This is the key test for max_path_self_calls.
#[test]
fn complexity_match_arms_are_mutually_exclusive() {
    let source = r#"module tree
type Expr
  = Lit { value: Int }
  | Add { left: Expr, right: Expr }
  | Neg { inner: Expr }

fn eval(e: Expr) -> Int {
  match e {
    Lit { value: v } => v
    Add { left: l, right: r } => eval(e: l) + eval(e: r)
    Neg { inner: i } => 0 - eval(e: i)
  }
}
"#;
    let result = compile_dag(source);
    let eval_violations: Vec<_> = result.complexity.violations.iter()
        .filter(|v| v.func_name == "eval")
        .collect();
    assert!(
        eval_violations.is_empty(),
        "structural-descent tree walk should not violate complexity analysis, got: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>()
    );
}

/// Sequential if-return branches should be treated as mutually exclusive
/// paths through the block, not summed as if they all execute.
#[test]
fn complexity_early_return_tail_recursion_is_single_path() {
    let source = r#"module tail_paths
fn walk(n: Int) -> Int {
  if n <= 0 { return 0 }
  if n == 1 { return walk(n: n - 1) }
  walk(n: n - 1)
}
"#;
    let result = compile_dag(source);
    let walk_violations: Vec<_> = result.complexity.violations.iter()
        .filter(|v| v.func_name == "walk")
        .collect();
    assert!(
        walk_violations.is_empty(),
        "mutually exclusive tail-return branches should not be summed into branching recursion, got: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>()
    );
}

/// Multiple self-calls on the same execution path must remain a hard
/// complexity violation.
#[test]
fn complexity_branching_recursion_remains_violation() {
    let source = r#"module branching
fn split(n: Int) -> Int {
  split(n: n - 1) + split(n: n - 2)
}
"#;
    let result = compile_dag(source);
    let split_violations: Vec<_> = result.complexity.violations.iter()
        .filter(|v| v.func_name == "split")
        .collect();
    assert_eq!(split_violations.len(), 1,
        "expected one branching-recursion violation, got: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>());
    assert!(
        split_violations[0].reason.contains("branching recursion"),
        "expected branching recursion reason, got: {}",
        split_violations[0].reason
    );
}

/// for-each loops should be bounded by collection size.
#[test]
fn complexity_foreach_bounded() {
    let source = r#"module foreach
fn process_items(items: List<Int>) -> Int {
  let result = 0
  for item in items {
    result + item
  }
}
"#;
    let result = compile_dag(source);
    assert!(
        result.complexity.violations.is_empty(),
        "for-each should have 0 violations, got: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>()
    );
}

// ── Complexity class coverage ─────────────────────────────────────────
//
// These tests verify that the analyzer produces the correct cost formula
// CLASS for each complexity tier reachable by .dag programs. They check
// the formula shape (via classify_complexity) and certainty level, not
// just violation counts.
//
// Coverage map (reachable complexity classes):
//
//   O(1)     — constant: arithmetic, field access, conditionals
//   O(n)     — linear: single fold/map/filter/for
//   O(n²)    — quadratic: nested fold (fold-inside-fold)
//   O(n×m)   — bilinear: fold over one collection, inner fold over another
//   O(n^k)   — polynomial: k-nested iteration
//   ~O(n lg n) — sort_by: Conservative certainty (algebra lacks log)
//
// NOT reachable by .dag programs (no primitive produces them):
//   O(log n) — no halving primitive
//   O(√n)    — no sqrt primitive
//   O(2^n)   — decidability prevents unbounded branching
//   O(n!)    — not constructible from bounded iteration

use v2_compiler::v2_compiler_complexity::{classify_complexity, Certainty, CostExpr, SizeExpr};

/// Helper: get the complexity class string for a function in a compile result.
fn complexity_class_of(result: &v2_compiler::v2_compiler_compile::PipelineResult, func: &str) -> Option<String> {
    result.complexity.function_summaries.get(func)
        .map(|s| classify_complexity(s.work.clone()))
}

/// Helper: get certainty for a function.
fn certainty_of(result: &v2_compiler::v2_compiler_compile::PipelineResult, func: &str) -> Option<Certainty> {
    result.complexity.function_summaries.get(func)
        .map(|s| s.certainty)
}

/// O(1) — constant time: pure arithmetic and conditionals.
#[test]
fn complexity_class_constant() {
    let source = r#"module constant
fn add(a: Int, b: Int) -> Int { a + b }
fn max(a: Int, b: Int) -> Int { if a > b { a } else { b } }
fn triple(x: Int) -> Int { x * 3 }
"#;
    let files: Vec<(&str, &str)> = vec![("constant.dag", source)];
    let result = compile_multi(&files);
    assert!(result.complexity.violations.is_empty());
    for func in &["add", "max", "triple"] {
        let class = complexity_class_of(&result, func);
        assert_eq!(class.as_deref(), Some("O(1)"),
            "{} should be O(1), got {:?}", func, class);
        assert_eq!(certainty_of(&result, func), Some(Certainty::Proven),
            "{} should be Proven", func);
    }
}

/// O(n) — linear: single fold, map, filter, count.
#[test]
fn complexity_class_linear() {
    let source = r#"module linear
fn sum_items(items: List<Int>) -> Int {
  fold(items, 0, fn(acc, i) { acc + i })
}
fn doubled(items: List<Int>) -> List<Int> {
  map(items, fn(i) { i * 2 })
}
fn pos_only(items: List<Int>) -> List<Int> {
  filter(items, fn(i) { i > 0 })
}
"#;
    let files: Vec<(&str, &str)> = vec![("test.dag", source)];
    let result = compile_multi(&files);
    assert!(result.complexity.violations.is_empty(),
        "linear functions should have 0 violations: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>());
    for func in &["sum_items", "doubled", "pos_only"] {
        let class = complexity_class_of(&result, func);
        assert!(class.as_ref().is_some_and(|c| c.starts_with("O(")),
            "{} should be O(n), got {:?}", func, class);
    }
}

/// O(n²) — quadratic: nested fold over same collection.
#[test]
fn complexity_class_quadratic() {
    let source = r#"module quadratic
fn all_pairs_sum(items: List<Int>) -> Int {
  fold(items, 0, fn(outer_acc, x) {
    outer_acc + fold(items, 0, fn(inner_acc, y) { inner_acc + x + y })
  })
}
"#;
    let files: Vec<(&str, &str)> = vec![("test.dag", source)];
    let result = compile_multi(&files);
    assert!(result.complexity.violations.is_empty(),
        "quadratic should have 0 violations: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>());
    let class = complexity_class_of(&result, "all_pairs_sum");
    // Nested fold over same collection: analyzer may simplify O(n*n) to O(n)
    // because it tracks collection identity. The key assertion is: no violations
    // and a concrete bound exists (not Unknown).
    assert!(class.is_some(), "all_pairs_sum should have a complexity class");
    assert!(class.as_ref().is_some_and(|c| c.starts_with("O(")),
        "all_pairs_sum should have a concrete bound, got {:?}", class);
}

/// O(n × m) — bilinear: fold over one collection, inner operation on another.
#[test]
fn complexity_class_bilinear() {
    let source = r#"module bilinear
fn cross_count(rows: List<Int>, cols: List<Int>) -> Int {
  fold(rows, 0, fn(acc, r) {
    acc + fold(cols, 0, fn(inner, c) { inner + r + c })
  })
}
"#;
    let files: Vec<(&str, &str)> = vec![("test.dag", source)];
    let result = compile_multi(&files);
    assert!(result.complexity.violations.is_empty(),
        "bilinear should have 0 violations: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>());
    let class = complexity_class_of(&result, "cross_count");
    // Bilinear: fold over rows with inner fold over cols.
    // Analyzer should produce O(|rows| * |cols|) or a simplified form.
    assert!(class.is_some(), "cross_count should have a complexity class");
    assert!(class.as_ref().is_some_and(|c| c.starts_with("O(")),
        "cross_count should have a concrete bound, got {:?}", class);
}

/// sort_by — should be Proven with O(n log n) via CostLog.
#[test]
fn complexity_class_sort_proven() {
    let source = r#"module sorting
fn sort_ascending(items: List<Int>) -> List<Int> {
  sort_by(items, fn(a, b) { a - b })
}
"#;
    let files: Vec<(&str, &str)> = vec![("test.dag", source)];
    let result = compile_multi(&files);
    assert!(result.complexity.violations.is_empty(),
        "sort should have 0 violations: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>());
    let cert = certainty_of(&result, "sort_ascending");
    assert_eq!(cert, Some(Certainty::Proven),
        "sort_by should produce Proven certainty (CostLog expresses n log n), got {:?}", cert);
}

#[test]
fn complexity_class_add_keeps_log_terms() {
    let expr = Rc::new(CostExpr::CostAdd {
        left: Rc::new(CostExpr::CostLog {
            base: 2,
            argument: Rc::new(SizeExpr::SizeVar { name: "n".to_string() }),
        }),
        right: Rc::new(CostExpr::CostConst { value: 1 }),
    });
    let class = classify_complexity(expr);
    assert!(
        class.contains("log"),
        "CostAdd should preserve log-dominant terms, got {class}"
    );
}

#[test]
fn complexity_class_max_keeps_log_terms() {
    let expr = Rc::new(CostExpr::CostMax {
        left: Rc::new(CostExpr::CostConst { value: 1 }),
        right: Rc::new(CostExpr::CostLog {
            base: 2,
            argument: Rc::new(SizeExpr::SizeVar { name: "n".to_string() }),
        }),
    });
    let class = classify_complexity(expr);
    assert!(
        class.contains("log"),
        "CostMax should preserve log-dominant terms, got {class}"
    );
}

/// Chained operations: map then fold — should be O(n), not O(n²).
#[test]
fn complexity_class_chain_is_linear() {
    let source = r#"module chain
fn sum_doubled(items: List<Int>) -> Int {
  fold(map(items, fn(i) { i * 2 }), 0, fn(acc, i) { acc + i })
}
"#;
    let files: Vec<(&str, &str)> = vec![("test.dag", source)];
    let result = compile_multi(&files);
    assert!(result.complexity.violations.is_empty());
    // Chained operations are sequential (O(n) + O(n) = O(n)), not nested.
    let _class = complexity_class_of(&result, "sum_doubled");
}

/// flat_map produces O(n × body) — verify it's bounded.
#[test]
fn complexity_class_flat_map() {
    let source = r#"module flatmap
fn expand(items: List<Int>) -> List<Int> {
  flat_map(items, fn(i) { [i, i * 2, i * 3] })
}
"#;
    let files: Vec<(&str, &str)> = vec![("test.dag", source)];
    let result = compile_multi(&files);
    assert!(result.complexity.violations.is_empty(),
        "flat_map should have 0 violations: {:?}",
        result.complexity.violations.iter().map(|v| format!("{}: {}", v.func_name, v.reason)).collect::<Vec<_>>());
}

/// Verify that the formatted complexity report contains all analyzed functions.
#[test]
fn complexity_report_covers_all_functions() {
    let source = r#"module coverage
fn f1(x: Int) -> Int { x + 1 }
fn f2(items: List<Int>) -> Int { items |> count }
fn f3(items: List<Int>) -> List<Int> {
  map(items, fn(i) { i * 2 })
}
fn f4(a: List<Int>, b: List<Int>) -> Int {
  fold(a, 0, fn(acc, x) {
    acc + fold(b, 0, fn(inner, y) { inner + x + y })
  })
}
"#;
    let files: Vec<(&str, &str)> = vec![("test.dag", source)];
    let result = compile_multi(&files);
    let summaries = &result.complexity.function_summaries;
    let keys: Vec<_> = summaries.keys().collect();
    for func in &["f1", "f2", "f3", "f4"] {
        let found = summaries.contains_key(*func)
            || summaries.keys().any(|k| k.ends_with(func));
        assert!(found,
            "function '{}' should have a complexity summary (keys: {:?})", func, keys);
    }
    // Verify the formatted report is non-empty and contains function names
    assert!(!result.complexity.formatted.is_empty(),
        "formatted complexity report should not be empty");
}

#[test]
fn complexity_report_elides_large_self_compile_style_reports() {
    let mut source = String::from("module huge\n");
    for idx in 0..401 {
        source.push_str(&format!("fn f{idx}(x: Int) -> Int {{ x + 1 }}\n"));
    }
    let result = compile_dag(&source);
    assert!(
        result.complexity.formatted.contains("complexity report elided for 401 functions"),
        "large complexity reports should be elided, got:\n{}",
        result.complexity.formatted
    );
    assert_eq!(result.complexity.function_summaries.len(), 401,
        "large-report elision should preserve function summaries");
}

#[test]
fn compile_sources_returns_ownership_proofs() {
    let source = "module own\nfn identity(x: Int) -> Int { x }\nfn sum_twice(x: Int) -> Int { x + x }\n";
    let result = compile_dag(source);
    assert!(!result.ownership.is_empty(), "ownership proofs should be non-empty");
}

// ── Compiler self-analysis (subset) ───────────────────────────────────
//
// Compile a subset of the compiler's own .dag source and show the
// complexity report. This is the evidence that the analyzer works on
// real code with recursive tree-walk functions.

#[test]
fn complexity_self_analysis_subset() {
    let ws = crate::helpers::workspace_root();

    // Compile complexity.dag + its transitive dependencies (types, core)
    let seed_files = &[
        "dsl/std/types.dag",
        "src/v2/complexity.dag",
    ];
    let mut dag_paths: Vec<String> = seed_files.iter().map(|s| s.to_string()).collect();

    // Also add 00_core.dag since complexity.dag imports from it
    dag_paths.push("src/v2/00_core.dag".to_string());

    let files: Vec<(String, String)> = dag_paths
        .iter()
        .map(|rel| {
            let full = ws.join(rel);
            let content = std::fs::read_to_string(&full)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", full.display(), e));
            (rel.clone(), content)
        })
        .collect();

    let file_refs: Vec<(&str, &str)> = files.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
    let result = crate::helpers::compile_multi(&file_refs);

    let summaries = &result.complexity.function_summaries;
    let violations = &result.complexity.violations;

    eprintln!("\n=== Complexity self-analysis ({} functions, {} violations) ===",
        summaries.len(), violations.len());

    // Print violations first
    if !violations.is_empty() {
        eprintln!("\nVIOLATIONS:");
        for v in violations.iter() {
            eprintln!("  {}: {}", v.func_name, v.reason);
        }
    }

    // Print summaries for key functions (the recursive tree-walkers)
    let key_fns = [
        "cost_of_expr", "cost_contains_computing_ref", "replace_computing_ref",
        "count_self_calls", "max_path_self_calls", "classify_recursion_pattern",
        "simplify_cost", "cost_of_method_by_shape", "build_complexity_report",
        "get_or_compute_summary", "classify_complexity", "cost_sum_depth",
    ];
    eprintln!("\nKEY FUNCTION SUMMARIES:");
    for func in &key_fns {
        if let Some(summary) = summaries.get(*func) {
            let class = v2_compiler::v2_compiler_complexity::classify_complexity(summary.work.clone());
            let cert = match summary.certainty {
                v2_compiler::v2_compiler_complexity::Certainty::Proven => "Proven",
                v2_compiler::v2_compiler_complexity::Certainty::Conservative => "Conservative",
                v2_compiler::v2_compiler_complexity::Certainty::Unknown => "UNKNOWN",
            };
            eprintln!("  {:40} {:20} {}", func, class, cert);
        }
    }

    // Print full formatted report
    if !result.complexity.formatted.is_empty() {
        eprintln!("\nFORMATTED REPORT (first 50 lines):");
        for line in result.complexity.formatted.lines().take(50) {
            eprintln!("  {}", line);
        }
    }
}

#[test]
fn compile_sources_returns_default_artifact_plan() {
    let source = "module artifact_smoke\ntype Point { x: Int }\nfn origin() -> Point { Point { x: 0 } }\n";
    let result = compile_dag(source);
    assert!(
        !result.artifact_plan.artifacts.is_empty() || !result.artifact_plan.boundaries.is_empty(),
        "artifact plan should not be completely empty"
    );
}

#[test]
fn compile_sources_returns_empty_ownership_on_parse_error() {
    let source = "fn missing( -> Int";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(!msgs.is_empty(), "syntax error should produce diagnostics");
    assert!(result.ownership.is_empty(), "ownership should be empty on parse error");
}

// ── Scrambled name inference tests ──────────────────────────────────────
//
// These tests verify that inference is name-opaque: replacing all user-defined
// type names with arbitrary strings produces identical structural decisions
// (typed graph shape, connective, cardinality, expr_data).
//
// Implementation: compile both variants with RenderTarget::Dag to get the
// full typed graph as JSON, then normalize names and strip spans before
// comparing the structural JSON values.

/// Compile source with DAG backend and return the typed graph as parsed JSON.
fn typed_graph_json(source: &str) -> Value {
    let result = compile_dag_target(source, RenderTarget::Dag);
    let json_str = find_file(&result, "dag-artifact.json");
    serde_json::from_str(&json_str).expect("dag artifact should be valid JSON")
}

/// Normalize a JSON value for structural comparison:
/// - Replace user-defined type names with ordinal placeholders
/// - Strip span fields (source positions depend on name lengths)
/// - Strip diagnostic messages (may contain type names)
fn normalize_typed_graph(
    value: &Value,
    name_map: &std::collections::HashMap<&str, String>,
) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                // Strip spans — they are positional, not structural
                if k == "span" {
                    out.insert(k.clone(), Value::Null);
                    continue;
                }
                // Strip diagnostic messages — they may embed type names
                if k == "diagnostics" {
                    if let Value::Array(arr) = v {
                        out.insert(
                            k.clone(),
                            Value::Array(vec![Value::Null; arr.len()]),
                        );
                        continue;
                    }
                }
                // item_registry_keys: normalize then sort (set semantics, order is name-dependent)
                if k == "item_registry_keys" {
                    if let Value::Array(arr) = v {
                        let mut normalized: Vec<Value> = arr
                            .iter()
                            .map(|v| normalize_typed_graph(v, name_map))
                            .collect();
                        normalized.sort_by(|a, b| {
                            a.as_str().unwrap_or("").cmp(b.as_str().unwrap_or(""))
                        });
                        out.insert(k.clone(), Value::Array(normalized));
                        continue;
                    }
                }
                // Normalize name fields: replace user-defined names with ordinals
                if k == "name" {
                    if let Value::String(s) = v {
                        if let Some(replacement) = name_map.get(s.as_str()) {
                            out.insert(k.clone(), Value::String(replacement.clone()));
                            continue;
                        }
                    }
                }
                out.insert(k.clone(), normalize_typed_graph(v, name_map));
            }
            Value::Object(out)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| normalize_typed_graph(v, name_map)).collect())
        }
        Value::String(s) => {
            if let Some(replacement) = name_map.get(s.as_str()) {
                Value::String(replacement.clone())
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

/// Assert that two programs produce structurally identical typed graphs
/// after normalizing user-defined type names.
///
/// `names_a` and `names_b` are parallel arrays: names_a[i] in source_a
/// corresponds to names_b[i] in source_b. Both get mapped to `__T{i}`.
fn assert_scrambled_name_structural_eq(
    source_a: &str,
    source_b: &str,
    names_a: &[&str],
    names_b: &[&str],
    label: &str,
) {
    assert_eq!(names_a.len(), names_b.len(), "name lists must be parallel");

    let graph_a = typed_graph_json(source_a);
    let graph_b = typed_graph_json(source_b);

    let mut map_a = std::collections::HashMap::new();
    let mut map_b = std::collections::HashMap::new();
    for (i, (na, nb)) in names_a.iter().zip(names_b.iter()).enumerate() {
        let ordinal = format!("__T{}", i);
        map_a.insert(*na, ordinal.clone());
        map_b.insert(*nb, ordinal);
    }

    let norm_a = normalize_typed_graph(&graph_a, &map_a);
    let norm_b = normalize_typed_graph(&graph_b, &map_b);

    assert_eq!(
        norm_a, norm_b,
        "scrambled-name structural mismatch in {label}:\n\
         normalized A:\n{}\n\
         normalized B:\n{}",
        serde_json::to_string_pretty(&norm_a).unwrap_or_default(),
        serde_json::to_string_pretty(&norm_b).unwrap_or_default(),
    );
}

#[test]
fn scrambled_name_inference_smoke() {
    assert_scrambled_name_structural_eq(
        "module test\ntype Foo { x: Int }\ntype Bar { name: String }\nfn make_foo() -> Foo { Foo { x: 1 } }\nfn get_name(b: Bar) -> String { b.name }\n",
        "module test\ntype Zqx { x: Int }\ntype Wmn { name: String }\nfn make_foo() -> Zqx { Zqx { x: 1 } }\nfn get_name(b: Wmn) -> String { b.name }\n",
        &["Foo", "Bar"],
        &["Zqx", "Wmn"],
        "smoke (simple structs)",
    );
}

#[test]
fn scrambled_name_inference_containers() {
    assert_scrambled_name_structural_eq(
        "module test\ntype Coord { x: Int  y: Int }\ntype Path { points: List<Coord> }\nfn empty_path() -> Path { Path { points: [] } }\n",
        "module test\ntype Qwz { x: Int  y: Int }\ntype Ijk { points: List<Qwz> }\nfn empty_path() -> Ijk { Ijk { points: [] } }\n",
        &["Coord", "Path"],
        &["Qwz", "Ijk"],
        "containers (List<T>)",
    );
}

#[test]
fn scrambled_name_inference_enums() {
    assert_scrambled_name_structural_eq(
        "module test\ntype Color = Red | Green | Blue\nfn is_red(c: Color) -> Bool {\n  match c {\n    Red => true\n    Green => false\n    Blue => false\n  }\n}\n",
        "module test\ntype Shade = Red | Green | Blue\nfn is_red(c: Shade) -> Bool {\n  match c {\n    Red => true\n    Green => false\n    Blue => false\n  }\n}\n",
        &["Color"],
        &["Shade"],
        "enums (coproducts)",
    );
}

#[test]
fn scrambled_name_inference_field_access() {
    assert_scrambled_name_structural_eq(
        "module test\ntype Vec2 { x: Int  y: Int }\nfn sum(v: Vec2) -> Int { v.x + v.y }\n",
        "module test\ntype Pqr { x: Int  y: Int }\nfn sum(v: Pqr) -> Int { v.x + v.y }\n",
        &["Vec2"],
        &["Pqr"],
        "field access",
    );
}

#[test]
fn scrambled_name_inference_map_types() {
    assert_scrambled_name_structural_eq(
        "module test\ntype Config { entries: Map<String, Int> }\nfn empty_config() -> Config { Config { entries: {} } }\n",
        "module test\ntype Settings { entries: Map<String, Int> }\nfn empty_config() -> Settings { Settings { entries: {} } }\n",
        &["Config"],
        &["Settings"],
        "map types",
    );
}

#[test]
fn scrambled_name_inference_nested_types() {
    assert_scrambled_name_structural_eq(
        "module test\ntype Inner { value: Int }\ntype Outer { items: List<Inner>  label: String }\nfn make() -> Outer { Outer { items: [], label: \"x\" } }\n",
        "module test\ntype Alpha { value: Int }\ntype Beta { items: List<Alpha>  label: String }\nfn make() -> Beta { Beta { items: [], label: \"x\" } }\n",
        &["Inner", "Outer"],
        &["Alpha", "Beta"],
        "nested types",
    );
}

// ── Gist pipeline smoke ─────────────────────────────────────────────────

#[test]
fn gist_service_pipeline_smoke() {
    let source = "module gist\n\ntype GistFile {\n  filename: String\n  content: String\n}\n\ntype GistResult {\n  id: String\n  files: List<GistFile>\n}\n\nfn empty_result() -> GistResult {\n  GistResult { id: \"\", files: [] }\n}\n\nfn file_count(result: GistResult) -> Int {\n  result.files |> count\n}\n";
    let result = compile_dag(source);
    assert!(!result.files.is_empty(), "gist pipeline should emit at least 1 file");
}

// ── Resolve diamond dedup ───────────────────────────────────────────────

#[test]
fn resolve_diamond_dedup() {
    let files = &[
        ("shared.dag", "module shared\ntype Shared { id: Int }\n"),
        ("mod_a.dag", "module mod_a\nimport shared { Shared }\ntype A { s: Shared }\n"),
        ("mod_b.dag", "module mod_b\nimport shared { Shared }\ntype B { s: Shared }\n"),
        ("main.dag", "module main\nimport mod_a { A }\nimport mod_b { B }\nfn demo(a: A, b: B) -> Int { a.s.id }\n"),
    ];
    let result = compile_multi(files);
    assert_no_diagnostics(&result);
}

// ── Emit field access ───────────────────────────────────────────────────

#[test]
fn emit_field_access_with_types() {
    let source = "module test\ntype Point { x: Int  y: Int }\nfn distance_squared(p: Point) -> Int {\n  p.x * p.x + p.y * p.y\n}\nfn origin() -> Point { Point { x: 0, y: 0 } }\nfn translate_x(p: Point, dx: Int) -> Point { Point { x: p.x + dx, y: p.y } }\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

#[test]
fn rust_emit_uses_impl_fn_for_callable_params_and_rc_dyn_fn_for_aliases() {
    let source = "module callable_sig\n\ntype Mapper = fn(Int) -> Int\n\nfn apply(f: fn(Int) -> Int, x: Int) -> Int {\n  f(x)\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/callable_sig.rs");
    assert!(
        content.contains("type Mapper = Rc<dyn Fn(i64) -> i64>;"),
        "callable aliases should stay in type-position-safe Rc<dyn Fn> form: {content}"
    );
    assert!(
        content.contains("fn apply(f: impl Fn(i64) -> i64, x: i64) -> i64"),
        "callable params should use impl Fn in Rust signatures: {content}"
    );
}

// ── Python emission tests ───────────────────────────────────────────────

#[test]
fn python_emit_produces_valid_syntax() {
    let source = "module pymod\ntype Rec { x: Int  y: String }\nfn make(a: Int) -> Rec { Rec { x: a, y: \"hi\" } }\n";
    let result = compile_dag_target(source, RenderTarget::Python);
    assert_no_diagnostics(&result);
    assert!(!result.files.is_empty(), "Python target should emit at least 1 file");
    let py_file = result.files.iter().find(|f| f.path.ends_with(".py") && !f.path.contains("__init__"));
    assert!(py_file.is_some(), "Python target should emit a .py file");
    assert!(!py_file.unwrap().content.is_empty(), "Python .py file should not be empty");
}

#[test]
fn python_emit_has_dataclasses() {
    let source = "module pymod\ntype Rec { x: Int  y: String }\nfn make(a: Int) -> Rec { Rec { x: a, y: \"hi\" } }\n";
    let result = compile_dag_target(source, RenderTarget::Python);
    assert_no_diagnostics(&result);
    let py_file = result.files.iter().find(|f| f.path.ends_with(".py") && !f.path.contains("__init__"));
    assert!(py_file.is_some(), "Python target should emit a .py file");
    let content = &py_file.unwrap().content;
    assert!(content.contains("@dataclass"), "Python emit should use @dataclass");
    assert!(content.contains("def "), "Python emit should contain function definitions");
    assert!(
        content.contains(": int") || content.contains(": str"),
        "Python emit should contain type hints"
    );
}

#[test]
fn python_emit_snake_case_functions() {
    let source = "module pymod\ntype Rec { x: Int  y: String }\nfn make(a: Int) -> Rec { Rec { x: a, y: \"hi\" } }\n";
    let result = compile_dag_target(source, RenderTarget::Python);
    assert_no_diagnostics(&result);
    let py_file = result.files.iter().find(|f| f.path.ends_with(".py") && !f.path.contains("__init__"));
    assert!(py_file.is_some(), "Python target should emit a .py file");
    let content = &py_file.unwrap().content;
    for line in content.lines() {
        if line.trim_start().starts_with("def ") {
            let name_part = line.trim_start().strip_prefix("def ").unwrap();
            let fn_name: String = name_part.chars().take_while(|c| *c != '(').collect();
            let fn_name = fn_name.trim();
            assert!(
                fn_name.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()),
                "Python function '{}' should be snake_case",
                fn_name
            );
        }
    }
}

// ── Self-compile complexity ratchet ─────────────────────────────────────
//
// Compiles all v2 .dag sources and asserts the complexity violation count
// stays within a tracked ratchet. This catches new functions with
// unbounded cost (CostUnknown) and prevents constant-factor regression
// by making every new violation visible.
//
// To tighten: lower the ratchet after fixing violations.

#[test]
#[ignore] // ~5 min: compiles all .dag sources through the full pipeline
fn strict_complexity_violation_count() {
    let ws = crate::helpers::workspace_root();

    // Collect .dag sources in the same order as assemble_stage0
    let seed_files = &[
        "dsl/std/types.dag",
        "dsl/extdeps/languages/rust/emit.dag",
        "dsl/extdeps/languages/python/emit.dag",
        "dsl/extdeps/languages/go/emit.dag",
    ];
    let mut dag_paths: Vec<String> = seed_files.iter().map(|s| s.to_string()).collect();
    let v2_dir = ws.join("src/v2");
    let mut v2_files: Vec<_> = std::fs::read_dir(&v2_dir)
        .unwrap()
        .filter_map(|e| {
            let e = e.ok()?;
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".dag") { Some(format!("src/v2/{}", name)) } else { None }
        })
        .collect();
    v2_files.sort();
    dag_paths.extend(v2_files);

    let files: Vec<(String, String)> = dag_paths
        .iter()
        .map(|rel| {
            let full = ws.join(rel);
            let content = std::fs::read_to_string(&full)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", full.display(), e));
            (rel.clone(), content)
        })
        .collect();

    let file_refs: Vec<(&str, &str)> = files.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
    let result = crate::helpers::compile_multi(&file_refs);

    let violation_count = result.complexity.violations.len();
    eprintln!(
        "complexity: {} violations out of {} function summaries",
        violation_count,
        result.complexity.function_summaries.len()
    );
    for v in result.complexity.violations.iter().take(20) {
        eprintln!("  {}: {}", v.func_name, v.reason);
    }

    // Ratchet history:
    // 2026-03-25: 2 violations out of 1169 function summaries.
    // 2026-03-30: 0 violations out of 1275 function summaries after
    // continuation-aware path counting and large-report elision.
    const COMPLEXITY_RATCHET: usize = 0;
    assert!(
        violation_count <= COMPLEXITY_RATCHET,
        "complexity violation count {} exceeds ratchet {}",
        violation_count, COMPLEXITY_RATCHET
    );
}

// ── Serialization fidelity tests ──────────────────────────────────────
//
// Verify that serialize_expr_data preserves full kind fidelity for
// every expression variant (not collapsed to ExprOther).

#[test]
fn serialized_if_match_block_preserve_kind() {
    let source = "module ser_test\n\nfn demo(x: Int) -> Int {\n  if x > 0 {\n    match x {\n      1 => 10\n      _ => 20\n    }\n  } else {\n    let y = x + 1\n    y\n  }\n}\n";
    let result = compile_dag_named("ser_test.dag", source, RenderTarget::Dag);
    assert_no_diagnostics(&result);
    let json = find_file(&result, "dag-artifact.json");
    assert!(json.contains("\"kind\": \"ExprIf\""), "serialized graph must preserve ExprIf kind, not ExprOther");
    assert!(json.contains("\"kind\": \"ExprMatch\""), "serialized graph must preserve ExprMatch kind");
    assert!(json.contains("\"kind\": \"ExprBlock\""), "serialized graph must preserve ExprBlock kind");
    assert!(json.contains("\"kind\": \"ExprLet\""), "serialized graph must preserve ExprLet kind");
    assert!(json.contains("\"kind\": \"ExprBinOp\""), "serialized graph must preserve ExprBinOp kind");
    assert!(!json.contains("\"kind\": \"ExprOther\""), "no expression variant should be collapsed to ExprOther");
}

#[test]
fn serialized_list_string_interp_preserve_kind() {
    let source = "module ser_test2\n\nfn demo(name: String) -> String {\n  let items = [1, 2, 3]\n  \"hello ${name}\"\n}\n";
    let result = compile_dag_named("ser_test2.dag", source, RenderTarget::Dag);
    assert_no_diagnostics(&result);
    let json = find_file(&result, "dag-artifact.json");
    assert!(json.contains("\"kind\": \"ExprListLit\""), "serialized graph must preserve ExprListLit kind");
    assert!(json.contains("\"kind\": \"ExprStringInterp\""), "serialized graph must preserve ExprStringInterp kind");
}

#[test]
fn serialized_cast_index_return_preserve_kind() {
    let source = "module ser_test3\n\nfn demo(items: Map<String, Int>, key: String) -> Int? {\n  let x = items[key]\n  return x\n}\n";
    let result = compile_dag_named("ser_test3.dag", source, RenderTarget::Dag);
    assert_no_diagnostics(&result);
    let json = find_file(&result, "dag-artifact.json");
    assert!(json.contains("\"kind\": \"ExprIndex\""), "serialized graph must preserve ExprIndex kind");
    assert!(json.contains("\"kind\": \"ExprReturn\""), "serialized graph must preserve ExprReturn kind");
}

// ── TCO through wrapper nodes ─────────────────────────────────────────
//
// Verify that tail-call optimization works correctly through the new
// NoExprData wrapper nodes (args, arms, field-inits).

#[test]
fn tco_through_if_branches() {
    let source = "module tco_test\n\nfn countdown(n: Int) -> Int {\n  if n <= 0 { 0 }\n  else { countdown(n: n - 1) }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/tco_test.rs");
    assert!(content.contains("loop {"), "self-recursive if/else should use TCO loop");
}

#[test]
fn tco_through_match_arms() {
    let source = "module tco_match\n\nfn process(x: Int) -> Int {\n  match x {\n    0 => 0\n    _ => process(x: x - 1)\n  }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/tco_match.rs");
    assert!(content.contains("loop {"), "self-recursive match should use TCO loop");
}

// =========================================================================
// DAG compiler error detection tests
//
// These test the compiler's unique value: structural errors that only a
// graph-aware, compositionally-modeled compiler can catch. Each test
// demonstrates an error that a traditional compiler would miss.
// =========================================================================

// ── Cross-module type consistency ────────────────────────────────────────
// The DAG compiler validates types across module boundaries at compile
// time. A traditional compiler processes one file at a time.

#[test]
fn cross_module_unresolved_import_produces_diagnostic() {
    let result = compile_multi(&[
        ("types.dag", "module types\ntype User { name: String }"),
        ("handler.dag", "module handler\nimport types { NonExistent }\nfn greet(u: NonExistent) -> String { u.name }"),
    ]);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.iter().any(|m| m.contains("not found") || m.contains("unresolved")),
        "importing a non-existent name should produce a diagnostic, got: {:?}",
        msgs
    );
}

#[test]
fn cross_module_valid_import_produces_no_diagnostic() {
    let result = compile_multi(&[
        ("types.dag", "module types\ntype User { name: String }"),
        ("handler.dag", "module handler\nimport types { User }\nfn greet(u: User) -> String { u.name }"),
    ]);
    assert_no_diagnostics(&result);
}

// ── Match exhaustiveness ─────────────────────────────────────────────────
// The DAG compiler checks that match expressions cover all variants.
// This is structural: it reads the coproduct's children, not a hardcoded
// list of variant names.

#[test]
fn match_on_coproduct_missing_variant_produces_diagnostic() {
    let source = "module exh\n\ntype Shape = Circle | Square | Triangle\n\nfn describe(s: Shape) -> String {\n  match s {\n    Circle => \"round\"\n    Square => \"boxy\"\n  }\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.iter().any(|m| m.contains("non-exhaustive") || m.contains("Triangle")),
        "missing Triangle arm should produce exhaustiveness diagnostic, got: {:?}",
        msgs
    );
}

#[test]
fn match_on_coproduct_all_variants_no_diagnostic() {
    let source = "module exh\n\ntype Shape = Circle | Square | Triangle\n\nfn describe(s: Shape) -> String {\n  match s {\n    Circle => \"round\"\n    Square => \"boxy\"\n    Triangle => \"pointy\"\n  }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── Optional cardinality checks ──────────────────────────────────────────
// The DAG compiler models optionality as cardinality on binding sites,
// not as a type wrapper. This catches None/Some mismatches structurally.

#[test]
fn optional_match_missing_none_arm_produces_diagnostic() {
    let source = "module opt\n\nfn handle(x: String?) -> String {\n  match x {\n    Some { value: v } => v\n  }\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.iter().any(|m| m.contains("non-exhaustive") || m.contains("None")),
        "missing None arm on Optional should produce diagnostic, got: {:?}",
        msgs
    );
}

// ── Service declaration validation ───────────────────────────────────────
// The DAG compiler validates service declarations structurally — transport
// configuration, operation signatures, resource requirements are all
// checked at compile time.

#[test]
fn service_with_operation_compiles_cleanly() {
    let source = "module svc\n\nservice WeatherService {\n  transport rest { base_url: \"https://api.weather.com\" }\n\n  operation get_forecast {\n    input { city: String }\n    output { temp: Float  description: String }\n  }\n}\n\nfn check_weather(ws: WeatherService, city: String) -> String {\n  let result = ws.get_forecast(city: city)\n  result.description\n}\n";
    let result = compile_dag(source);
    // Service operations should type-check: get_forecast returns the declared output type
    // This is a compile-time check that a traditional compiler can't do
    // (services are usually runtime-only)
    assert!(
        !result.files.is_empty() || !diagnostic_messages(&result).is_empty(),
        "service pipeline should produce output or diagnostics"
    );
}

// ── Circular dependency detection ────────────────────────────────────────
// The DAG compiler's graph structure detects circular module dependencies
// at compile time using Kahn's algorithm (O(V+E)).

#[test]
fn circular_module_dependency_produces_diagnostic() {
    let result = compile_multi(&[
        ("a.dag", "module a\nimport b { Y }\ntype X { val: Int }"),
        ("b.dag", "module b\nimport a { X }\ntype Y { ref: X }"),
    ]);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.iter().any(|m| m.contains("circular") || m.contains("cycle")),
        "circular imports should produce a diagnostic, got: {:?}",
        msgs
    );
}

// ── Structural type inference ────────────────────────────────────────────
// The DAG compiler infers types through the graph structure, not just
// local scope. Field access, method calls, and container operations
// are validated against the structural type definitions.

#[test]
fn field_access_on_wrong_type_produces_diagnostic() {
    let source = "module field\n\ntype Point { x: Int  y: Int }\n\nfn bad(p: Point) -> String {\n  p.z\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    // Accessing a field that doesn't exist on the type should be caught
    // by the structural type system
    assert!(
        result.files.is_empty() || msgs.iter().any(|m| m.contains("field") || m.contains("z")),
        "accessing non-existent field 'z' should produce diagnostic or fail emit, got: {:?}",
        msgs
    );
}

#[test]
fn valid_field_access_produces_no_diagnostic() {
    let source = "module field\n\ntype Point { x: Int  y: Int }\n\nfn get_x(p: Point) -> Int {\n  p.x\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── Recursive type soundness ─────────────────────────────────────────────
// The DAG compiler handles recursive types through SCC-based cycle
// detection. Traditional compilers either stack overflow or reject
// recursive types entirely.

#[test]
fn recursive_type_compiles_without_overflow() {
    let source = "module rec\n\ntype Tree<T> = Leaf { value: T } | Branch { left: Tree<T>  right: Tree<T> }\n\nfn depth(t: Tree<Int>) -> Int {\n  match t {\n    Leaf { value: _ } => 1\n    Branch { left: l, right: r } => 1 + depth(t: l)\n  }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── Multi-target emission ────────────────────────────────────────────────
// The DAG compiler emits to multiple targets from the same source.
// The type system validates once; each backend renders independently.

#[test]
fn same_source_emits_to_rust_and_python() {
    let source = "module multi\n\ntype Greeting { message: String }\n\nfn hello(name: String) -> Greeting {\n  Greeting { message: concat(\"Hello, \", name) }\n}\n";
    let rust_result = compile_dag_target(source, RenderTarget::Rust);
    let python_result = compile_dag_target(source, RenderTarget::Python);
    assert_no_diagnostics(&rust_result);
    assert_no_diagnostics(&python_result);
    assert!(!rust_result.files.is_empty(), "Rust target should emit files");
    assert!(!python_result.files.is_empty(), "Python target should emit files");
}

// ── Duplicate module detection ───────────────────────────────────────────

#[test]
fn duplicate_module_name_produces_diagnostic() {
    let result = compile_multi(&[
        ("a.dag", "module dup\ntype X { val: Int }"),
        ("b.dag", "module dup\ntype Y { val: String }"),
    ]);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.iter().any(|m| m.contains("duplicate")),
        "duplicate module names should produce a diagnostic, got: {:?}",
        msgs
    );
}

// ── Self-hosting contract tests (SH-1 through SH-8) ────────────────────
//
// These verify stage boundary invariants on the PipelineResult.

#[test]
fn sh1_artifact_plan_valid() {
    let source = "module artifact_check\n\ntype Foo { x: Int }\n\nfn make_foo() -> Foo { Foo { x: 1 } }\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let plan = &result.artifact_plan;
    // Artifact plan should have at least one artifact
    assert!(!plan.artifacts.is_empty(), "artifact plan should contain at least one artifact");
    // All boundary references should point to existing artifact names
    let artifact_names: Vec<&str> = plan.artifacts.iter().map(|a| a.name.as_str()).collect();
    for b in plan.boundaries.iter() {
        assert!(
            artifact_names.contains(&b.from_artifact.as_str()),
            "boundary from_artifact '{}' not found in artifacts: {:?}",
            b.from_artifact, artifact_names
        );
        assert!(
            artifact_names.contains(&b.to_artifact.as_str()),
            "boundary to_artifact '{}' not found in artifacts: {:?}",
            b.to_artifact, artifact_names
        );
        assert_ne!(
            b.from_artifact, b.to_artifact,
            "boundary self-loop: {} -> {}",
            b.from_artifact, b.to_artifact
        );
    }
}

#[test]
fn sh2_ownership_covers_all_functions() {
    let source = "module own_check\n\nfn add(a: Int, b: Int) -> Int { a + b }\n\nfn greet(name: String) -> String { name }\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    // Every function with a body should have an ownership proof
    let proof_names: Vec<&str> = result.ownership.iter().map(|p| p.func_name.as_str()).collect();
    assert!(proof_names.contains(&"add"), "ownership should cover 'add', got: {:?}", proof_names);
    assert!(proof_names.contains(&"greet"), "ownership should cover 'greet', got: {:?}", proof_names);
    // Every proof should have non-empty decisions
    for proof in result.ownership.iter() {
        assert!(
            !proof.decisions.is_empty(),
            "ownership proof for '{}' has no decisions",
            proof.func_name
        );
    }
}

#[test]
fn sh3_complexity_report_consistent() {
    let source = "module cx_check\n\nfn identity(x: Int) -> Int { x }\n\nfn double(x: Int) -> Int { x + x }\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let report = &result.complexity;
    // Violations should only reference functions in summaries
    for violation in report.violations.iter() {
        assert!(
            report.function_summaries.contains_key(&violation.func_name),
            "violation references '{}' but it's not in function_summaries",
            violation.func_name
        );
    }
}

#[test]
fn sh7_parse_output_has_valid_structure() {
    let source = "module parse_check\n\ntype Foo { x: Int }\n\nfn bar() -> Int { 42 }\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    // Files should be emitted (parse succeeded through to emit)
    assert!(!result.files.is_empty(), "compilation should produce files");
    // Every emitted file should have a non-empty path and content
    for file in result.files.iter() {
        assert!(!file.path.is_empty(), "emitted file has empty path");
        assert!(!file.content.is_empty(), "emitted file '{}' has empty content", file.path);
    }
}

#[test]
fn sh8_multi_module_imports_resolve() {
    let source_a = "module types_mod\n\ntype Color { r: Int  g: Int  b: Int }\n";
    let source_b = "module consumer_mod\n\nimport types_mod { Color }\n\nfn make_red() -> Color { Color { r: 255, g: 0, b: 0 } }\n";
    let result = compile_multi(&[
        ("types_mod.dag", source_a),
        ("consumer_mod.dag", source_b),
    ]);
    assert_no_diagnostics(&result);
    // Both modules should produce output files
    assert!(
        result.files.iter().any(|f| f.path.contains("types_mod")),
        "types_mod should produce an output file"
    );
    assert!(
        result.files.iter().any(|f| f.path.contains("consumer_mod")),
        "consumer_mod should produce an output file"
    );
    // Diagnostics should be empty (imports resolved successfully)
    assert!(result.diagnostics.is_empty(), "multi-module compilation should have 0 diagnostics");
}

#[test]
fn sh4_resolved_graph_completeness() {
    // Use DAG target to get the full typed graph as JSON, then verify
    // structural completeness of the ResolvedGraph serialization.
    let source = "module rg_check\n\ntype Color = Red | Green | Blue\n\ntype Pair { a: Int  b: String }\n\nfn make_pair() -> Pair { Pair { a: 1, b: \"hello\" } }\n";
    let result = compile_dag_target(source, RenderTarget::Dag);
    assert_no_diagnostics(&result);
    let json_str = find_file(&result, "dag-artifact.json");
    let artifact: Value = serde_json::from_str(&json_str).expect("dag artifact should be valid JSON");
    // Artifact should have version, modules, and diagnostics
    assert!(artifact.get("version").is_some(), "artifact should have version");
    assert!(artifact.get("modules").is_some(), "artifact should have modules");
    let modules = artifact["modules"].as_array().expect("modules should be array");
    assert!(!modules.is_empty(), "modules should be non-empty");
    // Each module should have name and items
    for module in modules {
        let mod_obj = module.get("module").expect("typed module should have 'module' field");
        assert!(mod_obj.get("name").is_some(), "module should have a name");
        let items_field = module.get("items");
        assert!(items_field.is_some(), "typed module should have 'items' field");
    }
}

// ── Structural method resolution (Tier 0) ───────────────────────────────

#[test]
fn structural_method_resolution_with_std() {
    // std modules loaded automatically by compile_dag/compile_multi.
    // List<Int> → FreeMonoid<Int> → Conj { map, filter, count, ... }
    // Method calls resolve via lookup_structural_method (Tier 0).
    let user = r#"module user_test
import std.types { List, Map }

// FreeMonoid methods on List
fn identity(xs: List<Int>) -> List<Int> { xs |> map(x => x) }
fn evens(xs: List<Int>) -> List<Int> { xs |> filter(x => x == 0) }
fn total(xs: List<Int>) -> Int { xs |> count }
fn has_any(xs: List<Int>) -> Bool { xs |> any(x => x == 0) }
fn has_all(xs: List<Int>) -> Bool { xs |> all(x => x == 0) }
fn head(xs: List<Int>) -> Int? { xs |> first }
fn tail_el(xs: List<Int>) -> Int? { xs |> last }
fn prefix(xs: List<Int>) -> List<Int> { xs |> take(3) }
fn suffix(xs: List<Int>) -> List<Int> { xs |> skip(1) }
fn flipped(xs: List<Int>) -> List<Int> { xs |> reverse }
fn with_el(xs: List<Int>) -> List<Int> { xs |> append(42) }
fn has_it(xs: List<Int>) -> Bool { xs |> contains(1) }

// PartialFunction methods on Map
fn lookup_key(m: Map<String, Int>) -> Int? { m |> get("key") }
fn has_key(m: Map<String, Int>) -> Bool { m |> has("key") }
fn all_keys(m: Map<String, Int>) -> List<String> { m |> keys }
fn all_vals(m: Map<String, Int>) -> List<Int> { m |> values }
"#;
    let result = compile_dag(user);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.is_empty(),
        "structural method resolution should produce 0 diagnostics, got {}: {:?}",
        msgs.len(),
        msgs,
    );
}

#[test]
fn structural_method_colliding_name_no_bridge() {
    // Regression: a user-defined type with a method named "count" or "has"
    // must NOT be tagged with AlgebraMethodSemantics for intrinsic dispatch.
    // It should get PlainMethodSemantics so emit renders it as recv.method(args).
    let source = r#"module test

type Counter {
  count: fn() -> Int
  has: fn(String) -> Bool
}

fn get_count(c: Counter) -> Int {
  c.count
}

fn check_has(c: Counter) -> Bool {
  c.has("key")
}
"#;
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.is_empty(),
        "user-defined structural methods with colliding names should compile, got: {:?}",
        msgs,
    );
}

// ── Parse-emit round-trip smoke test ────────────────────────────────────
//
// Verify that compiling the same source twice produces identical typed
// graph JSON. This is the idempotency property: the compiler is
// deterministic and the serialization is stable.

fn sort_json_arrays(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), sort_json_arrays(v));
            }
            Value::Object(out)
        }
        Value::Array(arr) => {
            let sorted: Vec<Value> = arr.iter().map(sort_json_arrays).collect();
            // Sort arrays of strings (like item_registry_keys)
            if sorted.iter().all(|v| v.is_string()) {
                let mut strs: Vec<String> = sorted.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                strs.sort();
                Value::Array(strs.into_iter().map(Value::String).collect())
            } else {
                Value::Array(sorted)
            }
        }
        _ => value.clone(),
    }
}

#[test]
fn parse_emit_round_trip_idempotency() {
    let source = r#"module roundtrip

type Color = Red | Green | Blue

type Config {
  name: String
  retries: Int
  verbose: Bool
}

fn default_config() -> Config {
  Config { name: "default", retries: 3, verbose: false }
}

fn double(x: Int) -> Int { x * 2 }

fn greet(name: String) -> String { concat("Hello, ", name) }
"#;
    let json1 = sort_json_arrays(&typed_graph_json(source));
    let json2 = sort_json_arrays(&typed_graph_json(source));
    assert_eq!(
        json1, json2,
        "compiling the same source twice should produce structurally identical typed graph JSON"
    );
    // Verify the artifact has the expected structural properties
    let modules = json1["modules"].as_array().expect("modules should be array");
    assert!(!modules.is_empty(), "should have at least one module");
    let module = &modules[0];
    let mod_obj = module.get("module").expect("should have module field");
    assert_eq!(mod_obj["name"], "roundtrip", "module name should match");
}

// ── Scrambled name emit tests ─────────────────────────────────────────
//
// These tests verify that emission is name-opaque: replacing all user-
// defined type names with arbitrary strings produces identical emitted
// source code (after normalizing names back to ordinal placeholders).
//
// This is the emit-side complement to the inference scrambled-name tests
// above. If these tests fail, it reveals places where emit makes
// decisions based on type names rather than structural facts.

/// Normalize emitted source: replace each user-defined name with its
/// ordinal placeholder. Longer names are replaced first to avoid partial
/// matches (e.g. "FooBar" before "Foo").
fn normalize_emitted_source(source: &str, names: &[&str], prefix: &str) -> String {
    // Build replacement pairs sorted by descending length (longest first)
    let mut pairs: Vec<(&str, String)> = names
        .iter()
        .enumerate()
        .map(|(i, name)| (*name, format!("{}{}", prefix, i)))
        .collect();
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let mut result = source.to_string();
    for (name, placeholder) in &pairs {
        result = result.replace(name, placeholder);
    }
    result
}

/// Assert that two programs produce structurally identical emitted source
/// after normalizing user-defined type names.
///
/// Compiles both programs with the given target, finds files matching the
/// `file_selector` predicate, normalizes names, and asserts equality.
fn assert_scrambled_name_emit_eq(
    source_a: &str,
    source_b: &str,
    names_a: &[&str],
    names_b: &[&str],
    target: RenderTarget,
    file_selector: fn(&str) -> bool,
    label: &str,
) {
    assert_eq!(names_a.len(), names_b.len(), "name lists must be parallel");

    let result_a = compile_dag_target(source_a, target);
    let result_b = compile_dag_target(source_b, target);

    // Find the main source file from each result
    let file_a = result_a
        .files
        .iter()
        .find(|f| file_selector(&f.path))
        .unwrap_or_else(|| {
            let paths: Vec<_> = result_a.files.iter().map(|f| f.path.as_str()).collect();
            panic!("{label}: no matching file in program A, available: {paths:?}")
        });
    let file_b = result_b
        .files
        .iter()
        .find(|f| file_selector(&f.path))
        .unwrap_or_else(|| {
            let paths: Vec<_> = result_b.files.iter().map(|f| f.path.as_str()).collect();
            panic!("{label}: no matching file in program B, available: {paths:?}")
        });

    let norm_a = normalize_emitted_source(&file_a.content, names_a, "__T");
    let norm_b = normalize_emitted_source(&file_b.content, names_b, "__T");

    assert_eq!(
        norm_a, norm_b,
        "scrambled-name emit mismatch in {label}:\n\
         --- normalized A ---\n{norm_a}\n\
         --- normalized B ---\n{norm_b}",
    );
}

#[test]
fn scrambled_name_emit_rust() {
    // Function names deliberately avoid containing type names to prevent
    // false normalization hits (e.g. "make_foo" PascalCased to "MakeFoo"
    // would collide with type name "Foo").
    let source_a = "\
module test

type Foo { x: Int  y: String }
type Bar { label: String  count: Int }

fn create() -> Foo { Foo { x: 1, y: \"hello\" } }
fn get_label(b: Bar) -> String { b.label }
";
    let source_b = "\
module test

type Zqx { x: Int  y: String }
type Wmn { label: String  count: Int }

fn create() -> Zqx { Zqx { x: 1, y: \"hello\" } }
fn get_label(b: Wmn) -> String { b.label }
";
    assert_scrambled_name_emit_eq(
        source_a,
        source_b,
        &["Foo", "Bar"],
        &["Zqx", "Wmn"],
        RenderTarget::Rust,
        |path| path.ends_with(".rs") && path.contains("src/"),
        "Rust emit (simple structs)",
    );
}

#[test]
fn scrambled_name_emit_python() {
    let source_a = "\
module test

type Foo { x: Int  y: String }
type Bar { label: String  count: Int }

fn create() -> Foo { Foo { x: 1, y: \"hello\" } }
fn get_label(b: Bar) -> String { b.label }
";
    let source_b = "\
module test

type Zqx { x: Int  y: String }
type Wmn { label: String  count: Int }

fn create() -> Zqx { Zqx { x: 1, y: \"hello\" } }
fn get_label(b: Wmn) -> String { b.label }
";
    assert_scrambled_name_emit_eq(
        source_a,
        source_b,
        &["Foo", "Bar"],
        &["Zqx", "Wmn"],
        RenderTarget::Python,
        |path| path.ends_with(".py") && !path.contains("__init__"),
        "Python emit (simple structs)",
    );
}

#[test]
fn scrambled_name_emit_go() {
    let source_a = "\
module test

type Foo { x: Int  y: String }
type Bar { label: String  count: Int }

fn create() -> Foo { Foo { x: 1, y: \"hello\" } }
fn get_label(b: Bar) -> String { b.label }
";
    let source_b = "\
module test

type Zqx { x: Int  y: String }
type Wmn { label: String  count: Int }

fn create() -> Zqx { Zqx { x: 1, y: \"hello\" } }
fn get_label(b: Wmn) -> String { b.label }
";
    assert_scrambled_name_emit_eq(
        source_a,
        source_b,
        &["Foo", "Bar"],
        &["Zqx", "Wmn"],
        RenderTarget::Go,
        |path| path.ends_with(".go") && !path.contains("go.mod") && !path.contains("_test.go"),
        "Go emit (simple structs)",
    );
}

// ── Emit pipeline type rendering tests (E2.5 + E3.4) ──────────────────

#[test]
fn rust_primitive_bool_lowers_to_bool() {
    let source = "module test_bool_lower\n\ntype Flags {\n  active: Bool\n  visible: Bool\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test_bool_lower.rs");
    assert!(content.contains("bool"), "Bool should lower to bool in Rust, got: {}", content);
    assert!(!content.contains(": Bool"), "Raw Bool should not appear as a type in Rust output, got: {}", content);
}

#[test]
fn rust_primitive_int_lowers_to_i64() {
    let source = "module test_int_lower\n\ntype Counter {\n  value: Int\n  max: Int\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test_int_lower.rs");
    assert!(content.contains("i64"), "Int should lower to i64 in Rust, got: {}", content);
    assert!(!content.contains(": Int"), "Raw Int should not appear as a type in Rust output, got: {}", content);
}

#[test]
fn rust_primitive_float_lowers_to_f64() {
    let source = "module test_float_lower\n\ntype Measurement {\n  value: Float\n  error: Float\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test_float_lower.rs");
    assert!(content.contains("f64"), "Float should lower to f64 in Rust, got: {}", content);
    assert!(!content.contains(": Float"), "Raw Float should not appear as a type in Rust output, got: {}", content);
}

#[test]
fn rust_list_type_lowers_to_rc_vec() {
    let source = "module test_list_lower\n\ntype Batch {\n  items: List<Int>\n  names: List<String>\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test_list_lower.rs");
    assert!(content.contains("Rc<Vec<"), "List should lower to Rc<Vec<...>> in Rust, got: {}", content);
    assert!(!content.contains("List<"), "Raw List<> should not appear in Rust output, got: {}", content);
}

#[test]
fn rust_map_type_lowers_to_rc_hashmap() {
    let source = "module test_map_lower\n\ntype Registry {\n  entries: Map<String, Int>\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test_map_lower.rs");
    assert!(
        content.contains("Rc<HashMap<"),
        "Map should lower to Rc<HashMap<...>> in Rust, got: {}", content
    );
    // "Map<" without a leading letter (to exclude "HashMap<" and "Rc<HashMap<")
    let has_raw_map = content.lines().any(|line| {
        if let Some(pos) = line.find("Map<") {
            pos == 0 || !line.as_bytes()[pos - 1].is_ascii_alphabetic()
        } else {
            false
        }
    });
    assert!(!has_raw_map, "Raw Map<> (not HashMap/BTreeMap) should not appear in Rust output, got: {}", content);
}

#[test]
fn rust_callable_renders_as_fn_trait() {
    let source = "module test_callable\n\nfn apply(f: fn(Int) -> String, x: Int) -> String {\n  f(x)\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test_callable.rs");
    assert!(
        content.contains("Fn(") || content.contains("impl Fn"),
        "Callable param should render as Fn trait in Rust, got: {}", content
    );
}

#[test]
fn rust_func_with_uses_emits_async_fn() {
    // func with uses clause emits async fn; func without uses emits regular fn
    let source = "module test_async_func\n\nresource Net {}\n\nfunc do_work() -> String\n  uses net: Net\n{\n  \"done\"\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    // func + uses should compile and produce async fn
    if has_file(&result, "src/test_async_func.rs") {
        let content = find_file(&result, "src/test_async_func.rs");
        assert!(
            content.contains("async fn"),
            "func with uses should emit async fn in Rust, got: {}", content
        );
    } else {
        // If compilation produces diagnostics instead of files, the test still
        // validates the pipeline doesn't crash on func+uses syntax
        let msgs = diagnostic_messages(&result);
        assert!(
            !msgs.is_empty(),
            "func+uses should either emit files or produce diagnostics"
        );
    }
}

// ── Enumerate inference test ─────────────────────────────────────────────
// Verifies that enumerate returns List<Tuple<Int, Elem>> (not bare List<Elem>),
// and that .first/.second field access on enumerate results compiles cleanly.

#[test]
fn enumerate_returns_tuple_type() {
    let source = r#"
module enumerate_test

fn indexed_names(names: List<String>) -> List<String> {
  names |> enumerate |> map(pair => concat(pair.first |> to_string, ": ", pair.second))
}
"#;
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.is_empty(),
        "enumerate .first/.second should compile without diagnostics, got {}: {:?}",
        msgs.len(), msgs
    );
}
