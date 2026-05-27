//! **Layer:** integration
//!
//! **T-10 / Wave-3-B:** `06_translate.dag` + `05_emit.dag` + `mvp1_rust_add_translate.dag`
//! tokenize/parse cleanly (M1(2.7) single-file path; full `compile_to_dag` deferred until
//! multi-module v4 load lands). Peers: `v4_bin_main_dag_smoke_test.rs`, `v4_extdeps_file_system_dag_smoke_test.rs`.
//!
//! **TESTING.md:** M1(2.7) `.dag` brace-bodied `fn` items surface as `FnExternalBody` (no
//! expression AST), so call-site contracts are checked via **import rows** and **declared `fn`
//! inventory** — not raw `str::contains` probes. Semantic substantiation deferred to T-22/T-14.
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-10 (`src/v4/compiler/06_translate.dag`, `05_emit.dag`).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS §SG-0 hand-authored integration test receipts row
//! land in the same PR. **This PR expansion (+0 census paths):** interim ratchet rows for
//! `v4_translate_dag_dispatches_token_sequence_items`,
//! `v4_rust_language_model_declares_t11_translation_rules`,
//! `v4_java_language_model_declares_t11_translation_rules`,
//! `v4_python_language_model_declares_t11_translation_rules`,
//! `v4_go_language_model_declares_t11_translation_rules`,
//! `v4_cpp_language_model_declares_t11_translation_rules`,
//! `v4_typescript_language_model_declares_t11_translation_rules`,
//! `v4_swift_language_model_declares_t11_translation_rules`,
//! `v4_wasm_language_model_declares_t11_translation_rules`, and
//! `v4_dag_language_model_declares_surface_emit_rows` in INVARIANTS.md.
//!
//! **Dissolution:** remove when translate/emit/MVP-1 surfaces are exercised only by `.dag`
//! `TestClaim` rows / a generated harness without this per-file Rust probe (or when
//! `compile_to_dag` over v4 compiler modules resolves imports without substrate collision).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const FIND_WITNESS_DAG: &str = include_str!("../../../../v4/std/find_witness.dag");
const FIND_WITNESS_PATH: &str = "src/v4/std/find_witness.dag";
const TRANSLATE_DAG: &str = include_str!("../../../../v4/compiler/06_translate.dag");
const TRANSLATE_PATH: &str = "src/v4/compiler/06_translate.dag";
const EMIT_DAG: &str = include_str!("../../../../v4/compiler/05_emit.dag");
const EMIT_PATH: &str = "src/v4/compiler/05_emit.dag";
const RUST_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/rust.dag");
const RUST_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/rust.dag";
const PYTHON_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/python.dag");
const PYTHON_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/python.dag";
const JAVA_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/java.dag");
const JAVA_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/java.dag";
const TYPESCRIPT_LANGUAGE_DAG: &str =
    include_str!("../../../../v4/extdeps/languages/typescript.dag");
const TYPESCRIPT_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/typescript.dag";
const CPP_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/cpp.dag");
const CPP_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/cpp.dag";
const SWIFT_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/swift.dag");
const SWIFT_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/swift.dag";
const WASM_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/wasm.dag");
const WASM_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/wasm.dag";
const GO_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/go.dag");
const GO_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/go.dag";
const DAG_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/dag.dag");
const DAG_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/dag.dag";
const MVP1_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_rust_add_translate.dag");
const MVP1_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_rust_add_translate.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

#[test]
fn v4_find_witness_dag_tokenizes_and_parses() {
    let _module = parse_module(FIND_WITNESS_DAG, FIND_WITNESS_PATH);
}

#[test]
fn v4_find_witness_dag_declares_find_witness_entrypoint() {
    let module = parse_module(FIND_WITNESS_DAG, FIND_WITNESS_PATH);
    assert!(
        surface_declares_fn(&module, "find_witness"),
        "{FIND_WITNESS_PATH}: must declare find_witness primitive"
    );
}

#[test]
fn v4_translate_dag_tokenizes_and_parses() {
    let _module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
}

#[test]
fn v4_translate_dag_module_path_is_compiler_translate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "compiler", "translate"]],
        "{TRANSLATE_PATH}: module authority path"
    );
}

#[test]
fn v4_translate_dag_imports_coercion_fold_delegate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "coercion"], "coercion_fold"),
        "{TRANSLATE_PATH}: must import coercion_fold from v4.std.coercion (Practice 11)"
    );
}

#[test]
fn v4_translate_dag_imports_fold_node_traversal() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "node"], "fold_node"),
        "{TRANSLATE_PATH}: must import fold_node from v4.std.node"
    );
}

#[test]
fn v4_translate_dag_declares_coerce_grounded_node() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        surface_declares_fn(&module, "coerce_grounded_node"),
        "{TRANSLATE_PATH}: must declare coerce_grounded_node wrapper"
    );
}

#[test]
fn v4_translate_dag_declares_translate_node_and_translate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        surface_declares_fn(&module, "translate_node"),
        "{TRANSLATE_PATH}: must declare translate_node fold entry"
    );
    assert!(
        surface_declares_fn(&module, "translate"),
        "{TRANSLATE_PATH}: must declare translate stage entry"
    );
}

#[test]
fn v4_translate_dag_dispatches_token_sequence_items() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_field_kind"
        ),
        "{TRANSLATE_PATH}: must inspect concrete-token kind before treating class absence as nonterminal"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_kind_fixed"
        ),
        "{TRANSLATE_PATH}: fixed-token rows must validate the shared token-kind discriminator"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_kind_bound"
        ),
        "{TRANSLATE_PATH}: bound-token rows must validate the shared token-kind discriminator"
    );
    assert!(
        surface_declares_fn(&module, "token_sequence_item_kind"),
        "{TRANSLATE_PATH}: must classify concrete tokens and nonterminal emitted nodes explicitly"
    );
    assert!(
        surface_declares_fn(&module, "token_item_to_source"),
        "{TRANSLATE_PATH}: token_sequence_to_source must dispatch nonterminals recursively"
    );
    assert!(
        surface_declares_fn(&module, "target_serialize_source_from_model_bounded"),
        "{TRANSLATE_PATH}: recursive nonterminal serialization must be explicitly bounded"
    );
}

#[test]
fn v4_translate_dag_imports_find_witness_types_not_inline_fn() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "find_witness"], "CandidateSet"),
        "{TRANSLATE_PATH}: may import find_witness carrier types"
    );
    assert!(
        !import_includes_name(&module, &["v4", "std", "find_witness"], "find_witness"),
        "{TRANSLATE_PATH}: must not import find_witness fn (delegates via coercion_fold)"
    );
}

#[test]
fn v4_emit_dag_tokenizes_and_parses() {
    let _module = parse_module(EMIT_DAG, EMIT_PATH);
}

#[test]
fn v4_emit_dag_module_path_is_compiler_emit() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "compiler", "emit"]],
        "{EMIT_PATH}: module authority path"
    );
}

#[test]
fn v4_emit_dag_imports_translate_stage() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        import_includes_name(&module, &["v4", "compiler", "translate"], "translate"),
        "{EMIT_PATH}: emit must import translate stage (serialize_target ∘ translate)"
    );
}

#[test]
fn v4_emit_dag_declares_emit_entrypoint() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        surface_declares_fn(&module, "emit"),
        "{EMIT_PATH}: must declare emit entrypoint"
    );
}

#[test]
fn v4_emit_dag_declares_shape_a_specialization_table() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        surface_declares_type(&module, "ShapeAEmitTarget"),
        "{EMIT_PATH}: must declare the closed Shape-A target selector"
    );
    assert!(
        surface_declares_type(&module, "EmitTargetSpecialization"),
        "{EMIT_PATH}: must declare specialization rows"
    );
    assert!(
        surface_declares_fn(&module, "shape_a_emit_specializations"),
        "{EMIT_PATH}: must declare the per-target specialization table"
    );
}

#[test]
fn v4_emit_dag_does_not_import_find_witness() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        !import_paths(&module)
            .iter()
            .any(|path| path.as_slice() == ["v4", "std", "find_witness"]),
        "{EMIT_PATH}: emit must not import find_witness (delegates via translate/coercion_fold)"
    );
}

#[test]
fn v4_rust_language_model_declares_t11_translation_rules() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{RUST_LANGUAGE_PATH}: Rust TargetModel must consume the shared translation-rules edge"
    );
    assert_imports_shared_token_kinds(&module, RUST_LANGUAGE_PATH);
    assert!(
        surface_declares_type(&module, "RustGrammarRelationRow"),
        "{RUST_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "rust_mvp1_translation_rules_node"),
        "{RUST_LANGUAGE_PATH}: must project MVP-1 Rust translation rules into the target model"
    );
}

#[test]
fn v4_rust_integer_overflow_disposition_is_mode_aware_and_axis_bound() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert_eq!(
        type_record_fields(&module, "OverflowDisposition")
            .iter()
            .map(|f| (f.name.as_str(), surface_type_name(&f.ty)))
            .collect::<Vec<_>>(),
        vec![
            ("ir_carrier", "IRCarrier".to_string()),
            (
                "checked_arithmetic_debug_default",
                "OverflowAction".to_string(),
            ),
            (
                "checked_arithmetic_release_default",
                "OverflowAction".to_string(),
            ),
            (
                "checked_arithmetic_overflow_checks_enabled",
                "OverflowAction".to_string(),
            ),
            (
                "checked_arithmetic_overflow_checks_disabled",
                "OverflowAction".to_string(),
            ),
        ],
        "{RUST_LANGUAGE_PATH}: Rust overflow disposition must model checked-arithmetic debug/release defaults and explicit overflow-checks behavior"
    );
    assert_eq!(
        type_record_fields(&module, "RustIntegerPrimitiveFacts")
            .iter()
            .map(|f| (f.name.as_str(), surface_type_name(&f.ty)))
            .collect::<Vec<_>>(),
        vec![
            ("surface_spelling", "Symbol".to_string()),
            (
                "overflow_disposition",
                "OverflowDisposition<RustIntegerCarrier>".to_string(),
            ),
            ("std_projection", "Symbol".to_string()),
        ],
        "{RUST_LANGUAGE_PATH}: integer primitive facts must make the overflow disposition's carrier the single kind/width authority"
    );
    assert_eq!(
        type_record_field_type(&module, "RustIntegerPrimitiveFacts", "overflow_disposition"),
        Some("OverflowDisposition<RustIntegerCarrier>".to_string()),
        "{RUST_LANGUAGE_PATH}: integer primitive facts must carry the mode-aware overflow disposition"
    );
    assert!(
        surface_declares_fn(&module, "rust_integer_carrier_interval_spec"),
        "{RUST_LANGUAGE_PATH}: Rust integer ranges must derive from the overflow disposition carrier"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "model_core"],
            "primitive_fact_axis_overflow_disposition"
        ),
        "{RUST_LANGUAGE_PATH}: Rust must import the shared overflow-disposition primitive fact axis"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "model_core"],
            "primitive_fact_axis_range"
        ),
        "{RUST_LANGUAGE_PATH}: Rust must import the shared range primitive fact axis"
    );
    assert!(
        surface_declares_fn(&module, "rust_integer_overflow_disposition"),
        "{RUST_LANGUAGE_PATH}: must declare the Rust Reference debug/release overflow disposition constructor"
    );
    assert!(
        surface_declares_fn(&module, "rust_overflow_disposition_node"),
        "{RUST_LANGUAGE_PATH}: must materialize overflow disposition facts as a Node for primitive bundles"
    );
}

#[test]
fn v4_java_language_model_declares_t11_translation_rules() {
    let module = parse_module(JAVA_LANGUAGE_DAG, JAVA_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{JAVA_LANGUAGE_PATH}: Java TargetModel must consume the shared translation-rules edge"
    );
    assert_imports_shared_token_kinds(&module, JAVA_LANGUAGE_PATH);
    assert!(
        surface_declares_type(&module, "JavaGrammarRelationRow"),
        "{JAVA_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "java_mvp1_translation_rules_node"),
        "{JAVA_LANGUAGE_PATH}: must project MVP-1 Java translation rules into the target model"
    );
}

#[test]
fn v4_python_language_model_declares_t11_translation_rules() {
    let module = parse_module(PYTHON_LANGUAGE_DAG, PYTHON_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{PYTHON_LANGUAGE_PATH}: Python TargetModel must consume the shared translation-rules edge"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "algebra"], "Empty"),
        "{PYTHON_LANGUAGE_PATH}: Python T-11 folds must import Empty from v4.std.algebra"
    );
    assert!(
        surface_declares_type(&module, "PythonGrammarRelationRow"),
        "{PYTHON_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "python_mvp1_translation_rules_node"),
        "{PYTHON_LANGUAGE_PATH}: must project MVP-1 Python translation rules into the target model"
    );
    assert!(
        surface_declares_fn(&module, "python_mvp1_target_model"),
        "{PYTHON_LANGUAGE_PATH}: must expose the MVP-1 TargetModel"
    );
}

#[test]
fn v4_go_language_model_declares_t11_translation_rules() {
    let module = parse_module(GO_LANGUAGE_DAG, GO_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{GO_LANGUAGE_PATH}: Go TargetModel must consume the shared translation-rules edge"
    );
    assert!(
        surface_declares_type(&module, "GoGrammarRelationRow"),
        "{GO_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "go_mvp1_translation_rules_node"),
        "{GO_LANGUAGE_PATH}: must project MVP-1 Go translation rules into the target model"
    );
    assert!(
        surface_declares_fn(&module, "go_mvp1_target_model"),
        "{GO_LANGUAGE_PATH}: must expose the MVP-1 TargetModel"
    );
}

#[test]
fn v4_cpp_language_model_declares_t11_translation_rules() {
    let module = parse_module(CPP_LANGUAGE_DAG, CPP_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{CPP_LANGUAGE_PATH}: C++ TargetModel must consume the shared translation-rules edge"
    );
    assert!(
        surface_declares_type(&module, "CppGrammarRelationRow"),
        "{CPP_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "cpp_mvp1_translation_rules_node"),
        "{CPP_LANGUAGE_PATH}: must project MVP-1 C++ translation rules into the target model"
    );
    assert!(
        surface_declares_fn(&module, "cpp_mvp1_target_model"),
        "{CPP_LANGUAGE_PATH}: must expose the target-profile-parameterized MVP-1 TargetModel"
    );
}

#[test]
fn v4_typescript_language_model_declares_t11_translation_rules() {
    let module = parse_module(TYPESCRIPT_LANGUAGE_DAG, TYPESCRIPT_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{TYPESCRIPT_LANGUAGE_PATH}: TypeScript TargetModel must consume the shared translation-rules edge"
    );
    assert_imports_shared_token_kinds(&module, TYPESCRIPT_LANGUAGE_PATH);
    assert!(
        surface_declares_type(&module, "TsGrammarRelationRow"),
        "{TYPESCRIPT_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "ts_mvp1_translation_rules_node"),
        "{TYPESCRIPT_LANGUAGE_PATH}: must project MVP-1 TypeScript translation rules into the target model"
    );
}

#[test]
fn v4_swift_language_model_declares_t11_translation_rules() {
    let module = parse_module(SWIFT_LANGUAGE_DAG, SWIFT_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{SWIFT_LANGUAGE_PATH}: Swift TargetModel must consume the shared translation-rules edge"
    );
    assert_imports_shared_token_kinds(&module, SWIFT_LANGUAGE_PATH);
    assert!(
        surface_declares_type(&module, "SwiftGrammarRelationRow"),
        "{SWIFT_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "swift_mvp1_translation_rules_node"),
        "{SWIFT_LANGUAGE_PATH}: must project MVP-1 Swift translation rules into the target model"
    );
}

#[test]
fn v4_wasm_language_model_declares_t11_translation_rules() {
    let module = parse_module(WASM_LANGUAGE_DAG, WASM_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{WASM_LANGUAGE_PATH}: Wasm TargetModel must consume the shared translation-rules edge"
    );
    assert_imports_shared_token_kinds(&module, WASM_LANGUAGE_PATH);
    assert!(
        surface_declares_type(&module, "WasmGrammarRelationRow"),
        "{WASM_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "wasm_mvp1_translation_rules_node"),
        "{WASM_LANGUAGE_PATH}: must project MVP-1 Wasm translation rules into the target model"
    );
}

#[test]
fn v4_dag_language_model_declares_surface_emit_rows() {
    let module = parse_module(DAG_LANGUAGE_DAG, DAG_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "grammar_relation_field_tokens"
        ),
        "{DAG_LANGUAGE_PATH}: emit rows must use the shared grammar-relation field symbols"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_field_kind"
        ),
        "{DAG_LANGUAGE_PATH}: token rows must use the shared concrete-token field symbols"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_kind_fixed"
        ),
        "{DAG_LANGUAGE_PATH}: fixed-token rows must use the shared token-kind identity"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_kind_bound"
        ),
        "{DAG_LANGUAGE_PATH}: bound-token rows must use the shared token-kind identity"
    );
    assert!(
        surface_declares_fn(&module, "emit_fixed_token"),
        "{DAG_LANGUAGE_PATH}: must declare fixed-token grammar row emission"
    );
    assert!(
        surface_declares_fn(&module, "emit_bound_token"),
        "{DAG_LANGUAGE_PATH}: must declare bound-token grammar row emission"
    );
    assert!(
        surface_declares_fn(&module, "emit_grammar_relation_row"),
        "{DAG_LANGUAGE_PATH}: must declare grammar relation row emission"
    );
    assert!(
        surface_declares_fn(&module, "emit_data_decl_emitted_node"),
        "{DAG_LANGUAGE_PATH}: data-decl emit rows must carry concrete emitted identity"
    );
    assert!(
        surface_declares_fn(&module, "emit_module_header_emitted_node"),
        "{DAG_LANGUAGE_PATH}: module-header emit rows must carry concrete emitted identity"
    );
    assert!(
        surface_declares_fn(&module, "emit_import_decl_emitted_node"),
        "{DAG_LANGUAGE_PATH}: import-decl emit rows must carry concrete emitted identity"
    );
    assert!(
        surface_declares_fn(&module, "emit_row_module_header"),
        "{DAG_LANGUAGE_PATH}: must declare module-header grammar row emission"
    );
    assert!(
        surface_declares_fn(&module, "emit_row_import_decl"),
        "{DAG_LANGUAGE_PATH}: must declare import-decl grammar row emission"
    );
    assert!(
        surface_declares_fn(&module, "emit_row_data_decl"),
        "{DAG_LANGUAGE_PATH}: must declare data-decl grammar row emission"
    );
}

#[test]
fn v4_go_language_model_tokenizes_and_parses() {
    let _module = parse_module(GO_LANGUAGE_DAG, GO_LANGUAGE_PATH);
}

#[test]
fn v4_go_language_model_declares_wave1_carriers() {
    let module = parse_module(GO_LANGUAGE_DAG, GO_LANGUAGE_PATH);
    assert!(
        surface_declares_fn(&module, "go_wave1_primitive_fact_bundles"),
        "{GO_LANGUAGE_PATH}: must declare go_wave1_primitive_fact_bundles"
    );
    assert!(
        surface_declares_fn(&module, "go_model_core_wave1"),
        "{GO_LANGUAGE_PATH}: must declare go_model_core_wave1"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "model_core"], "ModelCore"),
        "{GO_LANGUAGE_PATH}: must import ModelCore from v4.std.model_core"
    );
}

#[test]
fn v4_mvp1_rust_add_claim_tokenizes_and_parses() {
    let _module = parse_module(MVP1_CLAIM_DAG, MVP1_CLAIM_PATH);
}

#[test]
fn v4_mvp1_rust_add_claim_imports_translate_and_emit() {
    let module = parse_module(MVP1_CLAIM_DAG, MVP1_CLAIM_PATH);
    assert!(
        import_includes_name(&module, &["v4", "compiler", "translate"], "translate"),
        "{MVP1_CLAIM_PATH}: claim must import translate stage"
    );
    assert!(
        import_includes_name(&module, &["v4", "compiler", "emit"], "emit"),
        "{MVP1_CLAIM_PATH}: claim must import emit stage"
    );
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Import { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return false;
        };
        item_path.len() == path.len()
            && item_path
                .iter()
                .zip(path.iter())
                .all(|(a, &b)| a.as_str() == b)
            && names.iter().any(|n| n == name)
    })
}

fn assert_imports_shared_token_kinds(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &str,
) {
    assert!(
        import_includes_name(
            module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_kind_fixed"
        ),
        "{path}: fixed-token rows must use the shared token-kind identity"
    );
    assert!(
        import_includes_name(
            module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_kind_bound"
        ),
        "{path}: bound-token rows must use the shared token-kind identity"
    );
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeAlias {
            name: item_name, ..
        }
        | SurfaceItem::TypeRecord {
            name: item_name, ..
        }
        | SurfaceItem::TypeSum {
            name: item_name, ..
        }
        | SurfaceItem::TypeAtom {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn type_record_fields<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a [SurfaceField] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record `{name}`"))
}

fn type_record_field_type(
    module: &v3_compiler::parse_surface::SurfaceModule,
    record_name: &str,
    field_name: &str,
) -> Option<String> {
    type_record_fields(module, record_name)
        .iter()
        .find(|field| field.name == field_name)
        .map(|field| surface_type_name(&field.ty))
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered_args = args
                .iter()
                .map(type_angle_arg_name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered_args}>")
        }
        SurfaceType::Optional { inner, .. } => format!("{}?", surface_type_name(inner)),
        SurfaceType::Arrow { inputs, output, .. } => {
            let rendered_inputs = inputs
                .iter()
                .map(surface_type_name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({rendered_inputs}) -> {}", surface_type_name(output))
        }
    }
}

fn type_angle_arg_name(arg: &TypeAngleArg) -> String {
    match arg {
        TypeAngleArg::TypeExpr { ty } => surface_type_name(ty),
        TypeAngleArg::WidthNatLiteral { decimal, .. } => decimal.clone(),
    }
}
