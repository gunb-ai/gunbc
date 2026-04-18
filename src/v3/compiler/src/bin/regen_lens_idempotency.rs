use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::CompileError;

const HEADER: &str = "// AUTO-GENERATED from `src/v3/lenses/idempotency.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn main() {
    let lens_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("idempotency.dag");
    let source = std::fs::read_to_string(&lens_path).expect("read idempotency.dag");
    let dag = match compile_to_dag(&source, lens_path.to_string_lossy().as_ref()) {
        Ok(d) => d,
        Err(CompileError::Semantic(d)) => {
            eprintln!("idempotency.dag diagnostics: {:?}", d.diagnostics());
            panic!("idempotency.dag compile failed (semantic)");
        }
        Err(e) => panic!("idempotency.dag compile failed: {e:?}"),
    };
    let raw = emit_rust_module(&dag).expect("emit lens module");
    let combined = format!("{HEADER}{raw}");

    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn rustfmt");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(combined.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("rustfmt");
    assert!(output.status.success(), "rustfmt failed");
    let formatted = String::from_utf8(output.stdout).expect("utf8");

    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lens_idempotency_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write lens_idempotency_generated.rs");
    println!("wrote {}", out_path.display());
}
