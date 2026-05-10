use std::collections::{HashMap, HashSet};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    algebra_profile_to_dimension, constant_bound_value, evidence_rank, is_constant_bound,
    join_evidence, literal_bits_int, lower_call_pattern, map_evidence_merge_at, merge_evidence,
    optional_evidence_meet, per_call_descent_evidence, per_call_pattern_at,
    positive_amount_from_i64, promote_to_strict, size_bound_param,
    sub_value_relation_to_call_pattern, tree_size_bound, type_iteration_dimension, AlgebraProfile,
    ArrowBody, AtomPayload, CallPattern, CardinalityBound, DescentEvidence, FieldMap, FieldValue,
    Interval, IntervalWidth, IterationDimension, IterationPrimitive, LoweringTarget,
    PositiveDescentAmount, PositiveIntervalWidth, ProportionalDivisor, ShrinkFactor, SizeBound,
    SubValueRelation, TypeConnective, ValueBody,
};
use v3_compiler::diagnostics::positive_interval_width_unit_count_requires_nonnegative_units_literal_message;
use v3_compiler::parse_surface;
use v3_compiler::CompileError;
use v3_compiler::Dag;
use v3_compiler::Diagnostic;
use v3_compiler::SourceSpan;
use v3_compiler::{parse_for_test, tokenize_for_test};

fn with_full_bootstrap_stack<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(f)
        .expect("spawn bootstrap-stack integration thread")
        .join()
        .expect("bootstrap-stack integration thread panicked")
}

fn find_named(dag: &Dag, name: &str) -> v3_compiler::dag::DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn structural_reference_field(
    decl: &v3_compiler::dag::Declaration,
    label: &str,
) -> Option<v3_compiler::dag::DeclarationId> {
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        return None;
    };
    fields
        .iter()
        .find_map(|(field_label, value)| match (field_label.as_str(), value) {
            (want, FieldValue::Reference(id)) if want == label => Some(*id),
            _ => None,
        })
}

fn record_fields(dag: &Dag, name: &str) -> Vec<String> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Conj { children } => {
            children.iter().map(|field| field.label.clone()).collect()
        }
        other => panic!("expected `{name}` to lower to a Conj, got {other:?}"),
    }
}

fn sum_variants(dag: &Dag, name: &str) -> Vec<(String, Vec<String>)> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .map(|variant| {
                let payload = match &dag.declaration(variant.ty).connective {
                    TypeConnective::Conj { children } => {
                        children.iter().map(|field| field.label.clone()).collect()
                    }
                    other => panic!(
                        "expected variant `{}` under `{name}` to lower to a Conj payload, got {other:?}",
                        variant.label
                    ),
                };
                (variant.label.clone(), payload)
            })
            .collect(),
        other => panic!("expected `{name}` to lower to a Disj, got {other:?}"),
    }
}

fn conj_field_by_id(
    dag: &Dag,
    id: v3_compiler::dag::DeclarationId,
    field_name: &str,
) -> v3_compiler::dag::DeclarationId {
    let decl = dag.declaration(id);
    match &decl.connective {
        TypeConnective::Conj { children } => {
            children
                .iter()
                .find(|field| field.label == field_name)
                .unwrap_or_else(|| panic!("declaration {id:?} missing `{field_name}` field"))
                .ty
        }
        other => panic!("declaration {id:?} is not a Conj: {other:?}"),
    }
}

fn positional_payload(
    dag: &Dag,
    id: v3_compiler::dag::DeclarationId,
) -> v3_compiler::dag::DeclarationId {
    conj_field_by_id(dag, id, "_0")
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("expected src/v3/compiler -> workspace root")
        .to_path_buf()
}

fn runtime_value_variant_payload(dag: &Dag, variant: &str) -> v3_compiler::dag::DeclarationId {
    let value = dag
        .declaration_by_name("Value")
        .expect("runtime Value missing from full bootstrap");
    match &value.connective {
        TypeConnective::Disj { variants } => {
            variants
                .iter()
                .find(|field| field.label == variant)
                .unwrap_or_else(|| panic!("Value missing `{variant}` variant"))
                .ty
        }
        other => panic!("runtime Value is not a Disj: {other:?}"),
    }
}

fn assert_runtime_value_instantiation(
    dag: &Dag,
    actual: v3_compiler::dag::DeclarationId,
    template: &str,
    argument: &str,
) {
    let expected_template = find_named(dag, template);
    let expected_argument = find_named(dag, argument);
    match &dag.declaration(actual).connective {
        TypeConnective::Instantiation {
            template: actual_template,
            arguments,
        } => {
            assert_eq!(
                *actual_template, expected_template,
                "expected instantiation template `{template}`"
            );
            assert_eq!(arguments.len(), 1, "expected one template argument");
            assert_eq!(
                arguments[0].value, expected_argument,
                "expected template argument `{argument}`"
            );
        }
        other => panic!("expected Instantiation, got {other:?}"),
    }
}

fn arrow_body(dag: &Dag, name: &str) -> ArrowBody {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Arrow { body, .. } => body.clone(),
        other => panic!("expected `{name}` to lower to an Arrow, got {other:?}"),
    }
}

fn semantic_dag_for(source: &str, file: &str) -> Dag {
    let err = compile_to_dag(source, file).expect_err("source must fail semantically");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostics, got {err:?}");
    };
    dag
}

fn has_resolve_error(dag: &Dag) -> bool {
    dag.diagnostics()
        .iter()
        .any(|(_, diagnostic)| matches!(diagnostic, Diagnostic::ResolveError { .. }))
}

#[test]
fn e_m_method_semantics_subsumption_receipt_is_verifiable() {
    let dag = Dag::new();
    assert!(
        dag.declaration_by_name("MethodSemantics").is_none(),
        "E-M chose M-b: v3 must not add a parallel MethodSemantics carrier"
    );
    assert_eq!(
        sum_variants(&dag, "TransformTarget"),
        vec![
            (String::from("Callable"), vec![String::from("_0")]),
            (
                String::from("UnresolvedFieldProject"),
                vec![String::from("field_label")],
            ),
            (
                String::from("ResolvedFieldProject"),
                vec![String::from("field_label")],
            ),
            (String::from("Operator"), vec![String::from("_0")]),
        ],
        "E-M receipt depends on callable, field-project, and operator dispatch facts"
    );
    assert_eq!(
        sum_variants(&dag, "OperatorKind"),
        vec![
            (String::from("Arithmetic"), vec![String::from("_0")]),
            (String::from("Comparison"), vec![String::from("_0")]),
            (String::from("Logical"), vec![String::from("_0")]),
        ],
        "AlgebraMethodSemantics.method_def maps through operator kind plus algebra resolution"
    );
    assert!(
        sum_variants(&dag, "TypeConnective").contains(&(
            String::from("Arrow"),
            vec![
                String::from("inputs"),
                String::from("output"),
                String::from("body"),
            ],
        )),
        "fold_accumulator_type maps to resolved callable/lambda Arrow signatures"
    );
    assert!(
        record_fields(&dag, "BindNode").contains(&String::from("params")),
        "fold_accumulator_type maps through callable/lambda BindNode params and port states"
    );

    let root = workspace_root();
    let register = std::fs::read_to_string(root.join("docs/v3-lens-capability-register.md"))
        .expect("read docs/v3-lens-capability-register.md");
    assert!(
        register.contains("PlainMethodSemantics` maps to ordinary `TransformTarget::Callable(DeclarationId)` dispatch or `TransformTarget::FieldProject"),
        "register must name the PlainMethodSemantics structural replacement"
    );
    assert!(
        register.contains("AlgebraMethodSemantics.method_def` maps to the callable declaration id or, for operators still in the surface scaffold, `TransformTarget::Operator(OperatorKind)`"),
        "register must name the AlgebraMethodSemantics structural replacement"
    );
    assert!(
        register.contains("fold_accumulator_type` maps to callable/lambda signature facts: v3 resolves the callable's `Arrow { inputs, output, body }`, binds callback arguments through `BindNode.params` and port states"),
        "register must name the fold_accumulator_type structural replacement"
    );
    assert!(
        register.contains("ServiceMethodSemantics` maps to typed service/effect declarations and operation metadata"),
        "register must name the ServiceMethodSemantics structural replacement"
    );

    let program =
        std::fs::read_to_string(root.join("docs/design-substrate-carrier-port-program.md"))
            .expect("read docs/design-substrate-carrier-port-program.md");
    assert!(
        program.contains("**E-M is closed via M-b structural subsumption**"),
        "carrier program must record E-M as closed via M-b"
    );
    assert!(
        program.contains("`AlgebraMethodSemantics.fold_accumulator_type` maps to resolved callable/lambda `Arrow { inputs, output, body }` signatures plus `BindNode.params`"),
        "carrier program must name the fold_accumulator_type structural replacement"
    );
    assert!(
        program.contains("**Sanity predicate passed:** v3 structural resolution carries all v2 `MethodSemantics` facts needed for carrier parity"),
        "closed E-M lane must retain its verifiable sanity predicate"
    );
}

#[test]
fn substrate_declares_expected_reflection_surface() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load substrate reflection files cleanly: {:?}",
        dag.diagnostics()
    );

    assert_eq!(record_fields(&dag, "TypeShape"), vec!["declaration"]);
    assert_eq!(
        record_fields(&dag, "DagPort"),
        vec!["id", "state", "produced_by"]
    );
    assert_eq!(record_fields(&dag, "FieldEntry"), vec!["label", "value"]);
    assert_eq!(
        record_fields(&dag, "Declaration"),
        vec![
            "id",
            "name",
            "connective",
            "type_params",
            "phantom_params",
            "meta_tag",
            "inhabits",
            "value_body",
            "nominal_opacity",
            "span",
        ]
    );
    assert_eq!(
        record_fields(&dag, "PayloadBinding"),
        vec!["binding_name", "payload_port"]
    );
    assert_eq!(
        record_fields(&dag, "BranchPath"),
        vec!["body", "result_port", "pattern", "binding"]
    );
    assert_eq!(record_fields(&dag, "NonEmptyList"), vec!["first", "rest"]);
    assert_eq!(
        record_fields(&dag, "NonSingletonList"),
        vec!["first", "second", "rest"]
    );
    assert_eq!(record_fields(&dag, "ElementRef"), vec!["index"]);
    assert_eq!(record_fields(&dag, "ParamRef"), vec!["member", "slot"]);
    assert_eq!(record_fields(&dag, "TransformRef"), vec!["node"]);
    assert_eq!(record_fields(&dag, "MemberDescent"), vec!["param"]);
    assert_eq!(record_fields(&dag, "IntraClusterCall"), vec!["transform"]);
    assert_eq!(
        record_fields(&dag, "Cluster"),
        vec!["members", "intra_cluster_calls"]
    );
    assert_eq!(
        record_fields(&dag, "ValueNode"),
        vec!["id", "payload", "result_port", "span", "lane2_workflow"]
    );
    assert_eq!(
        record_fields(&dag, "TransformNode"),
        vec!["id", "target", "inputs", "result_port", "span"]
    );
    assert_eq!(
        record_fields(&dag, "BranchNode"),
        vec![
            "id",
            "input",
            "paths",
            "result_port",
            "span",
            "emit_participation"
        ]
    );
    assert_eq!(
        record_fields(&dag, "LoopNode"),
        vec![
            "id",
            "source",
            "init",
            "body",
            "bound",
            "result_port",
            "span"
        ]
    );
    assert_eq!(
        record_fields(&dag, "BindNode"),
        vec![
            "id",
            "name",
            "result_port",
            "params",
            "span",
            "lane2_workflow",
            "emit_participation"
        ]
    );
    assert_eq!(
        record_fields(&dag, "Dag"),
        vec!["declarations", "nodes", "ports", "clusters"]
    );
    assert_eq!(record_fields(&dag, "SurfaceModule"), vec!["items"]);
    assert_eq!(
        record_fields(&dag, "SurfaceParam"),
        vec!["name", "ty", "refinement"]
    );
    assert_eq!(record_fields(&dag, "SurfaceField"), vec!["name", "ty"]);
    assert_eq!(
        record_fields(&dag, "SurfaceVariant"),
        vec!["name", "payload", "span"]
    );
    assert_eq!(
        record_fields(&dag, "SurfaceRecordField"),
        vec!["name", "value", "span"]
    );
    assert_eq!(
        record_fields(&dag, "SurfaceMapEntry"),
        vec!["key", "key_span", "value", "span"]
    );
    assert_eq!(
        record_fields(&dag, "SurfaceMatchArm"),
        vec!["pattern", "body", "span"]
    );
    assert_eq!(
        record_fields(&dag, "SurfacePatternField"),
        vec!["name", "binding", "span"]
    );
}

#[test]
fn termination_carriers_bootstrap_from_v3_std() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load termination carriers cleanly: {:?}",
        dag.diagnostics()
    );

    assert_eq!(
        sum_variants(&dag, "DescentEvidence"),
        vec![
            (String::from("Strict"), Vec::new()),
            (String::from("NonIncreasing"), Vec::new()),
            (String::from("DescentUnknown"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "RankingDimension"),
        vec![
            (String::from("TreeSize"), vec![String::from("param")]),
            (String::from("ListLength"), vec![String::from("param")]),
            (String::from("ArithmeticValue"), vec![String::from("param")]),
            (String::from("TokenPosition"), vec![String::from("param")]),
            (String::from("SetCardinality"), vec![String::from("param")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "PositiveDescentAmount"),
        vec![
            (String::from("OneStep"), Vec::new()),
            (
                String::from("AdditionalStep"),
                vec![String::from("previous")]
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "DescentSource"),
        vec![
            (
                String::from("ChildAccessor"),
                vec![String::from("accessor")]
            ),
            (String::from("ListShrink"), vec![String::from("amount")]),
            (
                String::from("ArithmeticSubtractDescent"),
                vec![String::from("steps")],
            ),
            (
                String::from("ArithmeticDivideDescent"),
                vec![String::from("divisor")],
            ),
            (String::from("ParserAdvance"), vec![String::from("witness")]),
            (String::from("SetRemoval"), vec![String::from("element")]),
            (String::from("FoldIteration"), Vec::new()),
        ]
    );
    assert_eq!(record_fields(&dag, "TerminationProof"), vec!["dimensions"]);
    assert_eq!(
        record_fields(&dag, "ProofEdge"),
        vec!["caller", "callee", "evidence"]
    );
}

#[test]
fn termination_lattice_functions_preserve_std_body_spans() {
    let dag = Dag::new();

    for name in [
        "evidence_rank",
        "merge_evidence",
        "join_evidence",
        "promote_to_strict",
        "proportional_divisor_to_int",
        "optional_evidence_meet",
        "map_evidence_merge_at",
    ] {
        assert!(
            matches!(arrow_body(&dag, name), ArrowBody::Unparsed(_)),
            "`{name}` should preserve its v3 std body span until std block bodies lower"
        );
    }
}

#[test]
fn termination_lattice_rust_mirror_matches_dag_authority() {
    use DescentEvidence::{DescentUnknown, NonIncreasing, Strict};

    assert_eq!(evidence_rank(Strict), 2);
    assert_eq!(evidence_rank(NonIncreasing), 1);
    assert_eq!(evidence_rank(DescentUnknown), 0);

    for evidence in [Strict, NonIncreasing, DescentUnknown] {
        assert_eq!(merge_evidence(Strict, evidence), evidence);
        assert_eq!(merge_evidence(evidence, Strict), evidence);
        assert_eq!(join_evidence(DescentUnknown, evidence), evidence);
        assert_eq!(join_evidence(evidence, DescentUnknown), evidence);
    }

    assert_eq!(merge_evidence(Strict, Strict), Strict);
    assert_eq!(merge_evidence(Strict, NonIncreasing), NonIncreasing);
    assert_eq!(merge_evidence(NonIncreasing, NonIncreasing), NonIncreasing);
    assert_eq!(
        merge_evidence(NonIncreasing, DescentUnknown),
        DescentUnknown
    );

    assert_eq!(join_evidence(NonIncreasing, Strict), Strict);
    assert_eq!(join_evidence(NonIncreasing, NonIncreasing), NonIncreasing);
    assert_eq!(join_evidence(Strict, DescentUnknown), Strict);

    assert_eq!(promote_to_strict(NonIncreasing), NonIncreasing);
    assert_eq!(promote_to_strict(Strict), Strict);
    assert_eq!(promote_to_strict(DescentUnknown), DescentUnknown);

    assert_eq!(optional_evidence_meet(None, Some(Strict)), Some(Strict));
    assert_eq!(
        optional_evidence_meet(Some(Strict), Some(NonIncreasing)),
        Some(NonIncreasing)
    );

    let mut base = HashMap::new();
    base.insert(String::from("n"), Strict);
    let merged = map_evidence_merge_at(base, String::from("n"), NonIncreasing);
    assert_eq!(merged.get("n"), Some(&NonIncreasing));
    let inserted = map_evidence_merge_at(merged, String::from("m"), Strict);
    assert_eq!(inserted.get("m"), Some(&Strict));
}

#[test]
fn e_p_per_call_descent_evidence_side_table_reads_recursive_call() {
    let dag = compile_to_dag(
        "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)
",
        "e_p_countdown.v3",
    )
    .expect("recursive countdown fixture compiles");

    let entries = per_call_descent_evidence(&dag);
    let countdown = entries
        .iter()
        .find(|entry| entry.caller == "countdown" && entry.callee == "countdown")
        .unwrap_or_else(|| panic!("expected countdown self-call evidence, got {entries:?}"));

    assert_eq!(countdown.evidence.len(), 1);
    match &countdown.evidence[0] {
        SubValueRelation::ArithmeticDescent { param, factor } => {
            assert_eq!(
                param, "param_0",
                "E-P side table uses the stable ordinal scaffold until BindNode exposes parameter names"
            );
            assert_eq!(
                factor,
                &ShrinkFactor::ConstantShrink {
                    steps: PositiveDescentAmount::OneStep
                }
            );
        }
        other => panic!("expected arithmetic descent for countdown(n - 1), got {other:?}"),
    }
}

#[test]
fn e_p_per_call_descent_evidence_classifies_match_payload_self_call_as_strict_sub_value() {
    // Phase-1 broadening: a recursive self-call passing a match-arm payload
    // binding whose scrutinee is the function's parameter directly is the
    // canonical structural-sub-value descent shape (cons-tail recursion).
    // This is the call-site class beyond `arithmetic` / `preserved-value` that
    // the lane needs for `e_p_per_call_descent_evidence_full_coverage`.
    let dag = compile_to_dag(
        "\
type EpList = EpNil | EpCons(EpList)
fn ep_count(xs: EpList) -> Int =
  match xs { EpCons(tail) => ep_count(tail), EpNil => 0 }
",
        "e_p_match_payload.v3",
    )
    .expect("recursive list-count fixture compiles");

    let entries = per_call_descent_evidence(&dag);
    let length = entries
        .iter()
        .find(|entry| entry.caller == "ep_count" && entry.callee == "ep_count")
        .expect("expected ep_count self-call evidence in per-call side table");

    assert_eq!(length.evidence.len(), 1);
    match &length.evidence[0] {
        SubValueRelation::StrictSubValue { field, factor } => {
            // The structural field accessor of a positional variant payload
            // is the lowered Conj field label (`_0`), NOT the user's pattern
            // binding name (`tail`). The pattern `Cons(tail) => length(tail)`
            // and `Cons(t) => length(t)` describe the same structural descent,
            // so substrate provenance must not vary with binding choice.
            assert_eq!(
                field.field_name, "_0",
                "positional-payload structural accessor is the variant Conj's `_0` field"
            );
            assert_eq!(
                field.variant_name, "EpCons",
                "variant_name comes from the parent Disj's variant label"
            );
            assert_eq!(
                field.type_name, "EpList",
                "type_name is the parent Disj declaration name"
            );
            assert_eq!(
                field.element_type, "EpList",
                "element_type is the resolved name of the payload's type"
            );
            assert_eq!(
                factor,
                &ShrinkFactor::UnitShrink,
                "match-payload descent is a unit-shrink: one constructor peeled"
            );
        }
        other => panic!("expected StrictSubValue for ep_count(tail), got {other:?}"),
    }
}

#[test]
fn e_p_per_call_descent_evidence_classifies_match_field_projection_self_call_as_strict_sub_value() {
    with_full_bootstrap_stack(|| {
        // Phase-1 broadening Slice 2 — record-payload sibling of Slice 1.
        // For `match t { EpNode { left: l } => ep_depth(l) }`, `lower.rs`
        // synthesizes a FieldProject transform between the payload port and the
        // user-scope name `l`, so the recursive arg is one indirection beyond the
        // payload port that Slice 1's direct-equality check catches.
        //
        // Same structural-fact discipline as Slice 1: field_name is the
        // FieldProject.field_label (the lowered Conj field label, NOT the user's
        // `l` binding name), and type_name/variant_name come from the parent-Disj
        // lookup of the resolved variant declaration.
        let dag = compile_to_dag(
            "\
type EpRec = EpLeaf | EpNode { left: EpRec }
fn ep_depth(t: EpRec) -> Int =
  match t { EpNode { left: l } => ep_depth(l), EpLeaf => 0 }
",
            "e_p_match_field_projection.v3",
        )
        .expect("recursive record-payload fixture compiles");

        let entries = per_call_descent_evidence(&dag);
        let depth = entries
            .iter()
            .find(|entry| entry.caller == "ep_depth" && entry.callee == "ep_depth")
            .expect("expected ep_depth self-call evidence in per-call side table");

        assert_eq!(depth.evidence.len(), 1);
        match &depth.evidence[0] {
            SubValueRelation::StrictSubValue { field, factor } => {
                assert_eq!(
                    field.field_name, "left",
                    "FieldProject.field_label is the variant Conj's structural field name"
                );
                assert_eq!(
                    field.variant_name, "EpNode",
                    "variant_name comes from the parent Disj's variant label"
                );
                assert_eq!(
                    field.type_name, "EpRec",
                    "type_name is the parent Disj declaration name"
                );
                assert_eq!(
                    field.element_type, "EpRec",
                    "element_type is the resolved name of the projected field's type"
                );
                assert_eq!(
                    factor,
                    &ShrinkFactor::UnitShrink,
                    "record-payload field-projection descent is a unit-shrink: \
                 one constructor + one field peeled"
                );
            }
            other => panic!("expected StrictSubValue for ep_depth(l), got {other:?}"),
        }
    });
}

#[test]
fn e_p_per_call_descent_evidence_classifies_nested_match_self_call_as_strict_sub_value() {
    // Phase-1 broadening Slice 3: a recursive self-call whose argument is
    // the payload binding of an INNER match arm whose scrutinee is itself
    // an outer-match payload binding (not the parameter directly). Slices 1
    // and 2 reject this because `branch.input != param`; Slice 3's
    // `scrutinee_traces_to_param` tracer walks the payload-binding chain
    // until it hits the parameter port (or the depth limit, fail-closed).
    //
    // Cumulative classifier coverage after this slice: direct payload,
    // direct field-projection, AND nested-match descent — all
    // structurally-sound `StrictSubValue` evidence.
    let dag = compile_to_dag(
        "\
type EpListN = EpNilN | EpConsN(EpListN)
fn ep_count2(xs: EpListN) -> Int =
  match xs {
    EpConsN(t1) => match t1 {
      EpConsN(t2) => ep_count2(t2),
      EpNilN => 0
    },
    EpNilN => 0
  }
",
        "e_p_nested_match.v3",
    )
    .expect("nested-match recursion fixture compiles");

    let entries = per_call_descent_evidence(&dag);
    let count = entries
        .iter()
        .find(|entry| entry.caller == "ep_count2" && entry.callee == "ep_count2")
        .expect("expected ep_count2 self-call evidence in per-call side table");

    assert_eq!(count.evidence.len(), 1);
    match &count.evidence[0] {
        SubValueRelation::StrictSubValue { field, factor } => {
            // Structural facts are taken from the INNERMOST variant pattern
            // (where the recursive arg's payload binding originates). The
            // outer-match level only contributes scrutinee-trace continuity.
            assert_eq!(
                field.field_name, "_0",
                "innermost positional payload structural accessor is the variant Conj's `_0`"
            );
            assert_eq!(field.variant_name, "EpConsN");
            assert_eq!(field.type_name, "EpListN");
            assert_eq!(field.element_type, "EpListN");
            assert_eq!(
                factor,
                &ShrinkFactor::UnitShrink,
                "nested descent is sound at any positive depth — \
                 every level peels one constructor"
            );
        }
        other => panic!("expected StrictSubValue for ep_count2(t2), got {other:?}"),
    }
}

#[test]
fn e_p_per_call_descent_evidence_classifies_nested_match_field_projection_self_call_as_strict_sub_value(
) {
    // Phase-1 broadening Slice 3 — nested-match × record-payload product
    // case: the recursive arg is the FieldProject of an INNER match arm
    // whose scrutinee is itself an outer-arm payload binding. Exercises the
    // tracer-extended Slice 2 classifier (the field-projection helper at
    // dag.rs:1780, which Slice 3 generalized but the nested-match-payload
    // test alone doesn't reach).
    let dag = compile_to_dag(
        "\
type EpRecN = EpLeafN | EpNodeN { left: EpRecN }
fn ep_depth2(t: EpRecN) -> Int =
  match t {
    EpNodeN { left: a } => match a {
      EpNodeN { left: b } => ep_depth2(b),
      EpLeafN => 0
    },
    EpLeafN => 0
  }
",
        "e_p_nested_match_field_projection.v3",
    )
    .expect("nested-match record-payload recursion fixture compiles");

    let entries = per_call_descent_evidence(&dag);
    let depth = entries
        .iter()
        .find(|entry| entry.caller == "ep_depth2" && entry.callee == "ep_depth2")
        .expect("expected ep_depth2 self-call evidence in per-call side table");

    assert_eq!(depth.evidence.len(), 1);
    match &depth.evidence[0] {
        SubValueRelation::StrictSubValue { field, factor } => {
            // Innermost FieldProject's structural label.
            assert_eq!(field.field_name, "left");
            assert_eq!(field.variant_name, "EpNodeN");
            assert_eq!(field.type_name, "EpRecN");
            assert_eq!(field.element_type, "EpRecN");
            assert_eq!(factor, &ShrinkFactor::UnitShrink);
        }
        other => panic!("expected StrictSubValue for ep_depth2(b), got {other:?}"),
    }
}

#[test]
fn e_p_per_call_descent_evidence_classifies_nested_match_in_mutual_recursion_scc() {
    // Phase-1 broadening Slice 3 — same nested-match shape as the
    // self-recursive case, but inside a mutually-recursive cluster (SCC).
    // The termination authority for clusters is `ClusterDescentChecker`,
    // a separate code path from `descent_provable`. Both must apply the
    // same nested-binding rule or the same termination fact has two
    // authorities with different acceptance rules (single-authority
    // INVARIANT violation).
    //
    // `ep_alpha`'s inner-match scrutinee `t1` is an outer-arm payload, not
    // the parameter — only the nested-binding rule lets the cluster
    // checker accept the inner arm's `ep_beta(t2)` call as descending.
    let dag = compile_to_dag(
        "\
type EpListM = EpNilM | EpConsM(EpListM)
fn ep_alpha(xs: EpListM) -> Int =
  match xs {
    EpConsM(t1) => match t1 {
      EpConsM(t2) => ep_beta(t2),
      EpNilM => 0
    },
    EpNilM => 0
  }
fn ep_beta(ys: EpListM) -> Int =
  match ys { EpConsM(z) => ep_alpha(z), EpNilM => 0 }
",
        "e_p_nested_match_mutual_recursion.v3",
    )
    .expect("nested-match mutual-recursion fixture compiles (cluster checker accepts)");

    let entries = per_call_descent_evidence(&dag);
    let alpha_to_beta = entries
        .iter()
        .find(|e| e.caller == "ep_alpha" && e.callee == "ep_beta")
        .expect("expected ep_alpha → ep_beta cluster edge in per-call side table");
    // The structural fact this test pins is that the FIXTURE COMPILES —
    // the cluster checker now accepts the nested-match descent shape
    // consistently with `descent_provable`. (`ep_alpha` and `ep_beta`
    // template through different declarations, so the per-call producer's
    // same-template guard emits a `SubValueUnknown` for the cross-edge
    // — that's the fail-closed cross-template default; in-SCC descent
    // proofs are the cluster checker's authority, not this side table's.)
    assert_eq!(alpha_to_beta.evidence.len(), 1);
}

#[test]
fn e_p_per_call_descent_evidence_emits_per_arg_relation_for_multi_arg_self_call() {
    // Phase-1 broadening Slice 4: multi-arg composition. The per-call
    // producer's contract is one `SubValueRelation` PER argument port —
    // the evidence vector matches v2's `ExprCall.descent_evidence:
    // List<SubValueRelation>?` shape (`src/v2/00_core.dag:199`). Slices
    // 1-3 only exercised single-arg recursion; this slice cements the
    // multi-arg contract so future producer broadening preserves
    // per-arg classification.
    //
    // Fixture: a length-with-accumulator self-call. arg 0 (`tail`) is a
    // match-payload binding — Slice 1 classifies as StrictSubValue.
    // arg 1 (`acc + 1`) is an arithmetic non-descent of the second
    // parameter — falls back to SubValueUnknown (sound: the accumulator
    // grows, so it cannot be a sub-value of itself). arg 2 (`limit`) is
    // the third parameter passed unchanged — PreservedValue.
    let dag = compile_to_dag(
        "\
type EpListA = EpNilA | EpConsA(EpListA)
fn ep_count_acc(xs: EpListA, acc: Int, limit: Int) -> Int =
  match xs {
    EpConsA(tail) => ep_count_acc(tail, acc + 1, limit),
    EpNilA => acc
  }
",
        "e_p_multi_arg.v3",
    )
    .expect("multi-arg accumulator fixture compiles");

    let entries = per_call_descent_evidence(&dag);
    let count_acc = entries
        .iter()
        .find(|entry| entry.caller == "ep_count_acc" && entry.callee == "ep_count_acc")
        .expect("expected ep_count_acc self-call evidence in per-call side table");

    assert_eq!(
        count_acc.evidence.len(),
        3,
        "evidence vector length matches argument count (per-arg classification)"
    );

    // arg 0: `tail` — match-payload positional binding, classified by Slice 1.
    match &count_acc.evidence[0] {
        SubValueRelation::StrictSubValue { field, factor } => {
            assert_eq!(field.field_name, "_0");
            assert_eq!(field.variant_name, "EpConsA");
            assert_eq!(field.type_name, "EpListA");
            assert_eq!(factor, &ShrinkFactor::UnitShrink);
        }
        other => panic!("expected StrictSubValue for arg 0 (tail), got {other:?}"),
    }

    // arg 1: `acc + 1` — arithmetic non-descent (Add, not Sub/Div), and the
    // existing arithmetic_descent_relation only matches Sub/Div anyway.
    // Sound fail-closed default: SubValueUnknown.
    assert_eq!(
        count_acc.evidence[1],
        SubValueRelation::SubValueUnknown,
        "non-descending accumulator argument must NOT classify as a sub-value"
    );

    // arg 2: `limit` — third parameter passed unchanged, port equality.
    assert_eq!(
        count_acc.evidence[2],
        SubValueRelation::PreservedValue,
        "unchanged forwarded parameter classifies as PreservedValue"
    );
}

#[test]
fn e_p_per_call_pattern_projects_multi_arg_self_call_from_per_arg_evidence() {
    with_full_bootstrap_stack(|| {
        // Gate 2 continuation: `per_call_pattern_at` is the lens-facing lookup,
        // so it must consume the per-argument evidence vector that Gate 1 emits.
        // A single-element-only projection would make multi-arg recursive calls
        // invisible to cost/complexity consumers even though the side table has
        // already classified their descent argument.
        let dag = compile_to_dag(
            "\
type EpListP = EpNilP | EpConsP(EpListP)
fn ep_count_acc_pattern(xs: EpListP, acc: Int, limit: Int) -> Int =
  match xs {
    EpConsP(tail) => ep_count_acc_pattern(tail, acc + 1, limit),
    EpNilP => acc
  }
",
            "e_p_multi_arg_pattern_lookup.v3",
        )
        .expect("multi-arg accumulator pattern fixture compiles");

        let entry = per_call_descent_evidence(&dag)
            .into_iter()
            .find(|entry| {
                entry.caller == "ep_count_acc_pattern" && entry.callee == "ep_count_acc_pattern"
            })
            .expect("expected ep_count_acc_pattern self-call evidence in per-call side table");

        assert_eq!(
            per_call_pattern_at(&dag, entry.call),
            Some(CallPattern::ChildAccessorCall {
                accessor: String::from("_0")
            }),
            "CallPattern lookup must project the provable descent relation from \
         a multi-arg evidence vector instead of rejecting the call site"
        );
    });
}

#[test]
fn e_p_per_call_descent_evidence_emits_distinct_per_arg_param_labels_for_arithmetic_descent() {
    // Phase-1 broadening Slice 4 (continued): when arithmetic descent
    // applies to multiple arguments at the same call site, each evidence
    // entry's `param` label must reflect the per-arg parameter index
    // (`param_0`, `param_1`, ...), not collapse to a single global label.
    // Pins the per-arg classifier's parameter-index independence at the
    // SubValueRelation level.
    //
    // (v3's existing termination prover requires the first arg to descend
    // structurally; `n - 1` on arg 0 satisfies that, while `m - 1` on arg
    // 1 lets us also exercise classification for a non-first parameter.)
    let dag = compile_to_dag(
        "\
fn ep_two_descent(n: Int, m: Int) -> Int =
  if n == 0 then m else ep_two_descent(n - 1, m - 1)
",
        "e_p_multi_arg_arith.v3",
    )
    .expect("two-param arithmetic-descent fixture compiles");

    let entries = per_call_descent_evidence(&dag);
    let two_descent = entries
        .iter()
        .find(|entry| entry.caller == "ep_two_descent" && entry.callee == "ep_two_descent")
        .expect("expected ep_two_descent self-call evidence");

    assert_eq!(two_descent.evidence.len(), 2);
    let expected_factor = ShrinkFactor::ConstantShrink {
        steps: PositiveDescentAmount::OneStep,
    };
    // arg 0: `n - 1` → ArithmeticDescent against param_0.
    match &two_descent.evidence[0] {
        SubValueRelation::ArithmeticDescent { param, factor } => {
            assert_eq!(param, "param_0");
            assert_eq!(factor, &expected_factor);
        }
        other => panic!("expected ArithmeticDescent for arg 0 (n - 1), got {other:?}"),
    }
    // arg 1: `m - 1` → ArithmeticDescent against param_1, with the
    // PER-ARG ordinal label (not collapsed to param_0).
    match &two_descent.evidence[1] {
        SubValueRelation::ArithmeticDescent { param, factor } => {
            assert_eq!(
                param, "param_1",
                "second-argument arithmetic descent's ordinal label tracks the per-arg parameter index"
            );
            assert_eq!(factor, &expected_factor);
        }
        other => panic!("expected ArithmeticDescent for arg 1 (m - 1), got {other:?}"),
    }
}

#[test]
fn e_p_per_call_descent_evidence_indirect_call_fail_closed_invariance() {
    // Phase-1 broadening Slice 6 — cementing-only. Pin the fail-closed
    // invariance for indirect-call dispatch (`w.f(x)` over a parameter
    // whose declared type is `Wrapper { f: fn(Int) -> Int }`).
    //
    // Substrate state at HEAD: `TransformDispatch::Indirect` /
    // `ArrowPortRef` are forward-looking comment-only references in
    // `prereq_x_call_on_field_access_ratchet_test.rs` — the substrate
    // types do not exist yet. Indirect calls lower to a typed
    // `Diagnostic::ResolveError` naming the X1.b prerequisite, never
    // reaching a callable Transform node. The per-call descent producer
    // therefore correctly emits NO evidence for them — fail-closed by
    // structural absence rather than a classifier extension.
    //
    // This test is a tripwire: when `TransformDispatch::Indirect` /
    // `ArrowPortRef` substrate eventually lands and indirect calls
    // start lowering to a real Transform, the X1.b ResolveError will
    // stop firing and `per_call_descent_evidence` will start producing
    // entries for these call sites. This test will then fail, surfacing
    // the producer-extension obligation atomically with the substrate
    // landing — same discipline pattern Slice 4's per-arg cementing
    // established.
    let src = r#"
type Wrapper { f: fn(Int) -> Int }

fn invoke(w: Wrapper, x: Int) -> Int = w.f(x)
"#;
    let dag = match compile_to_dag(src, "e_p_indirect_call_cementing.v3") {
        Ok(_) => panic!(
            "indirect-call lowering must remain blocked at HEAD until \
             the TransformDispatch::Indirect substrate lands; this test \
             is the tripwire surfacing that landing"
        ),
        Err(CompileError::Semantic(dag)) => dag,
        Err(err) => panic!("expected Semantic compile failure, got {err:?}"),
    };

    // (a) The X1.b ResolveError must fire for the indirect call site.
    let saw_x1b_diagnostic = dag.diagnostics().iter().any(|(_, d)| match d {
        Diagnostic::ResolveError { name, .. } => {
            name.contains("Prereq-X1.b") || name.contains("parameter")
        }
        _ => false,
    });
    assert!(
        saw_x1b_diagnostic,
        "expected X1.b ResolveError naming the indirect parameter call site"
    );

    // (b) The per-call descent producer must emit NO evidence entry
    // attributed to `invoke`'s body — there is no callable Transform
    // for the indirect call to populate the side table from. Any future
    // entry implies a substrate landing the producer must then classify.
    let entries = per_call_descent_evidence(&dag);
    let invoke_entries: Vec<_> = entries
        .iter()
        .filter(|entry| entry.caller == "invoke")
        .collect();
    assert!(
        invoke_entries.is_empty(),
        "indirect calls must not appear in the per-call side table at HEAD; \
         got entries — substrate has likely landed and the producer \
         needs an Indirect classifier slice"
    );
}

#[test]
fn e_p_per_call_descent_evidence_constant_descent_termination_matches_producer_acceptance_boundary()
{
    // Symmetric to the proportional-descent boundary test below but for
    // the Sub arm: the termination prover and per-call descent producer
    // MUST share the same acceptance boundary on `param - k`.
    // `positive_amount_from_i64` (dag.rs:1031-1032) materializes
    // `PositiveDescentAmount` for `1..=MAX_PEANO_MATERIALIZATION`;
    // accepting any larger literal in `is_strictly_smaller` would let
    // `f(n - 257)` pass termination while the producer fails to
    // materialize and emits `SubValueUnknown` — parallel-authority
    // split-brain (the discipline Slice 3 cemented for descent_provable
    // / ClusterDescentChecker single-authority alignment).
    //
    // This test pins the boundary by rejecting `n - 257` at compile
    // time. If either authority's range shifts, this test surfaces the
    // divergence.
    let err = compile_to_dag(
        "\
fn ep_oversize_subtractor(n: Int) -> Int =
  if n == 0 then 0 else ep_oversize_subtractor(n - 257)
",
        "e_p_oversize_subtractor.v3",
    )
    .expect_err("subtractor beyond producer's materialization range must be rejected");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostics, got {err:?}");
    };
    let saw_termination_diagnostic = dag.diagnostics().iter().any(|(_, d)| match d {
        Diagnostic::ResolveError { name, .. } => name.contains("ep_oversize_subtractor"),
        _ => false,
    });
    assert!(
        saw_termination_diagnostic,
        "expected a termination diagnostic naming `ep_oversize_subtractor`"
    );
}

#[test]
fn e_p_per_call_descent_evidence_fails_closed_for_non_self_call() {
    let dag = compile_to_dag(
        "\
fn helper(n: Int) -> Int = n - 1
fn caller(n: Int) -> Int = helper(n)
",
        "e_p_non_self_call.v3",
    )
    .expect("non-self call fixture compiles");

    let entries = per_call_descent_evidence(&dag);
    let non_self = entries
        .iter()
        .find(|entry| entry.caller == "caller" && entry.callee == "helper")
        .unwrap_or_else(|| panic!("expected non-self callable edge evidence, got {entries:?}"));

    assert_eq!(
        non_self.evidence,
        vec![SubValueRelation::SubValueUnknown],
        "resolved non-self callable edges must fail closed instead of disappearing"
    );
}

#[test]
fn e_p_per_call_pattern_projects_preserved_value_to_same_argument_call() {
    assert_eq!(
        sub_value_relation_to_call_pattern(&SubValueRelation::PreservedValue),
        Some(CallPattern::SameArgumentCall),
        "gate (1) broadens CallPattern coverage by projecting preserved self-call evidence \
         to SameArgumentCall without authoring a new lowered carrier or lens consumer"
    );
}

#[test]
fn e_p_per_call_pattern_preserves_induction_child_accessor_projection() {
    let field = v3_compiler::dag::InductiveField {
        type_name: String::from("Tree"),
        variant_name: String::from("Node"),
        field_name: String::from("left"),
        shape: v3_compiler::dag::RecursionShape::DirectRecursion,
        element_type: String::from("Tree"),
    };
    assert_eq!(
        sub_value_relation_to_call_pattern(&SubValueRelation::StrictSubValue {
            field: field.clone(),
            factor: ShrinkFactor::UnitShrink,
        }),
        Some(CallPattern::ChildAccessorCall {
            accessor: String::from("left")
        }),
        "StrictSubValue has an authoritative ChildAccessorCall projection in std.induction.dag"
    );
    assert_eq!(
        sub_value_relation_to_call_pattern(&SubValueRelation::IteratedSubValue { field }),
        Some(CallPattern::ChildAccessorCall {
            accessor: String::from("left")
        }),
        "IteratedSubValue has the same authoritative ChildAccessorCall projection in std.induction.dag"
    );
}

/// T-E-P-Producer-Broadening **gate (2)** — `e_p_call_pattern_lookup_authoritative`.
///
/// Pins `per_call_descent_evidence` as the **single** lookup authority for
/// per-call `SubValueRelation` evidence over `TransformTarget::Callable`
/// transforms. The gate guards against parallel producer growth while the
/// other two gates (`_full_coverage`, `_per_call_landed`) extend the
/// producer's coverage and move evidence onto a substrate carrier.
///
/// **Authoritativeness check (behavioral):**
/// - For every `Behavior::Transform` whose target is `TransformTarget::Callable`
///   and whose span sits in the user fixture file, the producer yields
///   **exactly one** `CallDescentEvidence` entry. (Bootstrap-scoped
///   Callable transforms are exercised by neighbouring `e_p_*` tests; this
///   test's candidate set is span-narrowed for tractable enumeration.)
/// - Each entry's `call: NodeId` is **unique** in the producer's output —
///   no parallel/duplicate producer is silently appending entries.
/// - Every producer entry corresponds to a real `Behavior::Transform` with
///   a `Callable` target — no synthetic entries.
/// - The producer-emitted set (filtered to the fixture file) **equals**
///   the candidate set — totality over the fixture-file domain.
///
/// **Why a behavioral test over a source-grep ratchet:** the helpers
/// `classify_call_argument` / `arithmetic_descent_relation` are already
/// private to `dag.rs`; the public surface is the producer fn + the
/// `SubValueRelation` / `CallDescentEvidence` types. The substantive
/// "single authority" claim is that no *callable-edge → evidence* mapping
/// is reconstructed elsewhere — this is observable as cardinality match
/// between the producer's output and the live Callable-transform set.
/// Future drift (a second producer adding entries; a missing call site) is
/// caught by these structural assertions without coupling to source text.
#[test]
fn per_call_descent_evidence_is_single_lookup_authority_over_callable_transforms() {
    use std::collections::HashSet;
    use v3_compiler::dag::{Behavior, NodeId, TransformTarget};

    // Two-function fixture exercises both branches of the producer's match
    // (`caller_template == callee_template` self-recursion + resolved
    // cross-template fail-closed). The single-authority claim must hold for
    // the cross-template branch too — a future producer that adds a parallel
    // walker for cross-template evidence (rather than going through
    // `per_call_descent_evidence`'s fail-closed path) would otherwise slip
    // past a self-recursion-only ratchet. Cardinality and uniqueness claims
    // are gate-level properties; future producer-broadening slices (gate 1)
    // add detection logic without changing this ratchet's shape.
    let source = "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)

fn helper(n: Int) -> Int = n - 1

fn caller(n: Int) -> Int = helper(n)
";
    let dag = compile_to_dag(source, "e_p_lookup_authority.v3")
        .expect("countdown self-call + caller→helper cross-template fixture compiles");

    // Candidate set: every `Callable`-target transform whose *span* sits in
    // the user fixture file. This narrows past the bootstrap dag's thousands
    // of std/spec Callable transforms — the producer covers them under the
    // same single-authority rule, but exhaustive iteration over the full
    // bootstrap would dwarf the signal and is unnecessary for a structural
    // cardinality assertion. The producer-emitted set (filtered identically)
    // is compared to this candidate set below.
    let fixture_callable_transforms: HashSet<NodeId> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_transform)
        .filter(|t| matches!(t.target, TransformTarget::Callable(_)))
        .filter(|t| t.span.file == "e_p_lookup_authority.v3")
        .map(|t| t.id)
        .collect();
    // Sanity floor: the fixture must exercise both producer match branches
    // (self-recursive countdown + cross-template caller→helper). Anything
    // less leaves the cross-template fail-closed branch unverified — the
    // coverage-equality assertion below would then trivially hold for a
    // self-recursion-only producer. The cross-template caller→helper edge
    // fails closed to `SubValueUnknown` (per
    // `e_p_per_call_descent_evidence_fails_closed_for_non_self_call`), but
    // it MUST still appear in the producer's entry set — single-authority
    // means the producer covers all Callable edges, not just self-recursive.
    assert!(
        fixture_callable_transforms.len() >= 2,
        "fixture must compile to >= 2 Callable transforms (countdown self-call + caller→helper \
         cross-template); got {}",
        fixture_callable_transforms.len()
    );

    let all_entries = per_call_descent_evidence(&dag);

    // Filter to user-fixture entries for the cardinality comparison; the
    // producer emits entries for the full bootstrap dag (thousands of std
    // callable transforms), but the structural authoritativeness claim is
    // gauged on the user fixture's narrow set where ground-truth is
    // tractable to enumerate. Bootstrap-coverage is exercised by the same
    // producer over std fixtures and validated by neighbouring tests
    // (`e_p_per_call_descent_evidence_*`).
    let entries: Vec<&v3_compiler::dag::CallDescentEvidence> = all_entries
        .iter()
        .filter(|entry| {
            dag.node(entry.call)
                .as_transform()
                .map(|t| t.span.file == "e_p_lookup_authority.v3")
                .unwrap_or(false)
        })
        .collect();

    // Uniqueness: every entry's NodeId appears exactly once. A second producer
    // appending evidence would either duplicate `call` ids or shift them.
    let unique_call_ids: HashSet<NodeId> = entries.iter().map(|entry| entry.call).collect();
    assert_eq!(
        unique_call_ids.len(),
        entries.len(),
        "per_call_descent_evidence must emit unique entries per Callable transform; \
         duplicate `call` ids indicate a parallel producer appending evidence"
    );

    // Reality: every entry must correspond to a real Callable-targeted
    // transform. A synthetic entry (parallel producer building from another
    // source) would fail this check.
    for entry in &entries {
        let node = dag.node(entry.call);
        let Behavior::Transform(t) = node else {
            panic!(
                "per_call_descent_evidence emitted entry for non-Transform node {:?}; \
                 call ids must reference live transforms",
                entry.call
            );
        };
        let TransformTarget::Callable(_) = t.target else {
            panic!(
                "per_call_descent_evidence emitted entry for non-Callable target on transform {:?}; \
                 the producer scope is Callable transforms only",
                entry.call
            );
        };
        // The transform must be in the candidate set — the producer cannot
        // reach Callable transforms outside the body-bind ownership rule
        // without a parallel walker.
        assert!(
            fixture_callable_transforms.contains(&entry.call),
            "per_call_descent_evidence emitted entry for transform {:?} not in the \
             owned-Callable-transforms candidate set; this indicates a parallel \
             walker reaching transforms outside the body-bind ownership rule",
            entry.call
        );
    }

    // Coverage: the producer must not silently drop call sites it owns. We
    // assert the producer-emitted set (filtered to the fixture file) equals
    // the candidate set (Callable transforms in the fixture file). Any
    // mismatch indicates either (a) a missing call site, or (b) the producer
    // overshooting its scope.
    let entries_call_set: HashSet<NodeId> = entries.iter().map(|entry| entry.call).collect();
    assert_eq!(
        entries_call_set, fixture_callable_transforms,
        "per_call_descent_evidence's emitted call set (restricted to the fixture file) \
         must equal the set of Callable transforms in the fixture file. Mismatch indicates \
         either (a) a missing call site (gate violation: producer is not the single \
         authority because something else covers what the producer drops) or (b) the \
         producer reaching outside body-owned transforms (gate violation: producer \
         overshoots its scope)"
    );
}

#[test]
fn e_p_runtime_mirror_matches_induction_carrier_shape() {
    let dag = Dag::new();

    assert_eq!(
        sum_variants(&dag, "RecursionShape"),
        vec![
            (String::from("DirectRecursion"), vec![]),
            (String::from("ListRecursion"), vec![]),
            (String::from("OptionalRecursion"), vec![]),
            (String::from("SetRecursion"), vec![]),
            (String::from("MapValueRecursion"), vec![]),
        ],
        "Rust RecursionShape mirror in dag.rs must stay aligned with src/v3/std/induction.dag"
    );
    assert_eq!(
        record_fields(&dag, "InductiveField"),
        vec![
            "type_name",
            "variant_name",
            "field_name",
            "shape",
            "element_type"
        ],
        "Rust InductiveField mirror in dag.rs must stay aligned with src/v3/std/induction.dag"
    );
    assert_eq!(
        sum_variants(&dag, "SubValueRelation"),
        vec![
            (
                String::from("StrictSubValue"),
                vec![String::from("field"), String::from("factor")]
            ),
            (
                String::from("IteratedSubValue"),
                vec![String::from("field")]
            ),
            (
                String::from("ArithmeticDescent"),
                vec![String::from("param"), String::from("factor")]
            ),
            (String::from("PreservedValue"), vec![]),
            (String::from("SubValueUnknown"), vec![]),
        ],
        "Rust SubValueRelation mirror in dag.rs must stay aligned with src/v3/std/induction.dag"
    );
}

#[test]
fn computation_carriers_bootstrap_from_v3_std() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load computation carriers cleanly: {:?}",
        dag.diagnostics()
    );

    assert_eq!(
        sum_variants(&dag, "SizeBound"),
        vec![
            (String::from("CollectionSize"), vec![String::from("param")]),
            (
                String::from("ParserStreamSize"),
                vec![String::from("witness")],
            ),
            (
                String::from("WorklistDrainSize"),
                vec![String::from("element")],
            ),
            (String::from("TreeSize"), vec![String::from("param")]),
            (String::from("ArithmeticParam"), vec![String::from("param")]),
            (String::from("ExplicitCountZero"), Vec::new()),
            (
                String::from("ExplicitCountPositive"),
                vec![String::from("steps")],
            ),
            (String::from("Forever"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "CallPattern"),
        vec![
            (
                String::from("ChildAccessorCall"),
                vec![String::from("accessor")],
            ),
            (
                String::from("CollectionShrinkCall"),
                vec![String::from("amount"), String::from("collection")],
            ),
            (
                String::from("ArithmeticSubtractCall"),
                vec![String::from("steps"), String::from("ring_param")],
            ),
            (
                String::from("ArithmeticDivideCall"),
                vec![String::from("divisor"), String::from("ring_param")],
            ),
            (
                String::from("ParserAdvanceCall"),
                vec![String::from("witness")],
            ),
            (
                String::from("WorklistDrainCall"),
                vec![String::from("element")],
            ),
            (
                String::from("FoldBodyCall"),
                vec![String::from("outer_collection")],
            ),
            (String::from("SameArgumentCall"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "ProportionalDivisor"),
        vec![
            (String::from("DivideByTwo"), Vec::new()),
            (String::from("StrictlyLarger"), vec![String::from("inner")],),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "ShrinkFactor"),
        vec![
            (String::from("UnitShrink"), Vec::new()),
            (String::from("ConstantShrink"), vec![String::from("steps")]),
            (
                String::from("ProportionalShrink"),
                vec![String::from("divisor")],
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "IterationPrimitive"),
        vec![
            (String::from("Fold"), Vec::new()),
            (String::from("Descend"), Vec::new()),
            (String::from("Repeat"), Vec::new()),
        ]
    );
    assert_eq!(
        record_fields(&dag, "LoweringTarget"),
        vec!["primitive", "bound", "evidence", "factor"]
    );
    assert_eq!(
        sum_variants(&dag, "IterationDimension"),
        vec![
            (String::from("TreeDescent"), Vec::new()),
            (String::from("CollectionFold"), Vec::new()),
            (String::from("ArithmeticRepeat"), Vec::new()),
        ]
    );
}

#[test]
fn computation_lowering_functions_preserve_std_body_spans() {
    let dag = Dag::new();

    for name in [
        "tree_size_bound",
        "lower_call_pattern",
        "positive_descent_count",
        "size_bound_param",
        "is_constant_bound",
        "forever_iteration_bound",
        "constant_bound_value",
        "algebra_profile_to_dimension",
        "type_iteration_dimension",
    ] {
        assert!(
            matches!(arrow_body(&dag, name), ArrowBody::Unparsed(_)),
            "`{name}` should preserve its v3 std body span until std block bodies lower"
        );
    }
}

#[test]
fn computation_lowering_rust_mirror_matches_dag_authority() {
    use v3_compiler::dag::ShrinkFactor::{ConstantShrink, ProportionalShrink};
    use CallPattern::{
        ArithmeticDivideCall, ArithmeticSubtractCall, ChildAccessorCall, CollectionShrinkCall,
        FoldBodyCall, ParserAdvanceCall, SameArgumentCall, WorklistDrainCall,
    };
    use DescentEvidence::{NonIncreasing, Strict};
    use IterationPrimitive::{Descend, Fold, Repeat};
    use SizeBound::{
        ArithmeticParam, CollectionSize, Forever, ParserStreamSize, TreeSize, WorklistDrainSize,
    };

    let cases = vec![
        (
            ChildAccessorCall {
                accessor: String::from("left"),
            },
            LoweringTarget {
                primitive: Descend,
                bound: TreeSize {
                    param: String::from("left"),
                },
                evidence: Strict,
                factor: None,
            },
        ),
        (
            CollectionShrinkCall {
                amount: PositiveDescentAmount::OneStep,
                collection: String::from("xs"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: CollectionSize {
                    param: String::from("xs"),
                },
                evidence: Strict,
                factor: Some(ConstantShrink {
                    steps: PositiveDescentAmount::OneStep,
                }),
            },
        ),
        (
            CollectionShrinkCall {
                amount: PositiveDescentAmount::AdditionalStep {
                    previous: Box::new(PositiveDescentAmount::OneStep),
                },
                collection: String::from("xs"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: CollectionSize {
                    param: String::from("xs"),
                },
                evidence: Strict,
                factor: Some(ConstantShrink {
                    steps: PositiveDescentAmount::AdditionalStep {
                        previous: Box::new(PositiveDescentAmount::OneStep),
                    },
                }),
            },
        ),
        (
            ArithmeticSubtractCall {
                steps: PositiveDescentAmount::OneStep,
                ring_param: String::from("n"),
            },
            LoweringTarget {
                primitive: Repeat,
                bound: ArithmeticParam {
                    param: String::from("n"),
                },
                evidence: Strict,
                factor: Some(ConstantShrink {
                    steps: PositiveDescentAmount::OneStep,
                }),
            },
        ),
        (
            ArithmeticDivideCall {
                divisor: ProportionalDivisor::DivideByTwo,
                ring_param: String::from("n"),
            },
            LoweringTarget {
                primitive: Repeat,
                bound: ArithmeticParam {
                    param: String::from("n"),
                },
                evidence: Strict,
                factor: Some(ProportionalShrink {
                    divisor: ProportionalDivisor::DivideByTwo,
                }),
            },
        ),
        (
            ArithmeticDivideCall {
                divisor: ProportionalDivisor::StrictlyLarger {
                    inner: Box::new(ProportionalDivisor::DivideByTwo),
                },
                ring_param: String::from("n"),
            },
            LoweringTarget {
                primitive: Repeat,
                bound: ArithmeticParam {
                    param: String::from("n"),
                },
                evidence: Strict,
                factor: Some(ProportionalShrink {
                    divisor: ProportionalDivisor::StrictlyLarger {
                        inner: Box::new(ProportionalDivisor::DivideByTwo),
                    },
                }),
            },
        ),
        (
            ParserAdvanceCall {
                witness: String::from("advance"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: ParserStreamSize {
                    witness: String::from("advance"),
                },
                evidence: Strict,
                factor: None,
            },
        ),
        (
            WorklistDrainCall {
                element: String::from("item"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: WorklistDrainSize {
                    element: String::from("item"),
                },
                evidence: Strict,
                factor: None,
            },
        ),
        (
            FoldBodyCall {
                outer_collection: String::from("items"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: CollectionSize {
                    param: String::from("items"),
                },
                evidence: NonIncreasing,
                factor: None,
            },
        ),
        (
            SameArgumentCall,
            LoweringTarget {
                primitive: Repeat,
                bound: Forever,
                evidence: NonIncreasing,
                factor: None,
            },
        ),
    ];

    for (pattern, expected) in cases {
        assert_eq!(lower_call_pattern(pattern), expected);
    }
}

#[test]
fn computation_size_bound_helpers_match_dag_authority() {
    let tree = tree_size_bound(String::from("node"));
    let collection = SizeBound::CollectionSize {
        param: String::from("items"),
    };
    let parser_stream = SizeBound::ParserStreamSize {
        witness: String::from("tok"),
    };
    let worklist_drain = SizeBound::WorklistDrainSize {
        element: String::from("wl"),
    };
    let arithmetic = SizeBound::ArithmeticParam {
        param: String::from("n"),
    };
    let explicit = SizeBound::ExplicitCountPositive {
        steps: positive_amount_from_i64(7).expect("literal 7 is in Peano materialization range"),
    };
    let explicit_zero = SizeBound::ExplicitCountZero;
    let forever = SizeBound::Forever;

    assert_eq!(size_bound_param(&tree), Some("node"));
    assert_eq!(size_bound_param(&collection), Some("items"));
    assert_eq!(size_bound_param(&parser_stream), Some("tok"));
    assert_eq!(size_bound_param(&worklist_drain), Some("wl"));
    assert_eq!(size_bound_param(&arithmetic), Some("n"));
    assert_eq!(size_bound_param(&explicit), None);
    assert_eq!(size_bound_param(&explicit_zero), None);
    assert_eq!(size_bound_param(&forever), None);

    assert!(!is_constant_bound(&tree));
    assert!(!is_constant_bound(&collection));
    assert!(!is_constant_bound(&arithmetic));
    assert!(is_constant_bound(&explicit));
    assert!(is_constant_bound(&explicit_zero));
    assert!(is_constant_bound(&forever));

    assert_eq!(constant_bound_value(&explicit_zero), Some(0));
    assert_eq!(constant_bound_value(&explicit), Some(7));
    assert_eq!(constant_bound_value(&forever), Some(i64::MAX));
    assert_eq!(constant_bound_value(&tree), None);
}

#[test]
fn computation_iteration_dimension_helpers_match_kernel_profile_authority() {
    use AlgebraProfile::{
        ApproximateFieldProfile, BooleanAlgebraCollectionProfile, BooleanAlgebraProfile,
        FreeMonoidCollectionProfile, FreeMonoidScalarProfile, OrderedRingProfile,
        PartialFunctionProfile,
    };
    use IterationDimension::{ArithmeticRepeat, CollectionFold, TreeDescent};

    for profile in [
        FreeMonoidCollectionProfile,
        FreeMonoidScalarProfile,
        BooleanAlgebraCollectionProfile,
        PartialFunctionProfile,
    ] {
        assert_eq!(algebra_profile_to_dimension(profile), Some(CollectionFold));
    }
    for profile in [OrderedRingProfile, ApproximateFieldProfile] {
        assert_eq!(
            algebra_profile_to_dimension(profile),
            Some(ArithmeticRepeat)
        );
    }
    assert_eq!(algebra_profile_to_dimension(BooleanAlgebraProfile), None);

    assert_eq!(type_iteration_dimension("Node"), Some(TreeDescent));
    assert_eq!(type_iteration_dimension("List"), Some(CollectionFold));
    assert_eq!(type_iteration_dimension("Map"), Some(CollectionFold));
    assert_eq!(type_iteration_dimension("Int"), Some(ArithmeticRepeat));
    assert_eq!(type_iteration_dimension("Float"), Some(ArithmeticRepeat));
    assert_eq!(type_iteration_dimension("Bool"), None);
    assert_eq!(type_iteration_dimension("UserType"), None);
}

#[test]
fn v3_kernel_algebra_profile_reads_lowered_dag_map_authority() {
    let dag = Dag::new();
    let decl = dag
        .declaration_by_name("kernel_algebra_profile")
        .expect("kernel_algebra_profile declaration exists");
    let Some(ValueBody::Map(entries)) = &decl.value_body else {
        panic!(
            "kernel_algebra_profile should lower from dsl/std/algebra.dag to ValueBody::Map, got {:?}",
            decl.value_body
        );
    };
    assert_eq!(entries.entries().len(), 7);
    assert_eq!(
        dag.kernel_algebra_profile("Int"),
        Some(AlgebraProfile::OrderedRingProfile)
    );
    assert_eq!(
        dag.kernel_algebra_profile("Float"),
        Some(AlgebraProfile::ApproximateFieldProfile)
    );
    assert_eq!(
        dag.kernel_algebra_profile("Bool"),
        Some(AlgebraProfile::BooleanAlgebraProfile)
    );
    assert_eq!(
        dag.kernel_algebra_profile("String"),
        Some(AlgebraProfile::FreeMonoidScalarProfile)
    );
    assert_eq!(
        dag.kernel_algebra_profile("List"),
        Some(AlgebraProfile::FreeMonoidCollectionProfile)
    );
    assert_eq!(
        dag.kernel_algebra_profile("Set"),
        Some(AlgebraProfile::BooleanAlgebraCollectionProfile)
    );
    assert_eq!(
        dag.kernel_algebra_profile("Map"),
        Some(AlgebraProfile::PartialFunctionProfile)
    );
    assert_eq!(dag.kernel_algebra_profile("UserType"), None);
}

#[test]
fn v3_kernel_algebra_profile_accessor_prefers_local_map_data_over_rust_fallback() {
    let source = "type AlgebraProfile\n  = OrderedRingProfile\n  | ApproximateFieldProfile\n  | BooleanAlgebraProfile\n  | BooleanAlgebraCollectionProfile\n  | FreeMonoidScalarProfile\n  | FreeMonoidCollectionProfile\n  | PartialFunctionProfile\n\n\
data kernel_algebra_profile: Map<String, AlgebraProfile> = {\n  \"Int\": BooleanAlgebraProfile,\n  \"Custom\": PartialFunctionProfile\n}\n";
    let dag = compile_to_dag(source, "local_kernel_profile_authority.v3")
        .expect("local kernel_algebra_profile map should lower");

    assert_eq!(
        dag.kernel_algebra_profile("Int"),
        Some(AlgebraProfile::BooleanAlgebraProfile),
        "accessor must read the lowered map instead of a hard-coded Rust profile table"
    );
    assert_eq!(
        dag.kernel_algebra_profile("Custom"),
        Some(AlgebraProfile::PartialFunctionProfile)
    );
    assert_eq!(dag.kernel_algebra_profile("Float"), None);
}

#[test]
fn substrate_coproducts_match_runtime_carriers() {
    let dag = Dag::new();

    assert_eq!(
        sum_variants(&dag, "PortState"),
        vec![
            (String::from("Uninferred"), Vec::new()),
            (String::from("Resolved"), vec![String::from("_0")]),
            (String::from("Unresolved"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "LiteralBits"),
        vec![
            (String::from("LitInt"), vec![String::from("_0")]),
            (String::from("LitBool"), vec![String::from("_0")]),
            (String::from("LitString"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "AtomPayload"),
        vec![
            (String::from("Literal"), vec![String::from("_0")]),
            (
                String::from("UnresolvedIdentifier"),
                vec![String::from("_0")],
            ),
            (
                String::from("ResolvedByStructure"),
                vec![String::from("_0")],
            ),
            (String::from("ResolvedByName"), vec![String::from("_0")],),
            (String::from("TypeParam"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "Interval"),
        vec![
            (
                String::from("BoundedInterval"),
                vec![String::from("lower"), String::from("width")],
            ),
            (String::from("Unbounded"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "BoundDeclaration"),
        vec![
            (String::from("StaticBound"), vec![String::from("_0")]),
            (String::from("PlatformDependent"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "TargetIntegerInhabitanceBound"),
        vec![
            (String::from("BoundUnspecified"), Vec::new()),
            (String::from("StaticBoundFact"), vec![String::from("_0")]),
            (String::from("PlatformDependentFact"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "PositiveIntervalWidth"),
        vec![
            (String::from("OneUnit"), Vec::new()),
            (
                String::from("AdditionalUnit"),
                vec![String::from("previous")],
            ),
            (String::from("UnitCount"), vec![String::from("units")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "IntervalWidth"),
        vec![
            (String::from("ZeroWidth"), Vec::new()),
            (String::from("PositiveWidth"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "CardinalityBound"),
        vec![
            (String::from("Exact"), vec![String::from("_0")]),
            (String::from("AtMostOne"), Vec::new()),
            (String::from("Unbounded"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "FieldValue"),
        vec![
            (String::from("Literal"), vec![String::from("_0")]),
            (String::from("Reference"), vec![String::from("_0")]),
            (String::from("Record"), vec![String::from("_0")]),
            (String::from("List"), vec![String::from("_0")]),
            (String::from("Map"), vec![String::from("_0")]),
            (
                String::from("Variant"),
                vec![String::from("constructor"), String::from("payload")],
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "ValueBody"),
        vec![
            (String::from("ValueBodyUnparsed"), vec![String::from("_0")]),
            (
                String::from("ValueBodyStructural"),
                vec![String::from("fields")]
            ),
            (String::from("ValueBodyScalar"), vec![String::from("_0")]),
            (String::from("ValueBodyList"), vec![String::from("_0")]),
            (String::from("ValueBodyMap"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "ArrowBody"),
        vec![
            (String::from("UserDefined"), vec![String::from("_0")]),
            (
                String::from("ExternalRealization"),
                vec![String::from("_0")],
            ),
            (String::from("Pending"), Vec::new()),
            (String::from("NoBody"), Vec::new()),
            (String::from("Unparsed"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "TypeConnective"),
        vec![
            (String::from("Atom"), vec![String::from("_0")]),
            (String::from("Conj"), vec![String::from("children")]),
            (String::from("Disj"), vec![String::from("variants")]),
            (
                String::from("Arrow"),
                vec![
                    String::from("inputs"),
                    String::from("output"),
                    String::from("body"),
                ],
            ),
            (
                String::from("Cardinality"),
                vec![String::from("element"), String::from("bound")],
            ),
            (
                String::from("Instantiation"),
                vec![String::from("template"), String::from("arguments")],
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "ArithmeticOp"),
        vec![
            (String::from("Add"), Vec::new()),
            (String::from("Sub"), Vec::new()),
            (String::from("Mul"), Vec::new()),
            (String::from("Div"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "ComparisonOp"),
        vec![
            (String::from("Eq"), Vec::new()),
            (String::from("Ne"), Vec::new()),
            (String::from("Lt"), Vec::new()),
            (String::from("Le"), Vec::new()),
            (String::from("Gt"), Vec::new()),
            (String::from("Ge"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "LogicalOp"),
        vec![
            (String::from("And"), Vec::new()),
            (String::from("Or"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "OperatorKind"),
        vec![
            (String::from("Arithmetic"), vec![String::from("_0")]),
            (String::from("Comparison"), vec![String::from("_0")]),
            (String::from("Logical"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "TransformTarget"),
        vec![
            (String::from("Callable"), vec![String::from("_0")]),
            (
                String::from("UnresolvedFieldProject"),
                vec![String::from("field_label")],
            ),
            (
                String::from("ResolvedFieldProject"),
                vec![String::from("field_label")],
            ),
            (String::from("Operator"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "BranchPattern"),
        vec![
            (
                String::from("UnresolvedVariant"),
                vec![String::from("name"), String::from("span")],
            ),
            (String::from("ResolvedVariant"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "LoopBound"),
        vec![
            (String::from("Cardinality"), vec![String::from("count")]),
            (
                String::from("Descent"),
                vec![String::from("cluster"), String::from("measure")],
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "Behavior"),
        vec![
            (String::from("Value"), vec![String::from("_0")]),
            (String::from("Transform"), vec![String::from("_0")]),
            (String::from("Branch"), vec![String::from("_0")]),
            (String::from("Loop"), vec![String::from("_0")]),
            (String::from("Bind"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "VariantPayload"),
        vec![
            (String::from("Positional"), vec![String::from("_0")]),
            (String::from("Record"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "SurfaceType"),
        vec![
            (
                String::from("Named"),
                vec![String::from("name"), String::from("span")],
            ),
            (
                String::from("Parameterized"),
                vec![
                    String::from("name"),
                    String::from("args"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Optional"),
                vec![String::from("inner"), String::from("span")],
            ),
            (
                String::from("Arrow"),
                vec![
                    String::from("inputs"),
                    String::from("output"),
                    String::from("span"),
                ],
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "SurfacePattern"),
        vec![
            (
                String::from("BareVariant"),
                vec![String::from("name"), String::from("span")],
            ),
            (
                String::from("VariantWith"),
                vec![
                    String::from("name"),
                    String::from("binding"),
                    String::from("span"),
                ],
            ),
            (
                String::from("VariantFields"),
                vec![
                    String::from("name"),
                    String::from("fields"),
                    String::from("span"),
                ],
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "SurfaceLiteral"),
        vec![
            (String::from("Int"), vec![String::from("_0")]),
            (String::from("Bool"), vec![String::from("_0")]),
            (String::from("String"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "SurfaceExpr"),
        vec![
            (
                String::from("Literal"),
                vec![String::from("value"), String::from("span")],
            ),
            (
                String::from("Var"),
                vec![String::from("name"), String::from("span")],
            ),
            (
                String::from("Path"),
                vec![
                    String::from("segments"),
                    String::from("segment_spans"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Call"),
                vec![
                    String::from("target"),
                    String::from("args"),
                    String::from("span"),
                ],
            ),
            (
                String::from("PathCall"),
                vec![
                    String::from("segments"),
                    String::from("segment_spans"),
                    String::from("args"),
                    String::from("span"),
                ],
            ),
            (
                String::from("VariantRecord"),
                vec![
                    String::from("target"),
                    String::from("fields"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Operator"),
                vec![
                    String::from("op"),
                    String::from("args"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Lambda"),
                vec![
                    String::from("params"),
                    String::from("body"),
                    String::from("span"),
                ],
            ),
            (
                String::from("If"),
                vec![
                    String::from("cond"),
                    String::from("then_branch"),
                    String::from("else_branch"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Match"),
                vec![
                    String::from("scrutinee"),
                    String::from("arms"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Record"),
                vec![String::from("fields"), String::from("span")],
            ),
            (
                String::from("List"),
                vec![String::from("elements"), String::from("span")],
            ),
            (
                String::from("Map"),
                vec![String::from("entries"), String::from("span")],
            ),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "SurfaceItem"),
        vec![
            (
                String::from("Let"),
                vec![
                    String::from("name"),
                    String::from("type_ann"),
                    String::from("expr"),
                ],
            ),
            (
                String::from("Fn"),
                vec![
                    String::from("name"),
                    String::from("type_params"),
                    String::from("params"),
                    String::from("return_type"),
                    String::from("body"),
                    String::from("span"),
                ],
            ),
            (
                String::from("FnExternalBody"),
                vec![
                    String::from("name"),
                    String::from("type_params"),
                    String::from("params"),
                    String::from("return_type"),
                    String::from("body_span"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Data"),
                vec![
                    String::from("name"),
                    String::from("ty"),
                    String::from("body"),
                    String::from("body_span"),
                    String::from("span"),
                ],
            ),
            (
                String::from("Module"),
                vec![String::from("path"), String::from("span")],
            ),
            (
                String::from("Import"),
                vec![
                    String::from("path"),
                    String::from("names"),
                    String::from("span"),
                ],
            ),
            (
                String::from("TypeAtom"),
                vec![
                    String::from("name"),
                    String::from("type_params"),
                    String::from("span"),
                ],
            ),
            (
                String::from("TypeRecord"),
                vec![
                    String::from("name"),
                    String::from("type_params"),
                    String::from("fields"),
                    String::from("span"),
                ],
            ),
            (
                String::from("TypeSum"),
                vec![
                    String::from("name"),
                    String::from("type_params"),
                    String::from("variants"),
                    String::from("inhabits"),
                    String::from("span"),
                ],
            ),
            (
                String::from("TypeAlias"),
                vec![
                    String::from("name"),
                    String::from("type_params"),
                    String::from("nominal_opaque"),
                    String::from("target"),
                    String::from("refinement"),
                    String::from("span"),
                ],
            ),
        ]
    );
}

// ValueBody mirror audit: on-disk `src/v3/std/substrate.dag` `type ValueBody` vs `dag::ValueBody`
// (Evaluator retirement / R3 debt paydown #1531). Complements `sum_variants(…, "ValueBody")` above.
const SUBSTRATE_VALUEBODY_SOURCE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../std/substrate.dag");

fn substrate_value_body_constructors_from_source() -> Vec<String> {
    let substrate = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../std/substrate.dag"));
    let start = substrate
        .find("type ValueBody")
        .unwrap_or_else(|| panic!("{SUBSTRATE_VALUEBODY_SOURCE}: missing `type ValueBody`"));
    let tail = &substrate[start..];
    let end = tail.find("\n// Type substrate.").unwrap_or_else(|| {
        panic!("{SUBSTRATE_VALUEBODY_SOURCE}: missing `// Type substrate.` after ValueBody")
    });
    let block = &tail[..end];
    let mut out = Vec::new();
    for line in block.lines() {
        let t = line.trim_start();
        if !(t.starts_with('=') || t.starts_with('|')) {
            continue;
        }
        let rest = t.trim_start_matches(['=', '|']).trim_start();
        let name = rest
            .split(|c: char| c == '(' || c == '{' || c.is_whitespace())
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                panic!("{SUBSTRATE_VALUEBODY_SOURCE}: malformed ValueBody variant line: {line:?}")
            });
        out.push(name.to_string());
    }
    out
}

fn rust_value_body_variant_tag(body: &ValueBody) -> &'static str {
    match body {
        ValueBody::Unparsed(_) => "Unparsed",
        ValueBody::Structural { .. } => "Structural",
        ValueBody::Scalar(_) => "Scalar",
        ValueBody::List(_) => "List",
        ValueBody::Map(_) => "Map",
    }
}

fn sample_value_body_instances_covering_all_rust_variants() -> Vec<ValueBody> {
    let span = SourceSpan::new("m2_value_body_mirror_audit.v3", 0, 1);
    vec![
        ValueBody::Unparsed(span),
        ValueBody::Structural { fields: Vec::new() },
        ValueBody::Scalar(literal_bits_int(0)),
        ValueBody::List(Vec::new()),
        ValueBody::Map(FieldMap::from_entries(Vec::new()).expect("empty FieldMap")),
    ]
}

#[test]
fn substrate_value_body_sum_matches_parsed_constructors() {
    let parsed = substrate_value_body_constructors_from_source();
    assert_eq!(
        parsed,
        vec![
            "ValueBodyUnparsed".to_string(),
            "ValueBodyStructural".to_string(),
            "ValueBodyScalar".to_string(),
            "ValueBodyList".to_string(),
            "ValueBodyMap".to_string(),
        ],
        "`{SUBSTRATE_VALUEBODY_SOURCE}` `type ValueBody` must expose the canonical five-constructor carrier; update this ratchet when the sum changes"
    );
}

#[test]
fn rust_value_body_runtime_variants_are_exhaustively_tagged() {
    let tags: HashSet<&str> = sample_value_body_instances_covering_all_rust_variants()
        .iter()
        .map(|b| rust_value_body_variant_tag(b))
        .collect();
    assert_eq!(
        tags,
        HashSet::from(["Unparsed", "Structural", "Scalar", "List", "Map"]),
        "dag::ValueBody gained/lost a variant — update rust_value_body_variant_tag + this test"
    );
}

#[test]
fn value_body_substrate_rust_mirror_audit_documents_known_gap() {
    let substrate_constructors = substrate_value_body_constructors_from_source();
    let rust_tags: HashSet<&str> = sample_value_body_instances_covering_all_rust_variants()
        .iter()
        .map(|b| rust_value_body_variant_tag(b))
        .collect();

    assert_eq!(substrate_constructors.len(), 5);
    assert_eq!(rust_tags.len(), 5);
    assert!(
        substrate_constructors.contains(&"ValueBodyScalar".to_string())
            && substrate_constructors.contains(&"ValueBodyList".to_string()),
        "substrate carries Scalar/List top-level bodies; Rust mirror must stay generated from it"
    );
}

#[test]
fn cardinality_bound_projects_to_interval_parent() {
    assert_eq!(
        CardinalityBound::Exact(3).interval(),
        Interval::BoundedInterval {
            lower: 3,
            width: IntervalWidth::ZeroWidth,
        }
    );
    assert_eq!(
        CardinalityBound::AtMostOne.interval(),
        Interval::BoundedInterval {
            lower: 0,
            width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::OneUnit),
        }
    );
    assert_eq!(CardinalityBound::Unbounded.interval(), Interval::Unbounded);
}

#[test]
fn bound_declaration_static_bound_wraps_interval_int() {
    let dag = Dag::new();
    let bound_declaration = dag
        .declaration_by_name("BoundDeclaration")
        .expect("BoundDeclaration missing from substrate bootstrap");
    assert_eq!(
        bound_declaration.span.file, "src/v3/std/substrate.dag",
        "BoundDeclaration is the substrate carrier consumed by Coercion-Fold; \
         parser/lowerer and target-row population land in later slices"
    );

    let variants = match &bound_declaration.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("BoundDeclaration must be a Disj, got {other:?}"),
    };
    let static_bound = variants
        .iter()
        .find(|variant| variant.label == "StaticBound")
        .expect("BoundDeclaration missing StaticBound variant");
    let platform_dependent = variants
        .iter()
        .find(|variant| variant.label == "PlatformDependent")
        .expect("BoundDeclaration missing PlatformDependent variant");

    assert_runtime_value_instantiation(
        &dag,
        positional_payload(&dag, static_bound.ty),
        "Interval",
        "Int",
    );
    assert!(
        matches!(
            &dag.declaration(platform_dependent.ty).connective,
            TypeConnective::Conj { children } if children.is_empty()
        ),
        "PlatformDependent must remain a distinct no-payload kind, not an \
         Interval<Int> value"
    );
}

#[test]
fn target_integer_type_inhabitance_rows_are_structural_slice_b_receipt() {
    // Full-bootstrap `infer` can overflow the default test thread stack on some hosts;
    // mirror `positive_interval_width_unit_count_rejects_negative_units_literal`.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let dag = Dag::new();
            assert!(
                dag.diagnostics().is_empty(),
                "bootstrap diagnostics: {:?}",
                dag.diagnostics()
            );
            let meta = dag
                .declaration_by_name("TargetIntegerTypeInhabitance")
                .expect("TargetIntegerTypeInhabitance meta");
            let mut rows: Vec<&str> = Vec::new();
            for decl in dag.declarations() {
                if decl.meta_tag != Some(meta.id) {
                    continue;
                }
                let name = decl.name.as_deref().expect("named inhabitance data row");
                rows.push(name);
                assert!(
                    matches!(&decl.value_body, Some(ValueBody::Structural { .. })),
                    "`{name}` must lower as ValueBody::Structural (Coercion-Fold Slice B); got {:?}",
                    decl.value_body
                );
            }
            assert!(
                rows.len() >= 8,
                "expected Rust/Python/Go TargetIntegerTypeInhabitance population; got {:?}",
                rows
            );
            assert!(
                rows.contains(&"go_integer_inhabit_int_platform_dependent"),
                "expected Track B Go native `int` inhabitance row (PlatformDependentFact); got {:?}",
                rows
            );
        })
        .expect("spawn structural slice B receipt test thread")
        .join()
        .expect("structural slice B receipt test thread panicked");
}

#[test]
fn positive_interval_width_unit_count_rejects_negative_units_literal() {
    // Full-bootstrap `infer` can overflow the default test thread stack on some hosts;
    // mirror v2 integration tests by giving this compile a dedicated larger stack.
    let hit = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let source = "module test_negative_unit_count\n\n\
import v3.std.substrate { PositiveIntervalWidth }\n\n\
type Holder {\n\
  w: PositiveIntervalWidth\n\
}\n\n\
data bad: Holder = { w: UnitCount { units: -1 } }\n";
            let dag = semantic_dag_for(source, "positive_interval_unit_count_negative.v3");
            let expected =
                positive_interval_width_unit_count_requires_nonnegative_units_literal_message("-1");
            let hit = dag
                .diagnostics()
                .iter()
                .any(|(_, diagnostic)| match diagnostic {
                    Diagnostic::MagnitudeOutOfRange { literal, .. } => literal == "-1",
                    Diagnostic::ResolveError { name, .. } => name == &expected,
                    _ => false,
                });
            hit
        })
        .expect("spawn unit_count negative test thread")
        .join()
        .expect("unit_count negative test thread panicked");
    assert!(
        hit,
        "expected UnitCount.units=-1 to fail (Nat magnitude gate or enforce fallback)"
    );
}

#[test]
fn rust_dag_realizes_reflected_substrate_types() {
    let dag = Dag::new();
    let type_realization_meta = find_named(&dag, "TypeRealization");
    for name in [
        "FieldEntry",
        "TypeShape",
        "DagPort",
        "Dag",
        "Declaration",
        "TemplateArgument",
        "Interval",
        "PositiveIntervalWidth",
        "IntervalWidth",
        "PhantomParameter",
        "FieldValue",
        "ValueBody",
        "TransformTarget",
        "Behavior",
        "ArithmeticOp",
        "ComparisonOp",
        "LogicalOp",
        "OperatorKind",
        "PayloadBinding",
        "BranchPath",
        "NonEmptyList",
        "NonSingletonList",
        "ElementRef",
        "ParamRef",
        "TransformRef",
        "MemberDescent",
        "IntraClusterCall",
        "Cluster",
        "LoopBound",
        "ValueNode",
        "TransformNode",
        "BranchNode",
        "LoopNode",
        "BindNode",
        "SurfaceModule",
        "SurfaceItem",
        "SurfaceParam",
        "SurfaceField",
        "SurfaceVariant",
        "VariantPayload",
        "SurfaceType",
        "SurfaceRecordField",
        "SurfaceMapEntry",
        "SurfaceMatchArm",
        "SurfacePatternField",
        "SurfacePattern",
        "SurfaceLiteral",
        "SurfaceExpr",
    ] {
        let target = find_named(&dag, name);
        let realized = dag.declarations().iter().find(|decl| {
            decl.meta_tag == Some(type_realization_meta)
                && matches!(
                    &decl.value_body,
                    Some(ValueBody::Structural { fields })
                        if matches!(
                            fields.iter().find(|(label, _)| label == "target").map(|(_, value)| value),
                            Some(FieldValue::Reference(id)) if *id == target
                        )
                )
        });
        assert!(
            realized.is_some(),
            "expected a TypeRealization entry targeting `{name}`"
        );
    }
}

#[test]
fn all_target_languages_realize_reflected_surface_types() {
    let dag = Dag::new();
    let type_realization_meta = find_named(&dag, "TypeRealization");
    let languages = [
        find_named(&dag, "rust_language"),
        find_named(&dag, "go_language"),
        find_named(&dag, "python_language"),
    ];

    for name in [
        "SurfaceModule",
        "SurfaceItem",
        "SurfaceParam",
        "SurfaceField",
        "SurfaceVariant",
        "VariantPayload",
        "SurfaceType",
        "SurfaceRecordField",
        "SurfaceMapEntry",
        "SurfaceMatchArm",
        "SurfacePatternField",
        "SurfacePattern",
        "SurfaceLiteral",
        "SurfaceExpr",
    ] {
        let target = find_named(&dag, name);
        for language in languages {
            let realized = dag.declarations().iter().find(|decl| {
                decl.meta_tag == Some(type_realization_meta)
                    && structural_reference_field(decl, "target") == Some(target)
                    && structural_reference_field(decl, "language") == Some(language)
            });
            assert!(
                realized.is_some(),
                "expected `{name}` to have a TypeRealization for language declaration {language:?}"
            );
        }
    }
}

#[test]
fn parse_sum_type_first_variant_may_be_named_inhabits() {
    let source = "type T = inhabits | Other\n";
    let tokens = tokenize_for_test(source, "kw_variant_sum.v3").expect("tokenize");
    let parsed = parse_for_test(&tokens, "kw_variant_sum.v3").expect("parse");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    assert_eq!(mirrored.items.len(), 1);
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeSum { name, variants, .. } => {
            assert_eq!(name, "T");
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "inhabits");
            assert_eq!(variants[1].name, "Other");
        }
        other => panic!("expected TypeSum, got {other:?}"),
    }
}

#[test]
fn parse_sum_type_first_variant_may_be_named_type_keyword() {
    let source = "type T = type | Other\n";
    let tokens = tokenize_for_test(source, "kw_type_variant_sum.v3").expect("tokenize");
    let parsed = parse_for_test(&tokens, "kw_type_variant_sum.v3").expect("parse");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    assert_eq!(mirrored.items.len(), 1);
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeSum { name, variants, .. } => {
            assert_eq!(name, "T");
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "type");
            assert_eq!(variants[1].name, "Other");
        }
        other => panic!("expected TypeSum, got {other:?}"),
    }
}

/// PR-A.3: single-variant sum with record payload — no top-level `|` before `{`.
#[test]
fn parse_single_variant_sum_record_payload_without_pipe() {
    let source = concat!(
        "type EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }\n",
        "type InputEvaluationOrder = LeftFirst\n",
    );
    let tokens = tokenize_for_test(source, "pr_a3_eval_strategy.v3").expect("tokenize");
    let parsed = parse_for_test(&tokens, "pr_a3_eval_strategy.v3").expect("parse");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    assert_eq!(mirrored.items.len(), 2);
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeSum { name, variants, .. } => {
            assert_eq!(name, "EvalStrategy");
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "ApplicativeOrder");
            assert!(matches!(
                &variants[0].payload,
                parse_surface::VariantPayload::Record(fields)
                    if fields.len() == 1 && fields[0].name == "input_order"
            ));
        }
        other => panic!("expected TypeSum EvalStrategy, got {other:?}"),
    }
    match &mirrored.items[1] {
        parse_surface::SurfaceItem::TypeSum { name, variants, .. } => {
            assert_eq!(name, "InputEvaluationOrder");
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "LeftFirst");
            assert!(matches!(
                &variants[0].payload,
                parse_surface::VariantPayload::Positional(ps) if ps.is_empty()
            ));
        }
        other => panic!("expected TypeSum InputEvaluationOrder, got {other:?}"),
    }
}

/// PR-A.3 alias preservation: bare RHS names that resolve to imports or sibling
/// declarations remain aliases rather than being reclassified as nullary sums.
#[test]
fn parse_bare_rhs_alias_reference_stays_type_alias() {
    let source = concat!(
        "import std.types { String }\n",
        "type LocalString = String\n",
        "type Forward = Later\n",
        "type Later = Only\n",
        "type Id<T> = T\n",
        "type Bad = T\n",
    );
    let tokens = tokenize_for_test(source, "pr_a3_alias_preservation.v3").expect("tokenize");
    let parsed = parse_for_test(&tokens, "pr_a3_alias_preservation.v3").expect("parse");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    assert_eq!(mirrored.items.len(), 6);
    match &mirrored.items[1] {
        parse_surface::SurfaceItem::TypeAlias { name, .. } => {
            assert_eq!(name, "LocalString");
        }
        other => panic!("expected imported bare RHS to stay TypeAlias, got {other:?}"),
    }
    match &mirrored.items[2] {
        parse_surface::SurfaceItem::TypeAlias { name, .. } => {
            assert_eq!(name, "Forward");
        }
        other => panic!("expected sibling bare RHS to stay TypeAlias, got {other:?}"),
    }
    match &mirrored.items[3] {
        parse_surface::SurfaceItem::TypeSum { name, variants, .. } => {
            assert_eq!(name, "Later");
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "Only");
        }
        other => panic!("expected unresolved bare RHS to parse as TypeSum, got {other:?}"),
    }
    match &mirrored.items[4] {
        parse_surface::SurfaceItem::TypeAlias {
            name,
            type_params,
            target,
            ..
        } => {
            assert_eq!(name, "Id");
            assert_eq!(type_params, &[String::from("T")]);
            assert!(matches!(
                target,
                parse_surface::SurfaceType::Named { name, .. } if name == "T"
            ));
        }
        other => panic!("expected type-param bare RHS to stay TypeAlias, got {other:?}"),
    }
    match &mirrored.items[5] {
        parse_surface::SurfaceItem::TypeSum { name, variants, .. } => {
            assert_eq!(name, "Bad");
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "T");
        }
        other => panic!("expected out-of-scope type param RHS to parse as TypeSum, got {other:?}"),
    }
}

/// Optional leading `|` before the first variant (same sum path as `A | B`).
#[test]
fn parse_sum_type_optional_leading_pipe_before_first_variant() {
    let source = "type T = | Only\n";
    let tokens = tokenize_for_test(source, "leading_pipe_sum.v3").expect("tokenize");
    let parsed = parse_for_test(&tokens, "leading_pipe_sum.v3").expect("parse");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeSum { name, variants, .. } => {
            assert_eq!(name, "T");
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "Only");
        }
        other => panic!("expected TypeSum, got {other:?}"),
    }
}

/// Regression: `type … inhabits … =` is v3 surface-only (std `types.dag` stays
/// v2-shaped); this pins the parser + `rhs_is_sum` lookahead for the clause.
#[test]
fn parse_type_inhabits_clause_with_parameterized_algebra_and_sum_rhs() {
    let source = concat!(
        "type Widget<T> inhabits AlgebraExpr<T> = ",
        "Leaf | Node { left: Widget<T>; right: Widget<T> }\n",
    );
    let tokens =
        tokenize_for_test(source, "inhabits_sum_surface.v3").expect("tokenize inhabits sum");
    let parsed = parse_for_test(&tokens, "inhabits_sum_surface.v3").expect("parse inhabits sum");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    assert_eq!(mirrored.items.len(), 1);
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeSum {
            name,
            type_params,
            variants,
            inhabits,
            ..
        } => {
            assert_eq!(name, "Widget");
            assert_eq!(type_params, &[String::from("T")]);
            let inh = inhabits
                .as_ref()
                .expect("inhabits clause should surface on TypeSum");
            assert!(matches!(
                inh,
                parse_surface::SurfaceType::Parameterized { name, args, .. }
                    if name == "AlgebraExpr"
                        && args.len() == 1
                        && matches!(&args[0], parse_surface::SurfaceType::Named { name, .. } if name == "T")
            ));
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Leaf");
            assert_eq!(variants[1].name, "Node");
            assert!(matches!(
                &variants[1].payload,
                parse_surface::VariantPayload::Record(fields)
                    if fields.len() == 2
                        && fields[0].name == "left"
                        && fields[1].name == "right"
            ));
        }
        other => panic!("expected TypeSum with inhabits, got {other:?}"),
    }
}

#[test]
fn parse_type_inhabits_clause_rejects_non_sum_rhs() {
    let source = "type T inhabits Algebra = Int\n";
    let tokens =
        tokenize_for_test(source, "inhabits_alias_reject.v3").expect("tokenize inhabits alias");
    let err = parse_for_test(&tokens, "inhabits_alias_reject.v3")
        .expect_err("non-sum RHS with inhabits must be rejected");
    match err {
        Diagnostic::ParseError { message, .. } => {
            assert!(
                message.contains("inhabits") && message.contains("sum"),
                "unexpected parse diagnostic: {message}"
            );
        }
        other => panic!("expected ParseError, got {other:?}"),
    }
}

#[test]
fn parse_type_nominal_opaque_clause_marks_alias_surface() {
    let source = "type Token nominal_opaque = String\n";
    let tokens =
        tokenize_for_test(source, "nominal_opaque_alias.v3").expect("tokenize nominal_opaque");
    let parsed =
        parse_for_test(&tokens, "nominal_opaque_alias.v3").expect("parse nominal_opaque alias");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    assert_eq!(mirrored.items.len(), 1);
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeAlias {
            name,
            nominal_opaque,
            target,
            ..
        } => {
            assert_eq!(name, "Token");
            assert!(*nominal_opaque);
            assert!(matches!(
                target,
                parse_surface::SurfaceType::Named { name, .. } if name == "String"
            ));
        }
        other => panic!("expected TypeAlias with nominal_opaque, got {other:?}"),
    }
}

#[test]
fn parse_type_nominal_opaque_rejects_sum_rhs() {
    let source = "type Token nominal_opaque = Plain | Redacted\n";
    let tokens =
        tokenize_for_test(source, "nominal_opaque_sum_reject.v3").expect("tokenize nominal sum");
    let err = parse_for_test(&tokens, "nominal_opaque_sum_reject.v3")
        .expect_err("nominal_opaque sum RHS must be rejected");
    match err {
        Diagnostic::ParseError { .. } => {}
        other => panic!("expected ParseError, got {other:?}"),
    }
}

#[test]
fn lower_type_nominal_opaque_clause_sets_declaration_carrier() {
    let source = "type NominalOpaqueFixture nominal_opaque = String\n";
    let dag = compile_to_dag(source, "nominal_opaque_lower.v3").expect("compile nominal_opaque");
    let token = dag
        .declaration_by_name("NominalOpaqueFixture")
        .expect("NominalOpaqueFixture declaration should lower");
    assert!(
        token.nominal_opacity.is_some(),
        "nominal_opaque source marker must lower to Declaration.nominal_opacity"
    );
}

/// `inhabits` is not in `dag_keyword_set` (shared syntax): it must tokenize as
/// an ordinary identifier so it can spell a declared type name — distinct from
/// the `type <Name> inhabits <Ty> = …` clause introducer.
#[test]
fn parse_type_declared_name_may_be_inhabits_sum() {
    let source = "type inhabits = True | False\n";
    let tokens =
        tokenize_for_test(source, "type_named_inhabits.v3").expect("tokenize type_named_inhabits");
    let parsed =
        parse_for_test(&tokens, "type_named_inhabits.v3").expect("parse type_named_inhabits");
    let mirrored: &parse_surface::SurfaceModule = &parsed;
    assert_eq!(mirrored.items.len(), 1);
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeSum {
            name,
            variants,
            inhabits,
            ..
        } => {
            assert_eq!(name, "inhabits");
            assert!(inhabits.is_none());
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "True");
            assert_eq!(variants[1].name, "False");
        }
        other => panic!("expected TypeSum, got {other:?}"),
    }
}

#[test]
fn parse_surface_generated_module_consumes_parser_output_structurally() {
    let source = "fn id<T>(x: T where x == x) -> T = x\nlet y = if true then id(1) else 2";
    let tokens = tokenize_for_test(source, "surface_reflection.v3").expect("tokenize source");
    let parsed = parse_for_test(&tokens, "surface_reflection.v3").expect("parse source");
    let mirrored: &parse_surface::SurfaceModule = &parsed;

    assert_eq!(mirrored.items.len(), 2);
    match &mirrored.items[0] {
        parse_surface::SurfaceItem::Fn {
            name,
            type_params,
            params,
            return_type,
            body,
            ..
        } => {
            assert_eq!(name, "id");
            assert_eq!(type_params, &vec![String::from("T")]);
            assert_eq!(params.len(), 1);
            assert!(matches!(
                params[0].refinement,
                Some(parse_surface::SurfaceExpr::Operator { .. })
            ));
            assert!(matches!(
                return_type,
                parse_surface::SurfaceType::Named { name, .. } if name == "T"
            ));
            assert!(matches!(body, parse_surface::SurfaceExpr::Var { name, .. } if name == "x"));
        }
        other => panic!("expected reflected fn item, got {other:?}"),
    }
    match &mirrored.items[1] {
        parse_surface::SurfaceItem::Let { expr, .. } => {
            assert!(matches!(expr, parse_surface::SurfaceExpr::If { .. }));
        }
        other => panic!("expected reflected let item, got {other:?}"),
    }
}

#[test]
fn parse_surface_generated_module_covers_recursive_surface_shapes() {
    let source = "\
type Point { x: Int }\n\
type Wrapped = Wrap { inner: Point } | Empty\n\
fn unwrap_or_zero(w: Wrapped) -> Int = match w { Wrap { inner: point } => point.x, Empty => 0 }\n\
let yes = true\n\
let note = \"ok\"\n";
    let tokens = tokenize_for_test(source, "surface_reflection_recursive.v3").expect("tokenize");
    let parsed = parse_for_test(&tokens, "surface_reflection_recursive.v3").expect("parse");
    let mirrored: &parse_surface::SurfaceModule = &parsed;

    assert_eq!(mirrored.items.len(), 5);

    match &mirrored.items[0] {
        parse_surface::SurfaceItem::TypeRecord {
            name,
            fields,
            type_params,
            ..
        } => {
            assert_eq!(name, "Point");
            assert!(type_params.is_empty());
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name, "x");
            assert!(matches!(
                fields[0].ty,
                parse_surface::SurfaceType::Named { ref name, .. } if name == "Int"
            ));
        }
        other => panic!("expected reflected type record, got {other:?}"),
    }

    match &mirrored.items[1] {
        parse_surface::SurfaceItem::TypeSum {
            name,
            variants,
            type_params,
            ..
        } => {
            assert_eq!(name, "Wrapped");
            assert!(type_params.is_empty());
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Wrap");
            assert!(matches!(
                &variants[0].payload,
                parse_surface::VariantPayload::Record(fields)
                    if fields.len() == 1 && fields[0].name == "inner"
            ));
            assert_eq!(variants[1].name, "Empty");
            assert!(matches!(
                &variants[1].payload,
                parse_surface::VariantPayload::Positional(fields) if fields.is_empty()
            ));
        }
        other => panic!("expected reflected type sum, got {other:?}"),
    }

    match &mirrored.items[2] {
        parse_surface::SurfaceItem::Fn {
            body, return_type, ..
        } => {
            assert!(matches!(
                return_type,
                parse_surface::SurfaceType::Named { name, .. } if name == "Int"
            ));
            match body {
                parse_surface::SurfaceExpr::Match { arms, .. } => {
                    assert_eq!(arms.len(), 2);
                    assert!(matches!(
                        &arms[0].pattern,
                        parse_surface::SurfacePattern::VariantFields { name, fields, .. }
                            if name == "Wrap"
                                && fields.len() == 1
                                && fields[0].name == "inner"
                                && fields[0].binding == "point"
                    ));
                    assert!(matches!(
                        &arms[0].body,
                        parse_surface::SurfaceExpr::Path { segments, .. }
                            if segments == &vec![String::from("point"), String::from("x")]
                    ));
                    assert!(matches!(
                        &arms[1].pattern,
                        parse_surface::SurfacePattern::BareVariant { name, .. } if name == "Empty"
                    ));
                    assert!(matches!(
                        &arms[1].body,
                        parse_surface::SurfaceExpr::Literal { value, .. }
                            if matches!(value, parse_surface::SurfaceLiteral::Int(s) if s == "0")
                    ));
                }
                other => panic!("expected reflected match expr, got {other:?}"),
            }
        }
        other => panic!("expected reflected fn item, got {other:?}"),
    }

    match &mirrored.items[3] {
        parse_surface::SurfaceItem::Let { name, expr, .. } => {
            assert_eq!(name, "yes");
            assert!(matches!(
                expr,
                parse_surface::SurfaceExpr::Literal {
                    value: parse_surface::SurfaceLiteral::Bool(true),
                    ..
                }
            ));
        }
        other => panic!("expected bool let, got {other:?}"),
    }

    match &mirrored.items[4] {
        parse_surface::SurfaceItem::Let { name, expr, .. } => {
            assert_eq!(name, "note");
            assert!(matches!(
                expr,
                parse_surface::SurfaceExpr::Literal {
                    value: parse_surface::SurfaceLiteral::String(s),
                    ..
                } if s == "ok"
            ));
        }
        other => panic!("expected string let, got {other:?}"),
    }
}

/// Parser sub-lane smoke test: a `kernel_algebra_profile`-shaped data
/// declaration parses into `SurfaceItem::Data` whose body is a
/// `SurfaceExpr::Map` with the asserted entries. Routes through
/// `parse_data_item` → `looks_like_map_literal` (`{ String :` lookahead)
/// → `parse_map_literal`; lowering then carries the entries as
/// `ValueBody::Map`.
#[test]
fn map_body_data_item_parses_and_lowers_to_value_body_map() {
    let source = "data kernel_algebra_profile: Map<String, AlgebraProfile> = {\n  \"Int\": OrderedRingProfile,\n  \"Float\": ApproximateFieldProfile,\n  \"Bool\": BooleanAlgebraProfile,\n  \"String\": FreeMonoidScalarProfile,\n  \"List\": FreeMonoidCollectionProfile,\n  \"Set\": BooleanAlgebraCollectionProfile,\n  \"Map\": PartialFunctionProfile\n}\n";
    let expected: Vec<(&str, &str)> = vec![
        ("Int", "OrderedRingProfile"),
        ("Float", "ApproximateFieldProfile"),
        ("Bool", "BooleanAlgebraProfile"),
        ("String", "FreeMonoidScalarProfile"),
        ("List", "FreeMonoidCollectionProfile"),
        ("Set", "BooleanAlgebraCollectionProfile"),
        ("Map", "PartialFunctionProfile"),
    ];

    let tokens = tokenize_for_test(source, "map_literal_data.v3").expect("tokenize");
    let parsed = parse_for_test(&tokens, "map_literal_data.v3").expect("parse map-literal data");
    assert_eq!(parsed.items.len(), 1);
    match &parsed.items[0] {
        parse_surface::SurfaceItem::Data {
            name,
            body: Some(parse_surface::SurfaceExpr::Map { entries, .. }),
            ..
        } => {
            assert_eq!(name, "kernel_algebra_profile");
            assert_eq!(entries.len(), expected.len());
            for (entry, (expected_key, expected_value_var)) in entries.iter().zip(expected.iter()) {
                assert_eq!(&entry.key, expected_key);
                match &entry.value {
                    parse_surface::SurfaceExpr::Var { name, .. } => {
                        assert_eq!(name, expected_value_var);
                    }
                    other => panic!(
                        "expected map entry value to be SurfaceExpr::Var({expected_value_var}), got {other:?}"
                    ),
                }
            }
        }
        other => panic!("expected SurfaceItem::Data with SurfaceExpr::Map body, got {other:?}"),
    }

    let dag = compile_to_dag(source, "map_literal_data.v3").expect("lower map-literal data");
    let algebra_profile = dag
        .declaration_by_name("AlgebraProfile")
        .expect("AlgebraProfile exists");
    let v3_compiler::dag::TypeConnective::Disj { variants } = &algebra_profile.connective else {
        panic!("expected AlgebraProfile to be a sum type");
    };
    let decl = dag
        .declaration_by_name("kernel_algebra_profile")
        .expect("kernel_algebra_profile declaration exists");
    let Some(ValueBody::Map(entries)) = &decl.value_body else {
        panic!(
            "expected kernel_algebra_profile to lower to ValueBody::Map, got {:?}",
            decl.value_body
        );
    };
    assert_eq!(entries.entries().len(), expected.len());
    for ((key, value), (expected_key, expected_value_name)) in
        entries.entries().iter().zip(expected.iter())
    {
        assert_eq!(key, expected_key);
        let expected_variant = variants
            .iter()
            .find(|variant| variant.label == *expected_value_name)
            .unwrap_or_else(|| panic!("missing AlgebraProfile variant {expected_value_name}"));
        let FieldValue::Variant {
            constructor,
            payload,
        } = value
        else {
            panic!("expected map value for {key} to lower as FieldValue::Variant, got {value:?}");
        };
        assert_eq!(
            *constructor, expected_variant.ty,
            "map value for {key} should reference AlgebraProfile::{expected_value_name}"
        );
        assert!(
            payload.is_empty(),
            "map value for {key} should use the zero-payload AlgebraProfile::{expected_value_name} constructor"
        );
    }
}

#[test]
fn map_body_duplicate_keys_fail_closed() {
    let dag = semantic_dag_for(
        "data duplicate_keys: Map<String, Bool> = {\n  \"same\": true,\n  \"same\": false\n}\n",
        "map_duplicate_keys.v3",
    );
    let decl = dag
        .declaration_by_name("duplicate_keys")
        .expect("duplicate_keys declaration should be allocated before lowering fails");

    assert!(
        has_resolve_error(&dag),
        "expected duplicate-key lowering to report a ResolveError, got {:?}",
        dag.diagnostics()
    );
    assert!(
        !matches!(decl.value_body, Some(ValueBody::Map(_))),
        "duplicate-key map must not construct ValueBody::Map, got {:?}",
        decl.value_body
    );
}

#[test]
fn record_body_duplicate_fields_fail_closed() {
    let dag = semantic_dag_for(
        "type Pair { a: Int, b: Int }\n\
         data duplicate_fields: Pair = { a: 1, a: 2, b: 3 }\n",
        "record_duplicate_fields.v3",
    );
    let decl = dag
        .declaration_by_name("duplicate_fields")
        .expect("duplicate_fields declaration should be allocated before lowering fails");

    assert!(
        has_resolve_error(&dag),
        "expected duplicate-field lowering to report a ResolveError, got {:?}",
        dag.diagnostics()
    );
    assert!(
        !matches!(decl.value_body, Some(ValueBody::Structural { .. })),
        "duplicate-field record must not construct ValueBody::Structural, got {:?}",
        decl.value_body
    );
}

#[test]
fn nested_record_body_duplicate_fields_fail_closed() {
    let dag = semantic_dag_for(
        "type Inner { a: Int, b: Int }\n\
         type Outer { inner: Inner, tag: Int }\n\
         data duplicate_nested_fields: Outer = { inner: { a: 1, a: 2, b: 3 }, tag: 0 }\n",
        "nested_record_duplicate_fields.v3",
    );
    let decl = dag
        .declaration_by_name("duplicate_nested_fields")
        .expect("duplicate_nested_fields declaration should be allocated before lowering fails");

    assert!(
        has_resolve_error(&dag),
        "expected nested duplicate-field lowering to report a ResolveError, got {:?}",
        dag.diagnostics()
    );
    assert!(
        !matches!(decl.value_body, Some(ValueBody::Structural { .. })),
        "duplicate nested record must not construct ValueBody::Structural, got {:?}",
        decl.value_body
    );
}

#[test]
fn data_body_named_variant_duplicate_payload_fields_fail_closed() {
    let dag = semantic_dag_for(
        "type Status = Ready { code: Int, retry: Bool } | Blocked\n\
         type Job { status: Status }\n\
         data duplicate_variant_payload_fields: Job = { status: Ready { code: 1, code: 2, retry: false } }\n",
        "data_body_named_variant_duplicate_payload_fields.v3",
    );
    let decl = dag
        .declaration_by_name("duplicate_variant_payload_fields")
        .expect(
        "duplicate_variant_payload_fields declaration should be allocated before lowering fails",
    );

    assert!(
        has_resolve_error(&dag),
        "expected named-variant duplicate payload field lowering to report a ResolveError, got {:?}",
        dag.diagnostics()
    );
    assert!(
        !matches!(decl.value_body, Some(ValueBody::Structural { .. })),
        "duplicate named-variant payload must not construct ValueBody::Structural, got {:?}",
        decl.value_body
    );
}

#[test]
fn expr_named_variant_duplicate_payload_fields_fail_closed() {
    let dag = semantic_dag_for(
        "type Status = Ready { code: Int, retry: Bool } | Blocked\n\
         fn make_status() -> Status =\n\
           Ready { code: 1, code: 2, retry: false }\n",
        "expr_named_variant_duplicate_payload_fields.v3",
    );
    assert!(
        has_resolve_error(&dag),
        "expected expression-position named-variant duplicate payload to report a ResolveError, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn map_body_on_non_map_type_fails_closed() {
    let dag = semantic_dag_for(
        "data not_a_map: Bool = {\n  \"x\": true\n}\n",
        "map_body_non_map_type.v3",
    );
    let decl = dag
        .declaration_by_name("not_a_map")
        .expect("not_a_map declaration should be allocated before lowering fails");

    assert!(
        has_resolve_error(&dag),
        "expected non-map body lowering to report a ResolveError, got {:?}",
        dag.diagnostics()
    );
    assert!(
        !matches!(decl.value_body, Some(ValueBody::Map(_))),
        "non-map type must not construct ValueBody::Map, got {:?}",
        decl.value_body
    );
}

#[test]
fn runtime_mirror_snapshots_are_fresh() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repo root");
    let status = std::process::Command::new("python3")
        .arg("scripts/regen_runtime_mirrors.py")
        .arg("--check")
        .current_dir(repo_root)
        .status()
        .expect("run runtime mirror freshness check");
    assert!(
        status.success(),
        "parse-surface / serialize snapshots are stale; run scripts/regen_runtime_mirrors.py"
    );
}

fn bin_shim_fields(dag: &Dag) -> Vec<(&str, v3_compiler::dag::DeclarationId)> {
    let decl = dag
        .declaration_by_name("BinShim")
        .expect("`BinShim` missing from full bootstrap");
    match &decl.connective {
        TypeConnective::Conj { children } => children
            .iter()
            .map(|field| (field.label.as_str(), field.ty))
            .collect(),
        other => panic!("`BinShim` must be a record carrier, got {other:?}"),
    }
}

#[test]
fn bin_shim_carrier_lives_in_v3_std_authority() {
    let dag = v3_compiler::generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name("BinShim")
        .expect("`BinShim` missing from full bootstrap");

    assert_eq!(
        decl.span.file, "src/v3/std/bin_shim.dag",
        "`BinShim` carrier authority must stay in the staged v3 std surface; \
         concrete shim rows belong under `dsl/std/runtime/bin_shims/`"
    );
}

#[test]
fn bin_shim_carrier_has_locked_three_field_shape() {
    let dag = v3_compiler::generated_full_bootstrap_dag();
    let labels: Vec<&str> = bin_shim_fields(&dag)
        .into_iter()
        .map(|(label, _)| label)
        .collect();

    assert_eq!(
        labels,
        ["entrypoint_name", "description", "entry"],
        "`BinShim` must remain metadata plus entry declaration; adding a \
         pipeline-step DSL or extra emitter state requires a substrate \
         amendment"
    );
}

#[test]
fn bin_shim_field_types_match_design_lock() {
    let dag = v3_compiler::generated_full_bootstrap_dag();
    let fields = bin_shim_fields(&dag);

    let expected = [
        ("entrypoint_name", find_named(&dag, "NonEmptyStr")),
        ("description", find_named(&dag, "String")),
        ("entry", find_named(&dag, "DeclarationRef")),
    ];

    assert_eq!(
        fields, expected,
        "`BinShim.entry` stays a DeclarationRef to a .dag `() -> \
         std.process.ProcessExit` function until DeclarationRef signature \
         refinement lands"
    );
}

fn disj_variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .map(|variant| variant.label.clone())
            .collect(),
        other => panic!("`{name}` must be a Disj, got {other:?}"),
    }
}

fn variant_payload_field_types(
    dag: &Dag,
    type_name: &str,
    variant_name: &str,
) -> Vec<(String, v3_compiler::dag::DeclarationId)> {
    let decl = dag
        .declaration_by_name(type_name)
        .unwrap_or_else(|| panic!("`{type_name}` missing from full bootstrap"));
    let variant_ty = match &decl.connective {
        TypeConnective::Disj { variants } => {
            variants
                .iter()
                .find(|variant| variant.label == variant_name)
                .unwrap_or_else(|| panic!("`{type_name}` missing variant `{variant_name}`"))
                .ty
        }
        other => panic!("`{type_name}` must be a Disj, got {other:?}"),
    };
    match &dag.declaration(variant_ty).connective {
        TypeConnective::Conj { children } => children
            .iter()
            .map(|field| (field.label.clone(), field.ty))
            .collect(),
        other => panic!("`{type_name}.{variant_name}` payload must be a Conj, got {other:?}"),
    }
}

#[test]
fn approximate_field_axes_live_in_v3_std_authority() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    for name in [
        "RoundingMode",
        "Precision",
        "NanPolicy",
        "InfinityPolicy",
        "SignedZeroPolicy",
        "SubnormalPolicy",
        "SpecialValues",
        "ApproximateField",
    ] {
        let decl = dag
            .declaration_by_name(name)
            .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
        assert_eq!(
            decl.span.file, "src/v3/std/approximate_field.dag",
            "`{name}` must stay in the single approximate-field authority module"
        );
    }
}

/// T-Numeric-Construction — `ApproximateField<F>` carrier introduction per 6Q audit
/// (`SpecialValues` + `ApproximateField<F>` slice). Pins `base: Field<F>` with `F`
/// as the carrier parameter matching `dsl/std/algebra.dag`'s `Field<T>`.
///
/// For `Real`, see `real_default_alias_resolves_to_approximate_field_over_field_of_fractions_of_int`
/// (Option A spelling per `docs/audit/t-numeric-construction-approximate-field-real-parameter-stop.md`).
#[test]
fn approximate_field_carrier_record_shape_ratchets() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let approx_id = find_named(&dag, "ApproximateField");
    let field_template = dag
        .declaration_by_name("Field")
        .expect("`Field` must load from dsl/std/algebra.dag");

    let mut labels = record_fields(&dag, "ApproximateField");
    labels.sort();
    assert_eq!(
        labels,
        vec![
            "base".to_string(),
            "precision".to_string(),
            "rounding".to_string(),
            "special_values".to_string(),
            "subnormal_policy".to_string(),
        ],
        "`ApproximateField<F>` must carry exactly the five 6Q axes"
    );

    let approx_decl = dag.declaration(approx_id);
    assert_eq!(
        approx_decl.type_params.len(),
        1,
        "ApproximateField<F> takes exactly one carrier type parameter"
    );

    let base_ty = conj_field_by_id(&dag, approx_id, "base");
    match &dag.declaration(base_ty).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, field_template.id,
                "`base` must instantiate `Field<F>`, not an ad hoc record"
            );
            assert_eq!(
                arguments.len(),
                1,
                "`Field<F>` carries exactly one type argument"
            );
            assert_eq!(
                arguments[0].parameter, field_template.type_params[0],
                "`Field` instantiation must fill `Field`'s formal `<T>` slot"
            );
            assert_eq!(
                arguments[0].value,
                approx_decl.type_params[0],
                "`base: Field<F>` must pass ApproximateField's carrier `<F>` as the `Field` argument \
                 (parameter = template slot; value = argument binding)"
            );
        }
        other => panic!("`base` must lower to a Field instantiation; got {other:?}"),
    }

    let mut sv_labels = record_fields(&dag, "SpecialValues");
    sv_labels.sort();
    assert_eq!(
        sv_labels,
        vec![
            "infinity".to_string(),
            "nan".to_string(),
            "signed_zero".to_string(),
        ],
        "`SpecialValues` aggregates nan / infinity / signed-zero policy only"
    );
}

#[test]
fn rounding_mode_axis_is_closed_sum() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    assert_eq!(
        disj_variant_labels(&dag, "RoundingMode"),
        [
            "ToNearestEven",
            "ToZero",
            "ToPositiveInfinity",
            "ToNegativeInfinity",
            "ToAwayFromZero"
        ]
        .map(String::from),
        "`RoundingMode` must remain a typed closed sum, not a string tag"
    );
}

#[test]
fn precision_axis_splits_binary_and_decimal_payloads() {
    let dag = v3_compiler::generated_full_bootstrap_dag();
    let positive_int = find_named(&dag, "PositiveInt");

    assert_eq!(
        disj_variant_labels(&dag, "Precision"),
        ["Unbounded", "BinaryPrecision", "DecimalPrecision"].map(String::from),
        "`Precision` must not regress to a compressed total-width token"
    );
    assert_eq!(
        variant_payload_field_types(&dag, "Precision", "BinaryPrecision"),
        [
            (String::from("significand_bits"), positive_int),
            (String::from("exponent_bits"), positive_int)
        ],
        "`BinaryPrecision` must name significand/exponent counts with a \
         positive-count type"
    );
    assert_eq!(
        variant_payload_field_types(&dag, "Precision", "DecimalPrecision"),
        [
            (String::from("digits"), positive_int),
            (String::from("exponent_digits"), positive_int)
        ],
        "`DecimalPrecision` must name decimal precision counts with a \
         positive-count type"
    );
}

#[test]
fn special_value_policy_axes_are_closed_sums() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    assert_eq!(
        disj_variant_labels(&dag, "NanPolicy"),
        ["NoNaN", "QuietNaN", "QuietAndSignalingNaN"].map(String::from),
        "`NanPolicy` must distinguish no-NaN from quiet/signaling support"
    );
    assert_eq!(
        disj_variant_labels(&dag, "InfinityPolicy"),
        ["NoInfinity", "SignedInfinity"].map(String::from),
        "`InfinityPolicy` must stay typed, not a target-specific string"
    );
    assert_eq!(
        disj_variant_labels(&dag, "SignedZeroPolicy"),
        ["NoSignedZero", "SignedZero"].map(String::from),
        "`SignedZeroPolicy` must stay typed"
    );
    assert_eq!(
        disj_variant_labels(&dag, "SubnormalPolicy"),
        ["NoSubnormals", "GradualUnderflow", "FlushToZero"].map(String::from),
        "`SubnormalPolicy` must distinguish absent, gradual, and flushed \
         underflow behavior"
    );
}

#[test]
fn string_diagnostic_ordering_axes_live_in_emit_model_authority() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    for name in [
        "StringOwnershipAxis",
        "StringLifetimeAxis",
        "StringGrowabilityAxis",
        "StringEncodingAxis",
    ] {
        let decl = dag
            .declaration_by_name(name)
            .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
        assert_eq!(
            decl.span.file, "src/v3/std/emit_model.dag",
            "`{name}` must stay with target emission-model substrate; \
             per-target diagnostic-ordering rows land in target specs later"
        );
    }
}

#[test]
fn string_diagnostic_ordering_axes_are_closed_structural_values() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    assert_eq!(
        disj_variant_labels(&dag, "StringOwnershipAxis"),
        ["Owned", "Borrowed"].map(String::from),
        "string ownership must be a typed axis, not a boolean or target string"
    );
    assert_eq!(
        disj_variant_labels(&dag, "StringLifetimeAxis"),
        ["SelfContained", "Caller"].map(String::from),
        "string lifetime must use canonical substrate names, not Rust enum \
         escaping or target-local strings"
    );
    assert_eq!(
        disj_variant_labels(&dag, "StringGrowabilityAxis"),
        ["Growable", "Fixed", "NotApplicable"].map(String::from),
        "`NotApplicable` must be an explicit growability value, not absence"
    );
    assert_eq!(
        disj_variant_labels(&dag, "StringEncodingAxis"),
        ["Utf8FreeMonoidChar"].map(String::from),
        "R2 string encoding vocabulary must name the FreeMonoid<Char> / UTF-8 \
         row shape structurally"
    );
}

#[test]
fn string_family_inhabitance_row_is_language_scoped_and_axis_typed() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let row = dag
        .declaration_by_name("StringFamilyInhabitanceRow")
        .expect("StringFamilyInhabitanceRow missing from full bootstrap");
    assert_eq!(
        row.span.file, "src/v3/std/emit_model.dag",
        "string-family rows must live beside the landed axis authority in emit_model"
    );
    assert_eq!(
        record_fields(&dag, "StringFamilyInhabitanceRow"),
        [
            "language",
            "target_type",
            "type_realization",
            "ownership",
            "lifetime",
            "growability",
            "encoding",
        ]
        .map(String::from),
        "string-family row host must stay structurally explicit and language-scoped"
    );

    let declaration_ref = find_named(&dag, "DeclarationRef");
    for field in [
        "language",
        "target_type",
        "type_realization",
        "ownership",
        "lifetime",
        "growability",
        "encoding",
    ] {
        assert_eq!(
            conj_field_by_id(&dag, row.id, field),
            declaration_ref,
            "`{field}` must remain a structural `DeclarationRef` edge"
        );
    }
}

#[test]
fn runtime_value_carrier_matches_pb_runtime_shape_and_marker_boundary() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let value = dag
        .declaration_by_name("Value")
        .expect("runtime Value missing from full bootstrap");
    assert_eq!(
        value.span.file, "src/v3/std/runtime.dag",
        "bare Value must be the runtime carrier, not an L1 behavior marker"
    );

    let variants = match &value.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("runtime Value is not a Disj: {other:?}"),
    };
    let labels: Vec<&str> = variants.iter().map(|field| field.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "LiteralValue",
            "RecordValue",
            "VariantValue",
            "NodeRef",
            "CardinalityValue"
        ],
        "runtime Value coproduct drifted from PB-Runtime section 3.2"
    );

    assert_eq!(
        positional_payload(&dag, runtime_value_variant_payload(&dag, "LiteralValue")),
        find_named(&dag, "LiteralBits")
    );
    assert_runtime_value_instantiation(
        &dag,
        positional_payload(&dag, runtime_value_variant_payload(&dag, "RecordValue")),
        "List",
        "NamedField",
    );
    assert_eq!(
        positional_payload(&dag, runtime_value_variant_payload(&dag, "NodeRef")),
        find_named(&dag, "NodeId")
    );
    assert_eq!(
        positional_payload(
            &dag,
            runtime_value_variant_payload(&dag, "CardinalityValue")
        ),
        find_named(&dag, "LoopBound")
    );

    let variant_value = runtime_value_variant_payload(&dag, "VariantValue");
    assert_eq!(
        conj_field_by_id(&dag, variant_value, "tag"),
        find_named(&dag, "DeclarationId")
    );
    assert_eq!(conj_field_by_id(&dag, variant_value, "payload"), value.id);

    assert_eq!(
        conj_field_by_id(&dag, find_named(&dag, "NamedField"), "label"),
        find_named(&dag, "String")
    );
    assert_eq!(
        conj_field_by_id(&dag, find_named(&dag, "NamedField"), "value"),
        find_named(&dag, "Value")
    );

    let runtime_value = find_named(&dag, "Value");
    let value_behavior = find_named(&dag, "ValueBehavior");
    assert_ne!(
        runtime_value, value_behavior,
        "runtime Value must not alias the L1 ValueBehavior marker"
    );
    assert_eq!(
        dag.value_marker(),
        Some(value_behavior),
        "Dag::value_marker() must keep returning the L1 behavior marker"
    );
}

#[test]
fn program_observation_carrier_is_producer_neutral_typed_envelope() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let observation = dag
        .declaration_by_name("ProgramObservation")
        .expect("PR-B.2 ProgramObservation missing from full bootstrap");
    assert_eq!(
        observation.span.file, "src/v3/std/runtime.dag",
        "ProgramObservation must live with the runtime observation authority"
    );
    let type_params = observation.type_params.clone();
    assert_eq!(
        type_params.len(),
        1,
        "ProgramObservation must have exactly one typed observation carrier"
    );
    match &dag.declaration(type_params[0]).connective {
        TypeConnective::Atom(AtomPayload::TypeParam(name)) => assert_eq!(
            name, "Carrier",
            "ProgramObservation's type parameter should name the typed observation domain"
        ),
        other => {
            panic!("ProgramObservation type parameter must be a TypeParam atom, got {other:?}")
        }
    }
    assert_eq!(
        conj_field_by_id(&dag, observation.id, "observed"),
        type_params[0],
        "ProgramObservation must wrap the typed observed value directly"
    );

    let TypeConnective::Conj { children } = &observation.connective else {
        panic!(
            "ProgramObservation must lower as a single-field Conj, got {:?}",
            observation.connective
        );
    };
    let labels: Vec<&str> = children.iter().map(|field| field.label.as_str()).collect();
    assert_eq!(
        labels,
        ["observed"],
        "ProgramObservation must not bake producer evidence such as stdout, target, \
         exit status, or evaluator strategy into the comparable carrier"
    );
}

#[test]
fn pr_a_2_eval_frame_and_state_stack_carriers_match_pb_runtime_section_3_3() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let frame = dag
        .declaration_by_name("EvalFrame")
        .expect("PR-A.2 EvalFrame missing from full bootstrap");
    assert_eq!(
        frame.span.file, "src/v3/std/runtime.dag",
        "EvalFrame must live in the single runtime authority module"
    );
    let bindings_ty = conj_field_by_id(&dag, frame.id, "bindings");
    match &dag.declaration(bindings_ty).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let template_name = dag
                .declaration(*template)
                .name
                .as_deref()
                .expect("Map template must be a named declaration");
            assert!(
                template_name == "Map" || template_name == "PartialFunction",
                "EvalFrame.bindings template must be Map / PartialFunction, got `{template_name}`"
            );
            assert_eq!(
                arguments.len(),
                2,
                "EvalFrame.bindings must have exactly two type arguments (key, value)"
            );
            assert_eq!(
                arguments[0].value,
                find_named(&dag, "PortId"),
                "EvalFrame.bindings key must be PortId"
            );
            assert_eq!(
                arguments[1].value,
                find_named(&dag, "Value"),
                "EvalFrame.bindings value must be runtime Value"
            );
        }
        other => panic!("EvalFrame.bindings is not an Instantiation: {other:?}"),
    }

    let stack = dag
        .declaration_by_name("EvalStateStack")
        .expect("PR-A.2 EvalStateStack missing from full bootstrap");
    assert_eq!(
        stack.span.file, "src/v3/std/runtime.dag",
        "EvalStateStack must live in the single runtime authority module"
    );
    assert_runtime_value_instantiation(
        &dag,
        conj_field_by_id(&dag, stack.id, "frames"),
        "List",
        "EvalFrame",
    );

    let value = dag
        .declaration_by_name("Value")
        .expect("runtime Value missing from full bootstrap");
    let labels: Vec<&str> = match &value.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|f| f.label.as_str()).collect(),
        other => panic!("runtime Value is not a Disj: {other:?}"),
    };
    assert!(
        !labels.contains(&"ClosureValue"),
        "PR-A.2 must not introduce ClosureValue; closed-over environments \
         are evaluator-internal state, not observable Value variants"
    );
    assert!(
        !labels.contains(&"EvalFrame") && !labels.contains(&"EvalStateStack"),
        "EvalFrame / EvalStateStack are evaluator state, never Value variants"
    );
}

// Test-harness containment: this carrier-shape ratchet loads the full generated
// bootstrap DAG, which can overflow the default harness stack in CI. Dissolution
// trigger: remove or centralize this wrapper once full-bootstrap shape tests run
// on the default stack.
const FULL_BOOTSTRAP_SHAPE_TEST_STACK_BYTES: usize = 32 * 1024 * 1024;

#[test]
fn pr_a_3_strategy_and_memo_key_carriers_match_eager_baseline_shape() {
    std::thread::Builder::new()
        .stack_size(FULL_BOOTSTRAP_SHAPE_TEST_STACK_BYTES)
        .spawn(pr_a_3_strategy_and_memo_key_carriers_match_eager_baseline_shape_impl)
        .expect("spawn larger-stack PR-A.3 carrier test thread")
        .join()
        .expect("larger-stack PR-A.3 carrier test thread panicked");
}

fn pr_a_3_strategy_and_memo_key_carriers_match_eager_baseline_shape_impl() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let eval_state_key_id = find_named(&dag, "EvalStateKey");
    let eval_memo_key_id = find_named(&dag, "EvalMemoKey");
    let eval_state_stack_id = find_named(&dag, "EvalStateStack");

    let state_key = dag
        .declaration_by_name("EvalStateKey")
        .expect("PR-A.3 EvalStateKey missing from full bootstrap");
    assert_eq!(
        state_key.span.file, "src/v3/std/runtime.dag",
        "EvalStateKey must live in the single runtime authority module"
    );
    assert_eq!(
        conj_field_by_id(&dag, eval_state_key_id, "state"),
        eval_state_stack_id,
        "EvalStateKey must key memoization by structural EvalStateStack state"
    );

    let strategy = dag
        .declaration_by_name("EvalStrategy")
        .expect("PR-A.3 EvalStrategy missing from full bootstrap");
    assert_eq!(
        strategy.span.file, "src/v3/std/runtime.dag",
        "EvalStrategy must live in the single runtime authority module"
    );
    let strategy_variants = match &strategy.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("EvalStrategy must lower as a one-variant Disj, got {other:?}"),
    };
    assert_eq!(
        strategy_variants.len(),
        1,
        "TC2 input-order expansion must stay under the single applicative strategy variant"
    );
    assert_eq!(strategy_variants[0].label, "ApplicativeOrder");
    assert_eq!(
        conj_field_by_id(&dag, strategy_variants[0].ty, "input_order"),
        find_named(&dag, "InputEvaluationOrder")
    );

    let input_order = dag
        .declaration_by_name("InputEvaluationOrder")
        .expect("PR-A.3 InputEvaluationOrder missing from full bootstrap");
    assert_eq!(
        input_order.span.file, "src/v3/std/runtime.dag",
        "InputEvaluationOrder must live in the single runtime authority module"
    );
    let input_order_variants = match &input_order.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("InputEvaluationOrder must lower as a two-variant Disj, got {other:?}"),
    };
    assert_eq!(
        input_order_variants.len(),
        2,
        "TC2 requires exactly the two executable eager input orders"
    );
    assert_eq!(input_order_variants[0].label, "LeftFirst");
    assert_eq!(input_order_variants[1].label, "RightFirst");
    for variant in input_order_variants {
        match &dag.declaration(variant.ty).connective {
            TypeConnective::Conj { children } => assert!(
                children.is_empty(),
                "{} is a bare nullary variant and must carry no payload fields",
                variant.label
            ),
            other => panic!(
                "{} payload must lower as empty Conj, got {other:?}",
                variant.label
            ),
        }
    }

    let memo_key = dag
        .declaration_by_name("EvalMemoKey")
        .expect("PR-A.3 EvalMemoKey missing from full bootstrap");
    assert_eq!(
        memo_key.span.file, "src/v3/std/runtime.dag",
        "EvalMemoKey must live in the single runtime authority module"
    );
    assert_eq!(
        conj_field_by_id(&dag, eval_memo_key_id, "program"),
        find_named(&dag, "DeclarationId"),
        "EvalMemoKey.program must name the evaluated program declaration"
    );
    assert_eq!(
        conj_field_by_id(&dag, eval_memo_key_id, "node"),
        find_named(&dag, "NodeId"),
        "EvalMemoKey.node must key by structural node identity"
    );
    assert_eq!(
        conj_field_by_id(&dag, eval_memo_key_id, "state_key"),
        eval_state_key_id,
        "EvalMemoKey.state_key must use EvalStateKey, not a string or name-only fingerprint"
    );
    assert_eq!(
        conj_field_by_id(&dag, eval_memo_key_id, "strategy"),
        find_named(&dag, "EvalStrategy"),
        "EvalMemoKey.strategy must use the closed EvalStrategy carrier"
    );
}

/// T-Numeric-Construction Slice 2 — `Nat = CommutativeSemiring<Magnitude>` resolves
/// structurally to a `CommutativeSemiring` instantiation whose carrier argument is the
/// `Magnitude` opaque atom landed by Slice 1.
///
/// The ratchet enforces both authorities at once:
/// - `Nat` must lower to a `TypeConnective::Instantiation` whose template is
///   `Semiring` (`dsl/std/algebra.dag`), not a fresh record or a name-keyed bridge.
/// - The single carrier argument must resolve to `Magnitude` (`dsl/std/magnitude.dag`),
///   not to a `Word*` storage carrier — those remain storage refinements per
///   `docs/audit/t-numeric-construction-magnitude-6q.md`.
#[test]
fn nat_resolves_to_semiring_over_magnitude() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let nat = dag
        .declaration_by_name("Nat")
        .expect("`Nat` missing from full bootstrap (T-Numeric-Construction Slice 2)");
    assert_eq!(
        nat.span.file, "dsl/std/nat.dag",
        "Nat must live in the canonical std/nat.dag module, not in a parallel authority"
    );

    let commutative_semiring = dag
        .declaration_by_name("CommutativeSemiring")
        .expect("`CommutativeSemiring` algebra must be loaded from dsl/std/algebra.dag");
    let magnitude = dag
        .declaration_by_name("Magnitude")
        .expect("`Magnitude` opaque carrier must be loaded from dsl/std/magnitude.dag (Slice 1)");

    match &nat.connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, commutative_semiring.id,
                "Nat must instantiate `CommutativeSemiring`, not an alternate algebra record"
            );
            assert_eq!(
                arguments.len(),
                1,
                "CommutativeSemiring takes exactly one carrier type parameter"
            );
            assert_eq!(
                arguments[0].value, magnitude.id,
                "Nat's CommutativeSemiring carrier argument must be `Magnitude`, not a Word* storage carrier"
            );
        }
        other => panic!(
            "Nat must lower to a CommutativeSemiring instantiation; got {other:?} — \
             a non-Instantiation connective indicates a parallel algebra authority \
             rather than a clean CommutativeSemiring<Magnitude> alias"
        ),
    }
}

/// T-Numeric-Construction algebra-strength sharpening — `CommutativeSemiring<T>`
/// exists as the stronger algebra surface for semirings whose multiplication is
/// commutative. Nat consumes that stronger algebra surface in
/// `nat_resolves_to_semiring_over_magnitude`.
#[test]
fn commutative_semiring_declares_semiring_shape_for_nat_sharpening() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let commutative = dag
        .declaration_by_name("CommutativeSemiring")
        .expect("CommutativeSemiring must be declared in std.algebra");
    assert_eq!(
        commutative.span.file, "dsl/std/algebra.dag",
        "CommutativeSemiring must live beside the algebra hierarchy authority"
    );

    assert_eq!(
        record_fields(&dag, "CommutativeSemiring"),
        record_fields(&dag, "Semiring"),
        "CommutativeSemiring adds the multiplicative-commutativity law, not a parallel field shape"
    );
}

/// T-Numeric-Construction `GroupCompletion<M>` substrate-introduction —
/// the Shape C opaque atom recommended by the Slice 3 prerequisite audit
/// (`docs/audit/t-numeric-construction-group-completion-6q.md`).
///
/// The ratchet pins:
/// - `GroupCompletion` lives in `dsl/std/algebra.dag` (audit's preferred home;
///   Director-confirmed boundary).
/// - It is a **bare opaque atom** — `TypeConnective::Conj { children: [] }`
///   with no fields. No quotient-of-pairs / sign-magnitude / explicit-carrier
///   structural facts admitted at this layer (per Director's hard boundary).
/// - It carries exactly one type parameter `<M>` (the input commutative
///   monoid; constraint is unenforced denotationally per the audit's
///   constrained-inhabitance gap).
///
/// This is the carrier construction; the algebra witness for Slice 3 will be
/// `AbelianGroup<GroupCompletion<Nat>>` — declared at the future `Int`
/// alias-pivot site, not here.
#[test]
fn group_completion_is_bare_opaque_atom_with_one_type_parameter() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let group_completion = dag.declaration_by_name("GroupCompletion").expect(
        "`GroupCompletion` substrate-introduction missing from full bootstrap \
         (T-Numeric-Construction Slice 3 prerequisite per #1422 audit)",
    );

    assert_eq!(
        group_completion.span.file, "dsl/std/algebra.dag",
        "GroupCompletion must live in dsl/std/algebra.dag per the audit's \
         preferred home (proximity to AbelianGroup<T>)"
    );

    match &group_completion.connective {
        TypeConnective::Conj { children } => assert!(
            children.is_empty(),
            "GroupCompletion must be a bare opaque atom (no fields) — Shape C \
             rejects quotient-of-pairs and sign-magnitude representation \
             facts per the audit and Director's dispatch"
        ),
        other => panic!(
            "GroupCompletion must lower as an opaque-atom Conj with no fields; \
             got {other:?} — a non-empty Conj would admit structural \
             representation facts (quotient/sign-magnitude) the audit rejects"
        ),
    }

    assert_eq!(
        group_completion.type_params.len(),
        1,
        "GroupCompletion takes exactly one type parameter `<M>` (the input \
         commutative-monoid type)"
    );

    assert!(
        group_completion.value_body.is_none(),
        "GroupCompletion is an opaque type declaration, not a data declaration; \
         no value_body should be present"
    );
}

/// T-Numeric-Construction `FieldOfFractions<R>` substrate-introduction —
/// Shape C opaque atom per `docs/audit/t-numeric-construction-field-of-fractions-6q.md`.
///
/// Mirrors the `GroupCompletion<M>` ratchet: single declaration in
/// `dsl/std/algebra.dag`, bare `Conj {{ children: [] }}`, exactly one type parameter `<R>`.
///
/// Also scans bootstrap for `FieldOfFractions<…>` instantiations: Slice 4 must keep
/// `Int` as the sole specialization (`Field<FieldOfFractions<Int>>` via `dsl/std/rational.dag`).
#[test]
fn field_of_fractions_substrate_introduction_ratchets() {
    with_full_bootstrap_stack(|| {
        let dag = v3_compiler::generated_full_bootstrap_dag();

        let fof = dag.declaration_by_name("FieldOfFractions").expect(
            "`FieldOfFractions` substrate-introduction missing from full bootstrap \
         (T-Numeric-Construction Slice 4 prerequisite per field-of-fractions 6Q audit)",
        );

        assert_eq!(
            fof.span.file, "dsl/std/algebra.dag",
            "FieldOfFractions must live in dsl/std/algebra.dag per the audit's preferred home"
        );

        match &fof.connective {
            TypeConnective::Conj { children } => assert!(
                children.is_empty(),
                "FieldOfFractions must be a bare opaque atom (no fields) — Shape C rejects \
             numerator/denominator pair and quotient representation at this layer"
            ),
            other => panic!(
                "FieldOfFractions must lower as an opaque-atom Conj with no fields; got {other:?}"
            ),
        }

        assert_eq!(
            fof.type_params.len(),
            1,
            "FieldOfFractions takes exactly one type parameter `<R>` (the input integral-domain type)"
        );

        assert!(
            fof.value_body.is_none(),
            "FieldOfFractions is an opaque type declaration, not a data declaration; \
         no value_body should be present"
        );

        let int_decl = dag
            .declaration_by_name("Int")
            .expect("`Int` must be present for Slice 4 integral-domain consumer");

        for decl in dag.declarations() {
            let TypeConnective::Instantiation {
                template,
                arguments,
            } = &decl.connective
            else {
                continue;
            };
            if *template != fof.id {
                continue;
            }
            assert_eq!(
                arguments.len(),
                1,
                "FieldOfFractions<R> instantiates with exactly one template argument"
            );
            assert_eq!(
                arguments[0].parameter, fof.type_params[0],
                "FieldOfFractions template argument must bind the `<R>` parameter"
            );
            assert_eq!(
                arguments[0].value,
                int_decl.id,
                "FieldOfFractions<R> instantiations must specialize `Int` only per \
             docs/audit/t-numeric-construction-field-of-fractions-6q.md; got argument {:?} on {}",
                arguments,
                decl.name.as_deref().unwrap_or("<anonymous>")
            );
        }
    });
}

/// T-Numeric-Construction Slice 4 — `Rational = Field<FieldOfFractions<Int>>` per
/// `docs/audit/t-numeric-construction-field-of-fractions-6q.md` (canonical Q6 form;
/// rejects compact `FieldOfFractions<Int>` alone).
///
/// Pins `Rational` authority to `dsl/std/rational.dag` and the two-step witness:
/// outer `Field<T>`, inner carrier `FieldOfFractions<Int>`.
#[test]
fn rational_default_alias_resolves_to_field_over_field_of_fractions_of_int() {
    with_full_bootstrap_stack(|| {
        let dag = v3_compiler::generated_full_bootstrap_dag();

        let rational = dag
            .declaration_by_name("Rational")
            .expect("`Rational` default alias missing from full bootstrap (Slice 4)");
        assert_eq!(
            rational.span.file, "dsl/std/rational.dag",
            "Rational must live in dsl/std/rational.dag (single authority)"
        );

        let field = dag
            .declaration_by_name("Field")
            .expect("`Field` algebra must be loaded from dsl/std/algebra.dag");
        let fof = dag
            .declaration_by_name("FieldOfFractions")
            .expect("`FieldOfFractions` carrier must be present (Slice 4 prerequisite)");
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("`Int` must be present for FieldOfFractions<Int>");

        let mut current = rational;
        let mut hops: usize = 0;
        let connective = loop {
            match &current.connective {
                TypeConnective::Atom(AtomPayload::ResolvedByName(next))
                | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => {
                    assert!(hops < 8, "Rational alias chain too deep (cycle?)");
                    hops += 1;
                    current = dag.declaration(*next);
                }
                other => break other,
            }
        };

        let inner_carrier = match connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                assert_eq!(
                    *template, field.id,
                    "Rational's outer witness must be `Field<T>`, not a compact \
                     `FieldOfFractions<Int>`-only form (Q6 single-authority)"
                );
                assert_eq!(
                    arguments.len(),
                    1,
                    "Field<T> takes exactly one carrier type parameter"
                );
                arguments[0].value
            }
            other => panic!("Rational must lower to a Field instantiation; got {other:?}"),
        };

        let carrier_decl = dag.declaration(inner_carrier);
        match &carrier_decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                assert_eq!(
                    *template, fof.id,
                    "Rational's Field carrier must be `FieldOfFractions<…>` — \
                     the localization-derived carrier, not `Int` directly \
                     (reject `Field<Int>` as dishonest reciprocal story)"
                );
                assert_eq!(
                    arguments.len(),
                    1,
                    "FieldOfFractions<R> takes exactly one integral-domain parameter"
                );
                assert_eq!(
                    arguments[0].value, int_decl.id,
                    "Slice 4 pins `FieldOfFractions<Int>` as the sole specialization — \
                     ℚ as field of fractions of ℤ"
                );
            }
            other => panic!(
                "Rational's Field carrier must be a FieldOfFractions instantiation; got {other:?}"
            ),
        }
    });
}

/// R3 gate #17 (`numeric_abstract_carriers_landed`) — `Real` abstract carrier per
/// `docs/r3-structure.md` + STOP Option A:
/// `Real = ApproximateField<FieldOfFractions<Int>>` in `dsl/std/float.dag`.
#[test]
fn real_default_alias_resolves_to_approximate_field_over_field_of_fractions_of_int() {
    with_full_bootstrap_stack(|| {
        let dag = v3_compiler::generated_full_bootstrap_dag();

        let real = dag
            .declaration_by_name("Real")
            .expect("`Real` abstract carrier missing from full bootstrap (R3 gate #17)");
        assert_eq!(
            real.span.file, "dsl/std/float.dag",
            "Real must live in dsl/std/float.dag (single authority)"
        );

        let approx = dag
            .declaration_by_name("ApproximateField")
            .expect("`ApproximateField` must be present");
        let fof = dag
            .declaration_by_name("FieldOfFractions")
            .expect("`FieldOfFractions` carrier must be present");
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("`Int` must be present for FieldOfFractions<Int>");

        let mut current = real;
        let mut hops: usize = 0;
        let connective = loop {
            match &current.connective {
                TypeConnective::Atom(AtomPayload::ResolvedByName(next))
                | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => {
                    assert!(hops < 8, "Real alias chain too deep (cycle?)");
                    hops += 1;
                    current = dag.declaration(*next);
                }
                other => break other,
            }
        };

        let (template_id, arguments) = match connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => (*template, arguments),
            other => panic!("Real must lower to an ApproximateField instantiation; got {other:?}"),
        };

        assert_eq!(
            arguments.len(),
            1,
            "`Real` must instantiate `ApproximateField` with exactly one carrier argument \
             (`FieldOfFractions<Int>` per STOP Option A)"
        );

        let mut resolved_template = template_id;
        while let TypeConnective::Atom(atom) = &dag.declaration(resolved_template).connective {
            match atom {
                AtomPayload::ResolvedByName(next) => resolved_template = *next,
                AtomPayload::ResolvedByStructure(next) => resolved_template = *next,
                _ => break,
            }
        }

        assert_eq!(
            resolved_template, approx.id,
            "Real must instantiate imported `ApproximateField<F>` (resolve import stubs)"
        );

        let carrier_id = arguments[0].value;
        let carrier_decl = dag.declaration(carrier_id);
        match &carrier_decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                assert_eq!(
                    *template,
                    fof.id,
                    "Real's type argument must be `FieldOfFractions<Int>` — not `Rational` \
                     (`Rational` names `Field<FieldOfFractions<Int>>`, a witness type)"
                );
                assert_eq!(
                    arguments.len(),
                    1,
                    "FieldOfFractions<R> takes exactly one integral-domain parameter"
                );
                assert_eq!(
                    arguments[0].value,
                    int_decl.id,
                    "Real approximates ℚ as field of fractions of ℤ — parameter must be `Int`"
                );
            }
            other => panic!(
                "Real's ApproximateField carrier argument must be a FieldOfFractions instantiation; got {other:?}"
            ),
        }
    });
}

/// T-Numeric-Construction Slice 3 — `Int = AbelianGroup<GroupCompletion<Nat>>`
/// resolves structurally as the canonical Q6 single-authority form per
/// `docs/audit/t-numeric-construction-group-completion-6q.md`.
///
/// The ratchet enforces three structural facts at once:
/// - `Int` lives in `dsl/std/integer.dag` (single authority).
/// - The alias-chain target is an `AbelianGroup` instantiation (NOT
///   `OrderedRing`/`Int64`, NOT the rejected compact `GroupCompletion<Nat>`
///   form, NOT a `Word*` storage carrier).
/// - The `AbelianGroup`'s carrier argument is itself an instantiation:
///   `GroupCompletion<Nat>` — proving the two-step construction
///   (`AbelianGroup<T>` standard parametric reading; `T = GroupCompletion<Nat>`
///   is the derived carrier; `<Nat>` is the input commutative-monoid).
#[test]
fn int_default_alias_resolves_to_abelian_group_over_group_completion_of_nat() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

    let int_default = dag
        .declaration_by_name("Int")
        .expect("`Int` default alias missing from full bootstrap (T-Numeric-Construction Slice 3)");
    assert_eq!(
        int_default.span.file, "dsl/std/integer.dag",
        "Int default alias must live in dsl/std/integer.dag, not in a parallel authority"
    );

    let abelian_group = dag
        .declaration_by_name("AbelianGroup")
        .expect("`AbelianGroup` algebra must be loaded from dsl/std/algebra.dag");
    let group_completion = dag
        .declaration_by_name("GroupCompletion")
        .expect("`GroupCompletion` carrier must be loaded from dsl/std/algebra.dag (#1448)");
    let nat = dag
        .declaration_by_name("Nat")
        .expect("`Nat` carrier must be loaded from dsl/std/nat.dag (Slice 2)");

    // Walk the alias chain to the underlying instantiation.
    let mut current = int_default;
    let mut hops: usize = 0;
    let connective = loop {
        match &current.connective {
            TypeConnective::Atom(AtomPayload::ResolvedByName(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => {
                assert!(hops < 8, "Int alias chain too deep (cycle?)");
                hops += 1;
                current = dag.declaration(*next);
            }
            other => break other,
        }
    };

    let outer_arg_decl = match connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, abelian_group.id,
                "Int's outer instantiation must be `AbelianGroup`; not `OrderedRing`/`Int64` \
                 (legacy storage chain), not the rejected compact `GroupCompletion<Nat>` form"
            );
            assert_eq!(
                arguments.len(),
                1,
                "AbelianGroup<T> takes exactly one carrier type parameter"
            );
            arguments[0].value
        }
        other => panic!("Int must lower to an AbelianGroup instantiation; got {other:?}"),
    };

    // The carrier argument resolves to `GroupCompletion<Nat>` — itself an
    // Instantiation { template: GroupCompletion, arguments: [Nat] }.
    let carrier_decl = dag.declaration(outer_arg_decl);
    match &carrier_decl.connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, group_completion.id,
                "Int's AbelianGroup carrier must be `GroupCompletion<...>` — \
                 the derived-carrier construction, not Nat directly (the \
                 rejected `AbelianGroup<Nat>` form would assert Nat has \
                 additive inverses) and not a `Word*` storage carrier"
            );
            assert_eq!(
                arguments.len(),
                1,
                "GroupCompletion<M> takes exactly one type parameter"
            );
            assert_eq!(
                arguments[0].value, nat.id,
                "GroupCompletion's input commutative-monoid must be `Nat`; \
                 the construction chain is Magnitude → Nat → Int"
            );
        }
        other => panic!(
            "Int's AbelianGroup carrier must be a GroupCompletion instantiation; got {other:?}"
        ),
    }
}
