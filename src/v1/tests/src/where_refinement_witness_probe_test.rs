use std::rc::Rc;

use v1_compiler::cli_run::{
    build_module_path_index_from_witness_roots, compile_clean_diagnostic_is_hard,
    compile_dag_rust_emit_check, resolve_virtual_source_with_imports,
};
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile;

fn diags_for(src: &str) -> im::Vector<Rc<v1_compiler::v1_std_core::ErrorNode>> {
    let module_index = build_module_path_index_from_witness_roots();
    let sources = resolve_virtual_source_with_imports("test.dag", src, &module_index);
    v1_compiler_compile::compile_sources(Rc::new(sources.into()), RenderTarget::Rust).diagnostics
}

#[test]
fn where_refinement_record_green_compiles() {
    let src = "module whereref_record_green\n\
      type PositiveInt = Nat where gt_zero\n\
      type Box { n: PositiveInt }\n\
      data b: Box = Box { n: 1 }\n";
    for d in diags_for(src).iter() {
        eprintln!(
            "hard={}: {:?}",
            compile_clean_diagnostic_is_hard(d),
            d.diagnostic
        );
    }
    assert!(compile_dag_rust_emit_check(
        src,
        "src/whereref_record_green.rs",
        &[],
        &[]
    ));
}

#[test]
fn where_refinement_cast_green_compiles() {
    let src = "module whereref_cast_green\n\
      type PositiveInt = Nat where gt_zero\n\
      fn ok() -> PositiveInt { 1 as PositiveInt }\n";
    for d in diags_for(src).iter() {
        eprintln!(
            "hard={}: {:?}",
            compile_clean_diagnostic_is_hard(d),
            d.diagnostic
        );
    }
    assert!(compile_dag_rust_emit_check(
        src,
        "src/whereref_cast_green.rs",
        &[],
        &[]
    ));
}
