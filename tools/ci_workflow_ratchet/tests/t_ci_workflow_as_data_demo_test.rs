//! **Layer:** integration
//!
//! PR-A relocation — gate-57 / lens_gate57 compile_to_dag receipts (ctrl#1467 follow-up).
//! **T-Workflow-As-Data** — `ci_workflow_modeled_as_dag` receipt (gunbc#1956): gunbc CI workflow
//! carriers from `dsl/extdeps/github/actions.dag` are authored as `.dag` **data** alongside a
//! `Lens<TimingMeasurement>` reporting shell. On the **gate-57 linked `gunbc.ci` compile** (see
//! `GUNBC_CI_LINKED_COMPILE_*`) we pin **structural** `ci_workflow_dag` receipts plus a **fail-closed
//! eval regression** (`BadTransformOperands` on `demo_ci_modeled_timing_dimension_report` until the
//! linked bundle agrees with eager eval). The **successful** `evaluate_body` receipt for that demo
//! lives on the PB-1 bootstrap shell (`ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell`).
//!
//! **T-Lens-Self-Application — gate `recursive_flex_demonstration_landed` (#59):** **CONSUMER_LANDED +
//! PASSING** via `recursive_flex_demonstration_landed`. Full `compile_to_dag` on `ci_emission.dag` is
//! **M1(2.8) user-range blocked** today (`data` bodies cannot apply calls; binary-shim `fn` uses a
//! block-bodied `Workflow` record literal — see `lower.rs` opaque scaffold rejection). The receipt
//! instead **pins the BinaryShim-only `WorkflowRuntime` surface** in `dsl/gunbc/ci_emission.dag`
//! after dissolution of the interim hand-sync carrier.
//!
//! **R3 gate #57** (`lens_self_application_demonstrated`, T-Lens-Self-Application): the same module
//! hosts the executable receipt: **`compile_to_dag` on `dsl/gunbc/ci.dag`** (see `GUNBC_CI_LINKED_COMPILE_*`
//! in this file) once (via `OnceLock`) to load
//! structural `ci_workflow_dag` (authority row, pipeline name, prerequisite edges, parallel fan-out),
//! paired symbolic-cost + E7 complexity on the lowered lane-2 subject, prerequisite **graph** fan-out
//! read from that carrier (no hand-staged `WorkflowEffect`), absence of a lowered lane-2 workflow
//! projection until the compiler owns it (P2), and a **timing-lens eval regression pin** on that
//! linked artifact (`assert_linked_carrier_demo_ci_modeled_timing_dimension_report_eval_blocked`).
//! The successful DimensionOk receipt for the same demo runs on the PB-1 bootstrap shell via
//! `ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell`.
//! Symbolic-cost and E7 complexity must both return `DimensionOk` where asserted (fail-closed).
//!
//! Runtime `Dag` binding is an **opaque substrate-shaped record** (empty `declarations` /
//! `nodes` / `ports` / `clusters` lists built from existing `List<τ>.Empty` tags — see
//! `_ci_wad_seed_*` in `t_ci_workflow_as_data_demo.dag`). The eager evaluator cannot execute
//! arbitrary bootstrap `Transform` nodes, so this harness does **not** embed
//! `reflect_behavior_list(full_bootstrap.nodes)` even though `behavior_spine` / `is_empty` are
//! live on the carrier type.
//!
//! **INVARIANTS P5 — checkable receipt:** this harness is still counted by SG-0's
//! `EXPECTED_HAND_AUTHORED_TEST` census and `sg0_v3_test_hand_authored_subratchet` fails on
//! unaccounted hand-Rust test drift. `.github/workflows/ci.yml` is the sole live Actions transport;
//! remaining hand-Rust test surface migrates to `.dag` `TestClaim` data per `sg0_census_test.rs`
//! R1C-E notes.

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use ci_workflow_ratchet::support::find_list_empty_constructor_tag;
use v3_compiler::dag::{
    AtomPayload, Behavior, DeclarationId, FieldValue, LiteralBits, Lookup, NodeId, TypeConnective,
    ValueNode,
};
use v3_compiler::evaluator::{
    evaluate_body, EvalError, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder,
    NamedField, Value, BAD_TRANSFORM_CALLABLE_TARGET_NOT_ARROW_REASON,
};
use v3_compiler::gunbc_ci::{
    select_affected_gates, select_affected_gates_for_binary_shim, CiBinaryShimAffectedSetReceipt,
    CiGateMeta, CiWorkflowDagInput, CiWorkflowDiff,
};
use v3_compiler::lens_cost::complexity_of;
use v3_compiler::{
    analyze_complexity, analyze_symbolic_cost_dimension, compile_to_dag,
    generated_full_bootstrap_dag, DimensionReport,
};

const DEMO_SPAN_FILE: &str = "src/v3/std/t_ci_workflow_as_data_demo.dag";
const GUNBC_CI_LINKED_COMPILE_SOURCE: &str = include_str!("../../../dsl/gunbc/ci.dag");
const GUNBC_CI_LINKED_COMPILE_FILE: &str = "dsl/gunbc/ci.dag";
const GUNBC_CI_EMISSION_SOURCE: &str = include_str!("../../../dsl/gunbc/ci_emission.dag");
const GUNBC_CI_EMISSION_FILE: &str = "dsl/gunbc/ci_emission.dag";

fn demo_bootstrap_dag() -> v3_compiler::dag::Dag {
    generated_full_bootstrap_dag()
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

fn eval_demo_ci_modeled_timing_dimension_report(
    dag: &v3_compiler::dag::Dag,
) -> Result<Value, EvalError> {
    assert!(
        dag.diagnostics().is_empty(),
        "dag diagnostics: {:?}",
        dag.diagnostics()
    );

    let (d_port, b_port) = {
        let bind_node_id = bind_node_id_for_fn(dag, "demo_ci_modeled_timing_dimension_report");
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

    let d_val = bootstrap_dag_runtime_carrier(dag);
    let b_beh = sample_demo_value_behavior(dag);
    let b_val = match &b_beh {
        Behavior::Value(v) => behavior_value_variant(dag, v),
        _ => unreachable!(),
    };

    let bind_node_id = bind_node_id_for_fn(dag, "demo_ci_modeled_timing_dimension_report");
    let frame = EvalFrame::from_bindings([(d_port, d_val), (b_port, b_val)]).expect("frame");
    let mut state = EvalStateStack::with_root_frame(frame);
    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };
    evaluate_body(dag, bind_node_id, &mut state, strategy)
}

/// Linked `gunbc.ci` + workflow carrier: `evaluate_body(demo_ci_modeled_timing_dimension_report, …)`
/// must remain blocked on the known eager-eval gap until appendix lowering/infer reconciles the bundle.
fn assert_linked_carrier_demo_ci_modeled_timing_dimension_report_eval_blocked(
    dag: &v3_compiler::dag::Dag,
) {
    match eval_demo_ci_modeled_timing_dimension_report(dag) {
        Err(EvalError::BadTransformOperands { reason })
            if reason == BAD_TRANSFORM_CALLABLE_TARGET_NOT_ARROW_REASON => {}
        other => panic!(
            "linked gunbc.ci timing-lens eval: expected BadTransformOperands(Callable target …); \
             if this flips to Ok, migrate success assertions onto `merged` and shrink this pin — got {other:?}"
        ),
    }
}

/// `evaluate_body` **success** receipt for `demo_ci_modeled_timing_dimension_report` on `dag`
/// (PB-1 bootstrap shell today; linked carrier still fails — see
/// `assert_linked_carrier_demo_ci_modeled_timing_dimension_report_eval_blocked`).
fn assert_demo_ci_modeled_timing_dimension_report_eval_on_dag(dag: &v3_compiler::dag::Dag) {
    let out = eval_demo_ci_modeled_timing_dimension_report(dag).expect("eval");

    let Value::VariantValue { tag, payload } = &out else {
        panic!("expected DimensionReport variant Value, got {out:?}");
    };
    let dim_ok = disj_variant_constructor_id(dag, "DimensionReport", "DimensionOk");
    assert_eq!(*tag, dim_ok, "expected DimensionOk");

    let Value::RecordValue(fields) = &**payload else {
        panic!("DimensionOk payload record");
    };
    let composed = fields
        .iter()
        .find(|f| f.label == "composed")
        .map(|f| &f.value)
        .expect("composed");

    let observed_tag = disj_variant_constructor_id(dag, "TimingMeasurement", "Observed");
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

fn literal_bool(value: &FieldValue) -> bool {
    let FieldValue::Literal(LiteralBits::Bool(value)) = value else {
        panic!("expected bool literal field, got {value:?}");
    };
    *value
}

fn ci_workflow_dag_input_from_compiled_ci(dag: &v3_compiler::dag::Dag) -> CiWorkflowDagInput {
    let fields = structural_value_body(dag, "ci_workflow_dag");
    let gate_records = workflow_gate_records(dag, fields);
    let gates: Vec<CiGateMeta> = gate_records
        .iter()
        .map(|gate| CiGateMeta {
            id: literal_string(structural_field(gate, "id")).to_string(),
            blocking: literal_bool(structural_field(gate, "blocking")),
        })
        .collect();
    let (_name, _node_ids, edge_pairs) = workflow_topology(dag, fields);
    let edges: Vec<(String, String)> = edge_pairs
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    CiWorkflowDagInput { gates, edges }
}

struct Gate57CiArtifacts {
    dag: v3_compiler::dag::Dag,
    input: CiWorkflowDagInput,
    subject: NodeId,
}

fn gate57_ci_artifacts() -> &'static Gate57CiArtifacts {
    static CACHE: OnceLock<Gate57CiArtifacts> = OnceLock::new();
    CACHE.get_or_init(|| {
        let dag = compile_to_dag(GUNBC_CI_LINKED_COMPILE_SOURCE, GUNBC_CI_LINKED_COMPILE_FILE)
            .unwrap_or_else(|err| panic!("compile {GUNBC_CI_LINKED_COMPILE_FILE}: {err:?}"));
        assert!(
            dag.diagnostics().is_empty(),
            "{GUNBC_CI_LINKED_COMPILE_FILE}: {:?}",
            dag.diagnostics()
        );
        let input = ci_workflow_dag_input_from_compiled_ci(&dag);
        let subject = dag.workflow_lane2_subject().expect(
            "compiled gunbc.ci must expose a workflow lane-2 subject bind for lens consumers",
        );
        Gate57CiArtifacts {
            dag,
            input,
            subject,
        }
    })
}

fn gate57_bootstrap_dag() -> &'static v3_compiler::dag::Dag {
    static BOOT: OnceLock<v3_compiler::dag::Dag> = OnceLock::new();
    BOOT.get_or_init(demo_bootstrap_dag)
}

/// Same lowered artifact as [`gate57_ci_artifacts`]: compiled `gunbc.ci` on the embedded bootstrap
/// DAG, which already includes `v3.std.t_ci_workflow_as_data_demo` for
/// `evaluate_body(demo_ci_modeled_timing_dimension_report, …)`.
fn gate57_ci_timing_lens_carrier_dag() -> &'static v3_compiler::dag::Dag {
    &gate57_ci_artifacts().dag
}

/// Floor prerequisite chain: `dsl-compile-clean` → `fmt` → `affected-tests`.
fn assert_ci_floor_prereq_chain(input: &CiWorkflowDagInput) {
    let edges: Vec<_> = input
        .edges
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    assert_eq!(
        edges,
        vec![
            ("dsl-compile-clean", "fmt"),
            ("fmt", "affected-tests"),
        ],
        "blocking floor must be a linear prerequisite chain"
    );
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
        "DslCompileCleanCommand",
        "FmtCommand",
        "AffectedTestsCommand",
    ] {
        if *constructor == disj_variant_constructor_id(dag, sum_name, label) {
            return label;
        }
    }
    panic!("unexpected {sum_name} constructor {constructor:?}");
}

fn disj_variant_labels(dag: &v3_compiler::dag::Dag, sum_name: &str) -> Vec<String> {
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
                return variants.iter().map(|v| v.label.clone()).collect();
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                decl_id = *next;
            }
            _ => panic!("unexpected connective while resolving `{sum_name}`"),
        }
    }
    panic!("peel depth exceeded resolving `{sum_name}`");
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
    let ci = compile_to_dag(GUNBC_CI_LINKED_COMPILE_SOURCE, GUNBC_CI_LINKED_COMPILE_FILE)
        .unwrap_or_else(|err| panic!("compile {GUNBC_CI_LINKED_COMPILE_FILE}: {err:?}"));
    let fields = structural_value_body(&ci, "ci_workflow_dag");
    let (name, node_ids, edges) = workflow_topology(&ci, fields);

    assert_eq!(
        name, "gunbc-ci",
        "modeled workflow DAG name must derive from the CI pipeline"
    );

    assert_eq!(
        node_ids,
        vec![
            "dsl-compile-clean",
            "fmt",
            "affected-tests",
        ],
        "CI workflow DAG must carry one node per structural gate"
    );

    assert_eq!(
        edges,
        vec![
            ("dsl-compile-clean", "fmt"),
            ("fmt", "affected-tests"),
        ],
        "CI workflow dependencies must be modeled as provider-neutral DAG edges"
    );

    // Edge endpoints are structural CIGate references; `workflow_topology`
    // projects ids only for readable assertions above.
}

#[test]
fn ci_workflow_as_data_demo_pins_interim_command_shape() {
    let ci = compile_to_dag(GUNBC_CI_LINKED_COMPILE_SOURCE, GUNBC_CI_LINKED_COMPILE_FILE)
        .unwrap_or_else(|err| panic!("compile {GUNBC_CI_LINKED_COMPILE_FILE}: {err:?}"));
    let fields = structural_value_body(&ci, "ci_workflow_dag");
    let gate_records = workflow_gate_records(&ci, fields);

    let mut commands = gate_records
        .iter()
        .map(|gate| {
            let id = literal_string(structural_field(gate, "id"));
            let command = structural_field(gate, "command");
            let label = variant_label(&ci, "CICommand", command);
            (id, label)
        })
        .collect::<Vec<_>>();
    commands.sort_by_key(|(id, ..)| *id);

    assert_eq!(
        commands,
        vec![
            ("affected-tests", "AffectedTestsCommand"),
            ("dsl-compile-clean", "DslCompileCleanCommand"),
            ("fmt", "FmtCommand"),
        ],
        "CICommand must keep impossible field combinations out of authored gate data"
    );
}

#[test]
fn ci_workflow_as_data_demo_uses_only_gunbc_ci_authority_topology() {
    let demo = demo_bootstrap_dag();
    let ci = compile_to_dag(GUNBC_CI_LINKED_COMPILE_SOURCE, GUNBC_CI_LINKED_COMPILE_FILE)
        .unwrap_or_else(|err| panic!("compile {GUNBC_CI_LINKED_COMPILE_FILE}: {err:?}"));

    assert!(
        demo.declaration_by_name("modeled_gunbc_ci_workflow_dag")
            .is_none(),
        "bootstrap demo must not carry a mirror of ci_workflow_dag"
    );
    assert_eq!(
        workflow_topology(&ci, structural_value_body(&ci, "ci_workflow_dag")),
        (
            "gunbc-ci",
            vec![
                "dsl-compile-clean",
                "fmt",
                "affected-tests",
            ],
            vec![
                ("dsl-compile-clean", "fmt"),
                ("fmt", "affected-tests"),
            ],
        ),
        "dsl/gunbc/ci.dag must remain the single CI DAG topology authority"
    );
}

/// **R3 gate #59** — recursive-flex / BinaryShim-only emission surface after hand-sync carrier
/// dissolution. Full `compile_to_dag(gunbc.ci_emission)` remains M1(2.8)-blocked on user range;
/// `ci_emission.dag` still carries the authoritative **source** binding ratcheted below.
#[test]
fn recursive_flex_demonstration_landed() {
    let ci_fields = structural_value_body(&gate57_ci_artifacts().dag, "ci_workflow_dag");
    assert!(
        !ci_fields
            .iter()
            .any(|(label, _)| label == "github_actions_workflow"),
        "gate #59: `CIWorkflowDag` must not carry a hand-synced Actions workflow carrier"
    );
    assert!(
        GUNBC_CI_EMISSION_SOURCE.contains("type WorkflowRuntime = | BinaryShim")
            || GUNBC_CI_EMISSION_SOURCE.contains("type WorkflowRuntime = BinaryShim"),
        "{GUNBC_CI_EMISSION_FILE} must keep BinaryShim as the sole WorkflowRuntime until T-24 YamlStatic emission"
    );
    assert!(
        GUNBC_CI_EMISSION_SOURCE.contains("BinaryShim => gunbc_ci_emission_binary_shim_workflow(dag)"),
        "{GUNBC_CI_EMISSION_FILE} must keep the BinaryShim projection arm of `project_github_actions`"
    );
    assert!(
        !GUNBC_CI_EMISSION_SOURCE.contains("gunbc_ci_yml_workflow"),
        "{GUNBC_CI_EMISSION_FILE} must not pin a YamlStatic hand-sync projection binding"
    );
}

#[test]
fn gunbc_ci_emission_substrate_contract_is_present() {
    assert!(
        GUNBC_CI_EMISSION_SOURCE.contains(
            "fn project_github_actions(dag: CIWorkflowDag, runtime: WorkflowRuntime) -> Workflow"
        ),
        "{GUNBC_CI_EMISSION_FILE} must declare the T-WAD projection-function contract"
    );
    assert!(
        GUNBC_CI_EMISSION_SOURCE
            .contains("fn gunbc_ci_emission_binary_shim_workflow(dag: CIWorkflowDag) -> Workflow"),
        "{GUNBC_CI_EMISSION_FILE} must declare the BinaryShim workflow projection"
    );
}

#[test]
fn workflow_runtime_initial_enum_matches_t_wad_gate_99() {
    let runtime_line = GUNBC_CI_EMISSION_SOURCE
        .lines()
        .find(|line| line.starts_with("type WorkflowRuntime ="))
        .expect("WorkflowRuntime type declaration");
    let dag = compile_to_dag(
        runtime_line,
        "dsl/gunbc/ci_emission.workflow_runtime.slice.dag",
    )
    .unwrap_or_else(|err| panic!("compile WorkflowRuntime slice: {err:?}"));
    assert!(
        dag.diagnostics().is_empty(),
        "WorkflowRuntime slice diagnostics: {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        disj_variant_labels(&dag, "WorkflowRuntime"),
        ["BinaryShim"],
        "T-WAD gate #99 WorkflowRuntime surface is BinaryShim-only until T-24 YamlStatic emission; \
         PythonShim and InlineGunbc remain design-only until their substrate-prereq PRs land"
    );
}

// --- R3 gate #57 (`lens_self_application_demonstrated`) — split receipts per TESTING.md §4.
// `cargo test lens_self_application_demonstrated` matches the shared name prefix.

#[test]
fn lens_self_application_demonstrated_ci_workflow_dag_authority_row_lowers() {
    let g = gate57_ci_artifacts();
    g.dag.declaration_by_name("ci_workflow_dag").expect(
        "compiled gunbc.ci must surface `ci_workflow_dag` as the CI workflow-as-data authority row",
    );
    let fields = structural_value_body(&g.dag, "ci_workflow_dag");
    assert!(
        !fields.is_empty(),
        "`ci_workflow_dag` must lower with a non-empty structural body (pipeline + edges)"
    );
}

#[test]
fn lens_self_application_demonstrated_bootstrap_ci_modeled_exclusivity() {
    let boot = gate57_bootstrap_dag();
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
}

#[test]
fn lens_self_application_demonstrated_ci_pipeline_name_is_gunbc_ci() {
    let g = gate57_ci_artifacts();
    let fields = structural_value_body(&g.dag, "ci_workflow_dag");
    let (pipe_name, _, _) = workflow_topology(&g.dag, fields);
    assert_eq!(
        pipe_name, "gunbc-ci",
        "`ci_workflow_dag.pipeline.name` must remain the gunbc-ci authority string"
    );
}

#[test]
fn lens_self_application_demonstrated_ci_touch_all_affected_gates_order() {
    let g = gate57_ci_artifacts();
    let affected = select_affected_gates(&g.input, &CiWorkflowDiff::TouchAll)
        .expect("affected-set selection must succeed on gunbc-ci topology");
    assert_eq!(
        affected.len(),
        3,
        "TouchAll must schedule the full gunbc-ci gate roster"
    );
    for id in [
        "dsl-compile-clean",
        "fmt",
        "affected-tests",
    ] {
        assert!(
            affected.iter().any(|g| g == id),
            "TouchAll roster must include `{id}`"
        );
    }
    let fmt_pos = affected.iter().position(|g| g == "fmt").expect("fmt");
    let dsl_pos = affected
        .iter()
        .position(|g| g == "dsl-compile-clean")
        .expect("dsl-compile-clean");
    let affected_pos = affected.iter().position(|g| g == "affected-tests").expect("affected-tests");
    assert!(dsl_pos < fmt_pos, "dsl-compile-clean must precede fmt in topo order");
    assert!(fmt_pos < affected_pos, "fmt must precede affected-tests in topo order");
}

// --- R3 gate #103 (`ci_uses_affected_set_selection`), Layer 1: gate-id receipt → `select_affected_gates`.
// Canvas §7 / Slice-5 runner wiring of PR #2713 `NodeRef` receipts is explicitly out of scope here.

#[test]
fn ci_uses_affected_set_selection_binary_shim_narrow_on_gunbc_ci_topology() {
    let g = gate57_ci_artifacts();
    let receipt = CiBinaryShimAffectedSetReceipt {
        narrowing_available: true,
        proven_direct_gate_touches: BTreeSet::from([String::from("fmt")]),
    };
    let plan = select_affected_gates_for_binary_shim(&g.input, &receipt)
        .expect("binary shim selection must succeed on gunbc-ci topology");
    let touch = select_affected_gates(
        &g.input,
        &CiWorkflowDiff::TouchedGates(BTreeSet::from([String::from("fmt")])),
    )
    .expect("baseline touched-gates selection");
    assert_eq!(plan, touch);
}

#[test]
fn ci_uses_affected_set_selection_binary_shim_unknown_receipt_full_roster() {
    let g = gate57_ci_artifacts();
    let receipt = CiBinaryShimAffectedSetReceipt {
        narrowing_available: false,
        proven_direct_gate_touches: BTreeSet::from([String::from("fmt")]),
    };
    let plan = select_affected_gates_for_binary_shim(&g.input, &receipt)
        .expect("binary shim selection must succeed on gunbc-ci topology");
    let full =
        select_affected_gates(&g.input, &CiWorkflowDiff::TouchAll).expect("TouchAll baseline");
    assert_eq!(plan, full);
}

// Inlined from deleted `scripts/workflow-path-regex-forbidden-substrings.txt` (operator 2026-06-01).
const WORKFLOW_PATH_REGEX_FORBIDDEN_SUBSTRINGS: &str = r#"
git diff --name-only
needs.changes.outputs
grep -vE '^(docs/.*|[^/]+\.md)$'
"#;

fn workflow_path_regex_forbidden_substrings() -> impl Iterator<Item = &'static str> {
    WORKFLOW_PATH_REGEX_FORBIDDEN_SUBSTRINGS
        .lines()
        .filter_map(|line| {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                None
            } else {
                Some(t)
            }
        })
}

// Scans tracked `.github/workflows/*.{yml,yaml}` via `git ls-files` (same enumeration as
// `check-workflow-path-regex-inventory.sh`), not `read_dir` (avoids untracked / non-repo files).
#[test]
fn workflow_no_path_regex_policy_ci_yml() {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let repo_root = repo_root
        .canonicalize()
        .unwrap_or_else(|e| panic!("canonicalize repo root {}: {e}", repo_root.display()));

    let output = Command::new("git")
        .current_dir(&repo_root)
        .args([
            "ls-files",
            "-z",
            ".github/workflows/*.yml",
            ".github/workflows/*.yaml",
        ])
        .output()
        .unwrap_or_else(|e| panic!("spawn git ls-files: {e}"));
    assert!(
        output.status.success(),
        "git ls-files failed (cwd={}): stderr={}",
        repo_root.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut scanned = 0usize;
    for rel in String::from_utf8_lossy(&output.stdout).split('\0') {
        if rel.is_empty() {
            continue;
        }
        let path = repo_root.join(rel);
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        scanned += 1;
        for forbidden in workflow_path_regex_forbidden_substrings() {
            assert!(
                !raw.contains(forbidden),
                "{} must not contain Layer-2 selection fingerprint `{forbidden}` (gate ci_uses_affected_set_selection; inlined forbidden-substrings roster)",
                path.display()
            );
        }
    }
    assert!(
        scanned > 0,
        "git ls-files must report at least one tracked workflow under .github/workflows/"
    );
}

#[test]
fn lens_self_application_demonstrated_ci_prereq_fanout_from_carrier() {
    let g = gate57_ci_artifacts();
    assert_ci_floor_prereq_chain(&g.input);
}

#[test]
fn lens_self_application_demonstrated_ci_lane2_workflow_projection_absent() {
    let g = gate57_ci_artifacts();
    assert!(
        g.dag.lane2_workflow_effect_at(&g.subject).is_none(),
        "lane-2 workflow projection must come from lowering when available — do not inject a parallel \
         `WorkflowEffect` mirror from Rust (P2 single authority)"
    );
}

/// R3 gate #57 — primary lens receipt: paired symbolic-cost + E7 `DimensionOk` on the CI lane-2 subject.
#[test]
fn lens_self_application_demonstrated() {
    let g = gate57_ci_artifacts();
    let cost_ci = analyze_symbolic_cost_dimension(&g.dag, g.subject);
    let complexity_ci = analyze_complexity(&g.dag, g.subject);
    let (
        DimensionReport::DimensionOk {
            dimension_name: a,
            composed: ca,
            ..
        },
        DimensionReport::DimensionOk {
            dimension_name: b,
            composed: cb,
            ..
        },
    ) = (&cost_ci, &complexity_ci)
    else {
        panic!(
            "gate #57 requires `DimensionOk` from both `analyze_symbolic_cost_dimension` and \
             `analyze_complexity` on the CI lane-2 subject (fail-closed); got cost={cost_ci:?} \
             complexity={complexity_ci:?}"
        );
    };
    assert_eq!(a, b);
    assert_eq!(ca, cb);
    assert_eq!(a.as_str(), "symbolic_cost");

    let Behavior::Bind(bind_ci) = g.dag.node(g.subject) else {
        panic!("lane-2 subject must remain a Bind shell");
    };
    let cx = complexity_of(&g.dag, &bind_ci.result_port());
    let Lookup::Hit(ref summary) = cx else {
        panic!(
            "gate #57 requires `complexity_of` Hit on the CI lane-2 bind result_port; got {cx:?}"
        );
    };
    assert_eq!(
        &summary.work, ca,
        "`complexity_of`.work must match `analyze_symbolic_cost_dimension` composed (single-authority with E7 complexity DimensionOk)"
    );
}

#[test]
fn lens_self_application_demonstrated_timing_dimension_report_on_ci_modeled_workflow() {
    let handle = std::thread::Builder::new()
        .name(
            "lens_self_application_demonstrated_timing_dimension_report_on_ci_modeled_workflow"
                .into(),
        )
        .stack_size(32 * 1024 * 1024)
        .spawn(
            lens_self_application_demonstrated_timing_dimension_report_on_ci_modeled_workflow_body,
        )
        .expect("spawn timing body");
    handle
        .join()
        .expect("lens_self_application_demonstrated CI-modeled timing thread panicked");
}

fn lens_self_application_demonstrated_timing_dimension_report_on_ci_modeled_workflow_body() {
    let g = gate57_ci_artifacts();
    let merged = gate57_ci_timing_lens_carrier_dag();
    let merged_input = ci_workflow_dag_input_from_compiled_ci(merged);
    assert_eq!(
        merged_input, g.input,
        "merged timing-lens carrier fixture must preserve `dsl/gunbc/ci.dag` `ci_workflow_dag` topology \
         (single authority — timing receipt is not scoped to a parallel CI mirror)"
    );
    merged
        .workflow_lane2_subject()
        .expect("merged gunbc.ci surface must expose the lane-2 CI workflow bind shell");
    assert_linked_carrier_demo_ci_modeled_timing_dimension_report_eval_blocked(merged);
}

/// T-Workflow-As-Data / PB-1 — bootstrap embed path for `demo_ci_modeled_timing_dimension_report`
/// (orthogonal to gate #57’s merged-carrier timing receipt).
#[test]
fn ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell() {
    let handle = std::thread::Builder::new()
        .name("ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell_body)
        .expect("spawn timing body");
    handle
        .join()
        .expect("ci_workflow_as_data_demo bootstrap timing thread panicked");
}

fn ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell_body() {
    assert_demo_ci_modeled_timing_dimension_report_eval_on_dag(gate57_bootstrap_dag());
}
