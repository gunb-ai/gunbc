//! **Layer:** integration
//!
//! E6-G1.a — static top-level `Lens<C>` fold through the public body evaluator (Evaluator E3).
//!
//! Authority: `docs/briefs/r3-pr-e6-g1a-static-lens-fold-dispatch-packet.md` (slice boundary +
//! acceptance gates). User `data … : Dag = …` literals remain class-5 blocked; the ratchet
//! reifies a substrate `Dag` [`v3_compiler::evaluator::Value`] via
//! [`v3_compiler::lens_apply::reify_substrate_dag_value_for_eval`] (reflection spine parity with
//! `reflect_program_dag_nodes_in_file`).

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{ArrowBody, Behavior, TypeConnective};
use v3_compiler::evaluator::{
    evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, Value,
};
use v3_compiler::lens_apply::reify_substrate_dag_value_for_eval;

const G1A_SOURCE: &str = r#"
import std.list { empty, fold }
import std.substrate { Dag, Behavior, LoopBound }
import v3.std.dimensions { Witness, OptionalDiagnostic, DimensionReport }
import v3.std.lens { Lens }

fn lens_read(d: Dag, b: Behavior) -> Witness<Int> { Inhabits(1) }
fn int_add(a: Int, b: Int) -> Int = a + b
fn int_max(a: Int, b: Int) -> Int = if a > b then a else b
fn lens_iterate(c: Int, bound: LoopBound) -> Int = c
fn lens_validate(d: Dag, c: Int) -> OptionalDiagnostic { NoDiagnostic }

data tc1_static_lens: Lens<Int> = {
  name: "tc1_static",
  read: lens_read,
  sequential: { op: int_add, identity: 0 },
  branch: int_max,
  iterate: lens_iterate,
  validate: lens_validate
}

fn tc1_composed(d: Dag) -> Int =
  fold(
    d.nodes,
    tc1_static_lens.sequential.identity,
    |acc, b|
      match tc1_static_lens.read(d, b) {
        Inhabits(c) => int_add(acc, c)
        Violates { reason: r, at: beh } => acc
      }
  )

fn tc1_static_dimension_fold(d: Dag) -> DimensionReport<Int> =
  match tc1_static_lens.validate(d, tc1_composed(d)) {
    NoDiagnostic =>
      DimensionOk {
        dimension_name: tc1_static_lens.name,
        composed: tc1_composed(d),
        witnesses: empty()
      }
    SomeDiagnostic { value: _diag } =>
      DimensionFail {
        dimension_name: tc1_static_lens.name,
        violations: empty(),
        witnesses: empty()
      }
  }

fn tc1_static_dimension_fold_eta(d: Dag) -> DimensionReport<Int> =
  match tc1_static_lens.validate(d, 0 + tc1_composed(d)) {
    NoDiagnostic =>
      DimensionOk {
        dimension_name: tc1_static_lens.name,
        composed: 0 + tc1_composed(d),
        witnesses: empty()
      }
    SomeDiagnostic { value: _diag } =>
      DimensionFail {
        dimension_name: tc1_static_lens.name,
        violations: empty(),
        witnesses: empty()
      }
  }
"#;

const G1A_FILE: &str = "tc1_g1a_static_lens_fold.v3";

fn eval_dimension_report_fn(dag: &v3_compiler::dag::Dag, fn_name: &str, d_arg: Value) -> Value {
    let decl = dag
        .declaration_by_name(fn_name)
        .unwrap_or_else(|| panic!("missing `{fn_name}`"));
    let TypeConnective::Arrow { body, .. } = &decl.connective else {
        panic!("`{fn_name}` must be an arrow");
    };
    let ArrowBody::UserDefined(bind_id) = body else {
        panic!("`{fn_name}` must have a UserDefined body");
    };
    let bind_node_id = bind_id.node_id();
    let Behavior::Bind(bind) = dag.node(bind_node_id) else {
        panic!("bind node");
    };
    assert_eq!(bind.params.len(), 1, "`{fn_name}` expects one Dag parameter");
    let d_port = bind.params[0];
    let frame =
        EvalFrame::from_bindings([(d_port, d_arg)]).expect("frame");
    let mut state = EvalStateStack::with_root_frame(frame);
    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };
    evaluate_body(dag, bind_node_id, &mut state, strategy)
        .unwrap_or_else(|e| panic!("evaluate `{fn_name}`: {e:?}"))
}

#[test]
fn e6_g1a_static_lens_fold_executes_tc1_dimension_report_pair() {
    let dag = cached_compile_to_dag(G1A_SOURCE, G1A_FILE);
    assert!(
        dag.diagnostics().is_empty(),
        "G1.a fixture should compile cleanly: {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let d_value = reify_substrate_dag_value_for_eval(&dag, G1A_FILE)
        .expect("reify substrate Dag Value for evaluator");

    let left = eval_dimension_report_fn(&dag, "tc1_static_dimension_fold", d_value.clone());
    let right = eval_dimension_report_fn(&dag, "tc1_static_dimension_fold_eta", d_value);

    assert_eq!(
        left, right,
        "Pattern-A TC1 static pair: eta-expanded fold body must agree with direct fold"
    );
}
