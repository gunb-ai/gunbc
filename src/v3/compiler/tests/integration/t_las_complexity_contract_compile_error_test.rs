//! **Layer:** integration
//!
//! §1.8 / gate #92 `complexity_violation_compile_error_demonstrated`: program
//! whose `complexity_of` asymptotic class strictly exceeds a `ClassLog` budget
//! under an `EnforcedApplication` must fail closed with a `ParseError`
//! diagnostic containing `lens enforcement violation`.

use std::fs;
use std::path::PathBuf;

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const DEMO_REL_PATH: &str = "src/v3/compiler/tests/fixtures/t_las_complexity_contract_demo.dag";

#[test]
fn complexity_violation_compile_error_demonstrated() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(DEMO_REL_PATH);
    let source = fs::read_to_string(&path).expect("read T-LAS complexity demo fixture");
    std::thread::Builder::new()
        .name("t-las-complexity-demo-compile".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let err = compile_to_dag(&source, "t_las_complexity_contract_demo.dag")
                .expect_err("witness asymptotic class exceeds ClassLog — compile must fail");
            let CompileError::Semantic(dag) = err else {
                panic!("expected Semantic(Dag) after violation, got {err:?}");
            };
            let receipts: Vec<_> = dag
                .diagnostics()
                .iter()
                .map(|(_, d)| (d.layer1_kind_label().to_string(), d.message()))
                .collect();
            let ok = receipts.iter().any(|(kind, msg)| {
                kind == "ParseError" && msg.contains("lens enforcement violation")
            });
            assert!(
                ok,
                "expected ParseError with `lens enforcement violation`; got {receipts:?}"
            );
        })
        .expect("spawn demo compile")
        .join()
        .expect("demo compile thread panicked");
}
