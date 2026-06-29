use std::rc::Rc;
use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_interpreter::{self, Value};

#[test]
fn value_null_split_model_witnesses_execute_green() {
    let entry = "src/v2/test/claim/value_null_split_model_witness_test.dag";
    let content = std::fs::read_to_string(entry).unwrap();
    let roots: Vec<std::path::PathBuf> = ["src/v2", "dsl"].iter().map(std::path::PathBuf::from).collect();
    let sources = v1_tests::helpers::resolve_imports_transitively_with_source_roots(entry, &content, &roots);
    let resolved = compile_to_resolved(Rc::new(sources));
    let msgs: Vec<String> = resolved.diagnostics.iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(msgs.is_empty() && resolved.graph.is_some(), "compile failed: {:?}", msgs);
    let graph = resolved.graph.as_ref().unwrap();
    for fn_name in [
        "value_null_split_carriers_pairwise_distinct",
        "value_null_split_optional_absent_projects",
        "value_null_split_witness_violates_projects",
        "value_null_split_map_lookup_witness_slice",
        "value_null_split_map_get_optional_absent_slice",
        "value_null_split_conflation_perturbation_red",
        "value_null_split_carrier_constructors_match_roles",
    ] {
        match v1_interpreter::run(graph, resolved.source_indices.clone(), fn_name) {
            Ok(Value::Bool(true)) => {}
            other => panic!("{}: {:?}", fn_name, other),
        }
    }
}
