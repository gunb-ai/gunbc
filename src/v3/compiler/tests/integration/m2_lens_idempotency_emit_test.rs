//! Stage 2b idempotency lens — emission smoke tests for `lenses/idempotency.dag`.
//!
//! **Rust:** `emit_rust_module` must succeed and surface `analyze_workflow` +
//! `lane2_workflow_effect_at` in the rendered module (the Stage 2b `.dag` lens
//! is part of the emit contract). **`m2_lens_idempotency_migration_test.rs`**
//! provides the rustc link-and-run check against `v3_compiler::analyze_workflow`
//! (crate exports `lane2_workflow_idempotency_report` /
//! `report_unsupported_workflow_variant` as std.effects mirrors for the emitted
//! module). Note: emitted `analyze_workflow` takes `&NodeId`; the oracle takes
//! `NodeId` — the migration test accounts for that at the call site.
//!
//! **Go / Python:** module emission succeeds on the same compiled lens DAG (class-5
//! surface: the lens is not Rust-only).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit::{
    emit_go_module_text as emit_go_module, emit_python_module_text as emit_python_module,
};
use v3_compiler::emit_rust::emit_rust_module;

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("idempotency.dag")
}

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read idempotency.dag")
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
        "rustfmt failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("rustfmt output should be utf-8")
}

fn compile_idempotency_lens() -> v3_compiler::Dag {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compile idempotency.dag");
    assert!(
        dag.diagnostics().is_empty(),
        "idempotency.dag should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    dag
}

#[test]
fn lens_idempotency_dag_emits_rust_module_with_analyze_workflow_entrypoint() {
    let dag = compile_idempotency_lens();
    let raw = emit_rust_module(&dag).expect("emit_rust_module(idempotency.dag)");
    let formatted = format_rust_source(&raw);
    assert!(
        formatted.contains("fn analyze_workflow"),
        "emitted Rust should declare analyze_workflow; got:\n{formatted}"
    );
    assert!(
        formatted.contains("lane2_workflow_effect_at"),
        "emitted Rust should call lane2_workflow_effect_at; got:\n{formatted}"
    );
}

#[test]
fn lens_idempotency_dag_emits_go_and_python_modules() {
    let dag = compile_idempotency_lens();
    let _go = emit_go_module(&dag).expect("emit_go_module(idempotency.dag)");
    let _py = emit_python_module(&dag).expect("emit_python_module(idempotency.dag)");
}
