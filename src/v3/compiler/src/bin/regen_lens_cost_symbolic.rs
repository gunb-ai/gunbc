// Regen driver for `src/v3/lenses/cost.dag` — Lane 2 Stage 2d
// symbolic-cost lens (DB-7). Mirrors the shape of
// `regen_lens_cost.rs` (the structural-cost lens) so the two stay
// reviewable side by side.
//
// Output: `src/v3/compiler/src/lens_cost_symbolic_generated.rs`.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

fn main() {
    let lens_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("cost.dag");
    let source = std::fs::read_to_string(&lens_path).expect("read cost.dag");
    let dag =
        compile_to_dag(&source, lens_path.to_string_lossy().as_ref()).expect("cost.dag compiles");
    let raw = emit_rust_module(&dag).expect("emit lens module");
    let header = "// AUTO-GENERATED from `src/v3/lenses/cost.dag` via\n\
                  // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";
    let combined = format!("{header}{raw}");

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
        .join("lens_cost_symbolic_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write lens_cost_symbolic_generated.rs");
    println!("wrote {}", out_path.display());
}
