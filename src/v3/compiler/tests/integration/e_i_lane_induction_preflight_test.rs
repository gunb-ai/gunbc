//! **Layer:** integration
//!
//! Lane E-I Step 0: prove `CostBound` with `List<CostBound>` parses, lowers, and
//! emits (v3 bootstrap is the authority; `regen_bootstrap` must succeed).

use v3_compiler::dag::TypeConnective;
use v3_compiler::Dag;

#[test]
fn e_i_lane_costbound_carries_list_of_costbound_in_bootstrap() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap dag should have no diagnostics, got {:?}",
        dag.diagnostics()
    );
    let cost = dag
        .declaration_by_name("CostBound")
        .expect("CostBound declaration");
    let TypeConnective::Disj { variants } = &cost.connective else {
        panic!("expected CostBound to be a sum, got {:?}", cost.connective);
    };
    let labels: Vec<&str> = variants.iter().map(|f| f.label.as_str()).collect();
    assert!(
        labels.contains(&"SumBound"),
        "CostBound should include SumBound {{ terms: List<CostBound> }}; got {labels:?}"
    );
    dag.declaration_by_name("sum_bound")
        .expect("sum_bound helper for List<CostBound>");
}
