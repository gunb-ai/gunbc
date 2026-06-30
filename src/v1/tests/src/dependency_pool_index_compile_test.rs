use crate::helpers::workspace_root;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn gunbc_bin() -> Option<std::path::PathBuf> {
    let gunbc = workspace_root().join("target/release/gunbc");
    gunbc.is_file().then_some(gunbc)
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("gunbc-{label}-{stamp}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn rm_rf(path: &std::path::Path) {
    if path.is_dir() {
        fs::remove_dir_all(path).expect("remove temp dir");
    }
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
    let root = temp_dir("within-root-dup");
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
    let out = temp_dir("within-root-dup-out");
    let output = compile_with_roots(&gunbc, &ws, &[&root], "primary-precedence", &out);
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
        combined.contains("duplicate module path"),
        "expected duplicate module path panic; got:\n{combined}"
    );
    rm_rf(&root);
    rm_rf(&out);
}

#[test]
fn strict_dependency_pool_index_panics_on_cross_root_extdeps_shell_collision() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let out = temp_dir("strict-cross-root-out");
    let output = compile_with_roots(
        &gunbc,
        &ws,
        &[&ws.join("dsl"), &ws.join("src/v2")],
        "strict",
        &out,
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
    rm_rf(&out);
}

#[test]
fn primary_precedence_keeps_dsl_extdeps_shell_for_dsl_only_services() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let entry_root = temp_dir("dsl-shell-primary-entry");
    fs::write(
        entry_root.join("probe.dag"),
        "module test.probe.dsl_shell_env_primary\n\
         import extdeps.shell\n\
         fn probe() -> String? {\n\
           shell.Env.Get(name: \"PATH\")\n\
         }\n",
    )
    .expect("write probe.dag");
    let out = temp_dir("dsl-shell-primary-out");
    let output = compile_with_roots(
        &gunbc,
        &ws,
        &[&entry_root, &ws.join("dsl"), &ws.join("src/v2")],
        "primary-precedence",
        &out,
    );
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "shell.Env.Get probe must compile under primary-precedence (dsl extdeps.shell wins over v2 overlay); exit {:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            output.status.code()
        );
    }
    rm_rf(&entry_root);
    rm_rf(&out);
}

#[test]
fn two_root_compile_with_shadow_masked_fixture_succeeds() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let out = temp_dir("shadow-masked-two-root-out");
    for pool_index in ["strict", "primary-precedence"] {
        let output = compile_with_roots(
            &gunbc,
            &ws,
            &[&ws.join("dsl"), &ws.join("src/v2")],
            pool_index,
            &out,
        );
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "two-root compile must succeed after shadow plant rename (pool_index={pool_index}); exit {:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
                output.status.code()
            );
        }
    }
    rm_rf(&out);
}
