#[test]
fn print_bootstrap_diagnostics() {
    let dag = v3_compiler::dag::Dag::new();
    for (port, diag) in dag.diagnostics().iter() {
        println!("DIAG port={:?}: {:?}", port, diag);
    }
    assert_eq!(dag.diagnostics().len(), 0, "see printed output");
}
