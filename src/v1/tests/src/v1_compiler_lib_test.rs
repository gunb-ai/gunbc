use crate::helpers::workspace_root;

fn cargo_binary() -> &'static str {
    if std::path::Path::new("/opt/cargo/bin/cargo").exists() {
        "/opt/cargo/bin/cargo"
    } else {
        "cargo"
    }
}

pub fn assert_v2_compiler_lib_tests_compile() {
    let output = std::process::Command::new(cargo_binary())
        .arg("test")
        .arg("-p")
        .arg("v1-compiler")
        .arg("--lib")
        .arg("--no-run")
        .arg("--release")
        .arg("--quiet")
        .current_dir(workspace_root())
        .output()
        .expect("failed to spawn cargo test -p v1-compiler --lib --no-run");

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
