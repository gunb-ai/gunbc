#[test]
fn print_bootstrap_diagnostics() {
    let dag = v3_compiler::dag::Dag::new();
    for (span, diag) in dag.diagnostics() {
        println!("DIAG {:?} @ {:?}:{}", diag, span.file, span.byte_start);
    }
    panic!("total diagnostics: {}", dag.diagnostics().len());
}
