//! **Layer:** integration
//!
//! **T-Workflow-As-Data** — `ci_workflow_modeled_as_dag` receipt (gunbc#1956): gunbc CI workflow
//! carriers from `dsl/extdeps/github/actions.dag` are authored as `.dag` **data** alongside a
//! `Lens<TimingMeasurement>` reporting shell; `demo_ci_modeled_timing_dimension_report` is exercised
//! via `evaluate_body` against bind ports resolved from **`generated_full_bootstrap_dag()`**.
//!
//! **R3 gate #57** (`lens_self_application_demonstrated`, T-Lens-Self-Application): the same module
//! hosts the executable receipt: `dsl/gunbc/ci.dag` source markers + bootstrap `modeled_gunbc_ci_workflow`,
//! timing via `evaluate_body` (large-stack thread for debug builds), and cost / complexity /
//! parallelism checks on a cached representative program (`lens_self_application_demonstrated`).
//!
//! Runtime `Dag` binding is an **opaque substrate-shaped record** (empty `declarations` /
//! `nodes` / `ports` / `clusters` lists built from existing `List<τ>.Empty` tags — see
//! `_ci_wad_seed_*` in `t_ci_workflow_as_data_demo.dag`). The eager evaluator cannot execute
//! arbitrary bootstrap `Transform` nodes, so this harness does **not** embed
//! `reflect_behavior_list(full_bootstrap.nodes)` even though `behavior_spine` / `is_empty` are
//! live on the carrier type.
//!
//! **INVARIANTS P5 — checkable receipt:** hand-Rust integration coverage here is transitional T-PB-B
//! surface; dissolution target is `.dag` `TestClaim` data per `sg0_census_test.rs` R1C-E notes.
//! This crate fails to build if the cited worker brief is removed from the worktree.

use crate::common::cached_compile_to_dag;
use crate::common::find_list_empty_constructor_tag;
use v3_compiler::dag::{
    AtomPayload, Behavior, CompositionVerdict, DeclarationId, EffectShape, FieldValue,
    IdempotentShape, LiteralBits, NonSingletonList, OperationEffect, TypeConnective, ValueNode,
    WorkflowEffect, WorkflowParallelismReport,
};
use v3_compiler::evaluator::{
    evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, NamedField, Value,
};
use v3_compiler::lens_cost::{complexity_of, ComplexityLookup};
use v3_compiler::{
    analyze_complexity, analyze_parallelism, analyze_symbolic_cost_dimension, compile_to_dag,
    generated_full_bootstrap_dag, CompileError, DimensionReport,
};

const DEMO_SPAN_FILE: &str = "src/v3/std/t_ci_workflow_as_data_demo.dag";
const GUNBC_CI_SOURCE: &str = include_str!("../../../../../dsl/gunbc/ci.dag");
const GUNBC_CI_FILE: &str = "dsl/gunbc/ci.dag";
const GUNBC_CI_GITHUB_WORKFLOW_SOURCE: &str =
    include_str!("../../../../../dsl/gunbc/ci_github_actions_workflow.dag");
const GUNBC_CI_GITHUB_WORKFLOW_FILE: &str = "dsl/gunbc/ci_github_actions_workflow.dag";
const GUNBC_CI_EMISSION_SOURCE: &str = include_str!("../../../../../dsl/gunbc/ci_emission.dag");
const GUNBC_CI_EMISSION_FILE: &str = "dsl/gunbc/ci_emission.dag";

// P5 checkable receipt (parent gate #1956 / brief linkage — same pattern as `tc1_*_strict_fire_test`).
const _: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md"
));

fn demo_bootstrap_dag() -> v3_compiler::dag::Dag {
    generated_full_bootstrap_dag()
}

fn bind_node_id_named(dag: &v3_compiler::dag::Dag, bind_name: &str) -> v3_compiler::dag::NodeId {
    dag.nodes()
        .iter()
        .find_map(|b| match b {
            Behavior::Bind(bind) if bind.name == bind_name => Some(bind.id),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing Bind named `{bind_name}`"))
}

fn lens_self_app_read_op(name: &str) -> OperationEffect {
    OperationEffect {
        operation_name: name.to_string(),
        shape: EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
    }
}

fn bind_node_id_for_fn(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::NodeId {
    use v3_compiler::dag::ArrowBody;
    let decl = dag.declaration_by_name(name).expect("decl");
    let TypeConnective::Arrow { body, .. } = &decl.connective else {
        panic!("arrow");
    };
    let ArrowBody::UserDefined(bind_id) = body else {
        panic!("user def");
    };
    bind_id.node_id()
}

fn disj_variant_constructor_id(
    dag: &v3_compiler::dag::Dag,
    sum_name: &str,
    variant_label: &str,
) -> DeclarationId {
    let mut decl_id = dag
        .declaration_by_name(sum_name)
        .unwrap_or_else(|| panic!("missing sum `{sum_name}`"))
        .id;
    const PEEL_MAX: usize = 64;
    for _ in 0..PEEL_MAX {
        let decl = dag.declaration(decl_id);
        match &decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                decl_id = *template;
            }
            TypeConnective::Disj { variants } => {
                return variants
                    .iter()
                    .find(|v| v.label == variant_label)
                    .unwrap_or_else(|| panic!("variant `{variant_label}` on {sum_name}"))
                    .ty;
            }
            TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name))
                if name == variant_label =>
            {
                return decl_id;
            }
            _ => panic!("unexpected connective while resolving `{sum_name}.{variant_label}`"),
        }
    }
    panic!("peel depth");
}

fn conj_field_ty(
    dag: &v3_compiler::dag::Dag,
    conj_decl_id: DeclarationId,
    label: &str,
) -> DeclarationId {
    let decl = dag.declaration(conj_decl_id);
    let TypeConnective::Conj { children } = &decl.connective else {
        panic!("expected Conj");
    };
    children
        .iter()
        .find(|c| c.label == label)
        .map(|c| c.ty)
        .unwrap_or_else(|| panic!("missing field `{label}`"))
}

fn named_record_root(dag: &v3_compiler::dag::Dag, name: &str) -> DeclarationId {
    let mut decl_id = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("missing record `{name}`"))
        .id;
    const PEEL_MAX: usize = 64;
    for _ in 0..PEEL_MAX {
        match &dag.declaration(decl_id).connective {
            TypeConnective::Conj { .. } => return decl_id,
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                decl_id = *template;
            }
            _ => panic!("`{name}` is not Conj"),
        }
    }
    panic!("record peel");
}

fn optional_workflow_effect_none(dag: &v3_compiler::dag::Dag) -> Value {
    let vn = named_record_root(dag, "ValueNode");
    let lane2_ty = conj_field_ty(dag, vn, "lane2_workflow");
    let card_id = peel_to_optional_cardinality_decl(dag, lane2_ty);
    let disj_id = dag
        .optional_match_disj(card_id)
        .unwrap_or_else(|| panic!("missing optional_match_disj row for ValueNode.lane2_workflow"));
    let decl = dag.declaration(disj_id);
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("optional disj");
    };
    let none_ty = variants
        .iter()
        .find(|v| v.label == "None")
        .expect("None variant")
        .ty;
    Value::VariantValue {
        tag: none_ty,
        payload: Box::new(Value::RecordValue(vec![])),
    }
}

fn peel_to_optional_cardinality_decl(
    dag: &v3_compiler::dag::Dag,
    mut ty: DeclarationId,
) -> DeclarationId {
    use v3_compiler::dag::CardinalityBound;
    const PEEL_MAX: usize = 64;
    for _ in 0..PEEL_MAX {
        match &dag.declaration(ty).connective {
            TypeConnective::Cardinality(p) if p.bound() == CardinalityBound::AtMostOne => {
                return ty;
            }
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                ty = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                ty = *next;
            }
            _ => panic!("optional peel failed at {ty:?}"),
        }
    }
    panic!("optional peel depth");
}

fn behavior_value_variant(dag: &v3_compiler::dag::Dag, v: &ValueNode) -> Value {
    let value_ctor = disj_variant_constructor_id(dag, "Behavior", "Value");
    let lane2 = optional_workflow_effect_none(dag);
    let span = v3_compiler::diagnostics::SourceSpan::new(DEMO_SPAN_FILE, 0, 0);
    let inner = Value::RecordValue(vec![
        NamedField {
            label: "id".to_string(),
            value: Value::LiteralValue(LiteralBits::Int("0".to_string())),
        },
        NamedField {
            label: "payload".to_string(),
            value: Value::LiteralValue(v.data.clone()),
        },
        NamedField {
            label: "result_port".to_string(),
            value: Value::LiteralValue(LiteralBits::Int(i64::from(v.output.raw()).to_string())),
        },
        NamedField {
            label: "span".to_string(),
            value: Value::RecordValue(vec![
                NamedField {
                    label: "file".to_string(),
                    value: Value::LiteralValue(LiteralBits::String(span.file.clone())),
                },
                NamedField {
                    label: "start".to_string(),
                    value: Value::LiteralValue(LiteralBits::Int(
                        i64::from(span.byte_start).to_string(),
                    )),
                },
                NamedField {
                    label: "end".to_string(),
                    value: Value::LiteralValue(LiteralBits::Int(
                        i64::from(span.byte_end).to_string(),
                    )),
                },
            ]),
        },
        NamedField {
            label: "lane2_workflow".to_string(),
            value: lane2,
        },
    ]);
    Value::VariantValue {
        tag: value_ctor,
        payload: Box::new(inner),
    }
}

fn empty_list_value(dag: &v3_compiler::dag::Dag, list_ty: DeclarationId) -> Value {
    let tag = find_list_empty_constructor_tag(dag, list_ty);
    Value::VariantValue {
        tag,
        payload: Box::new(Value::RecordValue(vec![])),
    }
}

/// Substrate-shaped `Dag` [`Value`] for PB-1 `evaluate_body` — empty component lists (evaluator-safe).
fn bootstrap_dag_runtime_carrier(dag: &v3_compiler::dag::Dag) -> Value {
    let dag_root = named_record_root(dag, "Dag");
    Value::RecordValue(vec![
        NamedField {
            label: "declarations".to_string(),
            value: empty_list_value(dag, conj_field_ty(dag, dag_root, "declarations")),
        },
        NamedField {
            label: "nodes".to_string(),
            value: empty_list_value(dag, conj_field_ty(dag, dag_root, "nodes")),
        },
        NamedField {
            label: "ports".to_string(),
            value: empty_list_value(dag, conj_field_ty(dag, dag_root, "ports")),
        },
        NamedField {
            label: "clusters".to_string(),
            value: empty_list_value(dag, conj_field_ty(dag, dag_root, "clusters")),
        },
    ])
}

fn sample_demo_value_behavior(dag: &v3_compiler::dag::Dag) -> Behavior {
    dag.nodes()
        .iter()
        .find_map(|n| match n {
            Behavior::Value(v) if v.span.file == DEMO_SPAN_FILE => Some(Behavior::Value(v.clone())),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "bootstrap must contain at least one Value behavior authored in {DEMO_SPAN_FILE}"
            )
        })
}

fn structural_field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> &'a FieldValue {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("missing structural field `{label}`"))
}

fn literal_string(value: &FieldValue) -> &str {
    let FieldValue::Literal(LiteralBits::String(value)) = value else {
        panic!("expected string literal field, got {value:?}");
    };
    value
}

fn structural_record(value: &FieldValue) -> &[(String, FieldValue)] {
    let FieldValue::Record(fields) = value else {
        panic!("expected structural record field, got {value:?}");
    };
    fields
}

fn structural_record_ref<'a>(
    dag: &'a v3_compiler::dag::Dag,
    value: &'a FieldValue,
) -> &'a [(String, FieldValue)] {
    match value {
        FieldValue::Record(fields) => fields,
        FieldValue::Reference(id) => {
            let decl = dag.declaration(*id);
            let Some(v3_compiler::dag::ValueBody::Structural { fields }) = &decl.value_body else {
                panic!(
                    "expected reference to structural data, got {:?}",
                    decl.value_body
                );
            };
            fields
        }
        _ => panic!("expected structural record or reference, got {value:?}"),
    }
}

fn variant_label(dag: &v3_compiler::dag::Dag, sum_name: &str, value: &FieldValue) -> &'static str {
    let FieldValue::Variant { constructor, .. } = value else {
        panic!("expected structural variant field, got {value:?}");
    };

    for label in [
        "LintCommand",
        "TestCommand",
        "IgnoredTestCommand",
        "ShellCommand",
    ] {
        if *constructor == disj_variant_constructor_id(dag, sum_name, label) {
            return label;
        }
    }
    panic!("unexpected {sum_name} constructor {constructor:?}");
}

fn structural_list(value: &FieldValue) -> &[FieldValue] {
    let FieldValue::List(items) = value else {
        panic!("expected structural list field, got {value:?}");
    };
    items
}

fn workflow_topology<'a>(
    dag: &'a v3_compiler::dag::Dag,
    fields: &'a [(String, FieldValue)],
) -> (&'a str, Vec<&'a str>, Vec<(&'a str, &'a str)>) {
    let pipeline = structural_record_ref(dag, structural_field(fields, "pipeline"));
    let name = literal_string(structural_field(pipeline, "name"));
    let node_ids = structural_list(structural_field(pipeline, "gates"))
        .iter()
        .map(|gate| literal_string(structural_field(structural_record_ref(dag, gate), "id")))
        .collect();
    let edges = structural_list(structural_field(fields, "edges"))
        .iter()
        .map(|edge| {
            let edge = structural_record(edge);
            let from = structural_record_ref(dag, structural_field(edge, "from"));
            let to = structural_record_ref(dag, structural_field(edge, "to"));
            (
                literal_string(structural_field(from, "id")),
                literal_string(structural_field(to, "id")),
            )
        })
        .collect();
    (name, node_ids, edges)
}

fn workflow_gate_records<'a>(
    dag: &'a v3_compiler::dag::Dag,
    fields: &'a [(String, FieldValue)],
) -> Vec<&'a [(String, FieldValue)]> {
    let pipeline = structural_record_ref(dag, structural_field(fields, "pipeline"));
    structural_list(structural_field(pipeline, "gates"))
        .iter()
        .map(|gate| structural_record_ref(dag, gate))
        .collect()
}

fn structural_value_body<'a>(
    dag: &'a v3_compiler::dag::Dag,
    name: &str,
) -> &'a [(String, FieldValue)] {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("{name} data must load"));
    let Some(v3_compiler::dag::ValueBody::Structural { fields }) = &decl.value_body else {
        panic!(
            "{name} must lower as structural data, got {:?}",
            decl.value_body
        );
    };
    fields
}

#[test]
fn ci_workflow_as_data_demo_pins_modeled_workflow_row() {
    let dag = demo_bootstrap_dag();
    assert!(
        dag.diagnostics().is_empty(),
        "fixture diagnostics: {:?}",
        dag.diagnostics()
    );
    dag.declaration_by_name("modeled_gunbc_ci_workflow")
        .expect("modeled_gunbc_ci_workflow data must load from t_ci_workflow_as_data_demo.dag");
    assert!(
        dag.declaration_by_name("modeled_gunbc_ci_workflow_dag")
            .is_none(),
        "bootstrap demo must not author a second CI DAG topology authority"
    );
}

#[test]
fn ci_workflow_as_data_demo_pins_structural_ci_dag_shape() {
    let ci = compile_to_dag(GUNBC_CI_SOURCE, GUNBC_CI_FILE)
        .unwrap_or_else(|err| panic!("compile {GUNBC_CI_FILE}: {err:?}"));
    let fields = structural_value_body(&ci, "ci_workflow_dag");
    let (name, node_ids, edges) = workflow_topology(&ci, fields);

    assert_eq!(
        name, "gunbc-ci",
        "modeled workflow DAG name must derive from the CI pipeline"
    );

    assert_eq!(
        node_ids,
        vec!["compile-gates", "lint", "tests", "l1-ratchet"],
        "CI workflow DAG must carry one node per structural gate"
    );

    assert_eq!(
        edges,
        vec![
            ("compile-gates", "lint"),
            ("compile-gates", "tests"),
            ("lint", "l1-ratchet"),
            ("tests", "l1-ratchet"),
        ],
        "CI workflow dependencies must be modeled as provider-neutral DAG edges"
    );

    // Edge endpoints are structural CIGate references; `workflow_topology`
    // projects ids only for readable assertions above.
}

#[test]
fn ci_workflow_as_data_demo_pins_interim_command_shape() {
    let ci = compile_to_dag(GUNBC_CI_SOURCE, GUNBC_CI_FILE)
        .unwrap_or_else(|err| panic!("compile {GUNBC_CI_FILE}: {err:?}"));
    let fields = structural_value_body(&ci, "ci_workflow_dag");
    let gate_records = workflow_gate_records(&ci, fields);

    let mut commands = gate_records
        .iter()
        .map(|gate| {
            let id = literal_string(structural_field(gate, "id"));
            let command = structural_field(gate, "command");
            let label = variant_label(&ci, "CICommand", command);
            let payload = match command {
                FieldValue::Variant { payload, .. } => payload.as_slice(),
                _ => unreachable!(),
            };
            let payload_text = payload
                .iter()
                .map(literal_string)
                .collect::<Vec<_>>()
                .join("|");
            (id, label, payload_text)
        })
        .collect::<Vec<_>>();
    commands.sort_by_key(|(id, ..)| *id);

    assert_eq!(
        commands,
        vec![
            ("compile-gates", "IgnoredTestCommand", "ci_".to_string()),
            (
                "l1-ratchet",
                "ShellCommand",
                "scripts/l1-ratchet.sh --check".to_string()
            ),
            ("lint", "LintCommand", String::new()),
            ("tests", "TestCommand", String::new()),
        ],
        "CICommand must keep impossible field combinations out of authored gate data"
    );
}

#[test]
fn ci_workflow_as_data_demo_uses_only_gunbc_ci_authority_topology() {
    let demo = demo_bootstrap_dag();
    let ci = compile_to_dag(GUNBC_CI_SOURCE, GUNBC_CI_FILE)
        .unwrap_or_else(|err| panic!("compile {GUNBC_CI_FILE}: {err:?}"));

    assert!(
        demo.declaration_by_name("modeled_gunbc_ci_workflow_dag")
            .is_none(),
        "bootstrap demo must not carry a mirror of ci_workflow_dag"
    );
    assert_eq!(
        workflow_topology(&ci, structural_value_body(&ci, "ci_workflow_dag")),
        (
            "gunbc-ci",
            vec!["compile-gates", "lint", "tests", "l1-ratchet"],
            vec![
                ("compile-gates", "lint"),
                ("compile-gates", "tests"),
                ("lint", "l1-ratchet"),
                ("tests", "l1-ratchet"),
            ],
        ),
        "dsl/gunbc/ci.dag must remain the single CI DAG topology authority"
    );
}

#[test]
fn gunbc_ci_github_actions_workflow_authority_compiles() {
    let dag = match compile_to_dag(
        GUNBC_CI_GITHUB_WORKFLOW_SOURCE,
        GUNBC_CI_GITHUB_WORKFLOW_FILE,
    ) {
        Ok(d) => d,
        Err(CompileError::Semantic(d)) => panic!(
            "compile {GUNBC_CI_GITHUB_WORKFLOW_FILE}: {:?}",
            d.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(e) => panic!("compile {GUNBC_CI_GITHUB_WORKFLOW_FILE}: {e:?}"),
    };
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());
}

/// Mechanical source gate: `.github/workflows/ci.yml` is the YAML authority; the committed
/// `dsl/gunbc/ci_github_actions_workflow.dag` row must match the generator library output
/// byte-for-byte (same implementation as the `gen_gunbc_ci_workflow_dag` binary).
#[test]
fn gunbc_ci_github_actions_workflow_dag_matches_yaml_generator_output() {
    use std::path::Path;

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let repo_root = repo_root
        .canonicalize()
        .unwrap_or_else(|e| panic!("canonicalize repo root {}: {e}", repo_root.display()));
    let ci_yml = repo_root.join(".github/workflows/ci.yml");
    let dag_path = repo_root.join("dsl/gunbc/ci_github_actions_workflow.dag");
    assert!(
        ci_yml.is_file(),
        "missing {} (repo root {})",
        ci_yml.display(),
        repo_root.display()
    );

    let yaml = std::fs::read_to_string(&ci_yml)
        .unwrap_or_else(|e| panic!("read {}: {e}", ci_yml.display()));
    let expected =
        std::fs::read(&dag_path).unwrap_or_else(|e| panic!("read {}: {e}", dag_path.display()));

    let fresh = gen_gunbc_ci_workflow_dag::emit_ci_github_actions_workflow_module(
        ".github/workflows/ci.yml",
        &yaml,
    )
    .unwrap_or_else(|e| panic!("emit_ci_github_actions_workflow_module: {e}"));

    assert_eq!(
        fresh.as_bytes(),
        expected.as_slice(),
        "`dsl/gunbc/ci_github_actions_workflow.dag` drifted from `.github/workflows/ci.yml`; regenerate with:\n  CTRL_BUILD_WRAP_CARGO=0 cargo run -q -p gen_gunbc_ci_workflow_dag -- .github/workflows/ci.yml > dsl/gunbc/ci_github_actions_workflow.dag"
    );
}

#[test]
fn gunbc_ci_emission_substrate_compiles() {
    let dag = compile_to_dag(GUNBC_CI_EMISSION_SOURCE, GUNBC_CI_EMISSION_FILE)
        .unwrap_or_else(|err| panic!("compile {GUNBC_CI_EMISSION_FILE}: {err:?}"));
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());
}

/// R3 gate #57 — `lens_self_application_demonstrated` (T-Lens-Self-Application).
///
/// Pins gunbc's CI workflow as `.dag` structural authority and exercises the repository's lens
/// stack: timing via `DimensionReport<TimingMeasurement>` (`evaluate_body` on the bootstrap shell),
/// symbolic-cost + E7 complexity entrypoints via `DimensionReport<SymbolicCost>`, structural
/// `complexity_of` read, and lane-2 parallelism composition (`WorkflowParallelismReport` — the
/// substrate report carrier until a `DimensionReport` projection lands for parallelism).
#[test]
fn lens_self_application_demonstrated() {
    // `compile_to_dag` on `dsl/gunbc/ci.dag` alone is not a supported entry shape today (the
    // `ci_workflow_dag` row carries a cross-module `Workflow` reference that M1(2.8) treats as an
    // opaque data body under single-file lowering). Gate #57 anchors on (a) the committed `ci.dag`
    // source markers and (b) the bootstrap `modeled_gunbc_ci_workflow` carrier from
    // `t_ci_workflow_as_data_demo.dag`, which is the CI-as-data modeling surface the timing lens
    // shell targets.
    assert!(
        GUNBC_CI_SOURCE.contains("data ci_workflow_dag"),
        "dsl/gunbc/ci.dag must retain the `ci_workflow_dag` authority row"
    );
    assert!(
        GUNBC_CI_SOURCE.contains("gunbc-ci"),
        "dsl/gunbc/ci.dag must retain the `gunbc-ci` pipeline name"
    );

    let boot = demo_bootstrap_dag();
    assert!(
        boot.diagnostics().is_empty(),
        "bootstrap diagnostics: {:?}",
        boot.diagnostics()
    );
    boot.declaration_by_name("modeled_gunbc_ci_workflow")
        .expect(
            "bootstrap must load `modeled_gunbc_ci_workflow` from t_ci_workflow_as_data_demo.dag",
        );
    assert!(
        boot.declaration_by_name("modeled_gunbc_ci_workflow_dag")
            .is_none(),
        "bootstrap demo must not author a second CI DAG topology authority"
    );

    // --- Timing lens: bootstrap `demo_ci_modeled_timing_dimension_report` (prior ignored receipt). ---
    // `evaluate_body` on the full bootstrap `Dag` recurses deeply enough to overflow the default
    // test thread stack in debug builds; run the receipt on a dedicated thread with a larger stack
    // (same pattern as other full-bootstrap evaluator harnesses).
    let dag_for_timing = boot.clone();
    let timing_join = std::thread::Builder::new()
        .name("lens_self_app_timing_dimension_report".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let dag = dag_for_timing;
            let (d_port, b_port) = {
                let bind_node_id =
                    bind_node_id_for_fn(&dag, "demo_ci_modeled_timing_dimension_report");
                let Behavior::Bind(bind) = dag.node(bind_node_id) else {
                    panic!("demo_ci_modeled_timing_dimension_report bind");
                };
                assert_eq!(
                    bind.params.len(),
                    2,
                    "demo_ci_modeled_timing_dimension_report expects Dag and Behavior"
                );
                (bind.params[0], bind.params[1])
            };

            let d_val = bootstrap_dag_runtime_carrier(&dag);
            let b_beh = sample_demo_value_behavior(&dag);
            let b_val = match &b_beh {
                Behavior::Value(v) => behavior_value_variant(&dag, v),
                _ => unreachable!(),
            };

            let bind_node_id = bind_node_id_for_fn(&dag, "demo_ci_modeled_timing_dimension_report");
            let frame =
                EvalFrame::from_bindings([(d_port, d_val), (b_port, b_val)]).expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = EvalStrategy::ApplicativeOrder {
                input_order: InputEvaluationOrder::LeftFirst,
            };
            let out = evaluate_body(&dag, bind_node_id, &mut state, strategy).expect("eval");

            let Value::VariantValue { tag, payload } = &out else {
                panic!("expected DimensionReport variant Value, got {out:?}");
            };
            let dim_ok = disj_variant_constructor_id(&dag, "DimensionReport", "DimensionOk");
            assert_eq!(*tag, dim_ok, "expected DimensionOk");

            let Value::RecordValue(fields) = &**payload else {
                panic!("DimensionOk payload record");
            };
            let composed = fields
                .iter()
                .find(|f| f.label == "composed")
                .map(|f| &f.value)
                .expect("composed");

            let observed_tag = disj_variant_constructor_id(&dag, "TimingMeasurement", "Observed");
            let Value::VariantValue {
                tag: ctag,
                payload: cpayload,
            } = composed
            else {
                panic!("composed must be TimingMeasurement variant, got {composed:?}");
            };
            assert_eq!(*ctag, observed_tag, "composed must be Observed");

            let Value::RecordValue(duration_fields) = &**cpayload else {
                panic!("Observed payload");
            };
            let duration = duration_fields
                .iter()
                .find(|f| f.label == "duration")
                .map(|f| &f.value)
                .expect("duration");
            let Value::RecordValue(count_fields) = duration else {
                panic!("Nanoseconds record");
            };
            let count = count_fields
                .iter()
                .find(|f| f.label == "count")
                .map(|f| &f.value)
                .expect("count");
            assert_eq!(
                count,
                &Value::LiteralValue(LiteralBits::Int("0".to_string())),
                "`timing_sequential_identity` pins Observed zero ns for this receipt"
            );

            let dim_name = fields
                .iter()
                .find(|f| f.label == "dimension_name")
                .map(|f| &f.value)
                .expect("dimension_name");
            assert_eq!(
                dim_name,
                &Value::LiteralValue(LiteralBits::String("ci_modeled_timing".to_string())),
                "dimension_name must match ci_modeled_timing_lens.name"
            );
        })
        .expect("spawn timing eval thread");
    timing_join.join().expect("timing eval thread panicked");

    // --- Cost + complexity + parallelism: same lens machinery the repo ships for user programs. ---
    let prog = cached_compile_to_dag("let x = 1 + 2", "lens_self_application_dim.v3");
    assert!(
        prog.diagnostics().is_empty(),
        "lens_self_application fixture diagnostics: {:?}",
        prog.diagnostics()
    );
    let root = bind_node_id_named(&prog, "x");

    let cost = analyze_symbolic_cost_dimension(&prog, root);
    let complexity = analyze_complexity(&prog, root);
    let (cost_composed, cost_name) = match &cost {
        DimensionReport::DimensionOk {
            composed,
            dimension_name,
            ..
        } => (composed, dimension_name.as_str()),
        other => panic!("expected symbolic-cost DimensionOk, got {other:?}"),
    };
    let (complexity_composed, complexity_name) = match &complexity {
        DimensionReport::DimensionOk {
            composed,
            dimension_name,
            ..
        } => (composed, dimension_name.as_str()),
        other => panic!("expected analyze_complexity DimensionOk, got {other:?}"),
    };
    assert_eq!(cost_name, "symbolic_cost");
    assert_eq!(complexity_name, "symbolic_cost");
    assert_eq!(
        cost_composed, complexity_composed,
        "analyze_complexity must delegate to the symbolic-cost dimension (single authority)"
    );

    let Behavior::Bind(bind_x) = prog.node(root) else {
        panic!("workflow root must be a Bind for this fixture");
    };
    assert!(
        matches!(
            complexity_of(&prog, &bind_x.result_port()),
            ComplexityLookup::Hit(_)
        ),
        "complexity lens read path must hit on the workflow bind result port"
    );

    let mut prog_parallel = prog.clone();
    let workflow = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![lens_self_app_read_op("read_user")],
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![lens_self_app_read_op("read_account")],
            }),
        ])
        .expect("two branches satisfy NonSingletonList"),
    };
    assert!(prog_parallel.try_register_lane2_workflow_effect(root, workflow));

    let par = analyze_parallelism(&prog_parallel, root);
    assert!(matches!(
        par,
        WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}
