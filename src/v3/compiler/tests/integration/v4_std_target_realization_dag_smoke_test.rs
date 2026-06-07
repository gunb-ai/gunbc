//! **Layer:** integration
//!
//! **P5 receipt (INVARIANTS.md §P5 per-PR gate — SG-0 `EXPECTED_HAND_AUTHORED_TEST` same-path
//! expansion):** explicit deferral to **`ROADMAP.md`** § **Public Operational Lanes** row
//! **T-PB-B** / `pb_rust_tests_outside_residual_zero` (tests-as-data / Pure Bootstrap test
//! floor), same
//! structural class as co-listed `t_gate_58_apply_lens_self_application_test.rs`: parse-surface
//! smokes discharge until `.dag` `TestClaim` asserts the same substrate facts without this file.
//! **Mechanism (b):** matching `EXPECTED_HAND_AUTHORED_TEST` line in `sg0_census_test.rs` +
//! `_internal/INVARIANTS_OPS.md` row land in the same PR. **+0 SG-0 paths** (no new census entry).
//! SG-RC ctor `binding_spellings` receipts live in `src/v4/test/claim/manual/sg_rc_layering.dag`.
//! Outcome/Rc<Outcome> receipt: this PR adds assertion-only same-file smoke over
//! `claim_sg_rc_outcome_inner_sg2_args_preserved`; SG-0 implementation-surface posture remains +0,
//! and the dissolve trigger is direct v4 manual-claim runner execution of `sg_rc_layering.dag`.
//!
//! SG-1 + SG-2 + SG-5/SG-6 + SG-RC receipt: `target_model.dag` carriers;
//! `bounded_lattice_completeness` + `04_infer` gate; Rust rows in `rust.dag`;
//! `06_translate.dag` consumers.
//!
//! **This PR (+0 SG-0 paths):** Go SG-1 same-path expansion —
//! `v4_go_language_model_declares_target_atom_realization_rows` per
//! `docs/planning/v4-go-target-atom-realization-worksheet-2026-06-01.md` §9 (Int row
//! fail-closed deferred until shared `TargetValueTemplateKind` gains integer literal arm).
//! **This PR (+0 SG-0 paths):** TS L0 TargetAtomRealization same-path expansion —
//! `v4_typescript_language_model_declares_target_atom_realization_rows` per
//! `docs/planning/v4-ts-target-atom-realization-worksheet-2026-06-01.md` §8; ROADMAP row
//! `ROADMAP.md` § **Public Operational Lanes** / T-PB-B
//! (`pb_rust_tests_outside_residual_zero`). Dissolves when `.dag` `TestClaim` /
//! generated harness execution covers TS Symbol/Bool/String TargetAtomRealization catalog
//! facts directly without this host Rust parse-surface smoke.
//! **This PR (+0 SG-0 paths):** TS SG-2 arrow wire-shape widening — extends
//! `v4_typescript_language_model_declares_type_expression_projection_row` with parse-surface
//! receipts for `ts_sg2_arrow_labeled_xy_emitted` and `ts_sg2_arrow_mixed_wire_emitted`
//! golden probes (`typescript.dag`). Behavioral contracts (wire-shape, labeled serialize,
//! mixed-wire rejection) live in `src/v4/test/claim/manual/sg2_typescript_type_expression_projection.dag`;
//! dissolves when manual-claim runner executes those `TestClaim`s without this file. ROADMAP:
//! `ROADMAP.md` § **Nine lanes** / T-PB-B (`pb_rust_tests_outside_residual_zero`).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType};
use v3_compiler::tokenize_for_test;

const TARGET_MODEL_DAG: &str = include_str!("../../../../v4/std/target_model.dag");
const TARGET_MODEL_PATH: &str = "src/v4/std/target_model.dag";
const TRANSLATE_DAG: &str = include_str!("../../../../v4/compiler/06_translate.dag");
const TRANSLATE_PATH: &str = "src/v4/compiler/06_translate.dag";
const RUST_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/rust.dag");
const RUST_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/rust.dag";
const TYPESCRIPT_LANGUAGE_DAG: &str =
    include_str!("../../../../v4/extdeps/languages/typescript.dag");
const TYPESCRIPT_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/typescript.dag";
const GO_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/go.dag");
const GO_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/go.dag";
const SG_RC_LAYERING_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/sg_rc_layering.dag");
const SG_RC_LAYERING_CLAIM_PATH: &str = "src/v4/test/claim/manual/sg_rc_layering.dag";
const SG_COLLECTION_PROJECTION_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/sg_collection_projection.dag");
const SG_COLLECTION_PROJECTION_CLAIM_PATH: &str =
    "src/v4/test/claim/manual/sg_collection_projection.dag";
const BOUNDED_LATTICE_COMPLETENESS_DAG: &str =
    include_str!("../../../../v4/std/bounded_lattice_completeness.dag");
const BOUNDED_LATTICE_COMPLETENESS_PATH: &str = "src/v4/std/bounded_lattice_completeness.dag";
const INFER_DAG: &str = include_str!("../../../../v4/compiler/04_infer.dag");
const INFER_PATH: &str = "src/v4/compiler/04_infer.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum { name: decl_name, .. }
                | SurfaceItem::TypeRecord { name: decl_name, .. }
                | SurfaceItem::TypeAlias { name: decl_name, .. }
                | SurfaceItem::TypeAtom { name: decl_name, .. }
                if decl_name == name
        )
    })
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

fn surface_declares_data(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module
        .items
        .iter()
        .any(|item| matches!(item, SurfaceItem::Data { name: decl_name, .. } if decl_name == name))
}

/// `map_insert` keys/values are often split across lines in `.dag` sources.
fn dag_source_contains_map_insert_pair(body: &str, key: &str, value: &str) -> bool {
    let collapsed = body.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.contains(&format!("{key}, {value}"))
}

fn dag_source_contains_collapsed(body: &str, needle: &str) -> bool {
    let collapsed = body.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.contains(&needle.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    v3_compiler::parse_surface::TypeAngleArg::TypeExpr { ty } => {
                        surface_type_name(ty)
                    }
                    v3_compiler::parse_surface::TypeAngleArg::WidthNatLiteral {
                        decimal, ..
                    } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        other => format!("{other:?}"),
    }
}

fn type_record_fields(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Vec<(String, String)> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(
                fields
                    .iter()
                    .map(|f: &SurfaceField| (f.name.clone(), surface_type_name(&f.ty)))
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Import {
            path: import_path,
            names,
            ..
        } => {
            import_path.iter().map(String::as_str).collect::<Vec<_>>() == path
                && names.iter().any(|n| n == name)
        }
        _ => false,
    })
}

#[test]
fn v4_std_target_realization_dag_tokenizes_and_parses() {
    let _module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
}

#[test]
fn v4_std_target_realization_declares_target_atom_realization_carrier() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_type(&module, "TargetAtomRealization"),
        "{TARGET_MODEL_PATH}: must declare TargetAtomRealization"
    );
    let fields = type_record_fields(&module, "TargetAtomRealization");
    assert!(
        fields.iter().any(|(n, _)| n == "source_carrier"),
        "source_carrier must be Node-keyed authority"
    );
    assert!(
        fields
            .iter()
            .any(|(n, ty)| n == "type_form" && ty.contains("TargetTypeExpression")),
        "type_form must use SG-2 TargetTypeExpression {{ kind, node }} (M6 product; no parallel atom vocab)"
    );
    assert!(
        fields
            .iter()
            .any(|(n, ty)| n == "value_form" && ty.contains("TargetValueTemplate")),
        "value_form must be parametric TargetValueTemplate"
    );
    assert!(
        fields.iter().any(|(n, _)| n == "target_model"),
        "target_model must flow through catalog encode/decode"
    );
    assert!(
        fields.iter().any(|(n, _)| n == "constructor_form"),
        "constructor_form must not be dropped at catalog boundary"
    );
}

#[test]
fn v4_std_target_realization_declares_catalog_lookup() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_fn(&module, "target_atom_realization_lookup_in_catalog_node"),
        "catalog lookup must be structural over encoded row nodes"
    );
    assert!(
        surface_declares_fn(&module, "target_atom_type_spelling"),
        "type and value consumers share row authority via target_atom_type_spelling"
    );
    assert!(
        surface_declares_fn(&module, "target_atom_value_expression"),
        "value_form application must be row-driven"
    );
    assert!(
        surface_declares_fn(&module, "target_value_expression_node"),
        "TargetValueExpression must have a canonical Node projection (Practice 3)"
    );
}

#[test]
fn v4_translate_dag_imports_target_atom_realization_consumer() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_atom_realization_lookup_in_catalog_node"
        ),
        "{TRANSLATE_PATH}: translate must import catalog lookup from target_model"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_atom_realizations"
        ),
        "{TRANSLATE_PATH}: translate must read atom_realizations bundle edge"
    );
    assert!(
        surface_declares_fn(&module, "translate_target_atom_realization_for_carrier"),
        "{TRANSLATE_PATH}: must declare translate_target_atom_realization_for_carrier"
    );
    assert!(
        surface_declares_fn(&module, "translate_coerced_with_atom_realization"),
        "{TRANSLATE_PATH}: type+value emit must consult the same TargetAtomRealization row"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_coerced_with_atom_realization("),
        "{TRANSLATE_PATH}: translate_fold_init must consult atom realization via translate_coerced_with_atom_realization (type shell via translate_coerced_shell inside)"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_atom_realization_value_from_source_at_use_site("),
        "{TRANSLATE_PATH}: value path must thread TargetOwnershipUseSite (facts-forward)"
    );
    assert!(
        TRANSLATE_DAG.contains("value: row.type_form.node"),
        "{TRANSLATE_PATH}: type shell must apply row.type_form.node from catalog lookup"
    );
    assert!(
        TRANSLATE_DAG.contains("target_value_expression_node(expr:"),
        "{TRANSLATE_PATH}: value realization must project TargetValueExpression onto emitted Node"
    );
    assert!(
        TARGET_MODEL_DAG.contains("source_atom_value_for_realization_row("),
        "{TARGET_MODEL_PATH}: translate must import row.value_form-driven source projection"
    );
    assert!(
        TARGET_MODEL_DAG.contains("ValueBoolLiteral") && TARGET_MODEL_DAG.contains("source_atom_bool("),
        "{TARGET_MODEL_PATH}: Bool rows must project literal witnesses, not type-carrier shells only"
    );
    assert!(
        TARGET_MODEL_DAG.contains("ValueCharUnicodeScalar")
            && TARGET_MODEL_DAG.contains("source_atom_char("),
        "{TARGET_MODEL_PATH}: Char rows must project unicode scalar witnesses"
    );
    assert!(
        TRANSLATE_DAG.contains("source_atom_value_for_realization_row("),
        "{TRANSLATE_PATH}: value path must call substrate source projection (no carrier-hash re-derive)"
    );
    assert!(
        TRANSLATE_DAG.contains("target_atom_realization_lookup_miss_diagnostic(source_carrier:"),
        "{TRANSLATE_PATH}: absent atom_realizations edge must decode as catalog lookup_miss"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_target_atom_realization_for_carrier("),
        "{TRANSLATE_PATH}: translate must consult catalog by source_carrier (row-driven)"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_outcome_is_catalog_lookup_miss("),
        "{TRANSLATE_PATH}: non-catalog coercions must pass through only on lookup miss"
    );
    assert!(
        TARGET_MODEL_DAG.contains("TargetAtomRealizationCatalogInvalid"),
        "{TARGET_MODEL_PATH}: malformed catalog rows must fail-closed decode"
    );
    assert!(
        TARGET_MODEL_DAG
            .contains("Rejected { diagnostics: _ } => TargetAtomRealizationCatalogInvalid"),
        "{TARGET_MODEL_PATH}: catalog lookup must not swallow row decode failures"
    );
}

#[test]
fn v4_rust_language_model_declares_target_atom_realization_rows() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_symbol"),
        "{RUST_LANGUAGE_PATH}: Symbol row is greenfield (no parallel std_projection sentinel)"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_bool"),
        "{RUST_LANGUAGE_PATH}: Bool TargetAtomRealization row coexists with rust_facts_bool"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_char"),
        "{RUST_LANGUAGE_PATH}: Char TargetAtomRealization row coexists with rust_facts_char"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_catalog"),
        "{RUST_LANGUAGE_PATH}: per-language catalog prepares Python/Go parallel rows"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("data rust_std_projection_bool:"),
        "rust_std_projection_bool sentinel must remain for rust_facts_bool std_projection"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("data rust_std_projection_char:"),
        "rust_std_projection_char sentinel must remain for rust_facts_char std_projection"
    );
    let bundle_core_start = RUST_LANGUAGE_DAG
        .find("fn rust_mvp1_target_model_bundle_core()")
        .expect("rust_mvp1_target_model_bundle_core");
    let bundle_core_end = RUST_LANGUAGE_DAG[bundle_core_start..]
        .find("fn rust_mvp1_binding_spellings()")
        .expect("rust_mvp1_binding_spellings after bundle_core");
    let bundle_core_body =
        &RUST_LANGUAGE_DAG[bundle_core_start..bundle_core_start + bundle_core_end];
    assert!(
        bundle_core_body.contains("target_model_edge_collection_realization"),
        "{RUST_LANGUAGE_PATH}: row/catalog host_bundle must include collection_realization so target_model_bundle_core(host) decode matches"
    );
}

#[test]
fn v4_go_language_model_declares_target_atom_realization_rows() {
    let module = parse_module(GO_LANGUAGE_DAG, GO_LANGUAGE_PATH);
    assert!(
        surface_declares_data(&module, "go_target_atom_realization_symbol"),
        "{GO_LANGUAGE_PATH}: Symbol row for kernel-ambient Symbol"
    );
    assert!(
        surface_declares_data(&module, "go_target_atom_realization_bool"),
        "{GO_LANGUAGE_PATH}: Bool TargetAtomRealization row"
    );
    assert!(
        surface_declares_data(&module, "go_target_atom_realization_char"),
        "{GO_LANGUAGE_PATH}: Char → rune TargetAtomRealization row"
    );
    assert!(
        !surface_declares_data(&module, "go_target_atom_realization_int"),
        "{GO_LANGUAGE_PATH}: platform int row deferred until shared TargetValueTemplateKind covers integer literals (R1 uses go_surface_spelling_int / go_facts_int facts)"
    );
    assert!(
        surface_declares_data(&module, "go_target_atom_realization_catalog"),
        "{GO_LANGUAGE_PATH}: per-language catalog wired on MVP target_model"
    );
    assert!(
        surface_declares_data(&module, "go_atom_realization_symbol"),
        "{GO_LANGUAGE_PATH}: dual-name fact_id for R3-external leaf-model claims"
    );
    assert!(
        GO_LANGUAGE_DAG.contains("target_model_edge_atom_realizations"),
        "{GO_LANGUAGE_PATH}: atom_realizations edge must be on live go_mvp1_target_model_node"
    );
    assert!(
        GO_LANGUAGE_DAG.contains("go_surface_spelling_rune"),
        "{GO_LANGUAGE_PATH}: Char kernel maps to Go rune surface spelling"
    );
    let bundle_core_start = GO_LANGUAGE_DAG
        .find("fn go_mvp1_target_model_bundle_core()")
        .expect("go_mvp1_target_model_bundle_core");
    let bundle_core_end = GO_LANGUAGE_DAG[bundle_core_start..]
        .find("fn go_mvp1_target_model_staging()")
        .expect("go_mvp1_target_model_staging after bundle_core");
    let bundle_core_body = &GO_LANGUAGE_DAG[bundle_core_start..bundle_core_start + bundle_core_end];
    assert!(
        !bundle_core_body.contains("target_model_edge_atom_realizations"),
        "{GO_LANGUAGE_PATH}: catalog host_bundle must exclude atom_realizations edge (rust SG-1 pattern)"
    );
    let bindings_start = GO_LANGUAGE_DAG
        .find("fn go_mvp1_binding_spellings()")
        .expect("go_mvp1_binding_spellings");
    let bindings_end = GO_LANGUAGE_DAG[bindings_start..]
        .find("fn go_mvp1_target_model_bundle_core()")
        .expect("go_mvp1_target_model_bundle_core after binding_spellings");
    let bindings_body = &GO_LANGUAGE_DAG[bindings_start..bindings_start + bindings_end];
    for (symbol, spelling) in [
        ("go_surface_spelling_string", "\"string\""),
        ("go_surface_spelling_bool", "\"bool\""),
        ("go_surface_spelling_rune", "\"rune\""),
    ] {
        assert!(
            dag_source_contains_map_insert_pair(bindings_body, symbol, spelling),
            "{GO_LANGUAGE_PATH}: SG-1 atom type_form identities must resolve via go_mvp1_binding_spellings (06_translate map_get)"
        );
    }
}

// P5 receipt: same-path smoke expansion (SG-2 type-expression-projection worksheet §6).
// Asserts `.dag` surface shape only — substrate edits landed #3962 on `main` (#4124).
// Deferral: retired when `.dag` `TestClaim` / T-22 eval covers same facts (T-PB-B).
#[test]
fn v4_std_target_model_declares_target_type_expression_projection_carrier() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_type(&module, "TargetTypeExpressionProjection"),
        "{TARGET_MODEL_PATH}: SG-2 canonical type-expression projection carrier"
    );
    assert!(
        surface_declares_type(&module, "TargetTypeExpression"),
        "{TARGET_MODEL_PATH}: emitted type-expression wire authority must be typed"
    );
    assert!(
        surface_declares_type(&module, "TargetTypeExprKind"),
        "{TARGET_MODEL_PATH}: connective kind axis must be a coproduct (M6)"
    );
    let wire_fields = type_record_fields(&module, "TargetTypeExpression");
    assert!(
        wire_fields.iter().any(|(n, _)| n == "kind"),
        "TargetTypeExpression.kind must be TargetTypeExprKind (no parallel atom vocab)"
    );
    assert!(
        wire_fields.iter().any(|(n, _)| n == "node"),
        "TargetTypeExpression.node must carry emitted subtree authority"
    );
    let proj_fields = type_record_fields(&module, "TargetTypeExpressionProjection");
    let generic_apply_fields = type_record_fields(&module, "TargetGenericApply");
    assert!(
        generic_apply_fields
            .iter()
            .any(|(n, ty)| n == "field_label_separator" && ty == "Optional<Symbol>"),
        "TargetGenericApply.field_label_separator must carry optional record field-label binding token"
    );
    assert!(
        proj_fields.iter().any(|(n, _)| n == "instantiation_form"),
        "instantiation_form must be present for generic-apply connective"
    );
    assert!(
        proj_fields.iter().any(|(n, _)| n == "arrow_form"),
        "arrow_form must be present for function-type connective"
    );
    assert!(
        surface_declares_fn(&module, "target_type_expr_emitted_wire_decode"),
        "{TARGET_MODEL_PATH}: bidirectional wire decode must share projection row (§10.6)"
    );
    assert!(
        surface_declares_fn(&module, "target_type_expr_emitted_labeled_slot_edges"),
        "{TARGET_MODEL_PATH}: record serializers must preserve slot labels, not just type nodes"
    );
}

#[test]
fn v4_translate_dag_imports_type_expression_projection_consumer() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_type_expression_projection"
        ),
        "{TRANSLATE_PATH}: translate must read type_expression_projection bundle edge"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_type_expr_emitted_wire_decode"
        ),
        "{TRANSLATE_PATH}: serialize path must import wire decode from target_model"
    );
    assert!(
        surface_declares_fn(&module, "type_expression_projection_from_target"),
        "{TRANSLATE_PATH}: projection rows must decode from TargetModel bundle"
    );
    assert!(
        surface_declares_fn(&module, "project_type_expression_node"),
        "{TRANSLATE_PATH}: TypeNode subtrees must project via SG-2 row"
    );
    assert!(
        surface_declares_fn(&module, "serialize_type_expr_record_field_label"),
        "{TRANSLATE_PATH}: record type serialization must bind field labels through target spellings"
    );
    assert!(
        TRANSLATE_DAG.contains("field_label_separator: field_label_separator"),
        "{TRANSLATE_PATH}: record serializer must consume the shared field-label separator carrier"
    );
    assert!(
        surface_declares_fn(&module, "translate_node_with_projection"),
        "{TRANSLATE_PATH}: projection-present targets must use dedicated translate path"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_node_with_projection("),
        "{TRANSLATE_PATH}: translate_node must route ProjectionPresent to projection path"
    );
}

#[test]
fn v4_translate_record_type_serialization_emits_field_labels() {
    assert!(
        dag_source_contains_collapsed(
            TRANSLATE_DAG,
            "o: target_type_expr_emitted_labeled_slot_edges(node: node)"
        ),
        "{TRANSLATE_PATH}: record serialization must retain labeled slot edges"
    );
    assert!(
        dag_source_contains_collapsed(
            TRANSLATE_DAG,
            "o: serialize_type_expr_record_field_label(target: target, edge: split.head)"
        ),
        "{TRANSLATE_PATH}: record serialization must resolve each field label through binding spellings"
    );
    assert!(
        dag_source_contains_collapsed(
            TRANSLATE_DAG,
            "o: lex_rules_literal(target: target, token_class: field_label_separator)"
        ),
        "{TRANSLATE_PATH}: record serialization must emit the target field-label separator"
    );
    assert!(
        dag_source_contains_collapsed(
            TRANSLATE_DAG,
            "let field_source = list_append(
              left: list_append(left: label_source, right: label_sep_source),
              right: field_type_source
            )"
        ),
        "{TRANSLATE_PATH}: record field emission must be label + ':' + serialized type, not bare type"
    );
}

#[test]
fn v4_rust_language_model_declares_type_expression_projection_row() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        surface_declares_fn(&module, "rust_type_expression_projection"),
        "{RUST_LANGUAGE_PATH}: per-language SG-2 row must be authored in rust.dag"
    );
    assert!(
        surface_declares_fn(&module, "rust_type_expression_projection_bundle_node"),
        "{RUST_LANGUAGE_PATH}: projection row must encode as TargetModel bundle child"
    );
    assert!(
        surface_declares_fn(&module, "rust_sg2_type_expr_target_model"),
        "{RUST_LANGUAGE_PATH}: falsification probe target must carry projection edge"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("target_model_edge_type_expression_projection"),
        "{RUST_LANGUAGE_PATH}: SG-2 bundle edge name must wire on rust TargetModel"
    );
    assert!(
        dag_source_contains_collapsed(
            RUST_LANGUAGE_DAG,
            "rust_lex_rule(token_class: rust_token_colon, text: \": \")"
        ),
        "{RUST_LANGUAGE_PATH}: SG-2 target model lex rules must realize the record field-label separator"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("fn rust_sg2_rc_foobar_xy_emitted()"),
        "{RUST_LANGUAGE_PATH}: golden Rc<FooBar<X,Y>> emitted node for falsification probe"
    );
    assert!(
        surface_declares_fn(&module, "rust_collection_realization_catalog_node"),
        "{RUST_LANGUAGE_PATH}: Rust target model must expose collection rows through one carrier-keyed catalog"
    );
}

#[test]
fn v4_typescript_language_model_binds_record_field_label_separator_in_shared_row() {
    let module = parse_module(TYPESCRIPT_LANGUAGE_DAG, TYPESCRIPT_LANGUAGE_PATH);
    assert!(
        surface_declares_fn(&module, "ts_type_expression_projection"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TypeScript SG-2 row must be authored"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "collection"],
            "Present"
        ) && import_includes_name(&module, &["v4", "std", "collection"], "Absent"),
        "{TYPESCRIPT_LANGUAGE_PATH}: projection bundle match arms must import Optional constructors explicitly"
    );
    assert!(
        TYPESCRIPT_LANGUAGE_DAG
            .contains("field_label_separator: optional_present(value: ts_token_colon)"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TS record row must bind ':' through shared TargetGenericApply"
    );
    assert!(
        !TYPESCRIPT_LANGUAGE_DAG.contains("feature:target-record-field-label-binding")
            && !TYPESCRIPT_LANGUAGE_DAG.contains("structural surface only"),
        "{TYPESCRIPT_LANGUAGE_PATH}: record-label Lane H disposition is live; stale gated prose must stay deleted"
    );
    assert!(
        TYPESCRIPT_LANGUAGE_DAG.contains("target_type_expr_field_field_label_separator"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TS projection bundle must encode the shared field-label separator edge"
    );
}

// P5 receipt: same-path smoke expansion (SG-5 collection-bounded-lattice worksheet §6).
// Asserts `.dag` surface shape only — substrate edits landed #3957 / #4085 on `main`.
// Deferral: retired when `.dag` `TestClaim` / T-22 eval covers same facts (T-PB-B).
#[test]
fn v4_std_target_model_declares_target_collection_realization_carrier() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_type(&module, "TargetCollectionRealization"),
        "{TARGET_MODEL_PATH}: SG-5 canonical collection realization carrier"
    );
    assert!(
        surface_declares_type(&module, "TargetCollectionReprKind"),
        "{TARGET_MODEL_PATH}: representation kind must be a coproduct (M6)"
    );
    assert!(
        TARGET_MODEL_DAG.contains("TargetCollectionReprVec"),
        "{TARGET_MODEL_PATH}: FreeMonoid sequence rows must have a Vec representation kind"
    );
    assert!(
        surface_declares_fn(
            &module,
            "target_collection_type_node_is_free_monoid_carrier"
        ),
        "{TARGET_MODEL_PATH}: FreeMonoid carrier recognition must be substrate-owned"
    );
    assert!(
        surface_declares_type(&module, "RequiredTraitWitness"),
        "{TARGET_MODEL_PATH}: per-alternative trait witnesses must be typed"
    );
    assert!(
        surface_declares_fn(&module, "target_collection_select_choice"),
        "{TARGET_MODEL_PATH}: primary/alternatives selection must be substrate authority"
    );
}

#[test]
fn v4_translate_dag_imports_collection_realization_consumer() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_collection_realization"
        ),
        "{TRANSLATE_PATH}: translate must read collection_realization bundle edge"
    );
    assert!(
        surface_declares_fn(&module, "project_set_collection_type_node"),
        "{TRANSLATE_PATH}: Set carrier projection must consume TargetCollectionRealization rows"
    );
    assert!(
        surface_declares_fn(&module, "project_free_monoid_collection_type_node"),
        "{TRANSLATE_PATH}: FreeMonoid carrier projection must consume TargetCollectionRealization rows"
    );
    assert!(
        surface_declares_fn(&module, "collection_realization_from_target"),
        "{TRANSLATE_PATH}: collection rows must decode from TargetModel bundle"
    );
}

// P5 receipt: same-path hand-Rust smoke expansion in an existing SG-0-listed file,
// explicitly deferred to `ROADMAP.md` § Public Operational Lanes / T-PB-B
// (`pb_rust_tests_outside_residual_zero`). Dissolves when the `.dag` manual-claim runner
// executes `src/v4/test/claim/manual/sg_collection_projection.dag` directly.
#[test]
fn v4_sg_collection_projection_claim_declares_vec_rc_receipt() {
    let module = parse_module(
        SG_COLLECTION_PROJECTION_CLAIM_DAG,
        SG_COLLECTION_PROJECTION_CLAIM_PATH,
    );
    assert!(
        surface_declares_data(&module, "claim_sg_collection_free_monoid_vec_rc_projection"),
        "{SG_COLLECTION_PROJECTION_CLAIM_PATH}: FreeMonoid<T> -> Vec<Rc<T>> claim must be authored"
    );
    assert!(
        SG_COLLECTION_PROJECTION_CLAIM_DAG.contains("Vec<Rc<Node>>"),
        "{SG_COLLECTION_PROJECTION_CLAIM_PATH}: claim must pin the Rust Vec<Rc<T>> boundary"
    );
}

#[test]
fn v4_bounded_lattice_completeness_and_infer_gate_are_wired() {
    let bl_module = parse_module(
        BOUNDED_LATTICE_COMPLETENESS_DAG,
        BOUNDED_LATTICE_COMPLETENESS_PATH,
    );
    assert!(
        surface_declares_fn(&bl_module, "bounded_lattice_instance_completeness"),
        "{BOUNDED_LATTICE_COMPLETENESS_PATH}: SG-6 completeness classifier must live in cycle-breaker module"
    );
    let infer_module = parse_module(INFER_DAG, INFER_PATH);
    assert!(
        import_includes_name(
            &infer_module,
            &["v4", "std", "bounded_lattice_completeness"],
            "bounded_lattice_instance_completeness"
        ),
        "{INFER_PATH}: infer must import completeness from bounded_lattice_completeness.dag"
    );
    assert!(
        surface_declares_fn(&infer_module, "infer_bounded_lattice_consumer_gate"),
        "{INFER_PATH}: consumer sites must reject partial BoundedLattice references"
    );
    assert!(
        surface_declares_fn(&infer_module, "partial_bounded_lattice_instances_in_tree"),
        "{INFER_PATH}: partial instances must be collected before consumer gate"
    );
}

#[test]
fn v4_typescript_language_model_declares_target_atom_realization_rows() {
    let module = parse_module(TYPESCRIPT_LANGUAGE_DAG, TYPESCRIPT_LANGUAGE_PATH);
    assert!(
        surface_declares_data(&module, "ts_target_atom_realization_symbol"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TS Symbol TargetAtomRealization row must be authored in typescript.dag"
    );
    assert!(
        surface_declares_data(&module, "ts_target_atom_realization_bool"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TS Bool TargetAtomRealization row must be authored in typescript.dag"
    );
    assert!(
        surface_declares_data(&module, "ts_target_atom_realization_string"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TS String TargetAtomRealization row substitutes for Rust Char"
    );
    assert!(
        surface_declares_data(&module, "ts_target_atom_realization_catalog"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TS TargetModel must expose atom rows through the shared catalog"
    );
    assert!(
        TYPESCRIPT_LANGUAGE_DAG.contains("target_model_edge_atom_realizations"),
        "{TYPESCRIPT_LANGUAGE_PATH}: live TS TargetModel bundle must expose atom_realizations edge"
    );
    assert!(
        TYPESCRIPT_LANGUAGE_DAG.contains("ts_type_expression_projection().atom_form"),
        "{TYPESCRIPT_LANGUAGE_PATH}: atom rows must consume the SG-2 TypeScript type-expression row"
    );
    assert!(
        !TYPESCRIPT_LANGUAGE_DAG.contains("ts_target_atom_realization_char"),
        "{TYPESCRIPT_LANGUAGE_PATH}: TS L0 substitutes String; it must not copy the Rust Char row"
    );
}

// P5 receipt (+0 SG-0): TS SG-2 same-path expansion beyond L0 row-2 — Instantiation/Arrow/Disj
// golden fn-decl probes; arrow labeled/mixed-wire behavioral contracts in
// sg2_typescript_type_expression_projection.dag (PR #4226).
#[test]
fn v4_typescript_language_model_declares_type_expression_projection_row() {
    let module = parse_module(TYPESCRIPT_LANGUAGE_DAG, TYPESCRIPT_LANGUAGE_PATH);
    assert!(
        surface_declares_fn(&module, "ts_type_expression_projection"),
        "{TYPESCRIPT_LANGUAGE_PATH}: per-language SG-2 row must be authored in typescript.dag"
    );
    assert!(
        surface_declares_fn(&module, "ts_type_expression_projection_bundle_node"),
        "{TYPESCRIPT_LANGUAGE_PATH}: projection row must encode as TargetModel bundle child"
    );
    assert!(
        surface_declares_fn(&module, "ts_sg2_type_expr_target_model"),
        "{TYPESCRIPT_LANGUAGE_PATH}: falsification probe target must carry projection edge"
    );
    assert!(
        TYPESCRIPT_LANGUAGE_DAG.contains("target_model_edge_type_expression_projection"),
        "{TYPESCRIPT_LANGUAGE_PATH}: SG-2 bundle edge name must wire on TS TargetModel"
    );
    assert!(
        surface_declares_fn(&module, "ts_sg2_foobar_xy_emitted"),
        "{TYPESCRIPT_LANGUAGE_PATH}: golden FooBar<X,Y> Instantiation emitted node (beyond row-2)"
    );
    assert!(
        surface_declares_fn(&module, "ts_sg2_arrow_xy_emitted"),
        "{TYPESCRIPT_LANGUAGE_PATH}: golden (X) => Y Arrow emitted node (beyond row-2)"
    );
    assert!(
        surface_declares_fn(&module, "ts_sg2_arrow_labeled_xy_emitted"),
        "{TYPESCRIPT_LANGUAGE_PATH}: golden (X: X) => Y labeled Arrow emitted node"
    );
    assert!(
        surface_declares_fn(&module, "ts_sg2_arrow_mixed_wire_emitted"),
        "{TYPESCRIPT_LANGUAGE_PATH}: mixed positional+labeled arrow wire falsification probe"
    );
    assert!(
        surface_declares_fn(&module, "ts_sg2_sum_xy_emitted"),
        "{TYPESCRIPT_LANGUAGE_PATH}: golden X | Y Disj emitted node (beyond row-2)"
    );
}

// P5 receipt: SG-RC-LAYERING worksheet §6 — same-path smoke expansion (parse surface only).
#[test]
fn v4_std_target_realization_declares_use_site_ownership_carrier() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_type(&module, "TargetUseSiteOwnershipRealization"),
        "{TARGET_MODEL_PATH}: must declare TargetUseSiteOwnershipRealization (SG-RC)"
    );
    assert!(
        surface_declares_type(&module, "TargetOwnershipUseSite"),
        "{TARGET_MODEL_PATH}: use_site must be a coproduct axis"
    );
    assert!(
        surface_declares_type(&module, "TargetReferenceLayer"),
        "{TARGET_MODEL_PATH}: reference_layer must be a coproduct axis"
    );
    assert!(
        surface_declares_type(&module, "TargetReferenceLayerWrapped"),
        "{TARGET_MODEL_PATH}: probe payload must exclude Owned (illegal-states-unrepresentable)"
    );
    assert!(
        surface_declares_fn(&module, "target_use_site_ownership_lookup_in_catalog_node"),
        "{TARGET_MODEL_PATH}: per (carrier, use_site) lookup must be structural"
    );
    assert!(
        surface_declares_fn(&module, "target_use_site_ownership_source_key"),
        "{TARGET_MODEL_PATH}: SG-RC lookup must use row-authored carrier key projection so Outcome rows compose with SG-2"
    );
    assert!(
        surface_declares_fn(&module, "target_reference_layer_apply_type_emitted"),
        "{TARGET_MODEL_PATH}: type emit must wrap SG-2 inner via reference_layer row"
    );
    assert!(
        surface_declares_fn(&module, "target_reference_layer_apply_value_expression"),
        "{TARGET_MODEL_PATH}: value emit must consult the same reference_layer row"
    );
    assert!(
        surface_declares_fn(&module, "target_reference_layer_tokens_from_node"),
        "{TARGET_MODEL_PATH}: token bundle decode must be structural Outcome"
    );
    assert!(
        surface_declares_data(&module, "target_reference_layer_tokens_decode_invalid"),
        "{TARGET_MODEL_PATH}: non-Conj token bundle must fail-closed"
    );
}

#[test]
fn v4_sg_rc_outcome_claim_preserves_sg2_inner_type_args() {
    assert!(
        SG_RC_LAYERING_CLAIM_DAG.contains("sg_rc_outcome_node_source_type"),
        "{SG_RC_LAYERING_CLAIM_PATH}: must include an Outcome<T> SG-RC fixture"
    );
    assert!(
        SG_RC_LAYERING_CLAIM_DAG.contains("project_type_expression_node"),
        "{SG_RC_LAYERING_CLAIM_PATH}: Outcome<T> fixture must project inner type through SG-2 before SG-RC wrapping"
    );
    assert!(
        SG_RC_LAYERING_CLAIM_DAG.contains("Rc<Outcome<Node>>"),
        "{SG_RC_LAYERING_CLAIM_PATH}: Outcome<T> receipt must assert Rc<Outcome<Node>>, not bare Rc<Outcome>"
    );
    assert!(
        SG_RC_LAYERING_CLAIM_DAG.contains("claim_sg_rc_outcome_inner_sg2_args_preserved"),
        "{SG_RC_LAYERING_CLAIM_PATH}: manual TestClaim must pin Outcome<T> SG-2 + SG-RC composition"
    );
}

#[test]
fn v4_translate_dag_imports_use_site_ownership_consumer() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_use_site_ownership_lookup_in_catalog_node"
        ),
        "{TRANSLATE_PATH}: translate must import SG-RC catalog lookup"
    );
    assert!(
        surface_declares_fn(&module, "translate_coerced_shell_at_use_site"),
        "{TRANSLATE_PATH}: type shell must consult use_site ownership row"
    );
    assert!(
        surface_declares_fn(&module, "translate_project_arrow_split_types"),
        "{TRANSLATE_PATH}: arrow boundaries must apply return/param ownership rows"
    );
    assert!(
        TRANSLATE_DAG.contains("OwnershipAtBindingProjection"),
        "{TRANSLATE_PATH}: fold coercion must use binding-projection catalog rows"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("OwnershipAtBindingProjection"),
        "{RUST_LANGUAGE_PATH}: catalog must include binding-projection rows for fold path"
    );
    assert!(
        surface_declares_fn(
            &module,
            "translate_apply_use_site_ownership_to_value_expression"
        ),
        "{TRANSLATE_PATH}: value path must consult use_site ownership row"
    );
    assert!(
        !surface_declares_fn(&module, "translate_sg_rc_bundle_ready"),
        "{TRANSLATE_PATH}: must not expose fail-open Bool SG-RC readiness beside apply_disposition"
    );
    assert!(
        surface_declares_fn(&module, "translate_sg_rc_bundle_apply_disposition"),
        "{TRANSLATE_PATH}: partial SG-RC bundle must fail-closed, not passthrough Owned"
    );
    assert!(
        surface_declares_fn(&module, "translate_sg_rc_bundle_edge_present"),
        "{TRANSLATE_PATH}: malformed bundle edge lookup must propagate Rejected"
    );
    assert!(
        !TRANSLATE_DAG.contains("shared_types"),
        "{TRANSLATE_PATH}: forbidden spelling-keyed shared_types table (§3)"
    );
}

#[test]
fn v4_rust_language_model_declares_use_site_ownership_rows() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        surface_declares_fn(&module, "rust_sg_rc_use_site_ownership_catalog"),
        "{RUST_LANGUAGE_PATH}: Rust TargetModel must carry SG-RC catalog rows"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("target_model_edge_use_site_ownership_realizations"),
        "{RUST_LANGUAGE_PATH}: live TargetModel bundle must expose ownership catalog edge"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("target_model_edge_reference_layer_tokens"),
        "{RUST_LANGUAGE_PATH}: Rc/Box surface tokens must live on TargetModel bundle"
    );
    assert!(
        !RUST_LANGUAGE_DAG.contains("shared_types"),
        "{RUST_LANGUAGE_PATH}: forbidden v2 shared_types re-port (§3)"
    );
}
