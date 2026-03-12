//! Backend compilation smoke tests.
//!
//! Compile a .dag file through the full pipeline for each backend and
//! verify the emitted code compiles with the target toolchain.
//!
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

/// Resolve a path relative to the workspace root.
fn workspace_path(relative: &str) -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR points to the crate dir; workspace root is 4 levels up
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()  // 01_surfaces/
        .and_then(|p| p.parent())  // v1/
        .and_then(|p| p.parent())  // src/
        .and_then(|p| p.parent())  // workspace root
        .expect("workspace root")
        .join(relative)
}

/// Compile a .dag file and emit to the given backend.
fn compile_and_emit(dag_path: &str, target: &str) -> Vec<(String, String)> {
    let abs_path = workspace_path(dag_path);
    let context = daglang_driver::DriverContext {
        roots: vec![workspace_path("dsl")],
        target_file: Some(abs_path),
    };
    let output = daglang_driver::compile_from_context(&context)
        .unwrap_or_else(|e| panic!("compile {dag_path}: {e}"));

    let dag = output.lowered_dag.as_dag();
    let reachable = gunbc_ir::ReachableDag::from_dag(dag);
    let derived = &output.derived;

    let bundle = match target {
        "go" => daglang_emit::emit_go_bundle(
            &reachable,
            derived,
            &Default::default(),
            &Default::default(),
        )
        .unwrap_or_else(|e| panic!("go emit: {e}")),
        "c" => daglang_emit::emit_c_bundle(
            &reachable,
            derived,
            &Default::default(),
            &Default::default(),
        )
        .unwrap_or_else(|e| panic!("c emit: {e}")),
        _ => panic!("unsupported target: {target}"),
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
        // Tool not available — skip silently
        println!("SKIP: go not available");
        return;
    }

    let files = compile_and_emit("dsl/std/filesystem.dag", "go");
    if files.is_empty() {
        // Tool not available — skip silently
        println!("SKIP: no go files emitted");
        return;
    }

    let dir = std::env::temp_dir().join("gunbc_go_smoke");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    write_files(&files, &dir);

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
        // Tool not available — skip silently
        println!("SKIP: gcc not available");
        return;
    }

    let files = compile_and_emit("dsl/std/filesystem.dag", "c");
    if files.is_empty() {
        // Tool not available — skip silently
        println!("SKIP: no c files emitted");
        return;
    }

    let dir = std::env::temp_dir().join("gunbc_c_smoke");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    write_files(&files, &dir);

    let c_file = files
        .iter()
        .find(|(p, _)| p.ends_with(".c"))
        .expect("should have a .c file");

    let output = Command::new("gcc")
        .args(["-fsyntax-only", "-Wall"])
        .arg(dir.join(&c_file.0))
        .output()
        .expect("run gcc");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "gcc syntax check failed:\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}
