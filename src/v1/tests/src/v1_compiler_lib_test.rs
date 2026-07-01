use crate::helpers::workspace_root;

fn cargo_binary() -> &'static str {
    if std::path::Path::new("/opt/cargo/bin/cargo").exists() {
        "/opt/cargo/bin/cargo"
    } else {
        "cargo"
    }
}

pub fn assert_v2_compiler_lib_tests_compile() {
    let output = crate::helpers::run_cargo_with_infra_retry(|| {
        let mut cmd = std::process::Command::new(cargo_binary());
        cmd.args([
            "test",
            "-p",
            "v1-compiler",
            "--lib",
            "--no-run",
            "--release",
            "--quiet",
        ])
        .current_dir(workspace_root());
        cmd
    });

    if output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    panic!(
        "v1-compiler lib test harness failed to compile (CI blind spot guard).\n\
         Fix Rc call-sites in compiler_tests_rust.dag and regen stage0.\n\
         --- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

#[test]
fn v1_compiler_lib_tests_compile_green() {
    assert_v2_compiler_lib_tests_compile();
}
