//! Hermetic pass/fail fixtures over live `v1-compiler compile` (candidate-B bridge transport).
//!
//! Requires `gunbc` on PATH or at `target/{debug,release}/gunbc` relative to workspace root,
//! or `V2_COMPILER` env override (same contract as `.github/ci-floor/v2-rust-full-tree-emit-probe.sh`).

use std::fs;
use std::path::{Path, PathBuf};

use compile_host_runner::{compile_accepted, run_compile_host_v2, CompileHostTransportInputs};

const PASS_FIXTURE: &str = r#"// compile_host_runner hermetic pass fixture
module v2.test.compile_host_fixture_pass

import v2.std.logic { Bool }

data witness_ok: Bool = true
"#;

const FAIL_FIXTURE: &str = r#"// compile_host_runner hermetic fail fixture
module v2.test.compile_host_fixture_fail

import v2.std.logic { Bool }

data broken: Bool = definitely_not_valid_syntax!!!
"#;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn resolve_v2_compiler() -> PathBuf {
    if let Ok(path) = std::env::var("V2_COMPILER") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return path;
        }
    }
    let root = workspace_root();
    for profile in ["release", "debug"] {
        let candidate = root.join("target").join(profile).join("gunbc");
        if candidate.is_file() {
            return candidate;
        }
    }
    panic!(
        "gunbc not found: build with `cargo build -p v1-compiler --release --bin gunbc` \
         or set V2_COMPILER"
    );
}

fn write_entry_fixture(work: &Path, rel: &str, source: &str) {
    let path = work.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture parent dir");
    }
    fs::write(path, source).expect("write fixture");
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// v2-only dependency closure — avoid scanning `src/v3` (duplicate module paths in fixtures).
fn materialize_v4_deps_root(work: &Path) -> PathBuf {
    let deps_root = work.join("deps");
    copy_dir_all(&workspace_root().join("src/v2"), &deps_root.join("v2"))
        .expect("copy src/v2 deps");
    deps_root
}

fn run_fixture_compile(
    entry_fixture_rel: &str,
    source: &str,
) -> compile_host_runner::CompileHostRunReceipt {
    let work = compile_host_runner::default_work_dir("compile_host_hermetic");
    fs::remove_dir_all(&work).ok();
    let entry_root = work.join("entry");
    let deps_root = materialize_v4_deps_root(&work);
    let output_dir = work.join("out");
    write_entry_fixture(&entry_root, entry_fixture_rel, source);

    let inputs = CompileHostTransportInputs {
        source_roots: vec![
            entry_root.display().to_string(),
            deps_root.display().to_string(),
        ],
        output_dir: output_dir.display().to_string(),
        target: "rust".into(),
    };
    let compiler = resolve_v2_compiler();
    run_compile_host_v2(&compiler, &inputs, &work.join("scratch"))
        .unwrap_or_else(|e| panic!("compile transport setup failed: {e}"))
}

#[test]
fn hermetic_compile_pass_fixture_accepted() {
    let receipt = run_fixture_compile("v2/test/compile_host_fixture_pass.dag", PASS_FIXTURE);
    assert!(
        compile_accepted(&receipt),
        "expected clean compile receipt; exit={:?} receipt={:?} log={:?}",
        receipt.exit,
        receipt.compiled_receipt,
        receipt.build_log.lines
    );
}

#[test]
fn hermetic_compile_fail_fixture_rejected() {
    let receipt = run_fixture_compile("v2/test/compile_host_fixture_fail.dag", FAIL_FIXTURE);
    assert!(
        !compile_accepted(&receipt),
        "expected compile failure or nonzero diagnostics; exit={:?} receipt={:?} log={:?}",
        receipt.exit,
        receipt.compiled_receipt,
        receipt.build_log.lines
    );
}
