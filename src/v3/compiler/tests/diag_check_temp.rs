#[test]
fn bootstrap_clean() {
    let dag = v3_compiler::dag::Dag::new();
    for (p, d) in dag.diagnostics().iter() {
        println!("DIAG {:?}: {:?}", p, d);
    }
    assert!(
        dag.diagnostics().is_empty(),
        "{} diags",
        dag.diagnostics().len()
    );
}
