// M2 — lens_structural_resolution migration test.
//
// Mirrors m2_lens_cost_migration_test.rs and
// m2_lens_provenance_migration_test.rs: the `.dag` declaration at
// src/v3/lenses/structural_resolution.dag is the live authority for
// the structural-resolution lens. This suite asserts:
//
// 1. The `.dag` file compiles cleanly (no diagnostics).
// 2. The emitted Rust module matches the checked-in snapshot at
//    src/v3/compiler/src/lens_structural_resolution_generated.rs — so
//    review of the emitted shape happens via the snapshot diff, not
//    behind it.
//
// Behavioral coverage of the lens itself lives in
// m1_lens_structural_resolution_test.rs (positive injection, bootstrap
// silence, named-UserDefined silence, anonymous arrow silence,
// co-existing real + injected declarations). This file exists solely
// to gate the `.dag` ↔ generated-Rust mirror so the single-authority
// claim is structural rather than conventional.
//
// If this test passes, the `.dag` is the authority and the
// `lens_structural_resolution_generated.rs` in-tree snapshot is in
// sync with it. The clone-count ratchet pattern from the provenance
// and unused_parameters lens tests is intentionally skipped here for
// the first pass — the cost-lens migration test followed the same
// minimal shape, and a clone budget can be added later if drift
// emerges.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

const GENERATED_LENS_HEADER: &str =
    "// AUTO-GENERATED from `src/v3/lenses/structural_resolution.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("structural_resolution.dag")
}

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read structural_resolution.dag")
}

fn emit_lens_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("structural_resolution.dag compiles");
    assert!(
        dag.diagnostics().is_empty(),
        "structural_resolution.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled lens module");
    format_rust_source(&format!("{GENERATED_LENS_HEADER}{raw}"))
}

fn checked_in_generated_module() -> &'static str {
    include_str!("../src/lens_structural_resolution_generated.rs")
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
        "rustfmt failed on emitted lens module"
    );
    String::from_utf8(output.stdout).expect("rustfmt output should be utf-8")
}

/// Helper: regenerate the checked-in generated module. Run with
/// `cargo test --test m2_lens_structural_resolution_migration_test --
/// --ignored emit_lens_structural_resolution_snapshot` whenever
/// `structural_resolution.dag` changes. The snapshot test below then
/// confirms the update landed on disk.
///
/// Equivalent to running the standalone
/// `bin/regen_lens_structural_resolution.rs` binary; both paths are
/// kept in lockstep with the cost / provenance / unused_parameters
/// migration tests.
#[test]
#[ignore]
fn emit_lens_structural_resolution_snapshot() {
    let fresh = emit_lens_module();
    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lens_structural_resolution_generated.rs");
    std::fs::write(&out_path, fresh).expect("write lens_structural_resolution_generated.rs");
    println!("wrote {}", out_path.display());
}

#[test]
fn lens_structural_resolution_dag_compiles_cleanly() {
    let dag = match compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref()) {
        Ok(dag) => dag,
        Err(v3_compiler::CompileError::Semantic(dag)) => {
            panic!(
                "structural_resolution.dag produced {} diagnostic(s): {:#?}",
                dag.diagnostics().len(),
                dag.diagnostics()
            );
        }
        Err(other) => panic!("structural_resolution.dag failed to compile: {other:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "structural_resolution.dag should compile without diagnostics, got {:#?}",
        dag.diagnostics()
    );
}

#[test]
fn lens_structural_resolution_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_lens_module();
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module().trim(),
        "checked-in generated module is stale; regenerate lens_structural_resolution_generated.rs from structural_resolution.dag (run with `--ignored emit_lens_structural_resolution_snapshot` or the regen binary)"
    );
}
