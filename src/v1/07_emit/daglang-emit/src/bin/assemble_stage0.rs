//! Assemble the stage0 crate from .dag sources.
//!
//! Usage: cargo run -p daglang-emit --bin assemble_stage0 -- <workspace-root> <output-dir>
//!
//! Example: cargo run -p daglang-emit --bin assemble_stage0 -- . src/v2/stage0

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <workspace-root> <output-dir>", args[0]);
        std::process::exit(1);
    }
    let root = std::path::PathBuf::from(&args[1]);
    let output_dir = std::path::PathBuf::from(&args[2]);

    let v2_files: &[(&str, &str)] = &[
        ("rust_emit", "dsl/extdeps/languages/rust/emit.dag"),
        ("python_emit", "dsl/extdeps/languages/python/emit.dag"),
        ("go_emit", "dsl/extdeps/languages/go/emit.dag"),
        ("00_core", "src/v2/00_core.dag"),
        ("01_tokenize", "src/v2/01_tokenize.dag"),
        ("02_parse", "src/v2/02_parse.dag"),
        ("03_resolve", "src/v2/03_resolve.dag"),
        ("03_normalize", "src/v2/03_normalize.dag"),
        ("04_types", "src/v2/04_types.dag"),
        ("04_env", "src/v2/04_env.dag"),
        ("04_method", "src/v2/04_method.dag"),
        ("04_cycle", "src/v2/04_cycle.dag"),
        ("04_infer", "src/v2/04_infer.dag"),
        ("05_emit", "src/v2/05_emit.dag"),
        ("05_emit_rust", "src/v2/05_emit_rust.dag"),
        ("05_emit_python", "src/v2/05_emit_python.dag"),
        ("05_emit_go", "src/v2/05_emit_go.dag"),
        ("compile", "src/v2/compile.dag"),
        ("complexity", "src/v2/complexity.dag"),
        ("ownership", "src/v2/ownership.dag"),
        ("artifact", "src/v2/artifact.dag"),
        ("runtime_rust", "src/v2/runtime_rust.dag"),
    ];

    let parsed: Vec<(String, daglang_syntax::ast::SourceFile)> = v2_files
        .iter()
        .map(|(stem, path)| {
            let full_path = root.join(path);
            let source = std::fs::read_to_string(&full_path)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", full_path.display(), e));
            let result = daglang_syntax::parser::parse_to_result(&source);
            (stem.to_string(), result.ast)
        })
        .collect();

    let modules: Vec<(&str, &daglang_syntax::ast::SourceFile)> = parsed
        .iter()
        .map(|(stem, sf)| (stem.as_str(), sf))
        .collect();

    let files = daglang_emit::v2_crate_emit::assemble_v2_crate(&modules);

    let _ = std::fs::remove_dir_all(&output_dir);
    daglang_emit::v2_crate_emit::write_crate(&output_dir, &files)
        .expect("failed to write crate");

    eprintln!(
        "stage0 assembled: {} files written to {}",
        files.len(),
        output_dir.display()
    );
}
