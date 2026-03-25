#![allow(clippy::disallowed_macros)]

use crate::helpers::*;
use v2_compiler::artifact::RenderTarget;

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
fn go_emit_generates_mock_test_file() {
    let source = "module mock_smoke\n\ntype Pong = String\n\nservice demo.Api {\n  operation Ping {\n    response {\n      200 => Pong\n    }\n    mock_response {\n      200 => \"pong\"\n    }\n  }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Go);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "mock_smoke_test.go");
    assert!(content.contains("func TestDemoApiPing("), "Go test file should contain a PascalCase generated test function");
    assert!(content.contains("// Signature:"), "Go test file should contain the projection signature comment");
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
fn null_coalesce_emits_rust_fallback() {
    let source = "module test\nfn fallback(name: String?) -> String {\n  name ?? \"guest\"\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test.rs");
    assert!(
        content.contains(".unwrap_or_else(||"),
        "null coalesce should emit Rust unwrap_or_else fallback"
    );
    assert!(
        content.contains("\"guest\".to_string()"),
        "null coalesce should preserve the fallback expression in emitted Rust"
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
fn for_each_emits_cloned_iteration_in_rust() {
    let source = "module test\nfn demo(items: List<Int>) -> List<Int> {\n  for item in items { item + 1 }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test.rs");
    assert!(
        content.contains(".iter().cloned()"),
        "for-each should emit cloned Rust iteration"
    );
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

#[test]
fn rust_target_emits_cargo_toml() {
    let source = "module test\nfn answer() -> Int { 42 }\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    assert!(has_file(&result, "Cargo.toml"), "Rust target should emit Cargo.toml");
    let cargo_toml = find_file(&result, "Cargo.toml");
    assert!(
        cargo_toml.contains("name = \"v2_compiled\""),
        "emitted Cargo.toml should contain the generated crate name"
    );
}

#[test]
fn recursive_function_without_return_type_reports_signature_resolution_error() {
    let source = "module test\nfn countdown(n: Int) {\n  if n == 0 {\n    0\n  } else {\n    countdown(n - 1)\n  }\n}\n";
    let result = compile_dag(source);
    let msgs = diagnostic_messages(&result);
    assert!(
        msgs.iter()
            .any(|m| m.contains("requires return type annotation")),
        "recursive function without return annotation should produce a signature diagnostic, got: {:?}",
        msgs
    );
}

#[test]
fn from_key_emits_serde_rename_attribute() {
    let source = "module test\ntype Payload {\n  access_token: String from \"accessToken\"\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test.rs");
    assert!(
        content.contains("#[serde(rename = \"accessToken\")]"),
        "from-key field rename should emit a serde rename attribute"
    );
}

#[test]
fn named_args_are_emitted_in_param_order() {
    let source = "module test\nfn pair(a: String, b: String) -> String {\n  a + b\n}\n\nfn demo() -> String {\n  pair(b: \"two\", a: \"one\")\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/test.rs");
    assert!(
        content.contains("pair(\"one\".to_string(), \"two\".to_string())"),
        "named arguments should be reordered into parameter order in emitted Rust"
    );
}

#[test]
fn tail_recursive_fn_emits_tco_loops_for_rust_and_python() {
    let source = "module test\nfn countdown(n: Int, acc: Int) -> Int {\n  if n == 0 {\n    acc\n  } else {\n    countdown(n - 1, acc + 1)\n  }\n}\n";

    let rust_result = compile_dag_target(source, RenderTarget::Rust);
    assert_no_diagnostics(&rust_result);
    let rust_content = find_file(&rust_result, "src/test.rs");
    assert!(
        rust_content.contains("loop {"),
        "tail-recursive Rust emission should use a loop"
    );
    assert!(
        rust_content.contains("continue;"),
        "tail-recursive Rust emission should continue the loop"
    );

    let python_result = compile_dag_target(source, RenderTarget::Python);
    assert_no_diagnostics(&python_result);
    let py_file = python_result
        .files
        .iter()
        .find(|f| f.path.ends_with(".py") && !f.path.contains("__init__"))
        .expect("Python target should emit a .py file");
    assert!(
        py_file.content.contains("while True:"),
        "tail-recursive Python emission should use a while True loop"
    );
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
}

#[test]
fn compile_sources_returns_ownership_proofs() {
    let source = "module own\nfn identity(x: Int) -> Int { x }\nfn sum_twice(x: Int) -> Int { x + x }\n";
    let result = compile_dag(source);
    assert!(!result.ownership.is_empty(), "ownership proofs should be non-empty");
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

#[test]
fn scrambled_name_inference_smoke() {
    let source_a = "module test\ntype Foo { x: Int }\ntype Bar { name: String }\nfn make_foo() -> Foo { Foo { x: 1 } }\nfn get_name(b: Bar) -> String { b.name }\n";
    let source_b = "module test\ntype Zqx { x: Int }\ntype Wmn { name: String }\nfn make_foo() -> Zqx { Zqx { x: 1 } }\nfn get_name(b: Wmn) -> String { b.name }\n";
    let result_a = compile_dag(source_a);
    let result_b = compile_dag(source_b);
    let diags_a = diagnostic_messages(&result_a);
    let diags_b = diagnostic_messages(&result_b);
    assert_eq!(
        diags_a.len(),
        diags_b.len(),
        "diagnostic counts should be equal (name-independent inference)\n  A: {:?}\n  B: {:?}",
        diags_a,
        diags_b
    );
    assert_eq!(
        result_a.files.len(),
        result_b.files.len(),
        "emitted file counts should be equal"
    );
}

#[test]
fn scrambled_name_inference_containers() {
    let source_a = "module test\ntype Coord { x: Int  y: Int }\ntype Path { points: List<Coord> }\nfn empty_path() -> Path { Path { points: [] } }\n";
    let source_b = "module test\ntype Qwz { x: Int  y: Int }\ntype Ijk { points: List<Qwz> }\nfn empty_path() -> Ijk { Ijk { points: [] } }\n";
    let result_a = compile_dag(source_a);
    let result_b = compile_dag(source_b);
    let diags_a = diagnostic_messages(&result_a);
    let diags_b = diagnostic_messages(&result_b);
    assert_eq!(
        diags_a.len(),
        diags_b.len(),
        "diagnostic counts should be equal (name-independent inference)\n  A: {:?}\n  B: {:?}",
        diags_a,
        diags_b
    );
    assert_eq!(
        result_a.files.len(),
        result_b.files.len(),
        "emitted file counts should be equal"
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
