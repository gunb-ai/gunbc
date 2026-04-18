#[test]
fn compile_record_variant_body() {
    let source = std::fs::read_to_string("/tmp/test_record_variant.dag").unwrap();
    let dag = v3_compiler::compile_to_dag(&source, "test.v3").expect("compiles");
    for (p, d) in dag.diagnostics().iter() {
        println!("{:?}: {:?}", p, d);
    }
    assert!(dag.diagnostics().is_empty());
}
