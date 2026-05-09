//! **Layer:** boundary-adjacent integration — emitted Rust lens linked via rustc.
//!
//! Lens-semantic coverage (Value/Transform/Branch/Loop/Bind, seeded
//! bind-param costs) lives as in-crate unit tests under
//! `src/v3/compiler/src/lib.rs::lens_cost::tests`, where the
//! `pub(crate)` Dag builder is reachable. This suite is the
//! cross-process receipt: it verifies that the `.dag` authority
//! compiles cleanly, that the checked-in generated module is in sync
//! with what `emit_rust_module(complexity.dag)` produces, and that the
//! emitted module links and runs end-to-end via rustc on one
//! representative fixture.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

use crate::common::{HarnessLinkMode, RustcHarness};

static HARNESS: OnceLock<RustcHarness> = OnceLock::new();
fn harness() -> &'static RustcHarness {
    HARNESS.get_or_init(|| RustcHarness::new("cost_lens_migration"))
}

const GENERATED_LENS_HEADER: &str = "// AUTO-GENERATED from `src/v3/lenses/complexity.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("complexity.dag")
}

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read complexity.dag")
}

fn emit_lens_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compiled complexity.dag");
    assert!(
        dag.diagnostics().is_empty(),
        "complexity.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled lens module");
    format_rust_source(&format!("{GENERATED_LENS_HEADER}{raw}"))
}

fn checked_in_generated_module() -> &'static str {
    include_str!("../../src/complexity_lens_generated.rs")
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

/// Compile the cost-lens roundtrip harness exactly once per test run.
/// The emitted module is embedded and `main` reads `program_source`,
/// `file_name`, and `bind_name` from argv, so each fixture re-run is a
/// single process spawn instead of a fresh `rustc` invocation.
fn build_roundtrip_harness(module_source: &str) -> PathBuf {
    let wrapped = format!(
        "#[allow(warnings, clippy::all)] \
         mod emitted {{ use v3_compiler::dag::*; use v3_compiler::diagnostics::*; \
         use v3_compiler::complexity_lattice::asymptotic_dominates; \
         use v3_compiler::Witness; \
         use v3_compiler::lens_t_las_carrier::{{EnforceableLens, Lens, LensEnforcement, Monoid, OptionalDiagnostic}}; \
         {module_source} }} \
         fn main() {{ \
           let mut __args = std::env::args(); __args.next(); \
           let program_source = __args.next().expect(\"program_source arg\"); \
           let file_name = __args.next().expect(\"file_name arg\"); \
           let bind_name = __args.next().expect(\"bind_name arg\"); \
           let dag = v3_compiler::compile_to_dag(&program_source, &file_name).expect(\"compiles\"); \
           let bind = dag.nodes().iter().find_map(|node| match node {{ \
             v3_compiler::dag::Behavior::Bind(bind) if bind.name == bind_name => Some(bind.clone()), \
             _ => None \
           }}).expect(\"bind\"); \
           match emitted::complexity_of(&dag, &bind.value) {{ \
             v3_compiler::dag::Lookup::Hit(summary) => {{ \
               let positive_work = match &summary.work {{ \
                 v3_compiler::dag::SymbolicCost::ConstantCost {{ _0 }} => *_0 > 0, \
                 _ => true, \
               }}; \
               println!(\"{{}}\", positive_work); \
             }} \
             v3_compiler::dag::Lookup::Miss => panic!(\"complexity lens returned Miss for bind `{{}}` — malformed DAG\", bind.name), \
           }} \
         }}"
    );
    harness().compile(&wrapped, "main_bin", HarnessLinkMode::WithV3Compiler)
}

fn roundtrip_positive_work(
    bin_path: &Path,
    program_source: &str,
    file_name: &str,
    bind_name: &str,
) -> bool {
    let run = Command::new(bin_path)
        .arg(program_source)
        .arg(file_name)
        .arg(bind_name)
        .output()
        .expect("run compiled binary");
    assert!(
        run.status.success(),
        "compiled binary failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout)
        .trim()
        .parse()
        .expect("printed positive-work flag should be bool")
}

#[test]
fn complexity_dag_compiles_cleanly() {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("complexity.dag should compile cleanly");
    assert!(
        dag.diagnostics().is_empty(),
        "complexity.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
}

/// Inbox #1130 / #1139 — STOP ratchet: production `complexity.dag` must not ship
/// iterate-shaped migration names; no L-8-violating emitted primitive helpers.
///
/// Gate #92 — `data complexity_lens` must wire **read** to `complexity_of` and
/// **monoid/branch/iterate** to the same `compose_*` spine as `compute_summaries`
/// (see `complexity.dag`). **Enforcement** (`project` / `violates`) stays substrate
/// authority; execution is via `complexity_lens_generated` +
/// `v3_compiler::complexity_lattice::asymptotic_dominates`.
#[test]
fn complexity_lens_migration_stop_surface_ratchet() {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("complexity.dag should compile cleanly");
    assert!(
        dag.declaration_by_name("complexity_lens_read_stub").is_none(),
        "`complexity_lens_read_stub` retired: read must delegate to `complexity_of`"
    );
    assert!(
        dag.declaration_by_name("complexity_lens_read").is_some(),
        "expected `complexity_lens_read` (authoritative read spine)"
    );
    assert!(
        dag.declaration_by_name("complexity_lens_validate").is_some(),
        "expected `complexity_lens_validate` (aggregate hook; see substrate limitation notes in complexity.dag)"
    );
    assert!(
        dag.declaration_by_name("complexity_summary_work_class_consistent").is_some(),
        "expected `complexity_summary_work_class_consistent` export for fold consumers"
    );
    assert!(
        dag.declaration_by_name("complexity_iterate").is_none(),
        "`Lens<Int>.iterate` must not ship as `complexity_iterate` until fold_lens owns loop-bound semantics (no LoopBound-ignoring stub)"
    );
    for name in [
        "complexity_behavior_result_port",
        "complexity_sequential_op",
        "complexity_branch",
    ] {
        assert!(
            dag.declaration_by_name(name).is_none(),
            "production `{name}` must not emit primitive-returning Rust on the migrated lens surface (L-8); see STOP comment in complexity.dag"
        );
    }
}

#[test]
fn complexity_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_lens_module();
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module().trim(),
        "checked-in generated module is stale; regenerate complexity_lens_generated.rs from complexity.dag"
    );
}

/// Cross-process receipt: the emitted module links against the
/// `v3_compiler` crate and runs under rustc on a representative
/// fixture that exercises Bind-with-params + seeded param costs + a
/// transform over fold/map/singleton. Deeper behavioral coverage
/// (Value/Transform/Branch/Loop/Bind/params) is pinned by the
/// in-crate `lens_cost::tests` unit tests against hand-built Dags.
#[test]
fn complexity_dag_runs_end_to_end_via_rustc_harness() {
    let module = emit_lens_module();
    let bin_path = build_roundtrip_harness(&module);
    let positive_work = roundtrip_positive_work(
        &bin_path,
        "let total: Int = fold(map(singleton(1), |x| x + 1), 0, |acc, x| acc + x)",
        "nested_fold.v3",
        "total",
    );
    assert!(
        positive_work,
        "nested fold fixture should report positive work"
    );
}
