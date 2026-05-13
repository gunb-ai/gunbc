//! **Layer:** integration
//!
//! Provenance `origin_of` seam check on `compile_to_dag` fixtures (Band-C v3-native
//! `COMPLETE` path). See retired `cementing_lens_registry_dispatch_test.rs` header for
//! rationale; the five-way `Behavior → Origin` mirror must stay in sync with
//! `src/v3/lenses/provenance.dag`.
//!
//! **Dual receipt with gate #87:** `tests/dag/t_r3_gate_87_cementing_regen_provenance.dag`
//! carries PB-B-1 enumeratable `LensOutputEquals` claims for the same glue programs. Those
//! projections intentionally assert only variant shape (Source vs Computed) because
//! `TestPredicate` cannot yet carry full `Origin` literals with `NodeId` equality. This module
//! remains the load-bearing payload mirror: `origin_of` must match the producer-derived
//! `Origin`, including carried `NodeId`, per review on PR #2894.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, PortId};
use v3_compiler::lens_provenance::{origin_of, Origin};

fn assert_origin_carriers_equal(a: &Origin, b: &Origin, context: &str) {
    match (a, b) {
        (Origin::NoProducer, Origin::NoProducer) => {}
        (Origin::MissingPort, Origin::MissingPort) => {}
        (Origin::MissingBehavior, Origin::MissingBehavior) => {}
        (Origin::Source { _0: x }, Origin::Source { _0: y }) => {
            assert_eq!(x, y, "{context}: Source NodeId mismatch")
        }
        (Origin::Computed { _0: x }, Origin::Computed { _0: y }) => {
            assert_eq!(x, y, "{context}: Computed NodeId mismatch")
        }
        (Origin::Selected { _0: x }, Origin::Selected { _0: y }) => {
            assert_eq!(x, y, "{context}: Selected NodeId mismatch")
        }
        (Origin::Accumulated { _0: x }, Origin::Accumulated { _0: y }) => {
            assert_eq!(x, y, "{context}: Accumulated NodeId mismatch")
        }
        _ => panic!("{context}: full Origin carrier mismatch\n  got: {a:?}\n  exp: {b:?}"),
    }
}

fn assert_provenance_origin_matches_lens_authority(dag: &Dag, bind_name: &str, context: &str) {
    let port = find_bind_value_port(dag, bind_name);
    let got = origin_of(dag, &port);
    let produced_by = dag
        .port_opt(&port)
        .unwrap_or_else(|| panic!("{context}: missing port for bind `{bind_name}`"))
        .produced_by
        .unwrap_or_else(|| {
            panic!("{context}: bind `{bind_name}` value port has no producer (expected one)")
        });
    let behavior = dag
        .nodes()
        .iter()
        .find(|b| b.id() == produced_by)
        .unwrap_or_else(|| panic!("{context}: missing producer node {produced_by:?}"));
    let expected = expected_origin_from_producer_behavior(behavior);
    assert_origin_carriers_equal(&got, &expected, context);
}

fn expected_origin_from_producer_behavior(behavior: &Behavior) -> Origin {
    match behavior {
        Behavior::Value(v) => Origin::Source { _0: v.id },
        Behavior::Transform(t) => Origin::Computed { _0: t.id },
        Behavior::Branch(b) => Origin::Selected { _0: b.id },
        Behavior::Loop(l) => Origin::Accumulated { _0: l.id },
        Behavior::Bind(bind) => Origin::Source { _0: bind.id },
    }
}

fn find_bind_value_port(dag: &Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

#[test]
fn provenance_origin_of_cements_complete_row_via_compile_to_dag_fixture() {
    let dag = compile_to_dag("let lit: Int = 7", "cementing_provenance_lit.v3").expect("compiles");
    assert_provenance_origin_matches_lens_authority(&dag, "lit", "cementing_provenance_lit");

    let dag =
        compile_to_dag("let sum: Int = 1 + 2", "cementing_provenance_sum.v3").expect("compiles");
    assert_provenance_origin_matches_lens_authority(&dag, "sum", "cementing_provenance_sum");
}
