#[test]
fn print_compile_to_dag_diagnostics() {
    match v3_compiler::compile_to_dag("fn id(x: Int) -> Int = x", "test.v3") {
        Ok(_) => println!("OK, no diagnostics"),
        Err(v3_compiler::CompileError::Semantic(dag)) => {
            for (p, d) in dag.diagnostics().iter() {
                println!("DIAG port={:?}: {:?}", p, d);
            }
            panic!("{} diagnostics", dag.diagnostics().len());
        }
        Err(e) => panic!("other error: {:?}", e),
    }
}
