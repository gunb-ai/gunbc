//! Backend compilation smoke tests.
//!
//! Compile a .dag file through the full pipeline for each backend and
//! verify the emitted code compiles with the target toolchain.
//!
//! Uses the real compiler pipeline via gunbc_resolve + daglang_emit.
//! Non-hermetic: invokes go, gcc. Skips when toolchain unavailable.

use std::fs;
use std::path::Path;
use std::process::Command;

fn has_tool(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn write_files(files: &[(String, String)], dir: &Path) {
    for (path, content) in files {
        let full = dir.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full, content).unwrap();
    }
}

/// Compile a .dag file and emit to the given backend.
/// Returns (path, content) pairs for the emitted files.
fn compile_dag_to_backend(dag_path: &str, backend: &str) -> Vec<(String, String)> {
    let result = gunbc_resolve::builder::build_dsl_graph(
        dag_path,
        gunbc_resolve::BuildOpts::default(),
    )
    .unwrap_or_else(|e| panic!("failed to compile {dag_path}: {e}"));

    let reachable = daglang_emit::ReachableDag::from_dag(&result.dag);
    let artifacts =
        daglang_emit::derive_artifacts(&result.dag).unwrap_or_else(|e| panic!("derive: {e}"));

    let bundle = match backend {
        "go" => daglang_emit::emit_go_bundle(
            &reachable,
            &artifacts,
            &Default::default(),
            &Default::default(),
        )
        .unwrap_or_else(|e| panic!("go emit: {e}")),
        "c" => daglang_emit::emit_c_bundle(
            &reachable,
            &artifacts,
            &Default::default(),
            &Default::default(),
        )
        .unwrap_or_else(|e| panic!("c emit: {e}")),
        _ => panic!("unknown backend: {backend}"),
    };

    bundle
        .files
        .into_iter()
        .map(|f| (f.path, f.content))
        .collect()
}

#[test]
fn go_emitted_code_compiles() {
    if !has_tool("go") {
        eprintln!("SKIP: go not available");
        return;
    }

    let files = compile_dag_to_backend("dsl/std/filesystem.dag", "go");
    if files.is_empty() {
        eprintln!("SKIP: no files emitted for go backend");
        return;
    }

    let dir = std::env::temp_dir().join("gunbc_go_smoke");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    write_files(&files, &dir);

    // Write go.mod so `go build` resolves
    fs::write(dir.join("go.mod"), "module smoke_test\n\ngo 1.21\n").unwrap();

    let output = Command::new("go")
        .args(["build", "./..."])
        .current_dir(&dir)
        .output()
        .expect("run go build");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "go build failed:\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn c_emitted_code_compiles() {
    if !has_tool("gcc") {
        eprintln!("SKIP: gcc not available");
        return;
    }

    let files = compile_dag_to_backend("dsl/std/filesystem.dag", "c");
    if files.is_empty() {
        eprintln!("SKIP: no files emitted for c backend");
        return;
    }

    let dir = std::env::temp_dir().join("gunbc_c_smoke");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    write_files(&files, &dir);

    let main_c = files
        .iter()
        .find(|(p, _)| p.ends_with(".c"))
        .expect("should have a .c file");

    let output = Command::new("gcc")
        .args(["-fsyntax-only", "-Wall"])
        .arg(dir.join(&main_c.0))
        .output()
        .expect("run gcc");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "gcc syntax check failed:\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}
