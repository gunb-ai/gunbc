use std::collections::HashMap;

use v3_compiler::dag::{
    algebra_profile_to_dimension, constant_bound_value, evidence_rank, is_constant_bound,
    join_evidence, lower_call_pattern, map_evidence_merge_at, merge_evidence,
    optional_evidence_meet, promote_to_strict, size_bound_param, tree_size_bound,
    type_iteration_dimension, AlgebraProfile, ArrowBody, CallPattern, DescentEvidence, FieldValue,
    IterationDimension, IterationPrimitive, LoweringTarget, ShrinkFactor, SizeBound,
    TypeConnective, ValueBody,
};
use v3_compiler::parse_surface;
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

fn variant_payload_field_type_name(
    dag: &Dag,
    sum_name: &str,
    variant_name: &str,
    field_name: &str,
) -> String {
    let id = find_named(dag, sum_name);
    let TypeConnective::Disj { variants } = &dag.declaration(id).connective else {
        panic!("expected `{sum_name}` to lower to a Disj");
    };
    let variant = variants
        .iter()
        .find(|variant| variant.label == variant_name)
        .unwrap_or_else(|| panic!("variant `{variant_name}` not found under `{sum_name}`"));
    let TypeConnective::Conj { children } = &dag.declaration(variant.ty).connective else {
        panic!("expected variant `{variant_name}` under `{sum_name}` to lower to a Conj payload");
    };
    let field = children
        .iter()
        .find(|field| field.label == field_name)
        .unwrap_or_else(|| {
            panic!("field `{field_name}` not found on variant `{variant_name}` under `{sum_name}`")
        });
    dag.declaration(field.ty)
        .name
        .clone()
        .unwrap_or_else(|| format!("<anonymous:{}>", field.ty.raw()))
}


fn arrow_body(dag: &Dag, name: &str) -> ArrowBody {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Arrow { body, .. } => body.clone(),
        other => panic!("expected `{name}` to lower to an Arrow, got {other:?}"),
    }
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
            "meta_tag",
            "inhabits",
            "value_body",
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
        vec!["id", "input", "paths", "result_port", "span"]
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
            "lane2_workflow"
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
        sum_variants(&dag, "DivisionDescentFactor"),
        vec![
            (String::from("Two"), Vec::new()),
            (String::from("GreaterThanTwo"), vec![String::from("extra")]),
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
            (String::from("ArithmeticSubtract"), vec![String::from("by")],),
            (String::from("ArithmeticDivide"), vec![String::from("by")],),
            (String::from("ParserAdvance"), vec![String::from("witness")]),
            (String::from("SetRemoval"), vec![String::from("element")]),
            (String::from("FoldIteration"), Vec::new()),
        ]
    );
    assert_eq!(
        variant_payload_field_type_name(&dag, "DivisionDescentFactor", "GreaterThanTwo", "extra"),
        "PositiveInt"
    );
    assert_eq!(
        variant_payload_field_type_name(&dag, "DescentSource", "ListShrink", "amount"),
        "PositiveInt"
    );
    assert_eq!(
        variant_payload_field_type_name(&dag, "DescentSource", "ArithmeticSubtract", "by"),
        "PositiveInt"
    );
    assert_eq!(
        variant_payload_field_type_name(&dag, "DescentSource", "ArithmeticDivide", "by"),
        "DivisionDescentFactor"
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
            (String::from("TreeSize"), vec![String::from("param")]),
            (String::from("ArithmeticParam"), vec![String::from("param")]),
            (String::from("ExplicitCount"), vec![String::from("n")]),
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
                vec![String::from("collection"), String::from("amount")],
            ),
            (
                String::from("ArithmeticSubtractCall"),
                vec![String::from("param"), String::from("by")],
            ),
            (
                String::from("ArithmeticDivideCall"),
                vec![String::from("param"), String::from("by")],
            ),
            (
                String::from("ParserAdvanceCall"),
                vec![String::from("stream"), String::from("witness")],
            ),
            (
                String::from("WorklistDrainCall"),
                vec![String::from("worklist"), String::from("element")],
            ),
            (
                String::from("FoldBodyCall"),
                vec![String::from("outer_collection")],
            ),
            (String::from("SameArgumentCall"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "ShrinkFactor"),
        vec![
            (String::from("UnitShrink"), Vec::new()),
            (String::from("ConstantShrink"), vec![String::from("amount")]),
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
        "size_bound_param",
        "is_constant_bound",
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
    use CallPattern::{
        ArithmeticDivideCall, ArithmeticSubtractCall, ChildAccessorCall, CollectionShrinkCall,
        FoldBodyCall, ParserAdvanceCall, SameArgumentCall, WorklistDrainCall,
    };
    use DescentEvidence::{NonIncreasing, Strict};
    use IterationPrimitive::{Descend, Fold, Repeat};
    use ShrinkFactor::{ConstantShrink, ProportionalShrink};
    use SizeBound::{ArithmeticParam, CollectionSize, Forever, TreeSize};

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
                collection: String::from("items"),
                amount: 1,
            },
            LoweringTarget {
                primitive: Fold,
                bound: CollectionSize {
                    param: String::from("items"),
                },
                evidence: Strict,
                factor: Some(ConstantShrink { amount: 1 }),
            },
        ),
        (
            ArithmeticSubtractCall {
                param: String::from("n"),
                by: 1,
            },
            LoweringTarget {
                primitive: Repeat,
                bound: ArithmeticParam {
                    param: String::from("n"),
                },
                evidence: Strict,
                factor: Some(ConstantShrink { amount: 1 }),
            },
        ),
        (
            ArithmeticDivideCall {
                param: String::from("k"),
                by: 2,
            },
            LoweringTarget {
                primitive: Repeat,
                bound: ArithmeticParam {
                    param: String::from("k"),
                },
                evidence: Strict,
                factor: Some(ProportionalShrink { divisor: 2 }),
            },
        ),
        (
            ParserAdvanceCall {
                stream: String::from("tokens"),
                witness: String::from("advance"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: CollectionSize {
                    param: String::from("tokens"),
                },
                evidence: Strict,
                factor: None,
            },
        ),
        (
            WorklistDrainCall {
                worklist: String::from("frontier"),
                element: String::from("item"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: CollectionSize {
                    param: String::from("frontier"),
                },
                evidence: Strict,
                factor: None,
            },
        ),
        (
            FoldBodyCall {
                outer_collection: String::from("outer"),
            },
            LoweringTarget {
                primitive: Fold,
                bound: CollectionSize {
                    param: String::from("outer"),
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
    let arithmetic = SizeBound::ArithmeticParam {
        param: String::from("n"),
    };
    let explicit = SizeBound::ExplicitCount { n: 7 };
    let forever = SizeBound::Forever;

    assert_eq!(size_bound_param(&tree), Some("node"));
    assert_eq!(size_bound_param(&collection), Some("items"));
    assert_eq!(size_bound_param(&arithmetic), Some("n"));
    assert_eq!(size_bound_param(&explicit), None);
    assert_eq!(size_bound_param(&forever), None);

    assert!(!is_constant_bound(&tree));
    assert!(!is_constant_bound(&collection));
    assert!(!is_constant_bound(&arithmetic));
    assert!(is_constant_bound(&explicit));
    assert!(is_constant_bound(&forever));

    assert_eq!(constant_bound_value(&explicit), Some(7));
    assert_eq!(constant_bound_value(&forever), Some(1));
    assert_eq!(constant_bound_value(&tree), None);
    assert_eq!(constant_bound_value(&collection), None);
    assert_eq!(constant_bound_value(&arithmetic), None);
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
                    String::from("target"),
                    String::from("refinement"),
                    String::from("span"),
                ],
            ),
        ]
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
