//! Stage 2b idempotency lens — rustc round-trip: emitted `analyze_workflow` vs oracle.
//!
//! `src/v3/lenses/idempotency.dag` reads `lane2_workflow` through the substrate
//! accessor (`lane2_workflow_at` → Rust `Dag::lane2_workflow_effect_at` per
//! `rust_lane2_workflow_accessor` in `rust.dag`). This file links
//! `emit_rust_module(idempotency.dag)` into a harness and asserts the generated
//! entrypoint matches [`v3_compiler::analyze_workflow`] — closing the
//! "binding exists vs consumer path is real" gap for the reflection boundary.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Behavior;
use v3_compiler::dag::{WorkflowEffect, WorkflowIdempotencyReport};
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::{analyze_workflow as oracle_analyze_workflow, Dag, NodeId};

mod common;
use common::{HarnessLinkMode, RustcHarness};

static HARNESS: OnceLock<RustcHarness> = OnceLock::new();
fn harness() -> &'static RustcHarness {
    HARNESS.get_or_init(|| RustcHarness::new("idempotency_lens_migration"))
}

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

fn emit_idempotency_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compile idempotency.dag");
    assert!(
        dag.diagnostics().is_empty(),
        "idempotency.dag should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit_rust_module(idempotency.dag)");
    format_rust_source(&raw)
}

fn lane2_anchor(dag: &Dag) -> NodeId {
    dag.nodes()
        .iter()
        .find(|b| matches!(b, Behavior::Value(_) | Behavior::Bind(_)))
        .expect("fixture should include a Value or Bind for lane2 staging")
        .id()
}

/// Compile the idempotency-lens roundtrip harness once per test run.
fn build_roundtrip_harness(module_source: &str) -> PathBuf {
    let wrapped = format!(
        "#[allow(warnings, clippy::all)] \
         mod emitted {{ \
           use v3_compiler::dag::*; \
           use v3_compiler::{{lane2_workflow_idempotency_report, report_unsupported_workflow_variant}}; \
           {module_source} \
         }} \
         fn main() {{ \
           let mut dag = v3_compiler::compile_to_dag(\"let _ = 1\", \"lane2_empty.v3\") \
             .expect(\"compile fixture\"); \
           let root = dag.nodes().iter() \
             .find(|b| matches!(b, v3_compiler::dag::Behavior::Value(_) | v3_compiler::dag::Behavior::Bind(_))) \
             .expect(\"lane2 root\").id(); \
           let wf = v3_compiler::dag::WorkflowEffect::LinearEffect {{ ops: vec![] }}; \
           assert!(dag.try_register_lane2_workflow_effect(root, wf)); \
           let emitted = emitted::analyze_workflow(&dag, &root); \
           let oracle = v3_compiler::analyze_workflow(&dag, root); \
           assert_eq!(emitted, oracle, \"emitted analyze_workflow should match crate oracle\"); \
           let mut dag2 = v3_compiler::compile_to_dag(\"let _ = 1\", \"lane2_none.v3\") \
             .expect(\"compile fixture 2\"); \
           let root2 = dag2.nodes().iter() \
             .find(|b| matches!(b, v3_compiler::dag::Behavior::Value(_) | v3_compiler::dag::Behavior::Bind(_))) \
             .expect(\"lane2 root 2\").id(); \
           let e2 = emitted::analyze_workflow(&dag2, &root2); \
           let o2 = v3_compiler::analyze_workflow(&dag2, root2); \
           assert_eq!(e2, o2, \"missing workflow: emitted vs oracle\"); \
           assert!(matches!(e2, v3_compiler::dag::WorkflowIdempotencyReport::IdempotencyUnsupported(_))); \
         }}"
    );
    harness().compile(&wrapped, "idempo_roundtrip", HarnessLinkMode::WithV3Compiler)
}

#[test]
fn idempotency_emitted_analyze_matches_oracle() {
    let module = emit_idempotency_module();
    let bin = build_roundtrip_harness(&module);
    let run = std::process::Command::new(&bin)
        .output()
        .expect("run idempotency roundtrip harness");
    assert!(
        run.status.success(),
        "harness failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn idempotency_dag_compiles_cleanly() {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("idempotency.dag should compile");
    assert!(
        dag.diagnostics().is_empty(),
        "idempotency.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
}
