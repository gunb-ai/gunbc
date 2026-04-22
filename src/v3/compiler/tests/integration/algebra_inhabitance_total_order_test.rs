//! **Layer:** integration

use v3_compiler::compile_std_bootstrap_dag;
use v3_compiler::dag::ArrowBody;

const ALGEBRA_DAG: &str = include_str!("../../../../../dsl/std/algebra.dag");
const FERMI_DAG: &str = include_str!("../../../../../dsl/std/fermi.dag");
const TERMINATION_DAG: &str = include_str!("../../../../../dsl/std/termination.dag");

fn source_for(file: &str) -> &'static str {
    match file {
        "dsl/std/algebra.dag" => ALGEBRA_DAG,
        "dsl/std/fermi.dag" => FERMI_DAG,
        "dsl/std/termination.dag" => TERMINATION_DAG,
        other => panic!("unexpected std source path `{other}`"),
    }
}

fn unparsed_body_text(dag: &v3_compiler::dag::Dag, name: &str) -> String {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("expected declaration `{name}` in std bootstrap DAG"));
    let ArrowBody::Unparsed(span) = &decl.connective.arrow_body().expect("arrow connective") else {
        panic!("expected `{name}` to retain an unparsed arrow body");
    };
    let source = source_for(&span.file);
    source[span.byte_start as usize..span.byte_end as usize].to_string()
}

trait ArrowConnectiveExt {
    fn arrow_body(&self) -> Option<&ArrowBody>;
}

impl ArrowConnectiveExt for v3_compiler::dag::TypeConnective {
    fn arrow_body(&self) -> Option<&ArrowBody> {
        match self {
            v3_compiler::dag::TypeConnective::Arrow { body, .. } => Some(body),
            _ => None,
        }
    }
}

#[test]
fn total_order_lattice_helpers_flow_through_shared_min_max_by() {
    let dag = compile_std_bootstrap_dag();

    assert!(dag.declaration_by_name("min_by").is_some(), "std bootstrap should export min_by");
    assert!(dag.declaration_by_name("max_by").is_some(), "std bootstrap should export max_by");

    let fermi_meet = unparsed_body_text(&dag, "fermi_meet");
    let fermi_join = unparsed_body_text(&dag, "fermi_join");
    let merge_evidence = unparsed_body_text(&dag, "merge_evidence");
    let join_evidence = unparsed_body_text(&dag, "join_evidence");

    assert!(
        fermi_meet.contains("min_by("),
        "fermi_meet should delegate through shared min_by, got: {fermi_meet}"
    );
    assert!(
        fermi_join.contains("max_by("),
        "fermi_join should delegate through shared max_by, got: {fermi_join}"
    );
    assert!(
        merge_evidence.contains("min_by("),
        "merge_evidence should delegate through shared min_by, got: {merge_evidence}"
    );
    assert!(
        join_evidence.contains("max_by("),
        "join_evidence should delegate through shared max_by, got: {join_evidence}"
    );
}
