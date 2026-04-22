//! **Layer:** integration

use v3_compiler::compile_std_bootstrap_dag;

const ALGEBRA_DAG: &str = include_str!("../../../../../dsl/std/algebra.dag");
const FERMI_DAG: &str = include_str!("../../../../../dsl/std/fermi.dag");
const TERMINATION_DAG: &str = include_str!("../../../../../dsl/std/termination.dag");

#[test]
fn total_order_lattice_helpers_flow_through_shared_min_max_by() {
    let dag = compile_std_bootstrap_dag();

    assert!(
        dag.declaration_by_name("min_by").is_some(),
        "std bootstrap should export min_by"
    );
    assert!(
        dag.declaration_by_name("max_by").is_some(),
        "std bootstrap should export max_by"
    );
    assert!(
        ALGEBRA_DAG.contains("fn min_by<T>("),
        "algebra.dag should define min_by"
    );
    assert!(
        ALGEBRA_DAG.contains("fn max_by<T>("),
        "algebra.dag should define max_by"
    );
    assert!(
        FERMI_DAG.contains(
            "fn fermi_meet(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth {\n  min_by(a: lhs, b: rhs, project: fermi_ordinal)\n}"
        ),
        "fermi_meet should delegate through shared min_by"
    );
    assert!(
        FERMI_DAG.contains(
            "fn fermi_join(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth {\n  max_by(a: lhs, b: rhs, project: fermi_ordinal)\n}"
        ),
        "fermi_join should delegate through shared max_by"
    );
    assert!(
        TERMINATION_DAG.contains(
            "fn merge_evidence(a: DescentEvidence, b: DescentEvidence) -> DescentEvidence {\n  min_by(a: a, b: b, project: evidence_rank)\n}"
        ),
        "merge_evidence should delegate through shared min_by"
    );
    assert!(
        TERMINATION_DAG.contains(
            "fn join_evidence(a: DescentEvidence, b: DescentEvidence) -> DescentEvidence {\n  max_by(a: a, b: b, project: evidence_rank)\n}"
        ),
        "join_evidence should delegate through shared max_by"
    );
}
