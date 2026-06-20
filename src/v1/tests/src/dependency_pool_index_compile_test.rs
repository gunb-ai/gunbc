//! Execution receipts for `gunbc compile --dependency-pool-index` (primary-precedence gate path).

use crate::helpers::workspace_root;
use std::fs;
use std::process::Command;

fn gunbc_bin() -> Option<std::path::PathBuf> {
    let gunbc = workspace_root().join("target/release/gunbc");
    gunbc.is_file().then_some(gunbc)
}

fn compile_with_roots(
    gunbc: &std::path::Path,
    ws: &std::path::Path,
    source_roots: &[&std::path::Path],
    pool_index: &str,
    output_dir: &std::path::Path,
) -> std::process::Output {
    let mut cmd = Command::new(gunbc);
    cmd.arg("compile")
        .arg("--dependency-pool-index")
        .arg(pool_index)
        .arg("--output-dir")
        .arg(output_dir)
        .arg("--target")
        .arg("rust")
        .current_dir(ws);
    for root in source_roots {
        cmd.arg("--source-root").arg(root);
    }
    cmd.output().expect("spawn gunbc compile")
}

#[test]
fn primary_precedence_within_root_duplicate_module_path_panics() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::write(
        root.join("a.dag"),
        "module duplicate.within.root\nfn a() -> Int { 1 }\n",
    )
    .expect("write a.dag");
    fs::write(
        root.join("b.dag"),
        "module duplicate.within.root\nfn b() -> Int { 2 }\n",
    )
    .expect("write b.dag");
    let out = tempfile::tempdir().expect("out dir");
    let output = compile_with_roots(&gunbc, &ws, &[root], "primary-precedence", out.path());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "expected within-root duplicate to fail closed; got success"
    );
    assert!(
        combined.contains("duplicate module path") && combined.contains("within source root"),
        "expected within-root duplicate panic message; got:\n{combined}"
    );
}

#[test]
fn strict_dependency_pool_index_panics_on_cross_root_extdeps_shell_collision() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let out = tempfile::tempdir().expect("out dir");
    let output = compile_with_roots(
        &gunbc,
        &ws,
        &[&ws.join("dsl"), &ws.join("src/v2")],
        "strict",
        out.path(),
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "strict mode must panic on cross-root extdeps.shell collision"
    );
    assert!(
        combined.contains("extdeps.shell"),
        "expected extdeps.shell collision diagnostic; got:\n{combined}"
    );
}

#[test]
fn primary_precedence_keeps_dsl_extdeps_shell_for_dsl_only_services() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let entry_root = tmp.path();
    fs::write(
        entry_root.join("probe.dag"),
        "module test.probe.dsl_shell_env_primary\n\
         import extdeps.shell\n\
         fn probe() -> String? {\n\
           shell.Env.Get(name: \"PATH\")\n\
         }\n",
    )
    .expect("write probe.dag");
    let out = tempfile::tempdir().expect("out dir");
    let output = compile_with_roots(
        &gunbc,
        &ws,
        &[entry_root, &ws.join("dsl"), &ws.join("src/v2")],
        "primary-precedence",
        out.path(),
    );
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "shell.Env.Get probe must compile under primary-precedence (dsl extdeps.shell wins over v2 overlay); exit {:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            output.status.code()
        );
    }
}
