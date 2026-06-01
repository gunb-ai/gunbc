// Throwaway profiling harness — times each pipeline phase separately.
#![allow(clippy::all)]
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use v2_compiler::v2_compiler_artifact::RenderTarget;
use v2_compiler::v2_compiler_compile as compile;
use v2_compiler::v2_compiler_compile::SourceFile;

fn collect(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            files.push(path);
        }
    }
}

fn main() {
    let root = std::env::args().nth(1).unwrap_or("src/v4".to_string());
    let mut paths = Vec::new();
    collect(std::path::Path::new(&root), &mut paths);
    let mut sources = Vec::new();
    for p in &paths {
        let content = std::fs::read_to_string(p).unwrap();
        sources.push(Rc::new(SourceFile {
            path: p.to_string_lossy().to_string(),
            content,
        }));
    }
    eprintln!("loaded {} sources", sources.len());
    let sources = Rc::new(sources);

    // Phase 1-N up through reconcile/ownership: compile_to_resolved
    let t = Instant::now();
    let resolved = compile::compile_to_resolved(sources.clone());
    eprintln!("compile_to_resolved (parse+resolve+normalize+infer+ownership): {:?}", t.elapsed());

    let graph = match resolved.graph.clone() {
        Some(g) => g,
        None => {
            eprintln!("no graph ({} diags); aborting", resolved.diagnostics.len());
            return;
        }
    };

    // emit_rust alone
    let t = Instant::now();
    let emit = v2_compiler::v2_compiler_emit_rust::emit_rust(graph.clone());
    eprintln!("emit_rust ONLY: {:?} ({} files)", t.elapsed(), emit.files.len());

    let _ = (HashMap::<String, String>::new(), RenderTarget::Rust);
}
