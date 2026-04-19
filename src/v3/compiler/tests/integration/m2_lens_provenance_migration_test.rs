//! **Layer:** boundary-adjacent integration — emitted Rust lens linked via rustc.
//!
//! Lens-semantic coverage (NoProducer, Source/Value, Computed/Transform,
//! Selected/Branch, Accumulated/Loop, Bind-value passthrough) lives as
//! in-crate unit tests under
//! `src/v3/compiler/src/lib.rs::lens_provenance::tests`, where the
//! `pub(crate)` Dag builder is reachable. This suite is the
//! cross-process receipt: the `.dag` authority compiles cleanly, the
//! checked-in generated module matches `emit_rust_module(provenance.dag)`,
//! the clone-count ratchet holds, and the emitted module links and
//! runs end-to-end on one representative fixture.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId};
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::Dag;

use crate::common::{HarnessLinkMode, RustcHarness};

static HARNESS: OnceLock<RustcHarness> = OnceLock::new();
fn harness() -> &'static RustcHarness {
    HARNESS.get_or_init(|| RustcHarness::new("provenance_lens_migration"))
}

const GENERATED_LENS_HEADER: &str = "// AUTO-GENERATED from `src/v3/lenses/provenance.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read provenance.dag")
}

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("provenance.dag")
}

fn emit_lens_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compiled lens source");
    assert!(
        dag.diagnostics().is_empty(),
        "lens_provenance.dag should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled lens module");
    format_rust_source(&format!("{GENERATED_LENS_HEADER}{raw}"))
}

fn checked_in_generated_module() -> &'static str {
    include_str!("../../src/lens_provenance_generated.rs")
}

fn clone_call_count(source: &str) -> usize {
    source.match_indices(".clone(").count()
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

/// Compile the provenance roundtrip harness once. `main` reads
/// `program_source` + `file_name` from argv so each fixture is a
/// single process spawn rather than a fresh rustc invocation.
fn build_roundtrip_harness(module_source: &str) -> PathBuf {
    let wrapped = format!(
        "#[allow(warnings, clippy::all)] \
         mod emitted {{ use v3_compiler::dag::*; use v3_compiler::diagnostics::*; {module_source} }} \
         fn origin_label(origin: &emitted::Origin) -> &'static str {{ \
           match origin {{ \
             emitted::Origin::NoProducer => \"Source\", \
             emitted::Origin::MissingPort => \"MissingPort\", \
             emitted::Origin::MissingBehavior => \"MissingBehavior\", \
             emitted::Origin::Source {{ _0: _ }} => \"Source\", \
             emitted::Origin::Computed {{ _0: _ }} => \"Computed\", \
             emitted::Origin::Selected {{ _0: _ }} => \"Selected\", \
             emitted::Origin::Accumulated {{ _0: _ }} => \"Accumulated\", \
           }} \
         }} \
         fn main() {{ \
           let mut __args = std::env::args(); __args.next(); \
           let program_source = __args.next().expect(\"program_source arg\"); \
           let file_name = __args.next().expect(\"file_name arg\"); \
           let dag = v3_compiler::compile_to_dag(&program_source, &file_name).expect(\"compiles\"); \
           let mut rendered: Vec<String> = Vec::new(); \
           for node in dag.nodes() {{ \
             if let v3_compiler::dag::Behavior::Bind(bind) = node {{ \
               let origin = emitted::origin_of(&dag, &bind.value); \
               rendered.push(format!(\"{{}}:{{}}\", bind.name, origin_label(&origin))); \
             }} \
           }} \
           rendered.sort(); \
           println!(\"{{}}\", rendered.join(\"|\")); \
         }}",
    );

    harness().compile(&wrapped, "main_bin", HarnessLinkMode::WithV3Compiler)
}

fn roundtrip_origin_labels(bin_path: &Path, program_source: &str, file_name: &str) -> String {
    let run = Command::new(bin_path)
        .arg(program_source)
        .arg(file_name)
        .output()
        .expect("run compiled binary");
    assert!(
        run.status.success(),
        "compiled binary failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

/// Helper: regenerate the checked-in generated module. Run with
/// `cargo test --test m2_lens_provenance_migration_test -- --ignored
/// emit_lens_provenance_snapshot` whenever `lens_provenance.dag`
/// changes. The hermetic snapshot check above then confirms the
/// update landed on disk.
#[test]
#[ignore]
fn emit_lens_provenance_snapshot() {
    const OUT_REL: &str = "src/v3/compiler/src/lens_provenance_generated.rs";
    assert!(
        v3_compiler::generated_files::GENERATED_FILES.contains(&OUT_REL),
        "SG-0 producer manifest mismatch: `{OUT_REL}` is not in GENERATED_FILES — \
         add it to REGEN_OUTPUTS in src/v3/compiler/build.rs."
    );
    let fresh = emit_lens_module();
    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lens_provenance_generated.rs");
    std::fs::write(&out_path, fresh).expect("write lens_provenance_generated.rs");
    println!("wrote {}", out_path.display());
}

#[test]
fn lens_provenance_dag_compiles_cleanly() {
    let dag = match compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref()) {
        Ok(dag) => dag,
        Err(v3_compiler::CompileError::Semantic(dag)) => {
            panic!(
                "lens_provenance.dag produced {} diagnostic(s): {:#?}",
                dag.diagnostics().len(),
                dag.diagnostics()
            );
        }
        Err(other) => panic!("lens_provenance.dag failed to compile: {other:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "lens_provenance.dag should compile without diagnostics, got {:#?}",
        dag.diagnostics()
    );
}

#[test]
fn lens_provenance_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_lens_module();
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module().trim(),
        "checked-in generated module is stale; regenerate lens_provenance_generated.rs from lens_provenance.dag"
    );
}

#[test]
fn lens_provenance_generated_module_clone_count_is_ratcheted() {
    // Ratchet: the emitted lens body allocates defensively in a handful
    // of spots (list-building helpers and Behavior clone in the lookup
    // helpers). If a refactor regresses clone generation, this ratchet
    // tightens the budget rather than letting it silently grow.
    const MAX_CLONE_CALLS: usize = 4;

    let fresh = emit_lens_module();
    let clone_calls = clone_call_count(&fresh);
    assert!(
        clone_calls <= MAX_CLONE_CALLS,
        "generated lens clone count regressed: observed {clone_calls}, ratchet allows at most {MAX_CLONE_CALLS}",
    );
}

fn handwritten_origin_label(dag: &Dag, port: PortId) -> &'static str {
    match dag.port(port).produced_by {
        None => "Source",
        Some(producer) => match dag.node(producer) {
            Behavior::Value(_) => "Source",
            Behavior::Transform(_) => "Computed",
            Behavior::Branch(_) => "Selected",
            Behavior::Loop(_) => "Accumulated",
            Behavior::Bind(_) => "Source",
        },
    }
}

fn render_handwritten_oracle(program_source: &str, file_name: &str) -> String {
    let dag = compile_to_dag(program_source, file_name).expect("program compiles");
    let mut rendered: Vec<String> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .map(|bind| {
            format!(
                "{}:{}",
                bind.name,
                handwritten_origin_label(&dag, bind.value)
            )
        })
        .collect();
    rendered.sort();
    rendered.join("|")
}

/// Cross-process receipt: the emitted lens module links against the
/// `v3_compiler` crate and runs under rustc on a representative
/// fixture that exercises all four producing Behaviors
/// (Value/Source, Transform/Computed, Branch/Selected,
/// Loop/Accumulated) plus Bind-value passthrough. Agreement is gated
/// against the handwritten oracle (not a hard-coded string) because
/// `compile_to_dag` also pulls in bootstrap `std/` binds whose names
/// are not stable to pin here. Per-variant coverage against
/// hand-built Dags lives in `lens_provenance::tests`.
#[test]
fn lens_provenance_dag_runs_end_to_end_via_rustc_harness() {
    let module = emit_lens_module();
    let bin_path = build_roundtrip_harness(&module);
    let source = "let a = 1\n\
         let b = a + 1\n\
         let c = if 1 > 0 then 42 else 0\n\
         fn count(n: Int) -> Int = if n == 0 then 0 else n + count(n - 1)\n\
         let d = count(3)";
    let file_name = "mixed_origins.v3";
    let oracle = render_handwritten_oracle(source, file_name);
    let emitted = roundtrip_origin_labels(&bin_path, source, file_name);
    assert_eq!(
        emitted, oracle,
        "emitted provenance lens should match handwritten oracle on the cross-process receipt fixture",
    );
}
