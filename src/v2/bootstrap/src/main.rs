//! v2 bootstrap driver — hand-written glue that combines v1-emitted Rust modules.
//!
//! This binary reads .dag source files, runs each through the v2 pipeline
//! (tokenize → parse → resolve → typecheck → emit_rust), and writes
//! the emitted Rust files to a target directory.
//!
//! The pipeline function stubs below will be replaced by v1-emitted code
//! once the v2 compiler is emitting valid Rust for its own source modules.

use std::path::PathBuf;

// Placeholder modules — each corresponds to a v2 compiler stage.
// These will be replaced by actual generated code from v1 once Phase 1c
// (native bootstrap) is reached.
mod v2_tokenize {
    pub fn tokenize(_source: &str) -> serde_json::Value {
        todo!("replaced by v1-emitted tokenize module")
    }
}

mod v2_parse {
    pub fn parse(_tokens: serde_json::Value) -> serde_json::Value {
        todo!("replaced by v1-emitted parse module")
    }
}

mod v2_resolve {
    pub fn resolve_modules(_modules: Vec<serde_json::Value>) -> serde_json::Value {
        todo!("replaced by v1-emitted resolve module")
    }
}

mod v2_typecheck {
    pub fn typecheck(_graph: serde_json::Value) -> serde_json::Value {
        todo!("replaced by v1-emitted typecheck module")
    }
}

mod v2_emit {
    pub fn emit_rust(_typed: serde_json::Value) -> Vec<(String, String)> {
        todo!("replaced by v1-emitted emit module")
    }
}

#[allow(clippy::disallowed_macros)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: v2-bootstrap <output-dir> <file1.dag> [file2.dag ...]");
        std::process::exit(1);
    }

    let output_dir = PathBuf::from(&args[1]);
    let source_files: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    // Read all source files
    let sources: Vec<(String, String)> = source_files
        .iter()
        .map(|path| {
            let content = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
            (path.to_string(), content)
        })
        .collect();

    // Stage 1: Tokenize each source
    let tokenized: Vec<serde_json::Value> = sources
        .iter()
        .map(|(_, content)| v2_tokenize::tokenize(content))
        .collect();

    // Stage 2: Parse each token stream
    let parsed: Vec<serde_json::Value> = tokenized
        .into_iter()
        .map(v2_parse::parse)
        .collect();

    // Stage 3: Resolve imports
    let graph = v2_resolve::resolve_modules(parsed);

    // Stage 4: Typecheck
    let typed = v2_typecheck::typecheck(graph);

    // Stage 5: Emit Rust
    let files = v2_emit::emit_rust(typed);

    // Write output files
    std::fs::create_dir_all(&output_dir)
        .unwrap_or_else(|e| panic!("failed to create output dir: {}", e));

    for (path, content) in &files {
        let full_path = output_dir.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&full_path, content)
            .unwrap_or_else(|e| panic!("failed to write {}: {}", full_path.display(), e));
    }

    println!(
        "v2-bootstrap: emitted {} files to {}",
        files.len(),
        output_dir.display()
    );
}
