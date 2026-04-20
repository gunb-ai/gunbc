//! SG-7 prep — freshness ratchet for `variant_payload_generated.rs`.
//!
//! Mirrors `sg4_prep_infer_helpers_freshness_test.rs`: recompile the `.dag`
//! authority, render through `emit_rust_module`, rustfmt the result, and
//! compare against the checked-in projection.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

const GENERATED_HEADER: &str = "// AUTO-GENERATED from `src/v3/lenses/variant_payload.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("variant_payload.dag")
}

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read variant_payload.dag")
}

fn emit_lens_module() -> String {
    let path = lens_path();
    let dag = compile_to_dag(&lens_source(), path.to_string_lossy().as_ref())
        .expect("compiled variant_payload source");
    assert!(
        dag.diagnostics().is_empty(),
        "variant_payload.dag should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled variant_payload module");
    format_rust_source(&format!("{GENERATED_HEADER}{raw}"))
}

fn checked_in_generated_module() -> &'static str {
    include_str!("../../src/variant_payload_generated.rs")
}

fn format_rust_source(source: &str) -> String {
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
        .expect("rustfmt stdin")
        .write_all(source.as_bytes())
        .expect("write source to rustfmt");
    let output = child.wait_with_output().expect("wait for rustfmt");
    assert!(
        output.status.success(),
        "rustfmt failed on emitted variant_payload module"
    );
    String::from_utf8(output.stdout).expect("rustfmt output should be utf-8")
}

#[test]
fn variant_payload_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_lens_module();
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module().trim(),
        "checked-in generated module is stale; regenerate variant_payload_generated.rs via `cargo run -p v3-compiler --bin regen_lens -- --lens variant_payload`"
    );
}

#[test]
#[ignore]
fn emit_variant_payload_snapshot() {
    let fresh = emit_lens_module();
    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("variant_payload_generated.rs");
    std::fs::write(&out_path, fresh).expect("write variant_payload_generated.rs");
    println!("wrote {}", out_path.display());
}
