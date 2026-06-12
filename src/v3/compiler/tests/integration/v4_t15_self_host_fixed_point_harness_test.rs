//! **T-15** bin/main.dag execution + self-host fixed-point validation harness
//! (former TASKS.md §T-15 — ledger deleted; operator-directive 2026-05-29).
//!
//! Fixed-point bar: **stage1 emitted == stage2 emitted** (`FixptStage1Stage2` in
//! `workflow/bootstrap.dag`), **not** stage0==stage1 (stage0 is v2-emission-style).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + `claim_t15_self_host_fixed_point.dag`
//! + matching `EXPECTED_HAND_AUTHORED_TEST` / INVARIANTS table row land in the same PR.
//!
//! **Dissolution:** retire when `.dag` `TestClaim` eval + bootstrap B1 `content_hash` pins
//! replace host structural receipts (`ROADMAP` T-PB-B / `pb_rust_tests_outside_residual_zero`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

const MAIN_DAG: &str = include_str!("../../../../v4/bin/main.dag");
const MAIN_DAG_PATH: &str = "src/v4/bin/main.dag";
const BOOTSTRAP_DAG: &str = include_str!("../../../../v4/workflow/bootstrap.dag");
const BOOTSTRAP_PATH: &str = "src/v4/workflow/bootstrap.dag";
const SELF_HOST_DAG: &str = include_str!("../../../../v4/compiler/self_host.dag");
const SELF_HOST_PATH: &str = "src/v4/compiler/self_host.dag";
const T15_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/self_host/claim_t15_self_host_fixed_point.dag");
const T15_CLAIM_PATH: &str = "src/v4/test/claim/self_host/claim_t15_self_host_fixed_point.dag";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn gunbc_release_binary(root: &Path) -> PathBuf {
    root.join("target").join("release").join("gunbc")
}

fn assert_parseable(source: &str, path: &str) {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"));
}

fn assert_main_trampoline_authority() {
    assert!(
        MAIN_DAG.contains("main_rs_trampoline_authority"),
        "{MAIN_DAG_PATH}: expected trampoline data id"
    );
    assert!(
        MAIN_DAG.contains(r#"include!(\"v4_main_generated.rs\");"#),
        "{MAIN_DAG_PATH}: expected trampoline include! spelling in String literal"
    );
    for name in [
        "stub_stage1_emitted_rust_digest_placeholder",
        "stub_stage2_emitted_rust_digest_placeholder",
    ] {
        assert!(
            MAIN_DAG.contains(name),
            "{MAIN_DAG_PATH}: expected digest stub data id {name}"
        );
    }
}

/// Bootstrap encodes fixpt over stage1/stage2 binaries and requires digest convergence on
/// `self1.produces_hash` (stage1) — not a stage0==stage1 check.
fn assert_bootstrap_fixpt_is_stage1_stage2_not_stage0() {
    assert!(
        BOOTSTRAP_DAG.contains("type FixptStage1Stage2")
            && BOOTSTRAP_DAG.contains("left: Symbol")
            && BOOTSTRAP_DAG.contains("right: Symbol"),
        "{BOOTSTRAP_PATH}: FixptStage1Stage2 must name left/right stage symbols"
    );
    assert!(
        BOOTSTRAP_DAG.contains("produces: v4_stage1_binary")
            && BOOTSTRAP_DAG.contains("compiled_by: v4_stage0_binary"),
        "{BOOTSTRAP_PATH}: self0 must emit stage1 from stage0 (not the fixpt pair)"
    );
    assert!(
        BOOTSTRAP_DAG.contains("produces: v4_stage2_binary")
            && BOOTSTRAP_DAG.contains("compiled_by: v4_stage1_binary"),
        "{BOOTSTRAP_PATH}: self1 must emit stage2 from stage1"
    );
    assert!(
        BOOTSTRAP_DAG.contains("p.fixpt.left_hash.digest == p.self0.produces_hash.digest")
            && BOOTSTRAP_DAG.contains("p.fixpt.right_hash.digest == p.self1.produces_hash.digest")
            && BOOTSTRAP_DAG
                .contains("p.self0.produces_hash.digest == p.self1.produces_hash.digest"),
        "{BOOTSTRAP_PATH}: bootstrap_plan_well_formed must require stage1==stage2 digest equality"
    );
    assert!(
        !BOOTSTRAP_DAG.contains("p.fixpt.left_hash.digest == p.seed.produces_hash.digest"),
        "{BOOTSTRAP_PATH}: fixpt must not equate stage1 directly to seed/stage0 output"
    );
}

fn assert_self_host_runner_shape_fail_closed() {
    assert!(
        SELF_HOST_DAG.contains("fn self_host_fixed_point_validate(")
            && SELF_HOST_DAG.contains("Rejected {")
            && SELF_HOST_DAG.contains("self_host_runner_not_realized"),
        "{SELF_HOST_PATH}: runner must stay fail-closed until execution substrate lands"
    );
}

fn assert_t15_claim_wiring() {
    assert!(
        T15_CLAIM_DAG.contains("data claim_t15_self_host_fixed_point: TestClaim = EqualsClaim")
            && T15_CLAIM_DAG.contains("stub_stage1_emitted_rust_digest_placeholder")
            && T15_CLAIM_DAG.contains("stub_stage2_emitted_rust_digest_placeholder"),
        "{T15_CLAIM_PATH}: EqualsClaim must name stage1/stage2 digest operands from main.dag"
    );
}

/// v2 semantic compile with `bin/main.dag` as the entry root (deps = full `src/v4` minus duplicate entry).
fn try_v2_compile_main_dag_entry(root: &Path) {
    let bin = gunbc_release_binary(root);
    if !bin.exists() {
        // Optional receipt: build `v2-compiler --release` locally to exercise entry-root compile.
        return;
    }

    let out = std::env::temp_dir().join(format!("v4-t15-main-entry-{}", std::process::id()));
    let entry_root = out.join("entry");
    let deps_root = out.join("deps");
    fs::remove_dir_all(&out).ok();
    fs::create_dir_all(entry_root.join("v4/bin")).expect("entry bin dir");
    fs::create_dir_all(deps_root.join("v4/bin")).expect("deps bin dir");

    fs::copy(
        root.join("src/v4/bin/main.dag"),
        entry_root.join("v4/bin/main.dag"),
    )
    .expect("copy main.dag entry");
    copy_dir_all(&root.join("src/v4"), &deps_root.join("v4")).expect("copy src/v4 deps");
    fs::remove_file(deps_root.join("v4/bin/main.dag"))
        .expect("remove duplicate main.dag from deps");

    let compile_log = out.join("compile.log");
    let status = Command::new(&bin)
        .args([
            "compile",
            "--source-root",
            entry_root.to_str().expect("entry utf8"),
            "--source-root",
            deps_root.to_str().expect("deps utf8"),
            "--output-dir",
            out.join("rust-out").to_str().expect("out utf8"),
            "--target",
            "rust",
        ])
        .output()
        .unwrap_or_else(|e| panic!("spawn gunbc compile: {e}"));
    let mut log_bytes = status.stdout.clone();
    log_bytes.extend_from_slice(&status.stderr);
    fs::write(&compile_log, &log_bytes).ok();

    assert!(
        status.status.success(),
        "v2 compile main.dag entry failed (log: {}):\n{}",
        compile_log.display(),
        String::from_utf8_lossy(&status.stderr)
    );
    let combined = String::from_utf8_lossy(&log_bytes);
    assert!(
        combined.contains("compiled:") && combined.contains("0 diagnostics"),
        "main.dag entry compile must emit a clean compiled receipt (log: {})",
        compile_log.display()
    );
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// CI gate name: `cargo test t_15_self_host_fixed_point`.
#[test]
fn t_15_self_host_fixed_point() {
    assert_parseable(MAIN_DAG, MAIN_DAG_PATH);
    assert_main_trampoline_authority();
    assert_parseable(BOOTSTRAP_DAG, BOOTSTRAP_PATH);
    assert_bootstrap_fixpt_is_stage1_stage2_not_stage0();
    assert_parseable(SELF_HOST_DAG, SELF_HOST_PATH);
    assert_self_host_runner_shape_fail_closed();
    assert_parseable(T15_CLAIM_DAG, T15_CLAIM_PATH);
    assert_t15_claim_wiring();
    try_v2_compile_main_dag_entry(&workspace_root());
}
