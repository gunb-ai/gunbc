//! **Layer:** integration
//!
//! Path B Brief 1 receipt: executable generic collection operations are
//! non-endomorphic. The current surface spells them as generic std-list
//! functions; dotted method-call type arguments remain a parser/lowerer
//! surface follow-up.

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust;

const NON_ENDOMORPHIC_MAP_DEMO: &str =
    include_str!("../fixtures/path_b_brief_1/non_endomorphic_map_demo.v3");

#[test]
fn non_endomorphic_map_and_accumulator_polymorphic_fold_compile_and_emit() {
    let dag = compile_to_dag(NON_ENDOMORPHIC_MAP_DEMO, "non_endomorphic_map_demo.v3")
        .expect("non-endomorphic map and accumulator-polymorphic fold should compile");
    assert!(
        dag.diagnostics().is_empty(),
        "fixture should compile without diagnostics: {:?}",
        dag.diagnostics()
    );

    let rendered = emit_rust(&dag).expect("generic collection fixture should emit Rust");
    assert!(
        rendered.contains("Vec<String>"),
        "map should infer and emit List<String>, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("String::from(\"\")") && rendered.contains(".iter().fold("),
        "fold should emit a String accumulator/result, rendered:\n{rendered}"
    );
}
