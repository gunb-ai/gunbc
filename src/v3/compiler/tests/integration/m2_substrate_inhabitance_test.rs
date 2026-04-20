use v3_compiler::dag::{FieldValue, TypeConnective, ValueBody};
use v3_compiler::parse_surface;
use v3_compiler::{parse_for_test, tokenize_for_test};
use v3_compiler::Dag;

fn find_named(dag: &Dag, name: &str) -> v3_compiler::dag::DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
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
                    String::from("span"),
                ],
            ),
            (
                String::from("TypeAlias"),
                vec![
                    String::from("name"),
                    String::from("type_params"),
                    String::from("target"),
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
fn parse_surface_generated_module_consumes_parser_output_structurally() {
    let source = "fn id<T>(x: T where x == x) -> T = x\nlet y = if true then id(1) else 2";
    let tokens = tokenize_for_test(source, "surface_reflection.v3").expect("tokenize source");
    let parsed = parse_for_test(&tokens, "surface_reflection.v3").expect("parse source");
    let mirrored = parse_surface::SurfaceModule::from(&parsed);

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
        "runtime mirror snapshots are stale; run scripts/regen_runtime_mirrors.py"
    );
}
