//! Stage 2b idempotency lens — `emit_rust_module` / `emit_go_module` / `emit_python_module`
//! smoke + rustc roundtrip proving `lenses/idempotency.dag` matches the crate oracle.
//!
//! Class-5 gap closure (β): the `.dag` lens must emit and execute, not only the
//! Rust re-export `v3_compiler::analyze_workflow`.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::emit_go::emit_go_module;
use v3_compiler::emit_python::emit_python_module;
use v3_compiler::emit_rust::emit_rust_module;

mod common;
use common::{HarnessLinkMode, RustcHarness};

static HARNESS: OnceLock<RustcHarness> = OnceLock::new();

fn harness() -> &'static RustcHarness {
    HARNESS.get_or_init(|| RustcHarness::new("lens_idempotency_emit"))
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

fn emit_idempotency_lens_rust() -> String {
    let header = "// AUTO-GENERATED from `src/v3/lenses/idempotency.dag` via \
                  `emit_rust_module`. Regenerate instead of hand-editing.\n\n";
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compile idempotency.dag");
    assert!(
        dag.diagnostics().is_empty(),
        "idempotency.dag should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit idempotency lens as Rust module");
    format_rust_source(&format!("{header}{raw}"))
}

/// `main` compares `v3_compiler::analyze_workflow` (Rust oracle) with
/// `emitted::analyze_workflow` from the `.dag` source.
const EMIT_CHECK_TAIL: &str = r#"
fn lane2_anchor(dag: &v3_compiler::Dag) -> v3_compiler::NodeId {
    dag.nodes()
        .iter()
        .find(|b| {
            matches!(
                b,
                v3_compiler::dag::Behavior::Value(_) | v3_compiler::dag::Behavior::Bind(_)
            )
        })
        .expect("fixture needs Value or Bind root")
        .id()
}

fn main() {
    let mut dag =
        v3_compiler::compile_to_dag("let _ = 42", "lens_idempotency_emit_fixture.v3")
            .expect("fixture compiles");
    let root = lane2_anchor(&dag);
    assert!(dag.try_register_lane2_workflow_effect(
        root,
        v3_compiler::dag::WorkflowEffect::LinearEffect { ops: vec![] }
    ));
    let oracle = v3_compiler::analyze_workflow(&dag, root);
    let via_lens = emitted::analyze_workflow(&dag, root);
    assert_eq!(oracle, via_lens);
}
"#;

static EMIT_CHECK_BIN: OnceLock<PathBuf> = OnceLock::new();

fn idempotency_emit_check_binary() -> &'static PathBuf {
    EMIT_CHECK_BIN.get_or_init(|| {
        let emitted = emit_idempotency_lens_rust();
        let mut src = String::new();
        src.push_str(
            "#[allow(warnings, clippy::all)]\n\
             mod emitted {\n  use v3_compiler::dag::*;\n  use v3_compiler::diagnostics::*;\n",
        );
        src.push_str(&emitted);
        src.push_str("\n}\n");
        src.push_str(EMIT_CHECK_TAIL);
        harness().compile(&src, "idempotency_emit_check", HarnessLinkMode::WithV3Compiler)
    })
}

#[test]
fn lens_idempotency_dag_emits_and_matches_analyze_workflow_oracle() {
    let run = Command::new(idempotency_emit_check_binary())
        .output()
        .expect("run idempotency emit-check harness");
    assert!(
        run.status.success(),
        "emit-check harness failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn lens_idempotency_dag_emits_go_and_python_modules() {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compile idempotency.dag");
    assert!(
        dag.diagnostics().is_empty(),
        "{:?}",
        dag.diagnostics()
    );
    let _go = emit_go_module(&dag).expect("emit_go_module(idempotency.dag)");
    let _py = emit_python_module(&dag).expect("emit_python_module(idempotency.dag)");
}
