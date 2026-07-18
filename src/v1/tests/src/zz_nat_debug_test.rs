use crate::helpers::compile_dag_named_with_source_roots;
use v1_compiler::v1_compiler_artifact::RenderTarget;

#[test]
fn debug_nat_repro() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let src = std::fs::read_to_string(root.join("dag/std/measure.dag")).unwrap();
    let result = compile_dag_named_with_source_roots(
        "dag/std/measure.dag",
        &src,
        RenderTarget::Rust,
        &[root.join("dag"), root.join("src/v2")],
    );
    for f in result.files.iter() {
        if f.path.contains("measure") {
            eprintln!("=== {} ===\n{}", f.path, f.content);
        }
    }
}
