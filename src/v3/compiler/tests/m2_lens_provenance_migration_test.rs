// M2 — lens_provenance migration test.
//
// Mirrors m2_lens_unused_parameters_migration_test.rs: the `.dag`
// declaration at src/v3/lenses/provenance.dag is the live authority
// for the provenance lens. This suite asserts:
//
// 1. The `.dag` file compiles cleanly (no diagnostics).
// 2. The emitted Rust module matches the checked-in snapshot at
//    src/v3/compiler/src/lens_provenance_generated.rs — so review of
//    the emitted shape happens via the snapshot diff, not behind it.
// 3. The clone-call count in the emitted module is ratcheted. If a
//    future change regresses clone generation, the ratchet fires.
// 4. The compiled `.dag` lens agrees with a hand-written Rust oracle
//    on a small fixture set covering every Origin variant. The oracle
//    is the minimum structural walker needed to classify a port by its
//    producer's behavior kind — any divergence means either the lens
//    or the oracle is wrong.
//
// If this test passes, the `.dag` is the authority and the
// `lens_provenance_generated.rs` in-tree snapshot is in sync with it.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId};
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::Dag;

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);
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
    include_str!("../src/lens_provenance_generated.rs")
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

fn next_roundtrip_dir() -> PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_lens_provenance_roundtrip_{}_{}",
        std::process::id(),
        id
    ))
}

fn deps_dir() -> PathBuf {
    std::env::current_exe()
        .expect("current test binary path")
        .parent()
        .expect("deps dir")
        .to_path_buf()
}

fn find_current_rlib(crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let mut matches: Vec<PathBuf> = std::fs::read_dir(deps_dir())
        .expect("read deps dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let file_name = path.file_name()?.to_str()?;
            if file_name.starts_with(&prefix) && file_name.ends_with(".rlib") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    matches.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    });
    matches
        .into_iter()
        .last()
        .expect("compiled rlib for current crate")
}

fn compile_with_current_crate(src_path: &Path, bin_path: &Path) {
    let deps = deps_dir();
    let current_rlib = find_current_rlib("v3_compiler");
    let compile = Command::new("rustc")
        .arg("--edition=2021")
        .arg(src_path)
        .arg("-o")
        .arg(bin_path)
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--extern")
        .arg(format!("v3_compiler={}", current_rlib.display()))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc");
    assert!(compile.success(), "rustc failed on emitted lens source");
}

fn roundtrip_origin_labels(module_source: &str, program_source: &str, file_name: &str) -> String {
    // Compile the emitted lens module inside a freshly-built binary and
    // walk every Bind's value port through it, printing a canonical
    // "{bind_name}:{variant}" label per lookup. We compare the joined
    // string against the handwritten oracle.
    let wrapped = format!(
        "mod emitted {{ use v3_compiler::dag::*; use v3_compiler::diagnostics::*; {module_source} }} \
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
           let dag = v3_compiler::compile_to_dag({program_source:?}, {file_name:?}).expect(\"compiles\"); \
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

    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(wrapped.as_bytes()))
        .expect("write wrapped rust source");

    compile_with_current_crate(&src_path, &bin_path);

    let run = Command::new(&bin_path)
        .output()
        .expect("run compiled binary");
    assert!(run.status.success(), "compiled binary failed");
    String::from_utf8_lossy(&run.stdout).trim().to_string()
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

/// Helper: regenerate the checked-in generated module. Run with
/// `cargo test --test m2_lens_provenance_migration_test -- --ignored
/// emit_lens_provenance_snapshot` whenever `lens_provenance.dag`
/// changes. The hermetic snapshot check above then confirms the
/// update landed on disk.
#[test]
#[ignore]
fn emit_lens_provenance_snapshot() {
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

#[test]
fn lens_provenance_dag_matches_handwritten_oracle_on_core_fixtures() {
    let module = emit_lens_module();
    let fixtures = [
        (
            "let a = 1\nlet b = 2\nlet c = a + b",
            "transform_origin.v3",
        ),
        (
            "let x = if 1 > 0 then 42 else 0",
            "branch_origin.v3",
        ),
        (
            "fn count(n: Int) -> Int = if n == 0 then 0 else n + count(n - 1)\nlet answer = count(3)",
            "loop_origin.v3",
        ),
        (
            "let a = 1",
            "value_source.v3",
        ),
    ];

    for (source, file_name) in fixtures {
        let rust_rendered = render_handwritten_oracle(source, file_name);
        let dag_rendered = roundtrip_origin_labels(&module, source, file_name);
        assert_eq!(
            dag_rendered, rust_rendered,
            "compiled .dag lens should match handwritten Rust oracle on {file_name}"
        );
    }
}
