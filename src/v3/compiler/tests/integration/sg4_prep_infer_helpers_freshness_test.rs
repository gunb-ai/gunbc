//! SG-4 prep — freshness ratchet for `infer_helpers_generated.rs`.
//!
//! Mirrors the per-file pattern used by every other committed
//! `*_generated.rs` module (`m2_lens_*_migration_test.rs::
//! *_generated_module_matches_checked_in_snapshot`): recompile the
//! `.dag` authority, render through `emit_rust_module`, and
//! trim-compare against the on-disk Rust projection. Drift fails
//! loudly with the regen command in the assertion message.
//!
//! Codex review on PR #562 (SHA `90939487a`) flagged the missing
//! ratchet as BLOCKING — without it, `src/v3/lenses/infer_helpers.dag`
//! and the committed `src/v3/compiler/src/infer_helpers_generated.rs`
//! can drift in violation of INVARIANTS.md's "no parallel
//! implementations" rule. SG-6 owns folding the per-file freshness
//! tests + per-file regen binaries into a single generic gate; until
//! then the per-helper test mirrors the existing precedent rather
//! than inventing new infra.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

const GENERATED_HEADER: &str = "// AUTO-GENERATED from `src/v3/lenses/infer_helpers.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn helper_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("infer_helpers.dag")
}

fn helper_source() -> String {
    std::fs::read_to_string(helper_path()).expect("read infer_helpers.dag")
}

fn emit_helper_module() -> String {
    let path = helper_path();
    let dag = compile_to_dag(&helper_source(), path.to_string_lossy().as_ref())
        .expect("compiled infer_helpers source");
    assert!(
        dag.diagnostics().is_empty(),
        "infer_helpers.dag should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled infer_helpers module");
    format_rust_source(&format!("{GENERATED_HEADER}{raw}"))
}

fn checked_in_generated_module() -> &'static str {
    include_str!("../../src/infer_helpers_generated.rs")
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
        "rustfmt failed on emitted infer_helpers module"
    );
    String::from_utf8(output.stdout).expect("rustfmt output should be utf-8")
}

#[test]
fn infer_helpers_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_helper_module();
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module().trim(),
        "checked-in generated module is stale; regenerate infer_helpers_generated.rs from infer_helpers.dag via `cargo run -p v3-compiler --bin regen_lens -- --lens infer_helpers` (absorbed into SG-6's unified regen driver on PR #560)"
    );
}

/// Regenerate helper. Writes the freshly emitted module straight to
/// disk so the next `cargo test` run sees a clean tree. Equivalent to
/// `cargo run -p v3-compiler --bin regen_lens -- --lens infer_helpers`
/// (SG-6 absorbed the former `regen_infer_helpers` bin into the unified
/// driver on PR #560) — kept as an `#[ignore]` test for parity with
/// the lens-migration regen helpers.
///
/// Run with: `cargo test -p v3-compiler --test integration -- --ignored
/// emit_infer_helpers_snapshot`.
#[test]
#[ignore]
fn emit_infer_helpers_snapshot() {
    let fresh = emit_helper_module();
    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("infer_helpers_generated.rs");
    std::fs::write(&out_path, fresh).expect("write infer_helpers_generated.rs");
    println!("wrote {}", out_path.display());
}
