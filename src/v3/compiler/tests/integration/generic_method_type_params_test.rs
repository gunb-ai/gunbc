//! **Layer:** integration
//!
//! Path B Brief 1 receipt: executable generic collection operations are
//! non-endomorphic. The current surface spells them as generic std-list
//! functions; dotted method-call type arguments remain a parser/lowerer
//! surface follow-up.

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust;

#[test]
fn non_endomorphic_map_and_accumulator_polymorphic_fold_compile_and_emit() {
    let source = "\
fn int_label(x: Int) -> String = \"one\"
fn keep_label(acc: String, x: Int) -> String = acc
let labels: List<String> = map(singleton(1), int_label)
let folded: String = fold(singleton(1), \"\", keep_label)
";

    let dag = compile_to_dag(source, "generic_method_type_params.v3")
        .expect("non-endomorphic map and accumulator-polymorphic fold should compile");
    assert!(
        dag.diagnostics().is_empty(),
        "fixture should compile without diagnostics: {:?}",
        dag.diagnostics()
    );

    let rendered = emit_rust(&dag).expect("generic collection fixture should emit Rust");
    assert!(
        rendered.contains("let labels: Vec<String>"),
        "map should infer and emit List<String>, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("let folded: String"),
        "fold should infer and emit String accumulator/result, rendered:\n{rendered}"
    );
}
