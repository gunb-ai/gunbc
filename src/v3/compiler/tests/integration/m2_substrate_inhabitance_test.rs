use std::collections::HashMap;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    algebra_profile_to_dimension, constant_bound_value, evidence_rank, is_constant_bound,
    join_evidence, kernel_algebra_profile, lower_call_pattern, map_evidence_merge_at,
    merge_evidence, optional_evidence_meet, per_call_descent_evidence, positive_amount_from_i64,
    promote_to_strict, size_bound_param, tree_size_bound, type_iteration_dimension, AlgebraProfile,
    ArrowBody, AtomPayload, CallPattern, CardinalityBound, DescentEvidence, FieldValue, Interval,
    IntervalWidth, IterationDimension, IterationPrimitive, LoweringTarget, PositiveDescentAmount,
    PositiveIntervalWidth, ProportionalDivisor, ShrinkFactor, SizeBound, SubValueRelation,
    TypeConnective, ValueBody,
};
use v3_compiler::parse_surface;
use v3_compiler::CompileError;
use v3_compiler::Dag;
use v3_compiler::Diagnostic;
use v3_compiler::{parse_for_test, tokenize_for_test};

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
                String::from("FieldProject"),
                vec![String::from("field_label"), String::from("field_child")],
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
fn v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority() {
    fn v2_profile_to_v3(p: v2_compiler::std_algebra::AlgebraProfile) -> AlgebraProfile {
        use v2_compiler::std_algebra::AlgebraProfile as V2;
        match p {
            V2::OrderedRingProfile => AlgebraProfile::OrderedRingProfile,
            V2::ApproximateFieldProfile => AlgebraProfile::ApproximateFieldProfile,
            V2::BooleanAlgebraProfile => AlgebraProfile::BooleanAlgebraProfile,
            V2::BooleanAlgebraCollectionProfile => AlgebraProfile::BooleanAlgebraCollectionProfile,
            V2::FreeMonoidScalarProfile => AlgebraProfile::FreeMonoidScalarProfile,
            V2::FreeMonoidCollectionProfile => AlgebraProfile::FreeMonoidCollectionProfile,
            V2::PartialFunctionProfile => AlgebraProfile::PartialFunctionProfile,
        }
    }

    let v2_map = v2_compiler::std_algebra::kernel_algebra_profile();
    assert!(
        !v2_map.is_empty(),
        "v2 stage0 kernel_algebra_profile table must be non-empty (dsl/std/algebra.dag authority)"
    );
    for (type_name, v2_profile) in v2_map.iter() {
        assert_eq!(
            kernel_algebra_profile(type_name),
            Some(v2_profile_to_v3(*v2_profile)),
            "v3 `dag::kernel_algebra_profile` must match v2 stage0 row for `{type_name}` \
             (stage0 is regenerated from dsl/std/algebra.dag `data kernel_algebra_profile`)"
        );
    }
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
        sum_variants(&dag, "PositiveIntervalWidth"),
        vec![
            (String::from("OneUnit"), Vec::new()),
            (
                String::from("AdditionalUnit"),
                vec![String::from("previous")],
            ),
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
                String::from("FieldProject"),
                vec![String::from("field_label"), String::from("field_child")],
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
            (String::from("Descent"), vec![String::from("cluster")]),
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
                        parse_surface::SurfaceExpr::Literal {
                            value: parse_surface::SurfaceLiteral::Int(0),
                            ..
                        }
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
    ] {
        let decl = dag
            .declaration_by_name(name)
            .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
        assert_eq!(
            decl.span.file, "src/v3/std/approximate_field.dag",
            "`{name}` must stay in the approximate-field axes precursor; the \
             full ApproximateField carrier and Float migration land later"
        );
    }
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

#[test]
fn pr_a_3_eval_strategy_carriers_match_eager_baseline_shape() {
    let dag = v3_compiler::generated_full_bootstrap_dag();

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
        "PR-A.3 eager baseline must not introduce fake strategy variants"
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
        other => panic!("InputEvaluationOrder must lower as a one-variant Disj, got {other:?}"),
    };
    assert_eq!(
        input_order_variants.len(),
        1,
        "PR-A.3 eager baseline must not introduce fake input-order variants"
    );
    assert_eq!(input_order_variants[0].label, "LeftFirst");
    match &dag.declaration(input_order_variants[0].ty).connective {
        TypeConnective::Conj { children } => assert!(
            children.is_empty(),
            "LeftFirst is a bare nullary variant and must carry no payload fields"
        ),
        other => panic!("LeftFirst payload must lower as empty Conj, got {other:?}"),
    }
}

/// T-Numeric-Construction Slice 2 — `Nat = Semiring<Magnitude>` resolves
/// structurally to a `Semiring` instantiation whose carrier argument is the
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

    let semiring = dag
        .declaration_by_name("Semiring")
        .expect("`Semiring` algebra must be loaded from dsl/std/algebra.dag");
    let magnitude = dag
        .declaration_by_name("Magnitude")
        .expect("`Magnitude` opaque carrier must be loaded from dsl/std/magnitude.dag (Slice 1)");

    match &nat.connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, semiring.id,
                "Nat must instantiate `Semiring`, not an alternate algebra record"
            );
            assert_eq!(
                arguments.len(),
                1,
                "Semiring takes exactly one carrier type parameter"
            );
            assert_eq!(
                arguments[0].value, magnitude.id,
                "Nat's Semiring carrier argument must be `Magnitude`, not a Word* storage carrier"
            );
        }
        other => panic!(
            "Nat must lower to a Semiring instantiation; got {other:?} — \
             a non-Instantiation connective indicates a parallel algebra authority \
             rather than a clean Semiring<Magnitude> alias"
        ),
    }
}
