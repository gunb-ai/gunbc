//! **Layer:** integration

use std::fs;
use std::path::PathBuf;

use v3_compiler::compile_to_dag;

#[test]
fn t_demo_fixture_skeleton_compiles() {
    let fixture = "src/v3/compiler/tests/t_demo/t_demo_fixtures.dag";
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(fixture);
    let source = fs::read_to_string(path).expect("read T-Demo fixture skeleton");

    let dag = compile_to_dag(&source, fixture).expect("T-Demo fixture skeleton compiles");

    assert!(
        dag.diagnostics().is_empty(),
        "T-Demo fixture skeleton should compile without diagnostics: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn t_demo_compiler_nerd_claim_sources_compile() {
    let claims = [
        (
            "fn pair_score(xs: List<Int>) -> Int = fold(xs, 0, |outer, x| outer + fold(xs, 0, |inner, y| inner + x + y))",
            "fixture_compiler_nerd_canonical_complexity.v3",
        ),
        (
            "type OwnedPayload { left: Int right: Int } fn combine_payload(payload: OwnedPayload) -> Int = payload.left + payload.right",
            "fixture_compiler_nerd_canonical_ownership.v3",
        ),
        (
            "let total: Int = fold(cons(1, cons(2, singleton(3))), 0, |acc, x| acc + x)",
            "fixture_compiler_nerd_canonical_parallelism.v3",
        ),
    ];

    for (source, file_name) in claims {
        let dag = compile_to_dag(source, file_name).expect("T-Demo claim source compiles");
        assert!(
            dag.diagnostics().is_empty(),
            "T-Demo claim source `{file_name}` should compile without diagnostics: {:?}",
            dag.diagnostics()
        );
    }
}
