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

// The live-tree extdeps.shell collision fixture is gone (the src/v2 fork was
// deleted — one module, one authority, DESIGN §3), so the strict-mode RED
// control synthesizes its own cross-root duplicate instead of depending on a
// defect persisting in the real corpus.
#[test]
fn strict_dependency_pool_index_panics_on_cross_root_collision() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let root_a = temp_dir("strict-cross-root-a");
    let root_b = temp_dir("strict-cross-root-b");
    fs::write(
        root_a.join("dup.dag"),
        "module duplicate.across.roots\nfn a() -> Int { 1 }\n",
    )
    .expect("write root_a dup.dag");
    fs::write(
        root_b.join("dup.dag"),
        "module duplicate.across.roots\nfn b() -> Int { 2 }\n",
    )
    .expect("write root_b dup.dag");
    let out = temp_dir("strict-cross-root-out");
    let output = compile_with_roots(&gunbc, &ws, &[&root_a, &root_b], "strict", &out);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "strict mode must panic on a cross-root duplicate module path; got success"
    );
    assert!(
        combined.contains("duplicate module path"),
        "expected duplicate module path panic; got:\n{combined}"
    );
    // GREEN control: the same fixture under primary-precedence indexes
    // first-root-wins instead of refusing at the index layer.
    let out_pp = temp_dir("strict-cross-root-pp-out");
    let output_pp = compile_with_roots(
        &gunbc,
        &ws,
        &[&root_a, &root_b],
        "primary-precedence",
        &out_pp,
    );
    let combined_pp = format!(
        "{}{}",
        String::from_utf8_lossy(&output_pp.stdout),
        String::from_utf8_lossy(&output_pp.stderr)
    );
    assert!(
        !combined_pp.contains("duplicate module path"),
        "primary-precedence must not refuse the cross-root duplicate at the index layer; got:\n{combined_pp}"
    );
    rm_rf(&root_a);
    rm_rf(&root_b);
    rm_rf(&out);
    rm_rf(&out_pp);
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
fn primary_precedence_two_root_compile_with_shadow_masked_fixture_succeeds() {
    let Some(gunbc) = gunbc_bin() else {
        eprintln!("skipping: release gunbc binary not found");
        return;
    };
    let ws = workspace_root();
    let out = temp_dir("shadow-masked-two-root-out");
    let output = compile_with_roots(
        &gunbc,
        &ws,
        &[&ws.join("dsl"), &ws.join("src/v2")],
        "primary-precedence",
        &out,
    );
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "primary-precedence two-root compile must succeed after shadow plant rename; exit {:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            output.status.code()
        );
    }
    rm_rf(&out);
}
