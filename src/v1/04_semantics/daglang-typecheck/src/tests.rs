use super::*;
use daglang_contract::FileId;
use std::collections::HashMap;
use std::path::PathBuf;

fn expr_identity(
    stmts: &[daglang_syntax::ast::Stmt],
    target: &daglang_syntax::ast::Expr,
) -> daglang_syntax::ast_utils::ExprIdentity {
    let mut found = None;
    daglang_syntax::ast_utils::walk_stmts_with_expr_identities(
        stmts,
        &mut |expr_identity, expr| {
            if std::ptr::eq(expr, target) {
                found = Some(expr_identity);
            }
        },
    );
    found.expect("expected walked expression identity")
}

fn module_graph_from_sources(sources: &[(&str, &str)]) -> ModuleGraph {
    let modules = sources
        .iter()
        .map(|(path, source)| {
            let ast = daglang_syntax::parser::parse(source).expect("source should parse");
            let module_path = ast
                .module_path
                .as_ref()
                .map(|module| module.node.clone())
                .expect("module declarations are required in tests");
            ResolvedModule {
                path: PathBuf::from(path),
                ast,
                module_path,
                dependencies: Vec::new(),
                source: source.to_string(),
            }
        })
        .collect::<Vec<_>>();
    let module_lookup = modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_path.as_dotted(), index))
        .collect::<HashMap<_, _>>();
    let mut modules = modules;
    for module in &mut modules {
        module.dependencies = module
            .ast
            .imports
            .iter()
            .filter_map(|import| module_lookup.get(&import.node.path.as_dotted()).copied())
            .collect::<Vec<_>>();
    }
    ModuleGraph { modules }
}

#[test]
fn typecheck_accepts_inline_module_with_fn_and_func_signatures() {
    let graph = module_graph_from_sources(&[(
        "sample/tooling.dag",
        r#"module sample.tooling

fn render_makefile_body() -> String {
  "all:\n\tcargo test"
}

func makegen() -> { content: String } {
  body = render_makefile_body()
  return { content: body }
}
"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect("inline tooling module should typecheck");

    assert_eq!(typed.module_count(), 1);
    let module = typed.module(0).expect("typed module should exist");
    assert_eq!(module.module_path.as_dotted(), "sample.tooling");
    assert!(module
        .signatures
        .iter()
        .any(|signature| matches!(signature, TypedItemSignature::Fn(_))));
    assert!(module
        .signatures
        .iter()
        .any(|signature| matches!(signature, TypedItemSignature::Func(_))));
}

#[test]
fn duplicate_param_names_are_reported() {
    let graph = module_graph_from_sources(&[(
        "dup_params.dag",
        "module sample.dup\nfn bad(a: String, a: Int) -> String { a }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("duplicate params should fail");
    assert!(errors
        .iter()
        .any(|error| matches!(error, TypeError::DuplicateParameter { item, param } if item == "bad" && param == "a")));
}

#[test]
fn located_signature_errors_carry_item_span() {
    let graph = module_graph_from_sources(&[(
        "dup_params.dag",
        "module sample.dup\nfn bad(a: String, a: Int) -> String { a }",
    )]);
    let errors = typecheck_module_graph_located(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect_err("duplicate params should fail");
    assert!(errors.iter().any(|error| matches!(
        &error.error,
        TypeError::DuplicateParameter { item, param } if item == "bad" && param == "a"
    )));
    assert!(
        errors.iter().any(|error| error.span.end > error.span.start),
        "signature validation errors should carry the enclosing item span"
    );
}

#[test]
fn duplicate_output_fields_are_reported() {
    let graph = module_graph_from_sources(&[(
        "dup_outputs.dag",
        r#"module sample.dup
func run() -> { ok: Bool, ok: Bool } {
  return { ok: true }
}
"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("duplicate outputs should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateOutputField { item, field }
            if item == "run" && field == "ok"
    )));
}

#[test]
fn undefined_types_are_reported() {
    let graph = module_graph_from_sources(&[(
        "unknown_type.dag",
        "module sample.unknown\nfn run(input: MissingType) -> String { \"ok\" }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("unknown type should fail");
    assert!(errors.iter().any(
        |error| matches!(error, TypeError::UndefinedType(msg) if msg.contains("MissingType"))
    ));
}

#[test]
fn unresolvable_type_defs_fail_closed_with_item_span() {
    let graph = module_graph_from_sources(&[(
        "bad_type_def.dag",
        r#"module sample.types
type Config {
  field: MissingType
}
"#,
    )]);
    let errors = typecheck_module_graph_located(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect_err("bad type definitions should fail typecheck");
    assert!(errors.iter().any(|error| matches!(
        &error.error,
        TypeError::UnresolvableType { ty, context }
            if ty == "MissingType" && context == "type Config.field"
    )));
    assert!(
        errors.iter().any(|error| error.span.end > error.span.start),
        "type registry errors should carry the enclosing item span"
    );
}

#[test]
fn non_string_map_keys_are_rejected_in_signatures() {
    let graph = module_graph_from_sources(&[(
        "bad_map_key.dag",
        "module sample.maps\nfn run(input: Map<Int,String>) -> String { \"ok\" }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("non-string map key should fail");
    assert!(
        errors.iter().any(|error| matches!(
            error,
            TypeError::UnresolvableType { ty, context }
                if ty == "Map<Int, String>" && context == "run.input"
        )),
        "{errors:?}"
    );
}

#[test]
fn located_callable_body_errors_carry_item_span() {
    let graph = module_graph_from_sources(&[(
        "missing_call.dag",
        "module sample.calls\nfn run() -> String { missing(value: \"ok\") }",
    )]);
    let errors = typecheck_module_graph_located(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("unresolved callable target should fail");
    assert!(errors.iter().any(|error| matches!(
        &error.error,
        TypeError::UnresolvedCallTarget { caller, callee }
            if caller == "run" && callee == "missing"
    )));
    assert!(
        errors.iter().any(|error| error.span.end > error.span.start),
        "callable body errors should carry the enclosing item span"
    );
}

#[test]
fn located_typecheck_diagnostics_use_contract_located_construction() {
    let graph = module_graph_from_sources(&[(
        "sample/invalid.dag",
        "module sample.invalid\nfn run() -> String { return 42 }",
    )]);
    let errors = typecheck_module_graph_located(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("type mismatch should fail");

    let diagnostic = errors[0].to_diagnostic();
    assert_eq!(diagnostic.file_id, Some(FileId(0)));
    assert_eq!(
        diagnostic.file.as_deref(),
        Some(std::path::Path::new("sample/invalid.dag"))
    );
    assert!(
        diagnostic.span.is_some(),
        "located diagnostics must carry a span"
    );
}

#[test]
fn duplicate_definition_is_reported() {
    let graph = module_graph_from_sources(&[(
        "duplicate_definition.dag",
        r#"module sample.dup
fn run() -> Unit {}
func run() -> { ok: Bool } {
  return { ok: true }
}
"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("duplicate item name should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.dup" && name == "run"
    )));
}

#[test]
fn duplicate_interface_definition_is_reported_and_causes_ambiguous_implements() {
    let graph = module_graph_from_sources(&[(
        "duplicate_interface_definition.dag",
        r#"module sample.dup
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )]);
    let errors =
        typecheck_module_graph(&graph).expect_err("duplicate interface definitions should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.dup" && name == "Storage"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousInterface { implementor, interface }
            if implementor == "FsStorage" && interface == "Storage"
    )));
}

#[test]
fn strict_mode_duplicate_interface_definition_also_reports_ambiguous_implements() {
    let graph = module_graph_from_sources(&[(
        "duplicate_interface_definition_strict.dag",
        r#"module sample.dup
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for duplicate interface definitions");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.dup" && name == "Storage"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousInterface { implementor, interface }
            if implementor == "FsStorage" && interface == "Storage"
    )));
}

#[test]
fn strict_mode_reports_unresolved_imports() {
    let graph = module_graph_from_sources(&[(
        "missing_import.dag",
        "module sample.main\nimport missing.dep\nfn run() -> Unit {}",
    )]);
    let options = TypecheckOptions {
        allow_unresolved_imports: false,
    };
    let errors = typecheck_module_graph_with_options(&graph, options)
        .expect_err("strict mode should fail on unresolved import");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnresolvedImport { module, target }
            if module == "sample.main" && target == "missing.dep"
    )));
}

#[test]
fn call_arity_mismatch_is_reported() {
    let graph = module_graph_from_sources(&[(
        "arity_mismatch.dag",
        "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt() }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("call arity mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::CallArityMismatch {
            caller,
            callee,
            expected,
            got
        } if caller == "run" && callee == "fmt" && *expected == 1 && *got == 0
    )));
}

#[test]
fn call_with_too_many_args_is_reported() {
    let graph = module_graph_from_sources(&[(
        "arity_overflow.dag",
        "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt(\"a\", \"b\") }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("too many call args should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::CallArityMismatch {
            caller,
            callee,
            expected,
            got
        } if caller == "run" && callee == "fmt" && *expected == 1 && *got == 2
    )));
}

#[test]
fn strict_mode_allows_call_omitting_defaulted_params() {
    let graph = module_graph_from_sources(&[(
        "defaulted_call.dag",
        r#"module sample.calls
fn greet(name: String, punctuation: String = "!") -> String {
  name
}
fn run() -> String {
  greet(name: "hi")
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("defaulted callable params should be optional at call sites");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_allows_pattern_calls_with_extra_wiring_named_args() {
    let graph = module_graph_from_sources(&[(
        "pattern_wiring_call.dag",
        r#"module sample.calls
pattern ensure(should_act: Bool = true) -> { acted: Bool } {
  return { acted: should_act }
}
fn run() -> Bool {
  let result = ensure(check: true, action: false)
  result.acted
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("pattern calls should allow extra named wiring arguments");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_reports_ambiguous_call_target() {
    let graph = module_graph_from_sources(&[
        (
            "sample/one.dag",
            "module sample.one\nfn render(value: String) -> String { value }",
        ),
        (
            "sample/two.dag",
            "module sample.two\nfn render(value: String) -> String { value }",
        ),
        (
            "sample/main.dag",
            "module sample.main\nfn run() -> String { render(value: \"ok\") }",
        ),
    ]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for ambiguous callable target");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousCallTarget { caller, callee }
            if caller == "run" && callee == "render"
    )));
}

#[test]
fn strict_mode_duplicate_callable_definition_also_reports_ambiguous_call_target() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for duplicate callable definition");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.main" && name == "helper"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousCallTarget { caller, callee }
            if caller == "run" && callee == "helper"
    )));
}

#[test]
fn relaxed_mode_duplicate_callable_definition_suppresses_ambiguous_call_target() {
    let graph = module_graph_from_sources(&[(
        "sample/single.dag",
        r#"module sample.single
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect_err("relaxed mode should still fail for duplicate callable definition");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.single" && name == "helper"
    )));
    assert!(!errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousCallTarget { caller, callee }
            if caller == "run" && callee == "helper"
    )));
}

#[test]
fn strict_mode_reports_unresolved_call_target() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        "module sample.main\nfn run() -> String { missing(value: \"ok\") }",
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for unresolved callable target");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnresolvedCallTarget { caller, callee }
            if caller == "run" && callee == "missing"
    )));
}

#[test]
fn relaxed_mode_allows_unresolved_call_target() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        "module sample.main\nfn run() -> String { missing(value: \"ok\") }",
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect("relaxed mode should allow unresolved callable target");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_collection_intrinsic_call_targets() {
    let graph = module_graph_from_sources(&[(
        "sample/intrinsics.dag",
        r#"module sample.intrinsics
type Stage {
  success: Bool,
  skipped: Bool,
  name: String
}
fn summarize(stages: List<Stage>) -> Int {
  let filtered = filter(stages, s => s.success)
  let passed = count(filtered)
  let names = map(stages, s => s.name)
  let labels = join(names, ",")
  let done = ends_with(labels, "ok")
  passed
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("collection intrinsics should be recognized in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_std_helper_intrinsic_call_targets() {
    let graph = module_graph_from_sources(&[(
        "sample/helpers.dag",
        r#"module sample.helpers

type DocgenSources { path: String }

fn run(sources: DocgenSources) -> String {
  let a = replace_section("template", "section", "value")
  let b = render_test_listings(sources: sources)
  let c = render_graph_structure(sources: sources)
  let d = render_source_artifacts(sources: sources)
  let e = compute_topology_diff(current: "{}", base: "{}")
  let f = render_annotated_mermaid(diff: e, topology: "{}", title: "title")
  let g = detect_runtime()
  let h = generate()
  let i = now()
  a
}
"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("std helper intrinsics should be recognized in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_generic_fn_type_params() {
    let graph = module_graph_from_sources(&[(
        "sample/generic_fn.dag",
        r#"module sample.generic
fn identity<T>(value: T) -> T {
  value
}
fn relay<T>(value: T) -> T {
  identity(value: value)
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("generic fn type parameters should be treated as known types");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_generic_pattern_type_params() {
    let graph = module_graph_from_sources(&[(
        "sample/generic_pattern.dag",
        r#"module sample.generic
pattern passthrough<T: Serializable>(value: T) -> { value: T } {
  return { value: value }
}
fn relay<T>(value: T) -> T {
  let result = passthrough(value: value)
  result.value
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("generic pattern type parameters should be treated as known types");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_untyped_record_literal_for_named_return() {
    let graph = module_graph_from_sources(&[(
        "sample/records.dag",
        r#"module sample.records
type StageResult {
  success: Bool,
  skipped: Bool
}
fn result() -> StageResult {
  { success: true, skipped: false }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("record literals should satisfy named-record return contracts");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn typecheck_tracks_let_bound_anonymous_record_constructor_targets() {
    let graph = module_graph_from_sources(&[(
        "sample/records.dag",
        r#"module sample.records
type ConfigA {
  value: String
}
type ConfigB {
  value: String
}
fn consume(cfg: ConfigB) -> String {
  cfg.value
}
fn make() -> String {
  cfg = { value: "ok" }
  consume(cfg)
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("let-bound record should inherit the typed call target");

    let module = typed.module(0).expect("typed module should exist");
    let make = module
        .ast
        .items
        .iter()
        .find_map(|item| match &item.node {
            daglang_syntax::ast::Item::FnDef(def) if def.name == "make" => Some(def),
            _ => None,
        })
        .expect("make fn should be present");
    let record_expr = match &make.body.stmts[0] {
        daglang_syntax::ast::Stmt::Let(_, expr) | daglang_syntax::ast::Stmt::Assign(_, expr) => {
            expr
        }
        other => panic!("expected first stmt to bind the record, got {other:?}"),
    };
    let metadata = module
        .callable_body_metadata("make")
        .expect("make should carry callable body metadata");
    assert_eq!(
        metadata
            .anonymous_record_target(expr_identity(&make.body.stmts, record_expr))
            .map(|target| target.0.as_str()),
        Some("ConfigB")
    );
}

#[test]
fn strict_mode_accepts_resource_config_named_type_returns() {
    let graph = module_graph_from_sources(&[(
        "sample/resources.dag",
        r#"module sample.resources
resource GcsBucket {
  config {
name: String,
project: String
  }
}
fn gcp_dev_storage() -> GcsBucket.Config {
  { name: "gunbc-dev-artifacts", project: "gunbai-auto" }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("resource config named types should be recognized in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_secret_builtin_type() {
    let graph = module_graph_from_sources(&[(
        "sample/secret.dag",
        r#"module sample.secret
fn identity(value: Secret) -> Secret {
  value
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("Secret should be recognized as builtin type");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_function_typed_parameter_call_targets() {
    let graph = module_graph_from_sources(&[(
        "sample/callback.dag",
        r#"module sample.callback
fn apply(value: Int, callback: fn(Int) -> Int) -> Int {
  callback(value)
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("function-typed parameters should be callable in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_associated_output_function_type_parameters() {
    let graph = module_graph_from_sources(&[(
        "sample/ensure.dag",
        r#"module sample.ensure
pattern ensure<Check, Action>(
  should_act: fn(Check.Output) -> Bool
) -> { acted: Bool } {
  return { acted: true }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("associated output references in function types should remain valid");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn function_typed_parameter_call_reports_arity_mismatch() {
    let graph = module_graph_from_sources(&[(
        "sample/callback_arity.dag",
        r#"module sample.callback
fn apply(callback: fn(Int) -> Int) -> Int {
  callback()
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("callback calls should enforce function-type arity");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::CallArityMismatch {
            caller,
            callee,
            expected,
            got
        } if caller == "apply" && callee == "callback" && *expected == 1 && *got == 0
    )));
}

#[test]
fn strict_mode_accepts_sum_variant_constructor_call_targets() {
    let graph = module_graph_from_sources(&[(
        "sample/constructors.dag",
        r#"module sample.constructors
type CloudConfig
  = GcpConfig { project: String, region: String }
  | AwsConfig { region: String }

fn make_gcp() -> CloudConfig {
  GcpConfig(project: "gunbc", region: "us-central1")
}
"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("sum variant constructors should resolve as callable targets");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn sum_variant_constructor_call_reports_arity_mismatch() {
    let graph = module_graph_from_sources(&[(
        "sample/constructor_arity.dag",
        r#"module sample.constructors
type CloudConfig
  = GcpConfig { project: String, region: String }

fn make_gcp() -> CloudConfig {
  GcpConfig(project: "gunbc")
}
"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("variant constructor calls should enforce field arity");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::CallArityMismatch {
            caller,
            callee,
            expected,
            got
        } if caller == "make_gcp" && callee == "GcpConfig" && *expected == 2 && *got == 1
    )));
}

#[test]
fn strict_mode_accepts_zero_arity_variant_as_identifier_value() {
    let graph = module_graph_from_sources(&[(
        "sample/variant_ident.dag",
        r#"module sample.variant
type Environment = Dev | Ci
fn env() -> Environment {
  Dev
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("zero-arity variants should be inferred from identifier expressions");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn match_trailing_expr_infers_return_type() {
    let graph = module_graph_from_sources(&[(
        "sample/match_return.dag",
        r#"module sample.match_return
type CloudConfig
  = GcpConfig { project: String }
  | AwsConfig { account: String }
type CloudProvider = Gcp | Aws

fn provider_of(config: CloudConfig) -> CloudProvider {
  match config {
GcpConfig { ... } => Gcp
AwsConfig { ... } => Aws
  }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("match as trailing expression should satisfy return type");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn unknown_named_call_argument_is_reported() {
    let graph = module_graph_from_sources(&[(
        "unknown_arg.dag",
        "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt(text: \"ok\") }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("unknown named argument should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnknownCallArgument {
            caller,
            callee,
            argument
        } if caller == "run" && callee == "fmt" && argument == "text"
    )));
}

#[test]
fn duplicate_named_call_argument_is_reported() {
    let graph = module_graph_from_sources(&[(
        "duplicate_arg.dag",
        "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt(value: \"a\", value: \"b\") }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("duplicate named argument should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateCallArgument {
            caller,
            callee,
            argument
        } if caller == "run" && callee == "fmt" && argument == "value"
    )));
}

#[test]
fn service_call_arity_mismatch_is_reported() {
    let graph = module_graph_from_sources(&[(
        "service_arity_mismatch.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read()
  return { ok: true }
}"#,
    )]);
    let errors =
        typecheck_module_graph(&graph).expect_err("service call arity mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::ServiceCallArityMismatch {
            caller,
            service_call,
            expected,
            got
        } if caller == "run"
            && service_call == "FsStorage.read"
            && *expected == 1
            && *got == 0
    )));
}

#[test]
fn service_call_with_too_many_args_is_reported() {
    let graph = module_graph_from_sources(&[(
        "service_arity_overflow.dag",
        r#"module sample.services
service FsStorage {
  operation read(path: String) -> { body: String }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read(path: "/tmp", extra: "value")
  return { ok: true }
}"#,
    )]);
    let errors =
        typecheck_module_graph(&graph).expect_err("too many service call arguments should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::ServiceCallArityMismatch {
            caller,
            service_call,
            expected,
            got
        } if caller == "run"
            && service_call == "FsStorage.read"
            && *expected == 1
            && *got == 2
    )));
}

#[test]
fn strict_mode_allows_service_call_omitting_defaulted_inputs() {
    let graph = module_graph_from_sources(&[(
        "service_default_input.dag",
        r#"module sample.services
service FsStorage {
  operation read(path: String, recursive: Bool = false) -> { ok: Bool }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read(path: "/tmp")
  return { ok: response.ok }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("service inputs with defaults should be optional at call sites");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn uses_bound_resource_capability_call_typechecks_and_infers_outputs() {
    let graph = module_graph_from_sources(&[(
        "resource_bound_service_call.dag",
        r#"module sample.resources
resource Filesystem {
  capability read {
input { path: String }
output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("resource-bound capability calls should typecheck in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn uses_bound_resource_capability_call_reports_arity_mismatch() {
    let graph = module_graph_from_sources(&[(
        "resource_bound_service_call_arity.dag",
        r#"module sample.resources
resource Filesystem {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } uses fs: Filesystem {
  let response = fs.read()
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("resource-bound capability calls should enforce arity");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::ServiceCallArityMismatch {
            caller,
            service_call,
            expected,
            got
        } if caller == "run"
            && service_call == "fs.read"
            && *expected == 1
            && *got == 0
    )));
}

#[test]
fn strict_mode_reports_unresolved_service_call() {
    let graph = module_graph_from_sources(&[(
        "service_unresolved_call.dag",
        r#"module sample.services
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for unresolved service call");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnresolvedServiceCall {
            caller,
            service_call
        } if caller == "run" && service_call == "MissingStorage.read"
    )));
}

#[test]
fn strict_mode_reports_ambiguous_service_call() {
    let graph = module_graph_from_sources(&[
        (
            "sample/first.dag",
            r#"module sample.first
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
        ),
        (
            "sample/second.dag",
            r#"module sample.second
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
        ),
        (
            "sample/main.dag",
            r#"module sample.main
func run(path: String) -> { body: String } {
  let response = SharedService.read(path: path)
  return { body: response.body }
}"#,
        ),
    ]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for ambiguous service call");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousServiceCall {
            caller,
            service_call
        } if caller == "run" && service_call == "SharedService.read"
    )));
}

#[test]
fn strict_mode_duplicate_service_definition_also_reports_ambiguous_service_call() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for duplicate service definition");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.main" && name == "FsStorage"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousServiceCall {
            caller,
            service_call
        } if caller == "run" && service_call == "FsStorage.read"
    )));
}

#[test]
fn relaxed_mode_duplicate_service_definition_suppresses_ambiguous_service_call() {
    let graph = module_graph_from_sources(&[(
        "sample/single.dag",
        r#"module sample.single
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect_err("relaxed mode should still fail for duplicate service definition");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.single" && name == "FsStorage"
    )));
    assert!(!errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousServiceCall {
            caller,
            service_call
        } if caller == "run" && service_call == "FsStorage.read"
    )));
}

#[test]
fn relaxed_mode_allows_unresolved_service_call() {
    let graph = module_graph_from_sources(&[(
        "service_unresolved_call_relaxed.dag",
        r#"module sample.services
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect("relaxed mode should allow unresolved service call for lower-stage validation");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn unknown_named_service_call_argument_is_reported() {
    let graph = module_graph_from_sources(&[(
        "service_unknown_arg.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(file: path)
  return { body: response.body }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph)
        .expect_err("unknown named service call argument should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnknownServiceCallArgument {
            caller,
            service_call,
            argument
        } if caller == "run" && service_call == "FsStorage.read" && argument == "file"
    )));
}

#[test]
fn duplicate_named_service_call_argument_is_reported() {
    let graph = module_graph_from_sources(&[(
        "service_duplicate_arg.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path, path: path)
  return { body: response.body }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph)
        .expect_err("duplicate named service call argument should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateServiceCallArgument {
            caller,
            service_call,
            argument
        } if caller == "run" && service_call == "FsStorage.read" && argument == "path"
    )));
}

#[test]
fn resource_missing_interface_capability_is_reported() {
    let graph = module_graph_from_sources(&[(
        "missing_capability.dag",
        r#"module sample.resources
interface ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
  capability write {
input { path: String, body: String }
output { ok: Bool }
  }
}
resource Disk implements ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
}"#,
    )]);
    let errors =
        typecheck_module_graph(&graph).expect_err("missing interface capability should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::MissingCapability {
            resource,
            interface,
            capability
        } if resource == "Disk" && interface == "ObjectStorage" && capability == "write"
    )));
}

#[test]
fn unresolved_interface_on_resource_is_reported() {
    let graph = module_graph_from_sources(&[(
        "missing_interface.dag",
        "module sample.resources\nresource Disk implements MissingStorage {}",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("unknown interface should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnresolvedInterface { implementor, interface }
            if implementor == "Disk" && interface == "MissingStorage"
    )));
}

#[test]
fn ambiguous_interface_on_resource_is_reported() {
    let graph = module_graph_from_sources(&[
        (
            "sample/first.dag",
            "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        ),
        (
            "sample/second.dag",
            "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        ),
        (
            "sample/main.dag",
            "module sample.main\nresource Disk implements Storage { capability read { input { path: String } output { body: String } } }",
        ),
    ]);
    let errors = typecheck_module_graph(&graph).expect_err("ambiguous interface should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousInterface {
            implementor,
            interface
        } if implementor == "Disk" && interface == "Storage"
    )));
}

#[test]
fn service_missing_interface_operation_is_reported() {
    let graph = module_graph_from_sources(&[(
        "missing_operation.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
  capability write {
input { path: String, body: String }
output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("missing operation should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::MissingOperation {
            service,
            interface,
            operation
        } if service == "FsStorage" && interface == "Storage" && operation == "write"
    )));
}

#[test]
fn unresolved_interface_on_service_is_reported() {
    let graph = module_graph_from_sources(&[(
        "missing_service_interface.dag",
        "module sample.services\nservice FsStorage implements MissingStorage { operation read(path: String) -> { body: String } }",
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("unknown interface should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnresolvedInterface { implementor, interface }
            if implementor == "FsStorage" && interface == "MissingStorage"
    )));
}

#[test]
fn ambiguous_interface_on_service_is_reported() {
    let graph = module_graph_from_sources(&[
        (
            "sample/first.dag",
            "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        ),
        (
            "sample/second.dag",
            "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        ),
        (
            "sample/main.dag",
            "module sample.main\nservice FsStorage implements Storage { operation read(path: String) -> { body: String } }",
        ),
    ]);
    let errors = typecheck_module_graph(&graph).expect_err("ambiguous interface should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousInterface {
            implementor,
            interface
        } if implementor == "FsStorage" && interface == "Storage"
    )));
}

#[test]
fn resource_capability_signature_mismatch_is_reported() {
    let graph = module_graph_from_sources(&[(
        "resource_sig_mismatch.dag",
        r#"module sample.resources
interface ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
}
resource Disk implements ObjectStorage {
  capability read {
input { path: Int }
output { body: String }
  }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("signature mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::InterfaceSignatureMismatch {
            implementor,
            interface,
            capability,
            detail,
        } if implementor == "Disk"
            && interface == "ObjectStorage"
            && capability == "read"
            && detail.contains("input field `path` expected `String` but found `Int`")
    )));
}

#[test]
fn service_operation_signature_mismatch_is_reported() {
    let graph = module_graph_from_sources(&[(
        "service_sig_mismatch.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: Int }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("signature mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::InterfaceSignatureMismatch {
            implementor,
            interface,
            capability,
            detail,
        } if implementor == "FsStorage"
            && interface == "Storage"
            && capability == "read"
            && detail.contains("output field `body` expected `String` but found `Int`")
    )));
}

#[test]
fn strict_mode_reports_unknown_used_resource_type() {
    let graph = module_graph_from_sources(&[(
        "unknown_uses.dag",
        "module sample.uses\nfunc run() -> { ok: Bool } uses fs: MissingResource { return { ok: true } }",
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for unknown used resource type");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnknownUsedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "fs" && resource_type == "MissingResource"
    )));
}

#[test]
fn strict_mode_accepts_uses_resource_type_with_runtime_config_suffix() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("configured resource type should resolve in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_reports_ambiguous_used_resource_type() {
    let graph = module_graph_from_sources(&[
        (
            "sample/one.dag",
            "module sample.one\nresource SharedResource {}",
        ),
        (
            "sample/two.dag",
            "module sample.two\nresource SharedResource {}",
        ),
        (
            "sample/main.dag",
            r#"module sample.main
func run() -> { ok: Bool } uses fs: SharedResource {
  return { ok: true }
}"#,
        ),
    ]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for ambiguous used resource type");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousUsedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "fs" && resource_type == "SharedResource"
    )));
}

#[test]
fn strict_mode_duplicate_resource_definition_also_reports_ambiguous_used_resource_type() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource {
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for duplicate resource definitions");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.main" && name == "SharedResource"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousUsedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "fs" && resource_type == "SharedResource"
    )));
}

#[test]
fn relaxed_mode_allows_unknown_used_resource_type() {
    let graph = module_graph_from_sources(&[(
        "unknown_uses_relaxed.dag",
        "module sample.uses\nfunc run() -> { ok: Bool } uses fs: MissingResource { return { ok: true } }",
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect("relaxed mode should allow unknown uses");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn relaxed_mode_duplicate_resource_definition_suppresses_ambiguous_used_resource_type() {
    let graph = module_graph_from_sources(&[(
        "sample/single.dag",
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource {
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect_err("relaxed mode should still fail for duplicate resource definition");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.single" && name == "SharedResource"
    )));
    assert!(!errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousUsedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "fs" && resource_type == "SharedResource"
    )));
}

#[test]
fn strict_mode_reports_unknown_provided_resource_type() {
    let graph = module_graph_from_sources(&[(
        "unknown_provides.dag",
        "module sample.provides\nfunc run() -> { ok: Bool } provides out: MissingResource { return { ok: true } }",
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for unknown provided resource type");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnknownProvidedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "out" && resource_type == "MissingResource"
    )));
}

#[test]
fn strict_mode_accepts_provides_resource_type_with_runtime_config_suffix() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
resource ArtifactStore {}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("configured provided resource type should resolve in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_provides_resource_type_reference() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
resource ArtifactStore {}
func run() -> { ok: Bool } provides out: ArtifactStore {
  return { ok: true }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("provided resource type should resolve in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_accepts_std_auth_context_resource_without_import() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
func run() -> { ok: Bool } provides auth: AuthContext {
  return { ok: true }
}"#,
    )]);
    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("std AuthContext should resolve in strict mode");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn strict_mode_reports_ambiguous_provided_resource_type() {
    let graph = module_graph_from_sources(&[
        (
            "sample/one.dag",
            "module sample.one\nresource SharedResource {}",
        ),
        (
            "sample/two.dag",
            "module sample.two\nresource SharedResource {}",
        ),
        (
            "sample/main.dag",
            r#"module sample.main
func run() -> { ok: Bool } provides out: SharedResource {
  return { ok: true }
}"#,
        ),
    ]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for ambiguous provided resource type");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousProvidedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "out" && resource_type == "SharedResource"
    )));
}

#[test]
fn relaxed_mode_duplicate_resource_definition_suppresses_ambiguous_provided_resource_type() {
    let graph = module_graph_from_sources(&[(
        "sample/single.dag",
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource {
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .expect_err("relaxed mode should still fail for duplicate resource definition");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.single" && name == "SharedResource"
    )));
    assert!(!errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousProvidedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "out" && resource_type == "SharedResource"
    )));
}

#[test]
fn strict_mode_duplicate_resource_definition_also_reports_ambiguous_provided_resource_type() {
    let graph = module_graph_from_sources(&[(
        "sample/main.dag",
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource {
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect_err("strict mode should fail for duplicate resource definitions");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateDefinition { module, name }
            if module == "sample.main" && name == "SharedResource"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::AmbiguousProvidedResourceType {
            item,
            binding,
            resource_type,
        } if item == "run" && binding == "out" && resource_type == "SharedResource"
    )));
}

#[test]
fn duplicate_uses_binding_is_reported() {
    let graph = module_graph_from_sources(&[(
        "duplicate_uses.dag",
        r#"module sample.uses
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage {
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("duplicate uses should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateUsesBinding { item, binding }
            if item == "run" && binding == "fs"
    )));
}

#[test]
fn duplicate_provides_binding_is_reported() {
    let graph = module_graph_from_sources(&[(
        "duplicate_provides.dag",
        r#"module sample.provides
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage provides out: Storage {
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("duplicate provides should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicateProvidesBinding { item, binding }
            if item == "run" && binding == "out"
    )));
}

#[test]
fn use_provide_binding_conflict_is_reported() {
    let graph = module_graph_from_sources(&[(
        "use_provide_conflict.dag",
        r#"module sample.conflict
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } uses io: Storage provides io: Storage {
  return { ok: true }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("binding conflict should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UseProvideBindingConflict { item, binding }
            if item == "run" && binding == "io"
    )));
}

#[test]
fn type_mismatch_in_fn_return_is_reported() {
    let graph = module_graph_from_sources(&[(
        "type_mismatch_fn_return.dag",
        r#"module sample.types
fn run() -> String { return 42 }"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("type mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::TypeMismatch { expected, got }
            if expected == "String" && got == "Int"
    )));
}

#[test]
fn implicit_type_mismatch_in_fn_return_is_reported() {
    let graph = module_graph_from_sources(&[(
        "implicit_type_mismatch_fn_return.dag",
        r#"module sample.types
fn run() -> String { 42 }"#,
    )]);
    let errors =
        typecheck_module_graph(&graph).expect_err("implicit return type mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::TypeMismatch { expected, got }
            if expected == "String" && got == "Int"
    )));
}

#[test]
fn missing_tail_expression_in_fn_return_is_reported_as_unit_mismatch() {
    let graph = module_graph_from_sources(&[(
        "missing_tail_expression_fn_return.dag",
        r#"module sample.types
fn run() -> String { let x = 42 }"#,
    )]);
    let errors = typecheck_module_graph(&graph)
        .expect_err("missing tail expression should fail for non-unit return type");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::TypeMismatch { expected, got }
            if expected == "String" && got == "Unit"
    )));
}

#[test]
fn missing_tail_expression_is_allowed_for_unit_return_type() {
    let graph = module_graph_from_sources(&[(
        "missing_tail_expression_unit_return.dag",
        r#"module sample.types
fn run() -> Unit { let x = 42 }"#,
    )]);
    let typed =
        typecheck_module_graph(&graph).expect("unit return type should allow no tail expression");
    assert_eq!(typed.module_count(), 1);
}

#[test]
fn no_such_field_on_record_literal_is_reported() {
    let graph = module_graph_from_sources(&[(
        "no_such_field.dag",
        r#"module sample.fields
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("no such field should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::NoSuchField { ty, field } if ty == "Record" && field == "missing"
    )));
}

#[test]
fn no_such_field_on_named_record_type_is_reported() {
    let graph = module_graph_from_sources(&[(
        "no_such_field_named_record.dag",
        r#"module sample.fields
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }"#,
    )]);
    let errors =
        typecheck_module_graph(&graph).expect_err("no such field on named record should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::NoSuchField { ty, field } if ty == "Payload" && field == "missing"
    )));
}

#[test]
fn unsatisfiable_refinement_is_reported() {
    let graph = module_graph_from_sources(&[(
        "unsat_refinement.dag",
        r#"module sample.refinement
fn run(value: Int where range(min: 5, max: 1)) -> Int { value }"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("unsatisfiable range should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnsatisfiableRefinement { ty, constraint }
            if ty == "Int" && constraint.contains("min 5 exceeds max 1")
    )));
}

#[test]
fn generic_arity_mismatch_is_reported() {
    let graph = module_graph_from_sources(&[(
        "generic_arity_mismatch.dag",
        r#"module sample.generics
fn run(items: Map<String>) -> Int { 1 }"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("generic arity mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::ArityMismatch {
            name,
            expected,
            got,
        } if name == "Map" && *expected == 2 && *got == 1
    )));
}

#[test]
fn user_defined_generic_arity_mismatch_is_reported() {
    let graph = module_graph_from_sources(&[(
        "user_generic_arity_mismatch.dag",
        r#"module sample.generics
type Box<T> = T
fn run(value: Box<String, Int>) -> String { value }"#,
    )]);
    let errors = typecheck_module_graph(&graph)
        .expect_err("user-defined generic arity mismatch should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::ArityMismatch {
            name,
            expected,
            got,
        } if name == "Box" && *expected == 1 && *got == 2
    )));
}

#[test]
fn pipeline_unknown_stage_dependency_is_reported() {
    let graph = module_graph_from_sources(&[(
        "pipeline_unknown_dep.dag",
        r#"module sample.pipeline
pipeline ci {
  stage build [after missing] {}
}"#,
    )]);
    let errors =
        typecheck_module_graph(&graph).expect_err("unknown pipeline stage dependency should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::UnknownPipelineStageDependency {
            pipeline,
            stage,
            dependency,
        } if pipeline == "ci" && stage == "build" && dependency == "missing"
    )));
}

#[test]
fn duplicate_pipeline_stage_is_reported() {
    let graph = module_graph_from_sources(&[(
        "pipeline_duplicate_stage.dag",
        r#"module sample.pipeline
pipeline ci {
  stage build {}
  stage build {}
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("duplicate pipeline stage should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::DuplicatePipelineStage { pipeline, stage }
            if pipeline == "ci" && stage == "build"
    )));
}

// --- WS3-5: Branch type unification ---

#[test]
fn if_else_branch_type_mismatch_string_vs_int() {
    let graph = module_graph_from_sources(&[(
        "branch_mismatch.dag",
        r#"module test.branch_mismatch

fn check(flag: Bool) -> String {
  if flag { "hello" } else { 42 }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("mismatched branches should fail");
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::BranchTypeMismatch { then_type, else_type }
                if then_type == "String" && else_type == "Int"
        )),
        "expected BranchTypeMismatch(String, Int), got: {errors:?}"
    );
}

#[test]
fn if_else_same_type_no_error() {
    let graph = module_graph_from_sources(&[(
        "branch_ok.dag",
        r#"module test.branch_ok

fn pick(flag: Bool) -> String {
  if flag { "hello" } else { "world" }
}"#,
    )]);
    let result = typecheck_module_graph(&graph);
    assert!(result.is_ok(), "same-type branches should pass: {result:?}");
}

#[test]
fn match_arm_type_mismatch_string_vs_bool() {
    let graph = module_graph_from_sources(&[(
        "match_mismatch.dag",
        r#"module test.match_mismatch

fn check(x: Int) -> String {
  match x {
    1 => "one"
    _ => true
  }
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("mismatched match arms should fail");
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::MatchArmTypeMismatch { first_type, mismatched_type }
                if first_type == "String" && mismatched_type == "Bool"
        )),
        "expected MatchArmTypeMismatch(String, Bool), got: {errors:?}"
    );
}

#[test]
fn match_arms_same_sum_type_variants_no_error() {
    let graph = module_graph_from_sources(&[(
        "match_variants.dag",
        r#"module test.match_variants

type Color = Red | Blue | Green

fn pick(x: Int) -> Color {
  match x {
    1 => Red
    2 => Blue
    _ => Green
  }
}"#,
    )]);
    let result = typecheck_module_graph(&graph);
    assert!(
        result.is_ok(),
        "same-parent variant arms should pass: {result:?}"
    );
}

#[test]
fn if_else_same_sum_type_variants_no_error() {
    let graph = module_graph_from_sources(&[(
        "if_variants.dag",
        r#"module test.if_variants

type Status = Active | Inactive

fn pick(flag: Bool) -> Status {
  if flag { Active } else { Inactive }
}"#,
    )]);
    let result = typecheck_module_graph(&graph);
    assert!(
        result.is_ok(),
        "same-parent variant if/else should pass: {result:?}"
    );
}

// --- WS3-6: Match exhaustiveness ---

#[test]
fn exhaustiveness_all_variants_covered() {
    let mut variants = HashMap::new();
    variants.insert(
        "Color".to_string(),
        HashSet::from(["Red".to_string(), "Blue".to_string(), "Green".to_string()]),
    );
    let matched = HashSet::from(["Red".to_string(), "Blue".to_string(), "Green".to_string()]);
    let result = check_match_exhaustiveness("Color", &matched, false, &variants);
    assert!(result.is_none(), "all variants covered should return None");
}

#[test]
fn exhaustiveness_missing_variants() {
    let mut variants = HashMap::new();
    variants.insert(
        "Color".to_string(),
        HashSet::from(["Red".to_string(), "Blue".to_string(), "Green".to_string()]),
    );
    let matched = HashSet::from(["Red".to_string()]);
    let missing = check_match_exhaustiveness("Color", &matched, false, &variants)
        .expect("should report missing variants");
    assert_eq!(missing, vec!["Blue", "Green"]);
}

#[test]
fn exhaustiveness_wildcard_suppresses_check() {
    let mut variants = HashMap::new();
    variants.insert(
        "Color".to_string(),
        HashSet::from(["Red".to_string(), "Blue".to_string(), "Green".to_string()]),
    );
    let matched = HashSet::from(["Red".to_string()]);
    let result = check_match_exhaustiveness("Color", &matched, true, &variants);
    assert!(
        result.is_none(),
        "wildcard should suppress exhaustiveness check"
    );
}

#[test]
fn exhaustiveness_unknown_type_returns_none() {
    let variants = HashMap::new();
    let matched = HashSet::from(["Foo".to_string()]);
    let result = check_match_exhaustiveness("UnknownType", &matched, false, &variants);
    assert!(
        result.is_none(),
        "unknown scrutinee type should return None"
    );
}

#[test]
fn non_exhaustive_match_error_format() {
    let err = TypeError::NonExhaustiveMatch {
        scrutinee_type: "Color".to_string(),
        missing_variants: vec!["Blue".to_string(), "Green".to_string()],
    };
    assert_eq!(err.code(), "TC040");
    let msg = format!("{err}");
    assert!(msg.contains("Color"), "error should mention type name");
    assert!(msg.contains("Blue"), "error should mention missing variant");
    assert!(
        msg.contains("Green"),
        "error should mention missing variant"
    );
}

#[test]
fn pipeline_when_condition_must_be_bool() {
    let graph = module_graph_from_sources(&[(
        "pipeline_when_type_mismatch.dag",
        r#"module sample.pipeline
pipeline ci {
  stage build [when 42] {}
}"#,
    )]);
    let errors = typecheck_module_graph(&graph).expect_err("non-bool when should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypeError::PipelineStageWhenTypeMismatch { pipeline, stage, got }
            if pipeline == "ci" && stage == "build" && got == "Int"
    )));
}
