//! **Layer:** integration

use std::fs;

use v3_compiler::compile_to_dag;

#[test]
fn t_demo_fixture_skeleton_compiles() {
    let path = "src/v3/demo/t_demo_fixtures.dag";
    let source = fs::read_to_string(path).expect("read T-Demo fixture skeleton");

    let dag = compile_to_dag(&source, path).expect("T-Demo fixture skeleton compiles");

    assert!(
        dag.diagnostics().is_empty(),
        "T-Demo fixture skeleton should compile without diagnostics: {:?}",
        dag.diagnostics()
    );
}
