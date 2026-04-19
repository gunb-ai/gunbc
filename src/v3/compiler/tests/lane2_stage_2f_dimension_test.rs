//! Lane 2 Stage 2f — DB-3 `Dimension` / `DimensionReport` wiring (symbolic cost).

use v3_compiler::analyze_symbolic_cost_dimension;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, PortId};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

fn find_bind_port(dag: &Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn find_bind_root(dag: &Dag, name: &str) -> v3_compiler::dag::NodeId {
    dag.nodes()
        .iter()
        .find(|b| b.as_bind().map(|bind| bind.name == name).unwrap_or(false))
        .map(|b| b.id())
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

#[test]
fn bootstrap_dimension_data_registry_empty_until_class5_instances_exist() {
    let dag = Dag::new();
    assert!(
        dag.dimension_value_declarations().is_empty(),
        "no `data _: Dimension<_> = ...` values ship until class-5 bodies unlock the receipt"
    );
}

#[test]
fn analyze_symbolic_cost_composed_matches_lens_at_workflow_root() {
    let dag = compile_to_dag("let x = 1 + 2", "lane2_2f_dim.v3").expect("compiles");
    let root = find_bind_root(&dag, "x");
    let report = analyze_symbolic_cost_dimension(&dag, root);
    let lens = match symbolic_cost_of(&dag, &find_bind_port(&dag, "x")) {
        SymbolicCostLookup::FoundCost { _0: c } => c,
        SymbolicCostLookup::MissingCost => panic!("expected FoundCost"),
    };
    assert_eq!(report.composed, lens);
    assert_eq!(report.dimension_name, "symbolic_cost");
    assert_eq!(report.witnesses.len(), dag.nodes().len());
}
