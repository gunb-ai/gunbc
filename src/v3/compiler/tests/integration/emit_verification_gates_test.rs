//! **Layer:** integration
//!
//! R3 §1.8 multi-target emit verification gates — arbitrary user programs
//! (the shared `PROGRAM_FIXTURES` corpus) must pass each target's declared
//! `CleanEmissionContract.post_emit_verifier`, not only the self-host M1
//! `src/v4` rust-emit probe.
//!
//! **Gap closed:** M1 probes v2 `--target rust src/v4` + `cargo check` on the
//! emitted self-host tree. `Compiles` TestClaims only check substrate lowering.
//! This gate ties arbitrary `.v3` programs to the E-5 contract verifiers for
//! Rust (`rustc -D warnings`), Go (`gofmt -l`), and Python (`py_compile`).
//!
//! **CI authority:** `r1c_e_emit_gates.template.dag` claim
//! `program_fixtures_post_emit_clean_all_targets` (`ExecuteCommand` →
//! `check_program_fixtures_post_emit_clean_all_targets`) via
//! `t_pb_b_1_dag_runner_test::r1c_e_emit_gates_suite_passes_through_runner`.
//! The `#[ignore]` tests below are optional local receipts when all toolchains
//! are on PATH.

use std::time::{SystemTime, UNIX_EPOCH};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust_roundtrip_fixtures::{
    ProgramFixture, GO_EMIT_EXCLUDE, PROGRAM_FIXTURES, PYTHON_EMIT_EXCLUDE,
};
use v3_compiler::post_emit_verifier::{
    verify_program_emitted_source, verify_program_emitted_source_all_targets,
    EmitVerificationTarget,
};
use v3_compiler::CompileError;

fn scratch_dir(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "gunbc_emit_verif_{label}_{}_{}",
        std::process::id(),
        stamp
    ))
}

fn compile_fixture(fixture: &ProgramFixture) -> v3_compiler::Dag {
    let file = format!("{}_emit_verification.v3", fixture.name);
    match compile_to_dag(fixture.source, &file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "fixture `{}` should compile cleanly, got diagnostics: {:?}",
                fixture.name,
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(err) => panic!("fixture `{}` compile failed: {err:?}", fixture.name),
    }
}

#[allow(dead_code)] // used by `#[ignore]` toolchain tests below
fn fixture_supports_target(fixture: &ProgramFixture, target: EmitVerificationTarget) -> bool {
    match target {
        EmitVerificationTarget::Rust => true,
        EmitVerificationTarget::Go => !GO_EMIT_EXCLUDE.contains(&fixture.name),
        EmitVerificationTarget::Python => !PYTHON_EMIT_EXCLUDE.contains(&fixture.name),
    }
}

/// Gate: every corpus program passes all applicable Shape-A post-emit verifiers.
#[test]
#[ignore = "requires rustc, gofmt, and python3 on PATH"]
fn arbitrary_program_fixtures_pass_post_emit_verifier_all_targets() {
    let mut failures = Vec::new();
    for fixture in PROGRAM_FIXTURES {
        let dag = compile_fixture(fixture);
        let scratch = scratch_dir(fixture.name);
        if let Err(msg) = verify_program_emitted_source_all_targets(&dag, &scratch, fixture.name) {
            failures.push(msg);
        }
        let _ = std::fs::remove_dir_all(&scratch);
    }
    if !failures.is_empty() {
        panic!(
            "{} PROGRAM_FIXTURES failed post_emit_verifier:\n{}",
            failures.len(),
            failures.join("\n---\n")
        );
    }
}

/// Per-target receipt so a missing toolchain surfaces as a single-target failure.
#[test]
#[ignore = "requires rustc on PATH"]
fn arbitrary_program_fixtures_pass_rust_post_emit_verifier() {
    for fixture in PROGRAM_FIXTURES {
        let dag = compile_fixture(fixture);
        let scratch = scratch_dir(&format!("{}_rust", fixture.name));
        verify_program_emitted_source(&dag, EmitVerificationTarget::Rust, &scratch, fixture.name)
            .unwrap_or_else(|e| panic!("fixture `{}`: {e}", fixture.name));
        let _ = std::fs::remove_dir_all(&scratch);
    }
}

#[test]
fn emit_verification_target_labels_are_stable() {
    assert_eq!(EmitVerificationTarget::Rust.label(), "rust");
    assert_eq!(EmitVerificationTarget::Go.label(), "go");
    assert_eq!(EmitVerificationTarget::Python.label(), "python");
    assert_eq!(EmitVerificationTarget::ALL.len(), 3);
}
