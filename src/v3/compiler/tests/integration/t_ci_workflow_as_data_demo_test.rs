//! **Layer:** integration
//!
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
//! instead **pins the YamlStatic projection lemma in lowering:** on linked `gunbc.ci` (already
//! loaded for gate #57), `ci_workflow_dag.github_actions_workflow` is structurally identical to
//! `gunbc_ci_github_actions_workflow`, matching the **`YamlStatic => dag.github_actions_workflow`**
//! arm of `project_github_actions` in `dsl/gunbc/ci_emission.dag`, plus a source-text ratchet on the
//! pinned `data gunbc_ci_yml_workflow` binding.
//!
//! **R3 gate #57** (`lens_self_application_demonstrated`, T-Lens-Self-Application): the same module
//! hosts the executable receipt: **`compile_to_dag` on a linked bundle** (`ci_github_actions_workflow.dag`
//! then `ci.dag` — see `GUNBC_CI_LINKED_COMPILE_*` in this file) once (via `OnceLock`) to load
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
//! unaccounted hand-Rust test drift. The added CI carrier projector is a bounded drift receipt for
//! the current hand-authored `.github/workflows/ci.yml` authority: `gunbc_ci_github_actions_workflow_*`
//! recomputes the source hash and structurally compares parsed YAML to the pinned `Workflow` data.
//! Dissolve-on: T-24 `src/v4/workflow/ci.dag` projection emits the Actions workflow and deletes
//! `dsl/gunbc/ci_github_actions_workflow.dag`; remaining hand-Rust test surface migrates to `.dag`
//! `TestClaim` data per `sg0_census_test.rs` R1C-E notes.

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use crate::common::find_list_empty_constructor_tag;
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use sha2::{Digest, Sha256};
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
    generated_full_bootstrap_dag, CompileError, DimensionReport,
};

const DEMO_SPAN_FILE: &str = "src/v3/std/t_ci_workflow_as_data_demo.dag";
/// [`compile_to_dag`] loads a single surface module — imports do not pull sibling files from
/// disk. Merge the pinned GitHub Actions workflow module before `gunbc.ci` so
/// `ci_workflow_dag.github_actions_workflow` resolves to `gunbc_ci_github_actions_workflow`.
const GUNBC_CI_LINKED_COMPILE_SOURCE: &str = concat!(
    include_str!("../../../../../dsl/gunbc/ci_github_actions_workflow.dag"),
    "\n\n",
    include_str!("../../../../../dsl/gunbc/ci.dag"),
);
const GUNBC_CI_LINKED_COMPILE_FILE: &str = "dsl/gunbc/ci_with_github_actions_workflow.dag";
const GUNBC_CI_GITHUB_WORKFLOW_SOURCE: &str =
    include_str!("../../../../../dsl/gunbc/ci_github_actions_workflow.dag");
const GUNBC_CI_GITHUB_WORKFLOW_FILE: &str = "dsl/gunbc/ci_github_actions_workflow.dag";
const GITHUB_ACTIONS_CI_YML_SOURCE: &str = include_str!("../../../../../.github/workflows/ci.yml");
const GUNBC_CI_EMISSION_SOURCE: &str = include_str!("../../../../../dsl/gunbc/ci_emission.dag");
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

/// Same lowered artifact as [`gate57_ci_artifacts`]: linked `gunbc.ci` (`ci_github_actions_workflow`
/// + `ci.dag`) on the embedded bootstrap DAG, which already includes `v3.std.t_ci_workflow_as_data_demo`
///   for `evaluate_body(demo_ci_modeled_timing_dimension_report, …)`.
fn gate57_ci_timing_lens_carrier_dag() -> &'static v3_compiler::dag::Dag {
    &gate57_ci_artifacts().dag
}

/// Single structural claim: `ci_workflow_dag` prerequisite edges expose exactly one 2-successor
/// fan-out, and it matches gunbc-ci’s `compile-gates` → `lint` + `tests` fork (no `WorkflowEffect`
/// staging — read-only mirror of the lowered `data ci_workflow_dag` carrier).
fn assert_ci_prereq_graph_has_single_parallel_fanout(input: &CiWorkflowDagInput) {
    let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
    for (from, to) in &input.edges {
        outgoing.entry(from.clone()).or_default().push(to.clone());
    }

    let mut two_branch: Vec<(String, Vec<String>)> = Vec::new();
    for (from, mut kids) in outgoing {
        kids.sort();
        kids.dedup();
        if kids.len() == 2 {
            two_branch.push((from, kids));
        }
    }
    two_branch.sort_by(|a, b| a.0.cmp(&b.0));
    match two_branch.as_slice() {
        [(parent, kids)] => {
            assert_eq!(parent.as_str(), "compile-gates");
            assert_eq!(kids, &vec!["lint".to_string(), "tests".to_string()]);
        }
        [] => panic!(
            "ci_workflow_dag must expose exactly one 2-branch prerequisite fan-out (parallel pair encoding); found none"
        ),
        found => panic!(
            "ci_workflow_dag must expose exactly one 2-branch prerequisite fan-out (parallel pair encoding); found {found:?}"
        ),
    }
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

fn yaml_key(key: &str) -> YamlValue {
    YamlValue::String(key.to_string())
}

fn yaml_string(value: &str) -> YamlValue {
    YamlValue::String(value.to_string())
}

fn insert_yaml(map: &mut YamlMapping, key: &str, value: YamlValue) {
    map.insert(yaml_key(key), value);
}

fn insert_yaml_if_some(map: &mut YamlMapping, key: &str, value: Option<YamlValue>) {
    if let Some(value) = value {
        insert_yaml(map, key, value);
    }
}

fn literal_int_yaml(value: &FieldValue) -> YamlValue {
    let FieldValue::Literal(LiteralBits::Int(value)) = value else {
        panic!("expected int literal field, got {value:?}");
    };
    serde_yaml::to_value(
        value
            .parse::<i64>()
            .unwrap_or_else(|err| panic!("invalid int literal `{value}`: {err}")),
    )
    .expect("integer serializes")
}

fn literal_yaml(value: &FieldValue) -> YamlValue {
    match value {
        FieldValue::Literal(LiteralBits::String(value)) => yaml_string(value),
        FieldValue::Literal(LiteralBits::Bool(value)) => YamlValue::Bool(*value),
        FieldValue::Literal(LiteralBits::Int(_)) => literal_int_yaml(value),
        other => panic!("expected scalar literal field, got {other:?}"),
    }
}

fn optional_payload<'a>(
    dag: &v3_compiler::dag::Dag,
    value: &'a FieldValue,
) -> Option<&'a FieldValue> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected optional variant field, got {value:?}");
    };
    let constructor_name = dag
        .declaration(*constructor)
        .name
        .as_deref()
        .unwrap_or("<anonymous>");
    match (constructor_name, payload.as_slice()) {
        ("None", []) => None,
        ("Some", [FieldValue::Record(fields)]) => Some(structural_field(fields, "value")),
        ("Some", [single]) => Some(single),
        _ if payload.is_empty() => None,
        _ if payload.len() == 1 => Some(&payload[0]),
        _ => panic!("unexpected optional payload `{constructor_name}`: {payload:?}"),
    }
}

fn optional_yaml(
    dag: &v3_compiler::dag::Dag,
    value: &FieldValue,
    project: impl FnOnce(&FieldValue) -> YamlValue,
) -> Option<YamlValue> {
    optional_payload(dag, value).map(project)
}

fn list_yaml(items: &[FieldValue], project: impl Fn(&FieldValue) -> YamlValue) -> YamlValue {
    YamlValue::Sequence(items.iter().map(project).collect())
}

fn string_list_yaml(items: &[FieldValue]) -> YamlValue {
    list_yaml(items, |item| yaml_string(literal_string(item)))
}

fn map_string_yaml(map: &v3_compiler::dag::FieldMap) -> YamlValue {
    let mut out = YamlMapping::new();
    for (key, value) in map.entries() {
        out.insert(yaml_key(key), literal_yaml(value));
    }
    YamlValue::Mapping(out)
}

fn action_with_scalar_yaml(map: &v3_compiler::dag::FieldMap) -> YamlValue {
    let mut out = YamlMapping::new();
    for (key, value) in map.entries() {
        let value = match value {
            FieldValue::Literal(LiteralBits::String(value)) if value == "true" => {
                YamlValue::Bool(true)
            }
            FieldValue::Literal(LiteralBits::String(value)) if value == "false" => {
                YamlValue::Bool(false)
            }
            FieldValue::Literal(LiteralBits::String(value)) => value
                .parse::<i64>()
                .map(|number| serde_yaml::to_value(number).expect("integer serializes"))
                .unwrap_or_else(|_| yaml_string(value)),
            other => literal_yaml(other),
        };
        out.insert(yaml_key(key), value);
    }
    YamlValue::Mapping(out)
}

fn direct_or_optional_map<'a>(
    dag: &v3_compiler::dag::Dag,
    value: &'a FieldValue,
) -> Option<&'a v3_compiler::dag::FieldMap> {
    match value {
        FieldValue::Map(map) if map.entries().is_empty() => None,
        FieldValue::Map(map) => Some(map),
        _ => optional_payload(dag, value).and_then(|payload| match payload {
            FieldValue::Map(map) if map.entries().is_empty() => None,
            FieldValue::Map(map) => Some(map),
            other => panic!("expected map payload, got {other:?}"),
        }),
    }
}

fn carrier_action_ref_yaml(fields: &[(String, FieldValue)]) -> YamlValue {
    let owner = literal_string(structural_field(fields, "owner"));
    let repo = literal_string(structural_field(fields, "repo"));
    let reference = literal_string(structural_field(fields, "ref"));
    yaml_string(&format!("{owner}/{repo}@{reference}"))
}

fn carrier_optional_value_yaml(
    dag: &v3_compiler::dag::Dag,
    value: &FieldValue,
) -> Option<YamlValue> {
    optional_yaml(dag, value, literal_yaml)
}

fn assert_elided_default_bash_shell(dag: &v3_compiler::dag::Dag, value: &FieldValue) {
    let FieldValue::Variant { constructor, .. } = value else {
        panic!("expected shell variant, got {value:?}");
    };
    assert_eq!(
        *constructor,
        disj_variant_constructor_id(dag, "ShellType", "Bash"),
        "RunStep.shell is elided from YAML only for the default Linux bash shell"
    );
}

fn carrier_runner_yaml(dag: &v3_compiler::dag::Dag, value: &FieldValue) -> YamlValue {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected runner variant, got {value:?}");
    };
    if *constructor == disj_variant_constructor_id(dag, "RunnerSpec", "SelfHosted") {
        return string_list_yaml(structural_list(&payload[0]));
    }
    if *constructor == disj_variant_constructor_id(dag, "RunnerSpec", "HostedRunner") {
        let label = &payload[0];
        if let FieldValue::Variant { constructor, .. } = label {
            if *constructor == disj_variant_constructor_id(dag, "RunnerLabel", "UbuntuLatest") {
                return yaml_string("ubuntu-latest");
            }
        }
    }
    if *constructor == disj_variant_constructor_id(dag, "RunnerSpec", "RunsOnExpression") {
        return yaml_string(literal_string(&payload[0]));
    }
    panic!("unsupported RunnerSpec payload {value:?}");
}

fn carrier_cancel_in_progress_yaml(dag: &v3_compiler::dag::Dag, value: &FieldValue) -> YamlValue {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected cancel-in-progress variant, got {value:?}");
    };
    if *constructor
        == disj_variant_constructor_id(dag, "CancelInProgressSpec", "CancelInProgressBool")
    {
        return literal_yaml(&payload[0]);
    }
    if *constructor
        == disj_variant_constructor_id(dag, "CancelInProgressSpec", "CancelInProgressExpression")
    {
        return yaml_string(literal_string(&payload[0]));
    }
    if *constructor
        == disj_variant_constructor_id(
            dag,
            "CancelInProgressWhenQueueMax",
            "QueueMaxCancelInProgressFalse",
        )
    {
        return YamlValue::Bool(false);
    }
    if *constructor
        == disj_variant_constructor_id(
            dag,
            "CancelInProgressWhenQueueMax",
            "QueueMaxCancelInProgressExpression",
        )
    {
        return yaml_string(literal_string(&payload[0]));
    }
    panic!("unsupported cancel-in-progress payload {value:?}");
}

fn carrier_concurrency_yaml(dag: &v3_compiler::dag::Dag, value: &FieldValue) -> YamlValue {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected concurrency variant, got {value:?}");
    };
    if *constructor == disj_variant_constructor_id(dag, "ConcurrencySpec", "ConcurrencyScalar") {
        return yaml_string(literal_string(&payload[0]));
    }
    let mut map = YamlMapping::new();
    insert_yaml(&mut map, "group", literal_yaml(&payload[0]));
    insert_yaml_if_some(
        &mut map,
        "cancel-in-progress",
        optional_yaml(dag, &payload[1], |payload| {
            carrier_cancel_in_progress_yaml(dag, payload)
        }),
    );
    if *constructor
        == disj_variant_constructor_id(dag, "ConcurrencySpec", "ConcurrencyMappingQueueMax")
    {
        insert_yaml(&mut map, "queue", yaml_string("max"));
    } else {
        insert_yaml_if_some(
            &mut map,
            "queue",
            optional_yaml(dag, &payload[2], |_| yaml_string("single")),
        );
    }
    YamlValue::Mapping(map)
}

fn carrier_step_yaml(dag: &v3_compiler::dag::Dag, value: &FieldValue) -> YamlValue {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected step variant, got {value:?}");
    };
    let mut map = YamlMapping::new();
    insert_yaml_if_some(
        &mut map,
        "name",
        carrier_optional_value_yaml(dag, &payload[0]),
    );
    insert_yaml_if_some(
        &mut map,
        "id",
        carrier_optional_value_yaml(dag, &payload[1]),
    );
    if *constructor == disj_variant_constructor_id(dag, "Step", "UsesStep") {
        insert_yaml(
            &mut map,
            "uses",
            carrier_action_ref_yaml(structural_record(&payload[2])),
        );
        if let Some(with) = direct_or_optional_map(dag, &payload[3]) {
            insert_yaml(&mut map, "with", action_with_scalar_yaml(with));
        }
        insert_yaml_if_some(
            &mut map,
            "env",
            optional_yaml(dag, &payload[4], |payload| match payload {
                FieldValue::Map(map) => map_string_yaml(map),
                other => panic!("expected env map, got {other:?}"),
            }),
        );
        insert_yaml_if_some(
            &mut map,
            "if",
            carrier_optional_value_yaml(dag, &payload[5]),
        );
        if literal_bool(&payload[6]) {
            insert_yaml(&mut map, "continue-on-error", YamlValue::Bool(true));
        }
        insert_yaml_if_some(
            &mut map,
            "timeout-minutes",
            optional_yaml(dag, &payload[7], literal_int_yaml),
        );
    } else if *constructor == disj_variant_constructor_id(dag, "Step", "RunStep") {
        insert_yaml(&mut map, "run", literal_yaml(&payload[2]));
        assert_elided_default_bash_shell(dag, &payload[3]);
        insert_yaml_if_some(
            &mut map,
            "working-directory",
            carrier_optional_value_yaml(dag, &payload[5]),
        );
        insert_yaml_if_some(
            &mut map,
            "env",
            optional_yaml(dag, &payload[4], |payload| match payload {
                FieldValue::Map(map) => map_string_yaml(map),
                other => panic!("expected env map, got {other:?}"),
            }),
        );
        insert_yaml_if_some(
            &mut map,
            "if",
            carrier_optional_value_yaml(dag, &payload[6]),
        );
        if literal_bool(&payload[7]) {
            insert_yaml(&mut map, "continue-on-error", YamlValue::Bool(true));
        }
        insert_yaml_if_some(
            &mut map,
            "timeout-minutes",
            optional_yaml(dag, &payload[8], literal_int_yaml),
        );
    } else {
        panic!("unsupported Step payload {value:?}");
    }
    YamlValue::Mapping(map)
}

fn carrier_job_yaml(dag: &v3_compiler::dag::Dag, fields: &[(String, FieldValue)]) -> YamlValue {
    let mut map = YamlMapping::new();
    insert_yaml_if_some(
        &mut map,
        "name",
        carrier_optional_value_yaml(dag, structural_field(fields, "name")),
    );
    insert_yaml(
        &mut map,
        "runs-on",
        carrier_runner_yaml(dag, structural_field(fields, "runner")),
    );
    let needs = structural_list(structural_field(fields, "needs"));
    if !needs.is_empty() {
        insert_yaml(&mut map, "needs", string_list_yaml(needs));
    }
    insert_yaml_if_some(
        &mut map,
        "env",
        optional_yaml(
            dag,
            structural_field(fields, "env"),
            |payload| match payload {
                FieldValue::Map(map) => map_string_yaml(map),
                other => panic!("expected job env map, got {other:?}"),
            },
        ),
    );
    insert_yaml_if_some(
        &mut map,
        "outputs",
        optional_yaml(
            dag,
            structural_field(fields, "outputs"),
            |payload| match payload {
                FieldValue::Map(map) => map_string_yaml(map),
                other => panic!("expected outputs map, got {other:?}"),
            },
        ),
    );
    insert_yaml_if_some(
        &mut map,
        "if",
        carrier_optional_value_yaml(dag, structural_field(fields, "if_condition")),
    );
    insert_yaml_if_some(
        &mut map,
        "timeout-minutes",
        optional_yaml(
            dag,
            structural_field(fields, "timeout_minutes"),
            literal_int_yaml,
        ),
    );
    if literal_bool(structural_field(fields, "continue_on_error")) {
        insert_yaml(&mut map, "continue-on-error", YamlValue::Bool(true));
    }
    insert_yaml_if_some(
        &mut map,
        "concurrency",
        optional_yaml(dag, structural_field(fields, "concurrency"), |payload| {
            carrier_concurrency_yaml(dag, payload)
        }),
    );
    insert_yaml(
        &mut map,
        "steps",
        list_yaml(structural_list(structural_field(fields, "steps")), |step| {
            carrier_step_yaml(dag, step)
        }),
    );
    YamlValue::Mapping(map)
}

fn carrier_trigger_yaml(dag: &v3_compiler::dag::Dag, triggers: &[FieldValue]) -> YamlValue {
    let mut map = YamlMapping::new();
    for trigger in triggers {
        let FieldValue::Variant {
            constructor,
            payload,
        } = trigger
        else {
            panic!("expected workflow trigger variant, got {trigger:?}");
        };
        if *constructor == disj_variant_constructor_id(dag, "WorkflowTrigger", "Push") {
            let mut push = YamlMapping::new();
            insert_yaml(
                &mut push,
                "branches",
                string_list_yaml(structural_list(&payload[0])),
            );
            let paths = structural_list(&payload[1]);
            if !paths.is_empty() {
                insert_yaml(&mut push, "paths", string_list_yaml(paths));
            }
            insert_yaml(&mut map, "push", YamlValue::Mapping(push));
        } else if *constructor == disj_variant_constructor_id(dag, "WorkflowTrigger", "PullRequest")
        {
            let mut pull_request = YamlMapping::new();
            insert_yaml(
                &mut pull_request,
                "branches",
                string_list_yaml(structural_list(&payload[0])),
            );
            let types = structural_list(&payload[1])
                .iter()
                .map(|activity| {
                    let FieldValue::Variant { constructor, .. } = activity else {
                        panic!("expected pull_request activity variant, got {activity:?}");
                    };
                    if *constructor
                        == disj_variant_constructor_id(dag, "PullRequestActivity", "Opened")
                    {
                        yaml_string("opened")
                    } else if *constructor
                        == disj_variant_constructor_id(dag, "PullRequestActivity", "Synchronize")
                    {
                        yaml_string("synchronize")
                    } else if *constructor
                        == disj_variant_constructor_id(dag, "PullRequestActivity", "Reopened")
                    {
                        yaml_string("reopened")
                    } else if *constructor
                        == disj_variant_constructor_id(dag, "PullRequestActivity", "ReadyForReview")
                    {
                        yaml_string("ready_for_review")
                    } else if *constructor
                        == disj_variant_constructor_id(dag, "PullRequestActivity", "Closed")
                    {
                        yaml_string("closed")
                    } else {
                        panic!("unsupported pull_request activity {activity:?}");
                    }
                })
                .collect();
            insert_yaml(&mut pull_request, "types", YamlValue::Sequence(types));
            insert_yaml(&mut map, "pull_request", YamlValue::Mapping(pull_request));
        } else {
            panic!("unsupported workflow trigger {trigger:?}");
        }
    }
    YamlValue::Mapping(map)
}

fn carrier_permissions_yaml(
    dag: &v3_compiler::dag::Dag,
    fields: &[(String, FieldValue)],
) -> YamlValue {
    let mut map = YamlMapping::new();
    for (field, yaml_field) in [
        ("contents", "contents"),
        ("pull_requests", "pull-requests"),
        ("issues", "issues"),
        ("actions", "actions"),
    ] {
        let value = structural_field(fields, field);
        let FieldValue::Variant { constructor, .. } = value else {
            panic!("expected permission variant, got {value:?}");
        };
        if *constructor == disj_variant_constructor_id(dag, "PermissionLevel", "PermRead") {
            insert_yaml(&mut map, yaml_field, yaml_string("read"));
        } else if *constructor == disj_variant_constructor_id(dag, "PermissionLevel", "PermWrite") {
            insert_yaml(&mut map, yaml_field, yaml_string("write"));
        } else if *constructor == disj_variant_constructor_id(dag, "PermissionLevel", "PermNone") {
            // Omitted in YAML: GitHub's unspecified permissions are modeled as PermNone here.
        } else {
            panic!("unsupported PermissionLevel {value:?}");
        }
    }
    YamlValue::Mapping(map)
}

fn carrier_workflow_yaml(
    dag: &v3_compiler::dag::Dag,
    fields: &[(String, FieldValue)],
) -> YamlValue {
    let mut map = YamlMapping::new();
    insert_yaml(
        &mut map,
        "name",
        literal_yaml(structural_field(fields, "name")),
    );
    insert_yaml(
        &mut map,
        "on",
        carrier_trigger_yaml(dag, structural_list(structural_field(fields, "on"))),
    );
    insert_yaml_if_some(
        &mut map,
        "permissions",
        optional_yaml(dag, structural_field(fields, "permissions"), |payload| {
            carrier_permissions_yaml(dag, structural_record(payload))
        }),
    );
    insert_yaml_if_some(
        &mut map,
        "concurrency",
        optional_yaml(dag, structural_field(fields, "concurrency"), |payload| {
            carrier_concurrency_yaml(dag, payload)
        }),
    );
    insert_yaml_if_some(
        &mut map,
        "env",
        optional_yaml(
            dag,
            structural_field(fields, "env"),
            |payload| match payload {
                FieldValue::Map(map) => map_string_yaml(map),
                other => panic!("expected workflow env map, got {other:?}"),
            },
        ),
    );
    let mut jobs = YamlMapping::new();
    for job in structural_list(structural_field(fields, "jobs")) {
        let fields = structural_record(job);
        let id = literal_string(structural_field(fields, "id"));
        jobs.insert(yaml_key(id), carrier_job_yaml(dag, fields));
    }
    insert_yaml(&mut map, "jobs", YamlValue::Mapping(jobs));
    YamlValue::Mapping(map)
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

#[test]
fn gunbc_ci_github_actions_workflow_pins_ci_yml_source_checksum() {
    let actual = Sha256::digest(GITHUB_ACTIONS_CI_YML_SOURCE.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let expected_prefix = "// Source-SHA256(.github/workflows/ci.yml): ";
    let expected = GUNBC_CI_GITHUB_WORKFLOW_SOURCE
        .lines()
        .find_map(|line| line.strip_prefix(expected_prefix))
        .expect("ci_github_actions_workflow.dag must pin the source ci.yml checksum");
    assert_eq!(
        actual, expected,
        "update {GUNBC_CI_GITHUB_WORKFLOW_FILE}'s Source-SHA256 when .github/workflows/ci.yml changes"
    );
}

#[test]
fn gunbc_ci_github_actions_workflow_matches_ci_yml_structure() {
    let dag = compile_to_dag(
        GUNBC_CI_GITHUB_WORKFLOW_SOURCE,
        GUNBC_CI_GITHUB_WORKFLOW_FILE,
    )
    .unwrap_or_else(|err| panic!("compile {GUNBC_CI_GITHUB_WORKFLOW_FILE}: {err:?}"));
    assert!(
        dag.diagnostics().is_empty(),
        "{GUNBC_CI_GITHUB_WORKFLOW_FILE}: {:?}",
        dag.diagnostics()
    );
    let modeled = carrier_workflow_yaml(
        &dag,
        structural_value_body(&dag, "gunbc_ci_github_actions_workflow"),
    );
    let parsed: YamlValue = serde_yaml::from_str(GITHUB_ACTIONS_CI_YML_SOURCE)
        .expect(".github/workflows/ci.yml parses");
    assert_eq!(
        modeled, parsed,
        "{GUNBC_CI_GITHUB_WORKFLOW_FILE} must structurally match .github/workflows/ci.yml"
    );
}

/// **R3 gate #59** — recursive-flex / YamlStatic emit-back lemma: `project_github_actions`'s
/// YamlStatic arm is `dag.github_actions_workflow`, which is exactly the carrier wired on
/// `ci_workflow_dag` (structural equality in the linked `gunbc.ci` compile). Full
/// `compile_to_dag(gunbc.ci_emission)` remains M1(2.8)-blocked on user range; `ci_emission.dag` still
/// carries the authoritative **source** binding ratcheted below.
#[test]
fn recursive_flex_demonstration_landed() {
    let g = gate57_ci_artifacts();
    let ci_fields = structural_value_body(&g.dag, "ci_workflow_dag");
    let carrier_wf = structural_field(ci_fields, "github_actions_workflow");
    let carrier_fields = structural_record_ref(&g.dag, carrier_wf);

    let pinned_decl = g
        .dag
        .declaration_by_name("gunbc_ci_github_actions_workflow")
        .expect("linked gunbc.ci must surface `gunbc_ci_github_actions_workflow`");
    let Some(v3_compiler::dag::ValueBody::Structural {
        fields: pinned_fields,
    }) = pinned_decl.value_body.as_ref()
    else {
        panic!(
            "gunbc_ci_github_actions_workflow must lower as structural data, got {:?}",
            pinned_decl.value_body
        );
    };

    assert_eq!(
        pinned_fields.as_slice(),
        carrier_fields,
        "gate #59: CI authority row must pin the generated GitHub Actions `Workflow` bytes as \
         `ci_workflow_dag.github_actions_workflow` (YamlStatic projection arm of `project_github_actions`)"
    );

    assert!(
        GUNBC_CI_EMISSION_SOURCE.contains("YamlStatic => dag.github_actions_workflow"),
        "{GUNBC_CI_EMISSION_FILE} must keep the YamlStatic forward arm on `dag.github_actions_workflow`"
    );
    assert!(
        GUNBC_CI_EMISSION_SOURCE.contains(
            "data gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)"
        ),
        "{GUNBC_CI_EMISSION_FILE} must keep the pinned `gunbc_ci_yml_workflow` projection binding"
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
            .contains("data gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)"),
        "{GUNBC_CI_EMISSION_FILE} must pin the YamlStatic projection binding"
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
        ["YamlStatic", "BinaryShim"],
        "T-WAD gate #99 initial WorkflowRuntime surface must stay paired to real consumers; \
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
        "`ci_workflow_dag` must lower with a non-empty structural body (pipeline + edges + transport fields)"
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
        affected,
        vec![
            "compile-gates".to_string(),
            "lint".to_string(),
            "tests".to_string(),
            "l1-ratchet".to_string(),
        ],
        "TouchAll must schedule the full gunbc-ci gate roster in prerequisite topo order"
    );
}

// --- R3 gate #103 (`ci_uses_affected_set_selection`), Layer 1: gate-id receipt → `select_affected_gates`.
// Canvas §7 / Slice-5 runner wiring of PR #2713 `NodeRef` receipts is explicitly out of scope here.

#[test]
fn ci_uses_affected_set_selection_binary_shim_narrow_on_gunbc_ci_topology() {
    let g = gate57_ci_artifacts();
    let receipt = CiBinaryShimAffectedSetReceipt {
        narrowing_available: true,
        proven_direct_gate_touches: BTreeSet::from([String::from("tests")]),
    };
    let plan = select_affected_gates_for_binary_shim(&g.input, &receipt)
        .expect("binary shim selection must succeed on gunbc-ci topology");
    let touch = select_affected_gates(
        &g.input,
        &CiWorkflowDiff::TouchedGates(BTreeSet::from([String::from("tests")])),
    )
    .expect("baseline touched-gates selection");
    assert_eq!(plan, touch);
}

#[test]
fn ci_uses_affected_set_selection_binary_shim_unknown_receipt_full_roster() {
    let g = gate57_ci_artifacts();
    let receipt = CiBinaryShimAffectedSetReceipt {
        narrowing_available: false,
        proven_direct_gate_touches: BTreeSet::from([String::from("tests")]),
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
    assert_ci_prereq_graph_has_single_parallel_fanout(&g.input);
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
