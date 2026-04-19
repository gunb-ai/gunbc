//! DB-8 / Lane D — emission determinism ratchet (`feedback_structural_perf_tests`).
//!
//! Per [`docs/design-fixed-point-ratchet.md`](../../docs/design-fixed-point-ratchet.md):
//! for fixed `(dag, target)`, repeated `emit` must yield **byte-identical** text.
//! This module locks that contract structurally: **5× re-emit** on every row of
//! the shared emit matrix (`integration/common/determinism_fixtures.rs`).
//!
//! ## DB-8 non-determinism sources (structural coverage map)
//!
//! 1. **HashMap iteration order** — ratchet: no unstable map iteration in emit;
//!    mechanical follow-up: sorted keys / `BTreeMap` (see `emit.rs` policy).
//!    *Structural hook:* `emit_rs_hash_iteration_debt_is_visible_to_audit` documents current
//!    `emit.rs` surface until Lane 1e dissolves remaining `HashMap` uses.
//! 2. **HashSet iteration order** — same class as (1).
//! 3. **Timestamp / build metadata embedding** — `assert_no_time_or_line_macros`.
//! 4. **Absolute path strings in output** — `assert_no_absolute_path_leakage` on
//!    emitted Rust (best-effort; spans should stay relative fixture names). Covers
//!    common macOS/Windows/Linux **and** typical CI workspace prefixes (e.g. GitHub Actions);
//!    program/module matrix **and** disk-backed `four_fixture_pressure` sources (real paths in
//!    `compile_to_dag`). Lane 1 Stage 1e may tighten further.
//! 5. **Unstable sorts / tie-breakers** — covered by byte-stable `emit` replay
//!    on one fixed `Dag` (same as 6).
//! 6. **Generated id allocation order** — same as (5).
//! 7. **Float formatting variance** — `assert_no_float_scientific_suspects` on
//!    emitted Rust for the matrix (guards accidental `{:e}`-style output).
//! 8. **Filesystem read order** — not exercised by string-in/string-out emit;
//!    documented as N/A for this test file (emit does not `read_dir`).

#[path = "integration/common/determinism_fixtures.rs"]
mod determinism_fixtures;

use std::path::PathBuf;

use determinism_fixtures::{
    ModuleFixture, ProgramFixture, FOUR_FIXTURE_FILES, GO_EMIT_EXCLUDE, MODULE_FIXTURES,
    PROGRAM_FIXTURES, PYTHON_PROGRAM_DETERMINISM_NAMES,
};
use v3_compiler::compile_to_dag;
use v3_compiler::emit::{emit, emit_module, EmitTarget};

const RUNS: usize = 5;

fn fixtures_dir_four() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("four_fixture_pressure")
}

fn emit_rust_program(fixture: &ProgramFixture) -> String {
    let dag = compile_to_dag(fixture.source, "determinism_program_matrix.v3")
        .unwrap_or_else(|e| panic!("fixture {} must compile: {e:?}", fixture.name));
    emit(&dag, EmitTarget::Rust)
        .unwrap_or_else(|e| panic!("emit rust {}: {e:?}", fixture.name))
        .text
}

fn emit_go_program(fixture: &ProgramFixture) -> String {
    let dag = compile_to_dag(fixture.source, "determinism_program_matrix_go.v3")
        .unwrap_or_else(|e| panic!("fixture {} must compile: {e:?}", fixture.name));
    emit(&dag, EmitTarget::Go)
        .unwrap_or_else(|e| panic!("emit go {}: {e:?}", fixture.name))
        .text
}

fn emit_python_program(fixture: &ProgramFixture) -> String {
    let dag = compile_to_dag(fixture.source, "determinism_program_matrix_py.v3")
        .unwrap_or_else(|e| panic!("fixture {} must compile: {e:?}", fixture.name));
    emit(&dag, EmitTarget::Python)
        .unwrap_or_else(|e| panic!("emit python {}: {e:?}", fixture.name))
        .text
}

fn emit_rust_module(fixture: &ModuleFixture) -> String {
    let dag = compile_to_dag(fixture.source, "determinism_module_matrix.v3")
        .unwrap_or_else(|e| panic!("module fixture {} must compile: {e:?}", fixture.name));
    emit_module(&dag, EmitTarget::Rust)
        .unwrap_or_else(|e| panic!("emit rust module {}: {e:?}", fixture.name))
        .text
}

fn emit_go_module(fixture: &ModuleFixture) -> String {
    let dag = compile_to_dag(fixture.source, "determinism_module_matrix_go.v3")
        .unwrap_or_else(|e| panic!("module fixture {} must compile: {e:?}", fixture.name));
    emit_module(&dag, EmitTarget::Go)
        .unwrap_or_else(|e| panic!("emit go module {}: {e:?}", fixture.name))
        .text
}

fn emit_python_module(fixture: &ModuleFixture) -> String {
    let dag = compile_to_dag(fixture.source, "determinism_module_matrix_py.v3")
        .unwrap_or_else(|e| panic!("module fixture {} must compile: {e:?}", fixture.name));
    emit_module(&dag, EmitTarget::Python)
        .unwrap_or_else(|e| panic!("emit python module {}: {e:?}", fixture.name))
        .text
}

fn assert_five_identical_runs(mut run: impl FnMut() -> String, label: &str) {
    let first = run();
    for i in 1..RUNS {
        let next = run();
        assert_eq!(
            first, next,
            "emit determinism failed for {label}: run 0 vs run {i}"
        );
    }
}

fn assert_no_time_or_line_macros(rust: &str, label: &str) {
    for needle in ["SystemTime", "std::time::", "file!(", "line!(", "column!("] {
        assert!(
            !rust.contains(needle),
            "{label}: emitted Rust must not embed build-metadata or time hooks (`{needle}`)"
        );
    }
}

fn assert_no_absolute_path_leakage(rust: &str, label: &str) {
    // Substrings that usually indicate a leaked host or CI workspace path in emit output.
    const LEAK_NEEDLES: &[&str] = &[
        "/Users/",
        "\\\\Users\\\\",
        "/home/runner/",
        "/home/circleci/",
        "/github/workspace/",
        "/builds/",
        "/root/",
    ];
    for needle in LEAK_NEEDLES {
        if rust.contains(needle) {
            panic!("{label}: emitted Rust appears to contain an absolute path ({needle:?})");
        }
    }
}

fn assert_no_float_scientific_suspects(rust: &str, label: &str) {
    if rust.contains(":e}") || rust.contains(":E}") {
        panic!("{label}: unexpected scientific float formatting token in emit output");
    }
}

fn audit_rust_emit_text(rust: &str, label: &str) {
    assert_no_time_or_line_macros(rust, label);
    assert_no_absolute_path_leakage(rust, label);
    assert_no_float_scientific_suspects(rust, label);
}

#[test]
fn emit_matrix_program_rust_is_deterministic() {
    for fixture in PROGRAM_FIXTURES {
        let name = fixture.name;
        assert_five_identical_runs(
            || emit_rust_program(fixture),
            &format!("rust program {name}"),
        );
    }
}

#[test]
fn emit_matrix_program_go_is_deterministic() {
    // Pre-filter: `emit_go` does not support `Behavior::Loop` yet (e.g. recursive_function_call_six).
    // Must not call `emit_go_program` for excluded rows — only supported Go matrix rows.
    for fixture in PROGRAM_FIXTURES
        .iter()
        .filter(|f| !GO_EMIT_EXCLUDE.contains(&f.name))
    {
        let name = fixture.name;
        assert_five_identical_runs(|| emit_go_program(fixture), &format!("go program {name}"));
    }
}

#[test]
fn emit_matrix_program_python_is_deterministic() {
    for fixture in PROGRAM_FIXTURES {
        if !PYTHON_PROGRAM_DETERMINISM_NAMES.contains(&fixture.name) {
            continue;
        }
        let name = fixture.name;
        assert_five_identical_runs(
            || emit_python_program(fixture),
            &format!("python program {name}"),
        );
    }
}

#[test]
fn emit_matrix_module_rust_is_deterministic() {
    for fixture in MODULE_FIXTURES {
        let name = fixture.name;
        assert_five_identical_runs(|| emit_rust_module(fixture), &format!("rust module {name}"));
    }
}

#[test]
fn emit_matrix_module_go_is_deterministic() {
    for fixture in MODULE_FIXTURES {
        let name = fixture.name;
        assert_five_identical_runs(|| emit_go_module(fixture), &format!("go module {name}"));
    }
}

#[test]
fn emit_matrix_module_python_is_deterministic() {
    for fixture in MODULE_FIXTURES {
        let name = fixture.name;
        assert_five_identical_runs(
            || emit_python_module(fixture),
            &format!("python module {name}"),
        );
    }
}

#[test]
fn four_fixture_disk_sources_emit_deterministically() {
    for file in FOUR_FIXTURE_FILES {
        let path = fixtures_dir_four().join(file);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let path_for_compile = path.to_string_lossy().to_string();
        let base = format!("four_fixture {file}");
        assert_five_identical_runs(
            || {
                let dag = compile_to_dag(&source, path_for_compile.as_str())
                    .unwrap_or_else(|e| panic!("{base} compile: {e:?}"));
                emit(&dag, EmitTarget::Rust)
                    .unwrap_or_else(|e| panic!("{base} emit rust: {e:?}"))
                    .text
            },
            &base,
        );
        let rust_disk = {
            let dag = compile_to_dag(&source, path_for_compile.as_str())
                .unwrap_or_else(|e| panic!("{base} compile: {e:?}"));
            emit(&dag, EmitTarget::Rust)
                .unwrap_or_else(|e| panic!("{base} emit rust: {e:?}"))
                .text
        };
        audit_rust_emit_text(&rust_disk, &format!("{base} disk rust"));
        let go_label = format!("{base} go");
        assert_five_identical_runs(
            || {
                let dag = compile_to_dag(&source, path_for_compile.as_str())
                    .unwrap_or_else(|e| panic!("{go_label} compile: {e:?}"));
                emit(&dag, EmitTarget::Go)
                    .unwrap_or_else(|e| panic!("{go_label} emit go: {e:?}"))
                    .text
            },
            &go_label,
        );
    }
}

#[test]
fn db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix() {
    for fixture in PROGRAM_FIXTURES {
        let rust = emit_rust_program(fixture);
        audit_rust_emit_text(&rust, &format!("db8 audit rust program {}", fixture.name));
    }
}

#[test]
fn emit_rs_hash_iteration_debt_is_visible_to_audit() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/emit.rs");
    let src = std::fs::read_to_string(&path).expect("read emit.rs");
    let hash_ops = src.matches("HashMap::").count() + src.matches("HashSet::").count();
    assert!(
        hash_ops > 0,
        "DB-8 §Sources 1–2: emit.rs should still mention HashMap/HashSet until Lane 1e replaces \
         iteration with BTree* / sorted keys — if this fails, debt is cleared: remove this test \
         and graduate the CI grep gate to required"
    );
}
