use std::path::PathBuf;

use v3_compiler::runtime_mirrors_codegen::render_runtime_mirrors;

fn main() {
    let generated = render_runtime_mirrors().expect("render runtime mirrors");
    write("types_generated.rs", &generated.types);
    write("diagnostics_generated.rs", &generated.diagnostics);
    write("serialize_generated.rs", &generated.serialize);
    write("dag_cost_generated.rs", &generated.dag_cost);
}

fn write(name: &str, body: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(name);
    std::fs::write(&path, body).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    println!("wrote {}", path.display());
}
