// Temp: check that lenses/cost.dag compiles cleanly.
#[test]
fn cost_lens_compiles_cleanly() {
    let source = include_str!("../../../../src/v3/lenses/cost.dag");
    match v3_compiler::compile_to_dag(source, "src/v3/lenses/cost.dag") {
        Ok(_) => println!("OK"),
        Err(v3_compiler::CompileError::Semantic(dag)) => {
            for (p, d) in dag.diagnostics().iter() {
                println!("DIAG port={:?}: {:?}", p, d);
            }
            panic!("{} diagnostics", dag.diagnostics().len());
        }
        Err(e) => panic!("structural error: {:?}", e),
    }
}
