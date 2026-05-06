//! **Layer:** integration
//!
//! E6-G1.a Option 3 — static `Lens<Int>` consumer wiring (argument-opaque `Dag` / `Behavior`).
//!
//! Authority: `docs/briefs/r3-pr-e6-g1a-option3-static-lens-worker.md` + feasibility probe.
//! This is a **mechanism demonstration only**; lens-over-`Dag` folding stays deferred to
//! `ReflectedProgram<T>` / typed declaration-reference carrier work (Q-Reification).
//!
//! **Hard bars (test-enforced):** no `lens_apply`, `eval_substrate_reify`, or reflection helper
//! imports; no compiled-program reification — opaque argument `Value`s are supplied manually.

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{
    AtomPayload, Behavior, DeclarationId, LiteralBits, TypeConnective, ValueNode,
};
use v3_compiler::evaluator::{
    evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, NamedField,
    Value,
};
use v3_compiler::diagnostics::SourceSpan;

const OPTION3_SOURCE: &str = r#"
import std.list { List, empty, cons }
import std.substrate { Dag, Behavior, LoopBound, DagPort, Cluster }
import v3.std.dimensions { Witness, OptionalDiagnostic, DimensionReport }
import v3.std.diagnostics { Diagnostic }
import v3.std.lens { Lens }

fn empty_dec() -> List<Declaration> = empty()
fn empty_beh() -> List<Behavior> = empty()
fn empty_ports() -> List<DagPort> = empty()
fn empty_clusters() -> List<Cluster> = empty()

fn opaque_dag() -> Dag =
  { declarations: empty_dec(), nodes: empty_beh(), ports: empty_ports(), clusters: empty_clusters() }

fn mini_read(d: Dag, b: Behavior) -> Witness<Int> = Inhabits(1)

fn int_add(a: Int, b: Int) -> Int = a + b
fn int_max(a: Int, b: Int) -> Int = if a > b then a else b
fn mini_iterate(c: Int, bound: LoopBound) -> Int = c
fn mini_validate(d: Dag, c: Int) -> OptionalDiagnostic = NoDiagnostic

data mini_lens: Lens<Int> = {
  name: "mini_static",
  read: mini_read,
  sequential: { op: int_add, identity: 0 },
  branch: int_max,
  iterate: mini_iterate,
  validate: mini_validate
}

fn mini_report(d: Dag, b: Behavior) -> DimensionReport<Int> =
  match mini_lens.read(d, b) {
    Inhabits(c) =>
      match mini_lens.validate(d, c) {
        NoDiagnostic =>
          DimensionOk {
            dimension_name: mini_lens.name,
            composed: c,
            witnesses: cons(Inhabits(c), empty())
          }
        SomeDiagnostic { value: diag } =>
          DimensionFail {
            dimension_name: mini_lens.name,
            violations: cons(diag, empty()),
            witnesses: cons(Inhabits(c), empty())
          }
      }
    Violates { reason: _r, at: _beh } =>
      mini_report(d, b)
  }
"#;

const OPTION3_FILE: &str = "e6_g1a_option3_static_lens.v3";

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

fn eval_nullary_fn(dag: &v3_compiler::dag::Dag, fn_name: &str) -> Value {
    let bind_node_id = bind_node_id_for_fn(dag, fn_name);
    let Behavior::Bind(bind) = dag.node(bind_node_id) else {
        panic!("bind");
    };
    assert!(
        bind.params.is_empty(),
        "`{fn_name}` must be nullary for this harness"
    );
    let frame = EvalFrame::empty();
    let mut state = EvalStateStack::with_root_frame(frame);
    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };
    evaluate_body(dag, bind_node_id, &mut state, strategy)
        .unwrap_or_else(|e| panic!("eval {fn_name}: {e:?}"))
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

fn conj_field_ty(dag: &v3_compiler::dag::Dag, conj_decl_id: DeclarationId, label: &str) -> DeclarationId {
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
            TypeConnective::Cardinality(p) if p.bound() == v3_compiler::dag::CardinalityBound::AtMostOne => {
                return p.element();
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
    let lane2_card = peel_to_optional_cardinality_decl(dag, conj_field_ty(dag, vn, "lane2_workflow"));
    let decl = dag.declaration(lane2_card);
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
            value: Value::LiteralValue(LiteralBits::Int(i64::from(v.id.raw()))),
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

#[test]
fn e6_g1a_option3_static_lens_mini_report_executes_without_reflection_imports() {
    const SOURCE: &str = concat!(
        include_str!("e6_g1a_option3_static_lens_test.rs"),
        "\n// sentinel for grep guards — actual program is OPTION3_SOURCE const above\n"
    );
    assert!(
        !SOURCE.contains("lens_apply::"),
        "guard: implementation must not mention lens_apply::"
    );
    assert!(
        !SOURCE.contains("eval_substrate_reify"),
        "guard: must not import eval_substrate_reify"
    );
    assert!(
        !SOURCE.contains("reflect_behavior"),
        "guard: must not reference reflect_behavior"
    );

    let dag = cached_compile_to_dag(OPTION3_SOURCE, OPTION3_FILE);
    assert!(
        dag.diagnostics().is_empty(),
        "option3 fixture must compile: {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let mini_read_id = dag
        .declaration_by_name("mini_read")
        .expect("mini_read")
        .id;
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
                    v3_compiler::dag::TransformTarget::Callable(id)
                        if *id == mini_read_id || *id == mini_validate_id
                )
            )
        }),
        "mini_read / mini_validate must lower to TransformTarget::Callable"
    );

    let d_port = {
        let bind_node_id = bind_node_id_for_fn(&dag, "mini_report");
        let Behavior::Bind(bind) = dag.node(bind_node_id) else {
            panic!("mini_report bind");
        };
        assert_eq!(bind.params.len(), 2, "mini_report expects Dag and Behavior");
        bind.params[0]
    };
    let b_port = {
        let bind_node_id = bind_node_id_for_fn(&dag, "mini_report");
        let Behavior::Bind(bind) = dag.node(bind_node_id) else {
            panic!("mini_report bind");
        };
        bind.params[1]
    };

    let d_val = eval_nullary_fn(&dag, "opaque_dag");
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
        "`dimension_name` must come from FieldProject on mini_lens.name"
    );
}
