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
//! land in the same PR. **This PR (+0 census paths):** structural serialize-measure ratchet
//! on `06_translate.dag` via parsed-surface `fn`/`import`/`data` inventory in
//! `v4_translate_dag_dispatches_token_sequence_items`; fail-closed semantics exercised by
//! `run_mvp1_serialize_rejects_missing_translation_rules` in `mvp1_rust_add_translate.dag`
//! (not host `str::contains` probes). **PR #3798 (+0 census paths):** extends
//! `v4_python_language_model_declares_t11_translation_rules` for T-4.17 python wave-2a
//! LanguageModel / lex/grammar surface on `python.dag`. **PR #3840 (+0 census paths):**
//! adds T-11 grammar-inverse compile-inferred TestClaim parse/import receipts for
//! python/go/cpp/typescript Shape-A MVP-1 add-fn fixtures (`mvp1_*_add_translate.dag`).
//! **PR #4297 (Branch C.1–C.5).** P5 / V3-HAND-RUST-GATE receipt (INVARIANTS §P5 Mechanism (b);
//! `_internal/INVARIANTS_OPS.md`): **same-path SG-0 deferral under ROADMAP row T-PB-B /
//! `pb_rust_tests_outside_residual_zero`** (ROADMAP.md:43,63 — "keeping same-path SG-0 expansions
//! at +0 new paths until the matching claim runner executes those facts directly"). This edit is
//! **assertion-only on an already-census-listed harness**: **+0 new census paths** (the
//! `EXPECTED_HAND_AUTHORED_TEST` row for this file is unchanged), **no new or deleted hand-Rust
//! test file**, and net Rust-test line count flat-to-down. The assertions were *retargeted*, not
//! added: `v4_dag_language_model_declares_surface_emit_rows` now requires `dag.dag` to import the
//! shared carrier + `concrete_syntax_token_to_node` (no longer spelling the wire-form
//! `concrete_syntax_token_kind_*` Symbols inline), and `assert_imports_shared_token_kinds` →
//! `assert_imports_shared_token_serializer` now requires the shared-serializer import
//! (rust/java/typescript/swift/wasm). Both track the T-11 single-author serialization morphism.
//! **PR #4321 (+0 census paths):** same-path structural assertions for Go G.1.3
//! `PerLanguageFactBundleEntry` rows and the B.2.2 `parse/go_wave2a.dag` corpus symmetry file;
//! no new hand-Rust test path, and the `.dag` rows are the authored substrate.
//! **PR #4360 (+0 census paths):** same-path G.1.1 Rust fact-bundle registry expansion in
//! `v4_rust_language_model_declares_g1_fact_bundle_registry`; the Rust registry consumes the
//! landed G.0 grounding API (`primitive_fact_bundle_for_subject(target: TargetModel) -> Outcome`)
//! and re-authors `rust_model_core_wave1` / `rust_language_model_wave1` as fail-closed
//! `Outcome<..>` projections, mirroring the merged Python G.1.2 pattern. Defers to **ROADMAP.md**
//! § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`; the ratchet stays in
//! this already-census-listed harness until `.dag` `TestClaim` / generated coverage executes the
//! Rust `PerLanguageFactBundleRegistry` construction and `ModelCore.primitives` projection
//! directly. It adds no new hand-Rust test path.
//! **PR #4341 (+0 census paths):** same-path helper fix — `data_body_source` reads call/scalar
//! data RHS via parser `body_span` (unblocks `v4_kotlin_language_model_declares_wave2b_algebra_inhabitance`
//! on fn-wrapper kotlin empty-type markers). **P5 Mechanism (b) disposition (3):** explicit deferral
//! **ROADMAP.md** § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`
//! (`ROADMAP.md:43-51`); +0 `EXPECTED_HAND_AUTHORED_TEST` paths (census row unchanged).
//! Dissolution trigger (= this file's existing trigger, unchanged): retires under T-PB-B when the
//! `.dag` `TestClaim` / generated-runner replacement executes these facts directly (see the
//! **Dissolution** note below).
//! See INVARIANTS.md row `v4_compiler_emit_translate_smoke_test.rs` for the checkable receipt
//! and **ROADMAP.md** § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`.
//!
//! **Dissolution:** remove when translate/emit/MVP-1 surfaces are exercised only by `.dag`
//! `TestClaim` rows / a generated harness without this per-file Rust probe (or when
//! `compile_to_dag` over v4 compiler modules resolves imports without substrate collision).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceLiteral, SurfaceType, TypeAngleArg,
};
use v3_compiler::tokenize_for_test;

const FIND_WITNESS_DAG: &str = include_str!("../../../../v4/std/find_witness.dag");
const FIND_WITNESS_PATH: &str = "src/v4/std/find_witness.dag";
const MVP_INT_CROSS_TARGET_COERCION_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp_int_cross_target_coercion.dag");
const MVP_INT_CROSS_TARGET_COERCION_CLAIM_PATH: &str =
    "src/v4/test/claim/manual/mvp_int_cross_target_coercion.dag";
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
const KOTLIN_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/kotlin.dag");
const KOTLIN_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/kotlin.dag";
const DAG_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/dag.dag");
const DAG_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/dag.dag";
const MVP1_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_rust_add_translate.dag");
const MVP1_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_rust_add_translate.dag";
const MVP1_PYTHON_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_python_add_translate.dag");
const MVP1_PYTHON_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_python_add_translate.dag";
const MVP1_GO_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_go_add_translate.dag");
const MVP1_GO_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_go_add_translate.dag";
const GO_WAVE2A_CLAIM_DAG: &str = include_str!("../../../../v4/test/claim/parse/go_wave2a.dag");
const GO_WAVE2A_CLAIM_PATH: &str = "src/v4/test/claim/parse/go_wave2a.dag";
const MVP1_CPP_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_cpp_add_translate.dag");
const MVP1_CPP_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_cpp_add_translate.dag";
const MVP1_TYPESCRIPT_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_typescript_add_translate.dag");
const MVP1_TYPESCRIPT_CLAIM_PATH: &str =
    "src/v4/test/claim/manual/mvp1_typescript_add_translate.dag";
const MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_typescript_record_task_translate.dag");
const MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_PATH: &str =
    "src/v4/test/claim/manual/mvp1_typescript_record_task_translate.dag";
const MVP1_TYPESCRIPT_PR3_TYPED_FN_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_typescript_pr3_typed_fn_translate.dag");
const MVP1_TYPESCRIPT_PR3_TYPED_FN_CLAIM_PATH: &str =
    "src/v4/test/claim/manual/mvp1_typescript_pr3_typed_fn_translate.dag";

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
    assert!(
        surface_declares_fn(&module, "target_translation_rules_budget"),
        "{TRANSLATE_PATH}: serialize measure must derive translation_rules budget structurally"
    );
    assert!(
        surface_declares_fn(&module, "translate_serialize_measure"),
        "{TRANSLATE_PATH}: emitted-node serialize budget must be a declared structural measure"
    );
    assert!(
        surface_declares_fn(&module, "target_serialize_source_from_model"),
        "{TRANSLATE_PATH}: public serialize entry must route through structural measure helpers"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "diagnostic"], "bind_outcome"),
        "{TRANSLATE_PATH}: bounded serializers must use bind_outcome for fail-closed measure propagation"
    );
    assert!(
        !surface_declares_data(&module, "translate_default_serialize_fuel"),
        "{TRANSLATE_PATH}: fixed serialize fuel data must not remain (structural measure replaces it)"
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
    assert_imports_shared_token_serializer(&module, RUST_LANGUAGE_PATH);
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
// P5 receipt for PR #4360: same-path G.1.1 Rust fact-bundle registry ratchet in an
// already-census-listed harness (+0 new hand-Rust paths). Defers to ROADMAP.md T-PB-B /
// `pb_rust_tests_outside_residual_zero`; dissolve when `.dag` TestClaim/generated coverage
// executes Rust `PerLanguageFactBundleRegistry` construction and `ModelCore.primitives`
// projection directly.
fn v4_rust_language_model_declares_g1_fact_bundle_registry() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    for name in [
        "PerLanguageFactBundleEntry",
        "PerLanguageFactBundleKey",
        "PerLanguageFactBundleRegistry",
        "empty_per_language_fact_bundle_registry",
        "insert_per_language_fact_bundle_entry",
        "primitive_fact_bundle_for_subject",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "grounding"], name),
            "{RUST_LANGUAGE_PATH}: Rust G.1.1 fact bundle must consume v4.std.grounding::{name}"
        );
    }
    for name in [
        "ModelCoreFactAxisEncoding",
        "ModelCoreFactAxisOverflowDisposition",
        "ModelCoreFactAxisRange",
        "ModelCoreFactAxisSignedness",
        "ModelCoreFactAxisSurfaceSpelling",
        "ModelCoreFactAxisWidth",
        "ModelCorePrimitiveFactAxis",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "model_core"], name),
            "{RUST_LANGUAGE_PATH}: Rust G.1.1 registry keys must consume model_core::{name}"
        );
    }
    for name in [
        "rust_per_language_fact_bundle_key",
        "rust_per_language_fact_bundle_entry",
        "rust_integer_per_language_fact_bundle_entries",
        "rust_float_per_language_fact_bundle_entries",
        "rust_noninteger_per_language_fact_bundle_entries",
        "rust_per_language_fact_bundle_entries",
        "rust_per_language_fact_bundle_registry",
        "rust_wave1_primitive_fact_bundles_from_registry",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{RUST_LANGUAGE_PATH}: Rust G.1.1 fact bundle must declare {name}"
        );
    }
    assert_eq!(
        surface_fn_param_type(&module, "rust_per_language_fact_bundle_key", 1).as_deref(),
        Some("ModelCorePrimitiveFactAxis"),
        "{RUST_LANGUAGE_PATH}: Rust G.1.1 registry key axis must be the closed model_core coproduct, not Symbol"
    );
    assert_eq!(
        surface_fn_param_type(&module, "rust_per_language_fact_bundle_entry", 1).as_deref(),
        Some("ModelCorePrimitiveFactAxis"),
        "{RUST_LANGUAGE_PATH}: Rust G.1.1 registry entry axis must be the closed model_core coproduct, not Symbol"
    );
    assert_eq!(
        surface_fn_return_type(&module, "rust_per_language_fact_bundle_registry").as_deref(),
        Some("Outcome<PerLanguageFactBundleRegistry>"),
        "{RUST_LANGUAGE_PATH}: Rust G.1.1 registry construction must be fail-closed Outcome<PerLanguageFactBundleRegistry>"
    );
    assert_eq!(
        surface_fn_return_type(&module, "rust_model_core_wave1").as_deref(),
        Some("Outcome<ModelCore>"),
        "{RUST_LANGUAGE_PATH}: Rust ModelCore construction must preserve registry rejection as Outcome<ModelCore>"
    );
    assert_eq!(
        type_record_field_type(&module, "RustLanguageModel", "core"),
        Some("ModelCore".to_string()),
        "{RUST_LANGUAGE_PATH}: RustLanguageModel.core must be plain ModelCore; registry failure rejects the whole model"
    );
    assert_eq!(
        surface_fn_return_type(&module, "rust_language_model_wave1").as_deref(),
        Some("Outcome<RustLanguageModel>"),
        "{RUST_LANGUAGE_PATH}: Rust language model construction must fail closed as Outcome<RustLanguageModel>"
    );
    assert!(
        fn_external_body_contains(
            &module,
            RUST_LANGUAGE_DAG,
            "rust_language_model_wave1",
            "Rejected { diagnostics: d } => Rejected { diagnostics: d }"
        ) && fn_external_body_contains(
            &module,
            RUST_LANGUAGE_DAG,
            "rust_language_model_wave1",
            "core: core"
        ),
        "{RUST_LANGUAGE_PATH}: RustLanguageModel must be constructed only after accepted ModelCore"
    );
    assert!(
        fn_external_body_contains(
            &module,
            RUST_LANGUAGE_DAG,
            "rust_model_core_wave1",
            "rust_per_language_fact_bundle_registry()"
        ) && fn_external_body_contains(
            &module,
            RUST_LANGUAGE_DAG,
            "rust_model_core_wave1",
            "rust_wave1_primitive_fact_bundles_from_registry(registry: registry)"
        ),
        "{RUST_LANGUAGE_PATH}: Rust ModelCore.primitives must consume the G.1.1 Outcome registry projection"
    );
    assert!(
        fn_external_body_contains(
            &module,
            RUST_LANGUAGE_DAG,
            "rust_model_core_wave1",
            "Rejected { diagnostics: d } => Rejected { diagnostics: d }"
        ),
        "{RUST_LANGUAGE_PATH}: Rust G.1.1 rejected registry construction must fail closed at ModelCore construction"
    );
    assert!(
        !surface_declares_fn(&module, "rust_checked_per_language_fact_bundle_registry")
            && !RUST_LANGUAGE_DAG.contains("rust_per_language_fact_bundle_registry_value"),
        "{RUST_LANGUAGE_PATH}: Rust G.1.1 must not collapse rejected registry construction into a plain value registry"
    );
    assert!(
        !RUST_LANGUAGE_DAG
            .contains("Rejected { diagnostics: _ } => empty_per_language_fact_bundle_registry()"),
        "{RUST_LANGUAGE_PATH}: Rust G.1.1 rejected registry construction must not fail open to an empty registry"
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
            "ModelCoreFactAxisOverflowDisposition"
        ),
        "{RUST_LANGUAGE_PATH}: Rust must import the closed overflow-disposition primitive fact axis"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "model_core"],
            "ModelCoreFactAxisRange"
        ),
        "{RUST_LANGUAGE_PATH}: Rust must import the closed range primitive fact axis"
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
    assert_imports_shared_token_serializer(&module, JAVA_LANGUAGE_PATH);
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
        import_includes_name(&module, &["v4", "std", "grammar"], "FormalProduction"),
        "{PYTHON_LANGUAGE_PATH}: Python grammar rows must consume canonical FormalProduction"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "grammar"],
            "formal_production_to_node"
        ),
        "{PYTHON_LANGUAGE_PATH}: Python grammar rows must consume the shared FormalProduction Node projection"
    );
    // Bounded FormalProduction → GrammarExpr operational parse shim (CP-1b interim):
    // grammar rows import projection carriers only for python_formal_productions_to_grammar_expr.
    for name in [
        "GrammarExpr",
        "Sequence",
        "Terminal",
        "Choice",
        "Nonterminal",
        "Optional",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "grammar"], name),
            "{PYTHON_LANGUAGE_PATH}: operational parse shim must import `{name}` from v4.std.grammar"
        );
    }
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
    assert!(
        surface_declares_fn(&module, "python_formal_rhs_from_token_classes"),
        "{PYTHON_LANGUAGE_PATH}: must build MVP-1 grammar production RHS as a flat formal-symbol list"
    );
    assert!(
        surface_declares_fn(&module, "python_formal_productions_to_grammar_expr"),
        "{PYTHON_LANGUAGE_PATH}: must derive GrammarExpr from FormalProduction authority (operational parse shim)"
    );
    assert!(
        surface_declares_fn(&module, "python_formal_production_mvp1_fn_add"),
        "{PYTHON_LANGUAGE_PATH}: must expose FormalProduction authority for MVP-1 relation rows"
    );
    for name in [
        "python_formal_nonterminal_node",
        "python_formal_terminal_node",
        "python_formal_grammar_symbol_node",
        "python_formal_rhs_edges",
        "python_formal_production_node",
    ] {
        assert_eq!(
            surface_fn_count(&module, name),
            0,
            "{PYTHON_LANGUAGE_PATH}: Python must not mirror std FormalProduction projection helper `{name}`"
        );
    }
    assert!(
        surface_declares_type(&module, "PythonLanguageModel"),
        "{PYTHON_LANGUAGE_PATH}: must declare the LanguageModel carrier"
    );
    assert!(
        surface_declares_fn(&module, "python_language_model_wave1"),
        "{PYTHON_LANGUAGE_PATH}: must expose wave-1 LanguageModel with lex/grammar data"
    );
    assert!(
        surface_declares_fn(&module, "python_wave1_grammar"),
        "{PYTHON_LANGUAGE_PATH}: must expose ModeledGrammar for bidirectional ingest"
    );
}

#[test]
fn v4_python_language_model_declares_wave2b_algebra_inhabitance() {
    let module = parse_module(PYTHON_LANGUAGE_DAG, PYTHON_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "model_core"],
            "AlgebraInhabitanceDecl"
        ),
        "{PYTHON_LANGUAGE_PATH}: Python Wave 2b must consume ModelCore algebra inhabitance rows"
    );
    for name in [
        "ordered_ring_node",
        "approximate_field_node",
        "boolean_algebra_node",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "algebra"], name),
            "{PYTHON_LANGUAGE_PATH}: Python Wave 2b must use grounded std.algebra Node constructor `{name}`"
        );
    }
    for name in [
        "python_integer_algebra_witness_node",
        "python_float_algebra_witness_node",
        "python_bool_algebra_witness_node",
        "python_integer_algebra_inhabitance",
        "python_float_algebra_inhabitance",
        "python_bool_algebra_inhabitance",
        "python_model_core_inhabitance_decls",
        "python_model_core_wave1",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{PYTHON_LANGUAGE_PATH}: Python Wave 2b must declare `{name}`"
        );
    }
    for name in [
        "python_complex_algebra_witness_node",
        "python_singleton_algebra_witness_node",
        "python_complex_algebra_inhabitance",
        "python_singleton_algebra_inhabitance",
        "python_complex_algebra_inhabitance_decls",
        "python_singleton_algebra_inhabitance_decls",
    ] {
        assert_eq!(
            surface_fn_count(&module, name),
            0,
            "{PYTHON_LANGUAGE_PATH}: Python Wave 2b must not declare faithful algebra inhabitance for deferred complex/singleton facts via `{name}`"
        );
    }
}

#[test]
fn v4_python_language_model_declares_g1_2_fact_bundle_registry() {
    let module = parse_module(PYTHON_LANGUAGE_DAG, PYTHON_LANGUAGE_PATH);
    for name in [
        "PerLanguageFactBundleEntry",
        "PerLanguageFactBundleKey",
        "PerLanguageFactBundleRegistry",
        "empty_per_language_fact_bundle_registry",
        "insert_per_language_fact_bundle_entry",
        "primitive_fact_bundle_for_subject",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "grounding"], name),
            "{PYTHON_LANGUAGE_PATH}: Python G.1.2 fact bundle must consume `{name}` from v4.std.grounding"
        );
    }
    for name in [
        "ModelCorePrimitiveFactAxis",
        "ModelCoreFactAxisEncoding",
        "ModelCoreFactAxisSurfaceSpelling",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "model_core"], name),
            "{PYTHON_LANGUAGE_PATH}: Python G.1.2 fact bundle must consume typed model_core axis `{name}`"
        );
    }
    for name in ["Outcome", "Accepted", "Rejected", "outcome_accepted"] {
        assert!(
            import_includes_name(&module, &["v4", "std", "diagnostic"], name),
            "{PYTHON_LANGUAGE_PATH}: Python G.1.2 fact bundle registry must consume diagnostic `{name}`"
        );
    }
    for name in [
        "python_g1_2_fact_bundle_key",
        "python_g1_2_fact_bundle_entry",
        "python_g1_2_integer_fact_bundle_entries",
        "python_g1_2_float_fact_bundle_entries",
        "python_g1_2_complex_fact_bundle_entries",
        "python_g1_2_bool_fact_bundle_entries",
        "python_g1_2_singleton_fact_bundle_entries",
        "python_g1_2_fact_bundle_entries",
        "python_g1_2_insert_fact_bundle_entry",
        "python_g1_2_insert_entries",
        "python_g1_2_fact_bundle_registry",
        "python_g1_2_snoc_primitive_bundle",
        "python_g1_2_integer_primitive_bundles_from_registry",
        "python_g1_2_float_primitive_bundles_from_registry",
        "python_g1_2_complex_primitive_bundles_from_registry",
        "python_g1_2_bool_primitive_bundles_from_registry",
        "python_g1_2_singleton_primitive_bundles_from_registry",
        "python_g1_2_append_primitive_bundle_outcomes",
        "python_g1_2_primitive_bundles_from_registry",
        "python_g1_2_primitive_fact_bundles",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{PYTHON_LANGUAGE_PATH}: Python G.1.2 fact bundle must declare `{name}`"
        );
    }
    assert!(
        PYTHON_LANGUAGE_DAG.contains("core: ModelCore")
            && PYTHON_LANGUAGE_DAG.contains("fn python_model_core_wave1() -> Outcome<ModelCore>")
            && PYTHON_LANGUAGE_DAG
                .contains("fn python_language_model_wave1() -> Outcome<PythonLanguageModel>")
            && PYTHON_LANGUAGE_DAG.contains("match python_g1_2_primitive_fact_bundles()"),
        "{PYTHON_LANGUAGE_PATH}: Python ModelCore primitive facts must propagate G.1.2 registry diagnostics"
    );
    assert!(
        PYTHON_LANGUAGE_DAG.contains(
            "fn python_g1_2_primitive_bundles_from_registry(\n  registry: PerLanguageFactBundleRegistry\n) -> Outcome<List<PrimitiveFactBundle>>"
        ) && PYTHON_LANGUAGE_DAG.contains("match primitive_fact_bundle_for_subject("),
        "{PYTHON_LANGUAGE_PATH}: Python registry projection must propagate primitive bundle lookup diagnostics"
    );
    assert!(
        PYTHON_LANGUAGE_DAG.contains("match python_model_core_wave1()")
            && PYTHON_LANGUAGE_DAG.contains("Rejected { diagnostics: d } =>\n      Rejected { diagnostics: d }"),
        "{PYTHON_LANGUAGE_PATH}: Python LanguageModel construction must reject when ModelCore construction rejects"
    );
    assert!(
        PYTHON_LANGUAGE_DAG.contains("primitives: primitives")
            && !PYTHON_LANGUAGE_DAG.contains("Rejected { diagnostics: _ } =>\n      []"),
        "{PYTHON_LANGUAGE_PATH}: Python G.1.2 registry rejection must not be coerced to an empty primitive list"
    );
    for name in [
        "python_integer_spec_facts",
        "python_float_spec_facts",
        "python_complex_spec_facts",
        "python_bool_spec_facts",
        "python_singleton_spec_facts",
        "python_primitive_bundle_from_integer_facts",
        "python_primitive_bundle_from_float_facts",
        "python_primitive_bundle_from_complex_facts",
        "python_primitive_bundle_from_bool_facts",
        "python_primitive_bundle_from_singleton_facts",
        "python_wave1_primitive_fact_bundles",
    ] {
        assert_eq!(
            surface_fn_count(&module, name),
            0,
            "{PYTHON_LANGUAGE_PATH}: Python G.1.2 must not retain legacy direct PrimitiveFactBundle builder `{name}`"
        );
    }
    assert_eq!(
        surface_fn_count(&module, "python_g1_2_concrete_syntax_token"),
        0,
        "{PYTHON_LANGUAGE_PATH}: Python G.1.2 must not revive target-local concrete syntax token helpers"
    );
    assert_eq!(
        surface_declares_type(&module, "PythonConcreteSyntaxToken"),
        false,
        "{PYTHON_LANGUAGE_PATH}: Python G.1.2 must consume shared ConcreteSyntaxToken authority"
    );
}

#[test]
fn v4_kotlin_language_model_declares_wave2b_algebra_inhabitance() {
    let module = parse_module(KOTLIN_LANGUAGE_DAG, KOTLIN_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "model_core"],
            "AlgebraInhabitanceDecl"
        ),
        "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must consume ModelCore algebra inhabitance rows"
    );
    for name in [
        "ordered_ring_node",
        "approximate_field_node",
        "boolean_algebra_node",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "algebra"], name),
            "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must use grounded std.algebra Node constructor `{name}`"
        );
    }
    for name in [
        "kotlin_ordered_ring_int_facts",
        "kotlin_ordered_ring_long_facts",
        "kotlin_integer_algebra_witness_node",
        "kotlin_float_algebra_witness_node",
        "kotlin_bool_algebra_witness_node",
        "kotlin_integer_algebra_inhabitance",
        "kotlin_float_algebra_inhabitance",
        "kotlin_bool_algebra_inhabitance",
        "kotlin_model_core_inhabitance_decls",
        "kotlin_model_core_wave1",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must declare `{name}`"
        );
    }
    assert!(
        surface_declares_type(&module, "KotlinOrderedRingIntegerFacts"),
        "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must declare algebra-closed integer inhabitance carrier `KotlinOrderedRingIntegerFacts`"
    );
    assert!(
        surface_declares_type(&module, "KotlinOrderedRingIntWidth"),
        "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must declare algebra-closed integer width coproduct `KotlinOrderedRingIntWidth`"
    );
    assert!(
        type_sum_has_variant(
            &module,
            "KotlinOrderedRingIntegerFacts",
            "KotlinOrderedRingIntFacts"
        ),
        "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must declare closed Int ordered-ring variant `KotlinOrderedRingIntFacts`"
    );
    assert!(
        type_sum_has_variant(
            &module,
            "KotlinOrderedRingIntegerFacts",
            "KotlinOrderedRingLongFacts"
        ),
        "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must declare closed Long ordered-ring variant `KotlinOrderedRingLongFacts`"
    );
    for name in [
        "kotlin_ordered_ring_integer_from_int_primitive_facts",
        "kotlin_ordered_ring_integer_from_long_primitive_facts",
    ] {
        assert_eq!(
            surface_fn_count(&module, name),
            0,
            "{KOTLIN_LANGUAGE_PATH}: must not expose broad `KotlinIntegerPrimitiveFacts` ordered-ring constructors via `{name}`"
        );
    }
    assert_eq!(
        surface_fn_first_param_named_type(&module, "kotlin_integer_algebra_inhabitance"),
        Some("KotlinOrderedRingIntegerFacts".to_string()),
        "{KOTLIN_LANGUAGE_PATH}: `kotlin_integer_algebra_inhabitance` must accept only `KotlinOrderedRingIntegerFacts` (Byte/Short excluded at API level)"
    );
    assert_eq!(
        data_list_element_var_names(&module, KOTLIN_LANGUAGE_DAG, "kotlin_integer_algebra_inhabitance_facts_catalog"),
        vec![
            "kotlin_ordered_ring_facts_int".to_string(),
            "kotlin_ordered_ring_facts_long".to_string(),
        ],
        "{KOTLIN_LANGUAGE_PATH}: algebra inhabitance catalog must name closed Int/Long ordered-ring rows"
    );
    assert!(
        data_body_source_contains(
            &module,
            KOTLIN_LANGUAGE_DAG,
            "kotlin_ordered_ring_facts_int",
            "kotlin_ordered_ring_int_facts()"
        ) && data_body_source_contains(
            &module,
            KOTLIN_LANGUAGE_DAG,
            "kotlin_ordered_ring_facts_long",
            "kotlin_ordered_ring_long_facts()"
        ),
        "{KOTLIN_LANGUAGE_PATH}: closed ordered-ring rows must be built only via zero-arg Int/Long constructors"
    );
    assert!(
        !data_body_source_contains(
            &module,
            KOTLIN_LANGUAGE_DAG,
            "kotlin_integer_algebra_inhabitance_facts_catalog",
            "kotlin_facts_byte"
        ) && !data_body_source_contains(
            &module,
            KOTLIN_LANGUAGE_DAG,
            "kotlin_integer_algebra_inhabitance_facts_catalog",
            "kotlin_facts_short"
        ),
        "{KOTLIN_LANGUAGE_PATH}: algebra inhabitance catalog must exclude Byte/Short primitive facts"
    );
    assert_eq!(
        data_named_type(&module, "kotlin_integer_algebra_inhabitance_facts_catalog"),
        Some("List<KotlinOrderedRingIntegerFacts>".to_string()),
        "{KOTLIN_LANGUAGE_PATH}: algebra inhabitance catalog must be typed as `List<KotlinOrderedRingIntegerFacts>`"
    );
    assert!(
        fn_external_body_contains(
            &module,
            KOTLIN_LANGUAGE_DAG,
            "kotlin_model_core_wave1",
            "inhabitance: kotlin_model_core_inhabitance_decls()"
        ),
        "{KOTLIN_LANGUAGE_PATH}: `kotlin_model_core_wave1` must wire `ModelCore.inhabitance` through `kotlin_model_core_inhabitance_decls()`"
    );
    for name in [
        "kotlin_char_algebra_witness_node",
        "kotlin_string_algebra_witness_node",
        "kotlin_unit_algebra_witness_node",
        "kotlin_nothing_algebra_witness_node",
        "kotlin_char_algebra_inhabitance",
        "kotlin_string_algebra_inhabitance",
        "kotlin_unit_algebra_inhabitance",
        "kotlin_nothing_algebra_inhabitance",
        "kotlin_char_algebra_inhabitance_decls",
        "kotlin_string_algebra_inhabitance_decls",
        "kotlin_unit_algebra_inhabitance_decls",
        "kotlin_nothing_algebra_inhabitance_decls",
    ] {
        assert_eq!(
            surface_fn_count(&module, name),
            0,
            "{KOTLIN_LANGUAGE_PATH}: Kotlin Wave 2b must not declare faithful algebra inhabitance for deferred char/string/unit/nothing facts via `{name}`"
        );
    }
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
    assert_imports_shared_token_serializer(&module, TYPESCRIPT_LANGUAGE_PATH);
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
    assert_imports_shared_token_serializer(&module, SWIFT_LANGUAGE_PATH);
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
fn v4_wasm_language_model_declares_wave2b_algebra_inhabitance() {
    let module = parse_module(WASM_LANGUAGE_DAG, WASM_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "model_core"],
            "AlgebraInhabitanceDecl"
        ),
        "{WASM_LANGUAGE_PATH}: Wasm Wave 2b must consume ModelCore algebra inhabitance rows"
    );
    for name in [
        "commutative_semiring_type_node",
        "approximate_field_type_node",
    ] {
        assert!(
            import_includes_name(&module, &["v4", "std", "algebra"], name),
            "{WASM_LANGUAGE_PATH}: Wasm Wave 2b must use grounded std.algebra Node constructor `{name}`"
        );
    }
    for name in [
        "wasm_integer_algebra_witness_node",
        "wasm_float_algebra_witness_node",
        "wasm_integer_algebra_inhabitance",
        "wasm_float_algebra_inhabitance",
        "wasm_model_core_inhabitance_decls",
        "wasm_model_core_wave2b",
        "wasm_language_model_wave2b",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{WASM_LANGUAGE_PATH}: Wasm Wave 2b must declare `{name}`"
        );
    }
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
    assert_imports_shared_token_serializer(&module, WASM_LANGUAGE_PATH);
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
fn v4_wasm_wat_lex_boundary_uses_trivia_and_numeric_literal_authority() {
    let module = parse_module(WASM_LANGUAGE_DAG, WASM_LANGUAGE_PATH);
    for stale in [
        "wasm_token_param_list",
        "wasm_token_result",
        "wasm_token_local_idx_0",
        "wasm_token_local_idx_1",
    ] {
        assert!(
            !surface_declares_data(&module, stale),
            "{WASM_LANGUAGE_PATH}: stale whitespace-bearing fixture token `{stale}` must be dissolved"
        );
    }
    for required in [
        "wasm_token_kw_func",
        "wasm_token_kw_param",
        "wasm_token_kw_result",
        "wasm_token_local_get",
        "wasm_token_int_numeric_literal",
        "wasm_token_float_numeric_literal",
        "wasm_token_localidx_literal",
        "wasm_token_i32_numeric_literal",
        "wasm_token_i64_numeric_literal",
        "wasm_token_f32_numeric_literal",
        "wasm_token_f64_numeric_literal",
        "wasm_binding_localidx_0",
        "wasm_binding_localidx_1",
        "wasm_numeric_immediate_validation_i32_range",
        "wasm_numeric_immediate_validation_i64_range",
        "wasm_numeric_immediate_validation_f32_payload",
        "wasm_numeric_immediate_validation_f64_payload",
        "wasm_production_i32_numeric_literal",
        "wasm_production_i64_numeric_literal",
        "wasm_production_f32_numeric_literal",
        "wasm_production_f64_numeric_literal",
        "wasm_production_localidx",
        "wasm_surface_localidx",
        "wasm_localidx_validation_u32_range",
    ] {
        assert!(
            surface_declares_data(&module, required),
            "{WASM_LANGUAGE_PATH}: WAT lex/grammar boundary must expose `{required}`"
        );
    }
    for stale_literal in [
        "func ",
        " (param i32 i32) ",
        "(result i32) ",
        "local.get ",
        "0 ",
        "1 ",
        "i32.const ",
        "i64.const ",
        "f32.const ",
        "f64.const ",
    ] {
        assert!(
            !module_string_literal_eq(&module, WASM_LANGUAGE_DAG, stale_literal),
            "{WASM_LANGUAGE_PATH}: WAT token literal `{stale_literal}` must not embed whitespace"
        );
    }
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
    // T-11: self-emission constructs the typed ConcreteSyntaxToken carrier and projects it
    // through the single shared serializer, rather than re-spelling the wire-form field/kind
    // Symbols inline. The carrier + serializer are the shared authority emit rows depend on.
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "FixedToken"
        ),
        "{DAG_LANGUAGE_PATH}: fixed-token rows must construct the shared ConcreteSyntaxToken carrier"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "BoundToken"
        ),
        "{DAG_LANGUAGE_PATH}: bound-token rows must construct the shared ConcreteSyntaxToken carrier"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_to_node"
        ),
        "{DAG_LANGUAGE_PATH}: token rows must serialize through the single shared token morphism"
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
fn v4_go_language_model_declares_g1_3_fact_bundle_entries() {
    let module = parse_module(GO_LANGUAGE_DAG, GO_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "grounding"],
            "PerLanguageFactBundleEntry"
        ),
        "{GO_LANGUAGE_PATH}: G.1.3 Go fact rows must consume the G.0 PerLanguageFactBundleEntry schema"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "grounding"],
            "PerLanguageFactBundleRegistry"
        ),
        "{GO_LANGUAGE_PATH}: G.1.3 Go fact rows must expose the keyed G.0 registry authority"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "grounding"],
            "insert_per_language_fact_bundle_entry"
        ),
        "{GO_LANGUAGE_PATH}: G.1.3 Go fact rows must populate through the fail-closed registry insert"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "grounding"],
            "PerLanguageFactBundleKey"
        ),
        "{GO_LANGUAGE_PATH}: G.1.3 Go fact rows must key by substrate carrier, TargetModel, and fact axis"
    );
    for name in [
        "go_per_language_fact_bundle_entry",
        "go_integer_per_language_fact_bundle_entries",
        "go_float_per_language_fact_bundle_entries",
        "go_complex_per_language_fact_bundle_entries",
        "go_bool_per_language_fact_bundle_entries",
        "go_string_per_language_fact_bundle_entries",
        "go_g1_3_per_language_fact_bundle_entry_rows",
        "go_g1_3_per_language_fact_bundle_registry_step",
        "go_g1_3_per_language_fact_bundle_registry",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{GO_LANGUAGE_PATH}: must declare G.1.3 Go fact-bundle row builder `{name}`"
        );
    }
    assert!(
        surface_fn_param_type(&module, "go_per_language_fact_bundle_key", 1)
            .as_deref()
            == Some("ModelCorePrimitiveFactAxis")
            && surface_fn_param_type(&module, "go_per_language_fact_bundle_entry", 1)
                .as_deref()
                == Some("ModelCorePrimitiveFactAxis")
            && surface_fn_return_type(&module, "go_per_language_fact_bundle_key").as_deref()
                == Some("PerLanguageFactBundleKey")
            && surface_fn_return_type(&module, "go_per_language_fact_bundle_entry").as_deref()
                == Some("PerLanguageFactBundleEntry"),
        "{GO_LANGUAGE_PATH}: G.1.3 keys must type fact axes as the closed G.0 ModelCorePrimitiveFactAxis carrier"
    );
    assert!(
        ["subject_carrier", "go_mvp1_target_model", "fact_axis"]
            .iter()
            .all(|needle| surface_fn_body_mentions_name(
                &module,
                GO_LANGUAGE_DAG,
                "go_per_language_fact_bundle_key",
                needle
            )),
        "{GO_LANGUAGE_PATH}: G.1.3 key builder must bind subject, Go target, and closed fact-axis fields in its body"
    );
    for (fn_name, axes) in [
        (
            "go_integer_per_language_fact_bundle_entries",
            &[
                "ModelCoreFactAxisSurfaceSpelling",
                "ModelCoreFactAxisWidth",
                "ModelCoreFactAxisSignedness",
                "ModelCoreFactAxisOverflowDisposition",
                "ModelCoreFactAxisEncoding",
            ][..],
        ),
        (
            "go_float_per_language_fact_bundle_entries",
            &[
                "ModelCoreFactAxisSurfaceSpelling",
                "ModelCoreFactAxisWidth",
                "ModelCoreFactAxisEncoding",
            ][..],
        ),
        (
            "go_complex_per_language_fact_bundle_entries",
            &[
                "ModelCoreFactAxisSurfaceSpelling",
                "ModelCoreFactAxisWidth",
                "ModelCoreFactAxisEncoding",
            ][..],
        ),
        (
            "go_bool_per_language_fact_bundle_entries",
            &[
                "ModelCoreFactAxisSurfaceSpelling",
                "ModelCoreFactAxisEncoding",
            ][..],
        ),
        (
            "go_string_per_language_fact_bundle_entries",
            &[
                "ModelCoreFactAxisSurfaceSpelling",
                "ModelCoreFactAxisEncoding",
            ][..],
        ),
    ] {
        assert!(
            axes.iter().all(|axis| surface_fn_body_mentions_name(
                &module,
                GO_LANGUAGE_DAG,
                fn_name,
                axis
            )),
            "{GO_LANGUAGE_PATH}: `{fn_name}` must key Go facts through the closed model_core axes"
        );
    }
    assert!(
        surface_fn_param_type(&module, "go_g1_3_per_language_fact_bundle_registry_step", 0)
            .as_deref()
            == Some("Outcome<PerLanguageFactBundleRegistry>")
            && surface_fn_param_type(&module, "go_g1_3_per_language_fact_bundle_registry_step", 1)
                .as_deref()
                == Some("PerLanguageFactBundleEntry")
            && surface_fn_return_type(&module, "go_g1_3_per_language_fact_bundle_registry_step")
                .as_deref()
                == Some("Outcome<PerLanguageFactBundleRegistry>")
            && surface_fn_return_type(&module, "go_g1_3_per_language_fact_bundle_registry")
                .as_deref()
                == Some("Outcome<PerLanguageFactBundleRegistry>"),
        "{GO_LANGUAGE_PATH}: G.1.3 registry builders must expose the fail-closed Outcome<PerLanguageFactBundleRegistry> surface"
    );
    assert!(
        surface_fn_body_mentions_name(
            &module,
            GO_LANGUAGE_DAG,
            "go_g1_3_per_language_fact_bundle_registry_step",
            "insert_per_language_fact_bundle_entry"
        ) && [
            "go_g1_3_per_language_fact_bundle_entry_rows",
            "outcome_accepted",
            "empty_per_language_fact_bundle_registry",
            "go_g1_3_per_language_fact_bundle_registry_step",
        ]
        .iter()
        .all(|needle| surface_fn_body_mentions_name(
            &module,
            GO_LANGUAGE_DAG,
            "go_g1_3_per_language_fact_bundle_registry",
            needle
        )),
        "{GO_LANGUAGE_PATH}: canonical G.1.3 Go registry builder must fold rows through the fail-closed insert step"
    );
}

#[test]
fn v4_go_wave2a_parse_claim_tokenizes_and_parses() {
    let _module = parse_module(GO_WAVE2A_CLAIM_DAG, GO_WAVE2A_CLAIM_PATH);
}

#[test]
fn v4_go_wave2a_parse_claim_is_mvp_scoped_symmetry_row() {
    let module = parse_module(GO_WAVE2A_CLAIM_DAG, GO_WAVE2A_CLAIM_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "extdeps", "languages", "go"],
            "go_mvp1_source_text"
        ) && import_includes_name(
            &module,
            &["v4", "extdeps", "languages", "go"],
            "go_wave1_lex"
        ) && import_includes_name(
            &module,
            &["v4", "extdeps", "languages", "go"],
            "go_wave1_grammar"
        ),
        "{GO_WAVE2A_CLAIM_PATH}: B.2.2 Go parse row must use the current MVP Go source and wave1 lex/grammar authority"
    );
    assert!(
        surface_declares_data(&module, "claim_go_wave2a_mvp_add_function_parses"),
        "{GO_WAVE2A_CLAIM_PATH}: must declare the missing Go parse/ wave2a corpus TestClaim"
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

#[test]
fn v4_mvp1_python_add_claim_tokenizes_and_parses() {
    let _module = parse_module(MVP1_PYTHON_CLAIM_DAG, MVP1_PYTHON_CLAIM_PATH);
}

#[test]
fn v4_mvp_int_cross_target_coercion_claim_tokenizes_and_parses() {
    let _module = parse_module(
        MVP_INT_CROSS_TARGET_COERCION_CLAIM_DAG,
        MVP_INT_CROSS_TARGET_COERCION_CLAIM_PATH,
    );
}

#[test]
fn v4_mvp1_go_add_claim_tokenizes_and_parses() {
    let _module = parse_module(MVP1_GO_CLAIM_DAG, MVP1_GO_CLAIM_PATH);
}

#[test]
fn v4_mvp1_cpp_add_claim_tokenizes_and_parses() {
    let _module = parse_module(MVP1_CPP_CLAIM_DAG, MVP1_CPP_CLAIM_PATH);
}

#[test]
fn v4_mvp1_typescript_add_claim_tokenizes_and_parses() {
    let _module = parse_module(MVP1_TYPESCRIPT_CLAIM_DAG, MVP1_TYPESCRIPT_CLAIM_PATH);
}

#[test]
fn v4_mvp1_typescript_record_task_claim_tokenizes_and_parses() {
    let _module = parse_module(
        MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_DAG,
        MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_PATH,
    );
}

#[test]
fn v4_mvp1_shape_a_add_claims_import_compile_inferred() {
    for (source, path) in [
        (MVP1_PYTHON_CLAIM_DAG, MVP1_PYTHON_CLAIM_PATH),
        (MVP1_GO_CLAIM_DAG, MVP1_GO_CLAIM_PATH),
        (MVP1_CPP_CLAIM_DAG, MVP1_CPP_CLAIM_PATH),
        (MVP1_TYPESCRIPT_CLAIM_DAG, MVP1_TYPESCRIPT_CLAIM_PATH),
    ] {
        let module = parse_module(source, path);
        assert!(
            import_includes_name(&module, &["v4", "compiler", "compile"], "compile_inferred"),
            "{path}: grammar-inverse claim must import compile_inferred"
        );
        assert!(
            import_includes_name(&module, &["v4", "compiler", "emit"], "emit"),
            "{path}: grammar-inverse claim must import emit stage"
        );
    }
}

#[test]
fn v4_mvp1_typescript_grammar_inverse_claims_name_l0_productions() {
    let add_module = parse_module(MVP1_TYPESCRIPT_CLAIM_DAG, MVP1_TYPESCRIPT_CLAIM_PATH);
    assert!(
        import_includes_name(
            &add_module,
            &["v4", "extdeps", "languages", "typescript"],
            "ts_production_mvp1_fn_add"
        ),
        "{MVP1_TYPESCRIPT_CLAIM_PATH}: G1 must import the MVP-1 add-fn production authority"
    );
    assert!(
        data_body_var_name_equals(
            &add_module,
            "mvp1_ts_g1_grammar_inverse_production",
            "ts_production_mvp1_fn_add"
        ),
        "{MVP1_TYPESCRIPT_CLAIM_PATH}: G1 anchor must bind ts_production_mvp1_fn_add"
    );

    let task_module = parse_module(
        MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_DAG,
        MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_PATH,
    );
    for production in [
        "ts_production_wave2a_type_alias_decl",
        "ts_production_wave2a_type_annotation",
        "ts_production_wave2a_record_type",
    ] {
        assert!(
            import_includes_name(
                &task_module,
                &["v4", "extdeps", "languages", "typescript"],
                production
            ),
            "{MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_PATH}: G2 must import {production}"
        );
    }
    for (anchor, expected) in [
        (
            "mvp1_ts_task_g2_grammar_inverse_production",
            "ts_production_wave2a_type_alias_decl",
        ),
        (
            "mvp1_ts_task_g2_type_annotation_production",
            "ts_production_wave2a_type_annotation",
        ),
        (
            "mvp1_ts_task_g2_record_type_production",
            "ts_production_wave2a_record_type",
        ),
    ] {
        assert!(
            data_body_var_name_equals(&task_module, anchor, expected),
            "{MVP1_TYPESCRIPT_RECORD_TASK_CLAIM_PATH}: G2 anchor {anchor} must bind {expected}"
        );
    }
}

#[test]
fn v4_mvp1_typescript_grammar_inverse_claims_name_g3_productions() {
    let pr3_module = parse_module(
        MVP1_TYPESCRIPT_PR3_TYPED_FN_CLAIM_DAG,
        MVP1_TYPESCRIPT_PR3_TYPED_FN_CLAIM_PATH,
    );
    for production in [
        "ts_production_pr3_typed_fn_decl",
        "ts_production_pr3_typed_param",
        "ts_production_wave2a_type_annotation",
        "ts_production_wave2a_type_number",
    ] {
        assert!(
            import_includes_name(
                &pr3_module,
                &["v4", "extdeps", "languages", "typescript"],
                production
            ),
            "{MVP1_TYPESCRIPT_PR3_TYPED_FN_CLAIM_PATH}: G3 must import {production}"
        );
    }
    for (anchor, expected) in [
        (
            "mvp1_ts_pr3_g3_grammar_inverse_production",
            "ts_production_pr3_typed_fn_decl",
        ),
        (
            "mvp1_ts_pr3_g3_type_annotation_production",
            "ts_production_wave2a_type_annotation",
        ),
        (
            "mvp1_ts_pr3_g3_typed_param_production",
            "ts_production_pr3_typed_param",
        ),
    ] {
        assert!(
            data_body_var_name_equals(&pr3_module, anchor, expected),
            "{MVP1_TYPESCRIPT_PR3_TYPED_FN_CLAIM_PATH}: G3 anchor {anchor} must bind {expected}"
        );
    }
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

fn assert_imports_shared_token_serializer(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &str,
) {
    // T-11: per-language `*_concrete_token_node` serialize through the single shared morphism
    // `concrete_syntax_token_to_node` rather than hand-spelling the wire-form kind/field Symbols
    // inline — so the Conj wire shape has exactly one author (INVARIANTS P2).
    assert!(
        import_includes_name(
            module,
            &["v4", "std", "target_model"],
            "concrete_syntax_token_to_node"
        ),
        "{path}: token serialization must project through the shared concrete_syntax_token_to_node morphism"
    );
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    surface_fn_count(module, name) > 0
}

fn surface_fn_count(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> usize {
    module
        .items
        .iter()
        .filter(|item| match item {
            SurfaceItem::Fn {
                name: item_name, ..
            }
            | SurfaceItem::FnExternalBody {
                name: item_name, ..
            } => item_name == name,
            _ => false,
        })
        .count()
}

fn surface_fn_first_param_named_type(
    module: &v3_compiler::parse_surface::SurfaceModule,
    fn_name: &str,
) -> Option<String> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Fn { name, params, .. } | SurfaceItem::FnExternalBody { name, params, .. }
            if name == fn_name =>
        {
            params.first().and_then(|param| match &param.ty {
                SurfaceType::Named { name, .. } => Some(name.clone()),
                _ => None,
            })
        }
        _ => None,
    })
}

fn surface_fn_param_type(
    module: &v3_compiler::parse_surface::SurfaceModule,
    fn_name: &str,
    index: usize,
) -> Option<String> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Fn { name, params, .. } | SurfaceItem::FnExternalBody { name, params, .. }
            if name == fn_name =>
        {
            params.get(index).map(|param| surface_type_name(&param.ty))
        }
        _ => None,
    })
}

fn surface_fn_return_type(
    module: &v3_compiler::parse_surface::SurfaceModule,
    fn_name: &str,
) -> Option<String> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Fn {
            name, return_type, ..
        }
        | SurfaceItem::FnExternalBody {
            name, return_type, ..
        } if name == fn_name => Some(surface_type_name(return_type)),
        _ => None,
    })
}

fn data_named_type(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Option<String> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Data {
            name: item_name,
            ty,
            ..
        } if item_name == name => Some(surface_type_name(ty)),
        _ => None,
    })
}

fn data_body_source<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    source: &'a str,
    name: &str,
) -> Option<&'a str> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Data {
            name: item_name,
            body: Some(body),
            body_span,
            ..
        } if item_name == name => Some(match body {
            SurfaceExpr::List { .. } | SurfaceExpr::Record { .. } => {
                source_span_text(source, &body.span())
            }
            _ => source_span_text(source, body_span),
        }),
        SurfaceItem::Data {
            name: item_name,
            body: None,
            body_span,
            ..
        } if item_name == name => Some(source_span_text(source, body_span)),
        _ => None,
    })
}

fn data_body_source_contains(
    module: &v3_compiler::parse_surface::SurfaceModule,
    source: &str,
    name: &str,
    needle: &str,
) -> bool {
    data_body_source(module, source, name).is_some_and(|body| body.contains(needle))
}

fn data_body_var_name_equals(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
    expected: &str,
) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::Data {
                name: item_name,
                body: Some(SurfaceExpr::Var { name: var_name, .. }),
                ..
            } if item_name == name && var_name == expected
        )
    })
}

fn data_list_element_surface_text(source: &str, element: &SurfaceExpr) -> String {
    match element {
        SurfaceExpr::Var { name, .. } => name.clone(),
        _ => source_span_text(source, &element.span()).trim().to_string(),
    }
}

fn data_list_element_var_names(
    module: &v3_compiler::parse_surface::SurfaceModule,
    source: &str,
    name: &str,
) -> Vec<String> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body: Some(SurfaceExpr::List { elements, .. }),
                ..
            } if item_name == name => Some(
                elements
                    .iter()
                    .map(|element| data_list_element_surface_text(source, element))
                    .collect(),
            ),
            SurfaceItem::Data {
                name: item_name,
                body: None,
                body_span,
                ..
            } if item_name == name => {
                let body = source_span_text(source, body_span);
                Some(
                    body.split(',')
                        .map(str::trim)
                        .filter(|line| !line.is_empty() && *line != "[" && *line != "]")
                        .map(|line| line.trim_end_matches(',').to_string())
                        .collect(),
                )
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data list body `{name}`"))
}

fn fn_external_body_contains(
    module: &v3_compiler::parse_surface::SurfaceModule,
    source: &str,
    fn_name: &str,
    needle: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::FnExternalBody {
            name, body_span, ..
        } if name == fn_name => source_span_text(source, body_span).contains(needle),
        SurfaceItem::Fn { name, body, .. } if name == fn_name => match body {
            SurfaceExpr::Record { fields, .. } => fields
                .iter()
                .any(|field| expr_source_contains(&field.value, needle)),
            _ => source_span_text(source, &body.span()).contains(needle),
        },
        _ => false,
    })
}

fn surface_fn_body_mentions_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    source: &str,
    fn_name: &str,
    needle: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn { name, body, .. } if name == fn_name => expr_mentions_name(body, needle),
        SurfaceItem::FnExternalBody {
            name, body_span, ..
        } if name == fn_name => source_span_without_comments(source, body_span)
            .split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
            .any(|token| token == needle),
        _ => false,
    })
}

fn source_span_without_comments(source: &str, span: &v3_compiler::SourceSpan) -> String {
    strip_dag_comments(source_span_text(source, span))
}

fn strip_dag_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '/' {
            match chars.peek().copied() {
                Some('/') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if next == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                }
                Some('*') => {
                    chars.next();
                    let mut prev = '\0';
                    for next in chars.by_ref() {
                        if prev == '*' && next == '/' {
                            break;
                        }
                        prev = next;
                    }
                }
                _ => out.push(ch),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn expr_mentions_name(expr: &SurfaceExpr, needle: &str) -> bool {
    match expr {
        SurfaceExpr::Literal { .. } => false,
        SurfaceExpr::Var { name, .. } => name == needle,
        SurfaceExpr::Path { segments, .. } => segments.iter().any(|segment| segment == needle),
        SurfaceExpr::Call { target, args, .. } => {
            target == needle || args.iter().any(|arg| expr_mentions_name(arg, needle))
        }
        SurfaceExpr::PathCall { segments, args, .. } => {
            segments.iter().any(|segment| segment == needle)
                || args.iter().any(|arg| expr_mentions_name(arg, needle))
        }
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            target == needle
                || fields
                    .iter()
                    .any(|field| expr_mentions_name(&field.value, needle))
        }
        SurfaceExpr::Operator { args, .. } => {
            args.iter().any(|arg| expr_mentions_name(arg, needle))
        }
        SurfaceExpr::Lambda { body, .. } => expr_mentions_name(body, needle),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_mentions_name(cond, needle)
                || expr_mentions_name(then_branch, needle)
                || expr_mentions_name(else_branch, needle)
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            expr_mentions_name(scrutinee, needle)
                || arms.iter().any(|arm| expr_mentions_name(&arm.body, needle))
        }
        SurfaceExpr::Record { fields, .. } => fields
            .iter()
            .any(|field| field.name == needle || expr_mentions_name(&field.value, needle)),
        SurfaceExpr::List { elements, .. } => elements
            .iter()
            .any(|element| expr_mentions_name(element, needle)),
        SurfaceExpr::Map { entries, .. } => entries
            .iter()
            .any(|entry| entry.key == needle || expr_mentions_name(&entry.value, needle)),
    }
}

fn source_span_text<'a>(source: &'a str, span: &v3_compiler::SourceSpan) -> &'a str {
    source
        .get(span.byte_start as usize..span.byte_end as usize)
        .unwrap_or_else(|| panic!("invalid source span {}..{}", span.byte_start, span.byte_end))
}

fn expr_source_contains(expr: &SurfaceExpr, needle: &str) -> bool {
    match expr {
        SurfaceExpr::Var { name, .. } => name.contains(needle),
        SurfaceExpr::Call { target, args, .. } => {
            target.contains(needle) || args.iter().any(|arg| expr_source_contains(arg, needle))
        }
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => fields
            .iter()
            .any(|field| expr_source_contains(&field.value, needle)),
        _ => false,
    }
}

trait SurfaceExprSpan {
    fn span(&self) -> v3_compiler::SourceSpan;
}

impl SurfaceExprSpan for SurfaceExpr {
    fn span(&self) -> v3_compiler::SourceSpan {
        match self {
            SurfaceExpr::Literal { span, .. }
            | SurfaceExpr::Var { span, .. }
            | SurfaceExpr::Path { span, .. }
            | SurfaceExpr::Call { span, .. }
            | SurfaceExpr::PathCall { span, .. }
            | SurfaceExpr::VariantRecord { span, .. }
            | SurfaceExpr::Operator { span, .. }
            | SurfaceExpr::Lambda { span, .. }
            | SurfaceExpr::If { span, .. }
            | SurfaceExpr::Match { span, .. }
            | SurfaceExpr::Record { span, .. }
            | SurfaceExpr::List { span, .. }
            | SurfaceExpr::Map { span, .. } => span.clone(),
        }
    }
}

fn surface_declares_data(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Data {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn module_string_literal_eq(
    module: &v3_compiler::parse_surface::SurfaceModule,
    source: &str,
    expected: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Let { expr, .. } => expr_string_literal_eq(expr, expected),
        SurfaceItem::Fn { body, .. } => expr_string_literal_eq(body, expected),
        SurfaceItem::FnExternalBody { body_span, .. } => {
            source_span_string_literal_eq(source, body_span, expected)
        }
        SurfaceItem::Data {
            body: Some(expr), ..
        } => expr_string_literal_eq(expr, expected),
        SurfaceItem::Data {
            body: None,
            body_span,
            ..
        } => source_span_string_literal_eq(source, body_span, expected),
        _ => false,
    })
}

fn source_span_string_literal_eq(
    source: &str,
    span: &v3_compiler::SourceSpan,
    expected: &str,
) -> bool {
    let Some(body) = source.get(span.byte_start as usize..span.byte_end as usize) else {
        return false;
    };
    body.contains(&dag_string_literal(expected))
}

fn dag_string_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn expr_string_literal_eq(expr: &SurfaceExpr, expected: &str) -> bool {
    match expr {
        SurfaceExpr::Literal {
            value: SurfaceLiteral::String(value),
            ..
        } => value == expected,
        SurfaceExpr::Call { args, .. }
        | SurfaceExpr::PathCall { args, .. }
        | SurfaceExpr::Operator { args, .. } => {
            args.iter().any(|arg| expr_string_literal_eq(arg, expected))
        }
        SurfaceExpr::Lambda { body, .. } => expr_string_literal_eq(body, expected),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_string_literal_eq(cond, expected)
                || expr_string_literal_eq(then_branch, expected)
                || expr_string_literal_eq(else_branch, expected)
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            expr_string_literal_eq(scrutinee, expected)
                || arms
                    .iter()
                    .any(|arm| expr_string_literal_eq(&arm.body, expected))
        }
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => fields
            .iter()
            .any(|field| expr_string_literal_eq(&field.value, expected)),
        SurfaceExpr::List { elements, .. } => elements
            .iter()
            .any(|element| expr_string_literal_eq(element, expected)),
        SurfaceExpr::Map { entries, .. } => entries
            .iter()
            .any(|entry| expr_string_literal_eq(&entry.value, expected)),
        _ => false,
    }
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

fn type_sum_has_variant(
    module: &v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
    variant_name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeSum { name, variants, .. } if name == type_name => {
            variants.iter().any(|variant| variant.name == variant_name)
        }
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
