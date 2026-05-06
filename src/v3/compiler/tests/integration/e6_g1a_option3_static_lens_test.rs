//! **Layer:** integration
//!
//! E6-G1.a Option 3 — static `Lens<Int>` consumer wiring (argument-opaque `Dag` / `Behavior`).
//!
//! Authority: `docs/briefs/r3-pr-e6-g1a-option3-static-lens-worker.md` + feasibility probe
//! (`docs/briefs/r3-pr-e6-g1a-option3-feasibility-probe.md`). Both are on-tree (landed
//! `gunbc#1853`); `include_str!` receipts at the end of this file bind the integration crate to
//! those paths (P1/P5 checkable receipt even when the PR diff is test-only).
//! Mechanism demonstration only; lens-over-`Dag` folding is deferred to `ReflectedProgram<T>` /
//! typed declaration-reference carrier work (Q-Reification).
//!
//! **Hard bars:** no `lens_apply`, `eval_substrate_reify`, or reflection-helper imports/calls.
//!
//! **Read-channel `Violates`:** `dimension_report_from_witness` still matches `Violates` so the
//! surface stays representative, but violation lifting into a declared `String` / `Behavior`
//! → `Diagnostic` path remains **deferred** (same #1853 scope bar as list monoid eval). The
//! `Violates` arm returns `DimensionFail` with **empty** `violations` / `witnesses` lists — a
//! fail-closed stub, not a claim that read-channel diagnostics are populated.
//!
//! **Witness list (Inhabits path):** `DimensionOk.witnesses` is built with the **List sum
//! variant constructors** `Cons { head: …, tail: … }` and `Empty`, not the std `cons` /
//! `singleton` **functions** (those arrows remain `Unparsed` in bootstrap and are not
//! body-evaluable here). That keeps #1844 / #1853 witness-flow acceptance: the same
//! `Inhabits(c)` payload from the read path feeds both `composed` and the report witness list.

use crate::common::{cached_compile_to_dag, find_list_empty_constructor_tag};
use v3_compiler::dag::{
    AtomPayload, Behavior, CardinalityBound, DeclarationId, LiteralBits, TransformTarget,
    TypeConnective, ValueNode,
};
use v3_compiler::diagnostics::SourceSpan;
use v3_compiler::evaluator::{
    evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, NamedField, Value,
};

const OPTION3_SOURCE: &str = r#"
import std.list { List, cons }
import std.substrate { Dag, Behavior, LoopBound, DagPort, Cluster, Declaration }
import v3.std.dimensions { Witness, OptionalDiagnostic, DimensionReport }
import v3.std.diagnostics { Diagnostic }
import v3.std.lens { Lens }

// Lowering seeds: each `Empty` below forces an `Instantiation` row for that `List<τ>.Empty` tag
// so the opaque-`Dag` harness can reuse stable `DeclarationId`s without mutating the `Dag`.
fn _e6_seed_list_declaration_empty() -> List<Declaration> = Empty
fn _e6_seed_list_behavior_empty() -> List<Behavior> = Empty
fn _e6_seed_list_dagport_empty() -> List<DagPort> = Empty
fn _e6_seed_list_cluster_empty() -> List<Cluster> = Empty

fn mini_read(d: Dag, b: Behavior) -> Witness<Int> = Inhabits(1)

fn int_add(a: Int, b: Int) -> Int = a + b
fn int_max(a: Int, b: Int) -> Int = if a > b then a else b
fn mini_iterate(c: Int, bound: LoopBound) -> Int = c
fn mini_validate(d: Dag, c: Int) -> OptionalDiagnostic = NoDiagnostic

fn empty_witness_int() -> List<Witness<Int>> = Empty
fn empty_diag_list() -> List<Diagnostic> = Empty

fn witnesses_inhabits(c: Int) -> List<Witness<Int>> =
  Cons { head: Inhabits(c), tail: Empty }

fn violations_singleton(diag: Diagnostic) -> List<Diagnostic> =
  cons(diag, empty_diag_list())

data mini_lens: Lens<Int> = {
  name: "mini_static",
  read: mini_read,
  sequential: { op: int_add, identity: 0 },
  branch: int_max,
  iterate: mini_iterate,
  validate: mini_validate
}

fn report_dim_ok(d: Dag, c: Int) -> DimensionReport<Int> =
  DimensionOk {
    dimension_name: mini_lens.name,
    composed: c,
    witnesses: witnesses_inhabits(c)
  }

fn report_dim_fail(d: Dag, c: Int, diag: Diagnostic) -> DimensionReport<Int> =
  DimensionFail {
    dimension_name: mini_lens.name,
    violations: violations_singleton(diag),
    witnesses: witnesses_inhabits(c)
  }

fn report_inhabits_branch(d: Dag, c: Int, od: OptionalDiagnostic) -> DimensionReport<Int> =
  match od {
    NoDiagnostic => report_dim_ok(d, c)
    SomeDiagnostic { value: diag } => report_dim_fail(d, c, diag)
  }

fn dimension_fail_closed() -> DimensionReport<Int> =
  DimensionFail {
    dimension_name: mini_lens.name,
    violations: empty_diag_list(),
    witnesses: empty_witness_int()
  }

fn dimension_report_from_witness(d: Dag, w: Witness<Int>) -> DimensionReport<Int> =
  match w {
    Inhabits(c) => report_inhabits_branch(d, c, mini_lens.validate(d, c))
    Violates { reason: _r, at: _beh } => dimension_fail_closed()
  }

fn mini_report(d: Dag, b: Behavior) -> DimensionReport<Int> =
  dimension_report_from_witness(d, mini_lens.read(d, b))
"#;

const OPTION3_FILE: &str = "e6_g1a_option3_static_lens.v3";

/// Split needles so this Rust source can be scanned later without self-matching fixture guards.
const GUARD_LENS_APPLY_MOD: &str = concat!("lens", "_apply", "::");
const GUARD_EVAL_SUBSTRATE_REIFY: &str = concat!("eval", "_substrate", "_reify");
const GUARD_REFLECT_BEHAVIOR: &str = concat!("reflect", "_behavior");

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

fn peel_to_optional_cardinality_decl(
    dag: &v3_compiler::dag::Dag,
    mut ty: DeclarationId,
) -> DeclarationId {
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

fn behavior_value_variant(dag: &v3_compiler::dag::Dag, v: &ValueNode) -> Value {
    let value_ctor = disj_variant_constructor_id(dag, "Behavior", "Value");
    let lane2 = optional_workflow_effect_none(dag);
    let span = SourceSpan::new(OPTION3_FILE, 0, 0);
    let inner = Value::RecordValue(vec![
        NamedField {
            label: "id".to_string(),
            value: Value::LiteralValue(LiteralBits::Int(0)),
        },
        NamedField {
            label: "payload".to_string(),
            value: Value::LiteralValue(v.data.clone()),
        },
        NamedField {
            label: "result_port".to_string(),
            value: Value::LiteralValue(LiteralBits::Int(i64::from(v.output.raw()))),
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
                    value: Value::LiteralValue(LiteralBits::Int(i64::from(span.byte_start))),
                },
                NamedField {
                    label: "end".to_string(),
                    value: Value::LiteralValue(LiteralBits::Int(i64::from(span.byte_end))),
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

fn sample_value_behavior(dag: &v3_compiler::dag::Dag) -> Behavior {
    dag.nodes()
        .iter()
        .find_map(|n| match n {
            Behavior::Value(v) => Some(Behavior::Value(v.clone())),
            _ => None,
        })
        .expect("fixture must contain at least one Value behavior for opaque Behavior harness")
}

/// Assemble a minimal substrate-shaped `Dag` [`Value::RecordValue`] using **existing**
/// `List<τ>.Empty` `Instantiation` rows from the same compile (see `_e6_seed_*` in the fixture).
fn opaque_dag_value(dag: &v3_compiler::dag::Dag) -> Value {
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

fn empty_list_value(dag: &v3_compiler::dag::Dag, list_ty: DeclarationId) -> Value {
    let tag = find_list_empty_constructor_tag(dag, list_ty);
    Value::VariantValue {
        tag,
        payload: Box::new(Value::RecordValue(vec![])),
    }
}

#[test]
fn e6_g1a_option3_static_lens_mini_report_executes_without_reflection_imports() {
    assert!(
        !OPTION3_SOURCE.contains(GUARD_LENS_APPLY_MOD),
        "fixture must not import the lens apply reflection path"
    );
    assert!(
        !OPTION3_SOURCE.contains(GUARD_EVAL_SUBSTRATE_REIFY),
        "fixture must not mention substrate reify helper"
    );
    assert!(
        !OPTION3_SOURCE.contains(GUARD_REFLECT_BEHAVIOR),
        "fixture must not mention reflect-behavior helper"
    );

    let dag = cached_compile_to_dag(OPTION3_SOURCE, OPTION3_FILE);
    assert!(
        dag.diagnostics().is_empty(),
        "option3 fixture must compile: {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let mini_read_id = dag.declaration_by_name("mini_read").expect("mini_read").id;
    let mini_validate_id = dag
        .declaration_by_name("mini_validate")
        .expect("mini_validate")
        .id;
    assert!(
        dag.nodes().iter().any(|n| {
            matches!(
                n,
                Behavior::Transform(t) if matches!(
                    &t.target,
                    TransformTarget::Callable(id)
                        if *id == mini_read_id || *id == mini_validate_id
                )
            )
        }),
        "mini_read / mini_validate must lower to TransformTarget::Callable"
    );

    let (d_port, b_port) = {
        let bind_node_id = bind_node_id_for_fn(&dag, "mini_report");
        let Behavior::Bind(bind) = dag.node(bind_node_id) else {
            panic!("mini_report bind");
        };
        assert_eq!(bind.params.len(), 2, "mini_report expects Dag and Behavior");
        (bind.params[0], bind.params[1])
    };

    let d_val = opaque_dag_value(&dag);
    let b_beh = sample_value_behavior(&dag);
    let b_val = match &b_beh {
        Behavior::Value(v) => behavior_value_variant(&dag, v),
        _ => unreachable!(),
    };

    let bind_node_id = bind_node_id_for_fn(&dag, "mini_report");
    let frame = EvalFrame::from_bindings([(d_port, d_val), (b_port, b_val)]).expect("frame");
    let mut state = EvalStateStack::with_root_frame(frame);
    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };
    let out = evaluate_body(&dag, bind_node_id, &mut state, strategy).expect("mini_report eval");

    let Value::VariantValue { tag, payload } = &out else {
        panic!("expected DimensionReport variant Value, got {out:?}");
    };
    let dim_ok = disj_variant_constructor_id(&dag, "DimensionReport", "DimensionOk");
    assert_eq!(
        *tag, dim_ok,
        "Inhabits path must produce DimensionOk; got non-Ok tag"
    );
    let Value::RecordValue(fields) = &**payload else {
        panic!("DimensionOk payload record");
    };
    let composed = fields
        .iter()
        .find(|f| f.label == "composed")
        .map(|f| &f.value)
        .expect("composed");
    assert_eq!(
        composed,
        &Value::LiteralValue(LiteralBits::Int(1)),
        "`composed` must carry the Inhabits witness payload `c` (= 1 from mini_read)"
    );
    let dim_name = fields
        .iter()
        .find(|f| f.label == "dimension_name")
        .map(|f| &f.value)
        .expect("dimension_name");
    assert_eq!(
        dim_name,
        &Value::LiteralValue(LiteralBits::String("mini_static".to_string())),
        "`dimension_name` must match the static `mini_lens` name field"
    );

    let witnesses = fields
        .iter()
        .find(|f| f.label == "witnesses")
        .map(|f| &f.value)
        .expect("witnesses");
    assert!(
        value_contains_int_literal(witnesses, 1),
        "witness list must carry the same `Inhabits(c)` payload as `composed` (c = 1 from mini_read)"
    );
}

fn value_contains_int_literal(v: &Value, needle: i64) -> bool {
    match v {
        Value::LiteralValue(LiteralBits::Int(i)) => *i == needle,
        Value::RecordValue(fs) => fs
            .iter()
            .any(|f| value_contains_int_literal(&f.value, needle)),
        Value::VariantValue { payload, .. } => value_contains_int_literal(payload, needle),
        _ => false,
    }
}

// INVARIANTS P1 / P5 — checkable receipt: this integration crate must not build if the cited
// briefs are missing from the worktree (the PR file list may omit `docs/` when unchanged).
const _: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../docs/briefs/r3-pr-e6-g1a-option3-static-lens-worker.md"
));
const _: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../docs/briefs/r3-pr-e6-g1a-option3-feasibility-probe.md"
));
