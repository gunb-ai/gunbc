//! Hermetic smoke: v2.5 std tree parses/resolves under v2-compiler with zero diagnostics.

#[test]
fn v2_5_std_modules_compile_with_zero_diagnostics() {
    let ws = crate::helpers::workspace_root();
    let v2_5_root = ws.join("src/v2.5");
    let out = std::env::temp_dir().join(format!(
        "v2-5-std-smoke-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("create output dir");

    let output = std::process::Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--release")
        .arg("--")
        .arg("compile")
        .arg("--source-root")
        .arg(&v2_5_root)
        .arg("--output-dir")
        .arg(&out)
        .arg("--target")
        .arg("dag")
        .current_dir(&ws)
        .output()
        .expect("run v2-compiler");

    assert!(
        output.status.success(),
        "v2.5 compile failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !combined.contains("error["),
        "expected zero diagnostics, got:\n{combined}"
    );
    assert!(
        combined.contains("resolved 6 sources") || combined.contains("resolved 6 source"),
        "expected six v2.5 modules in closure, got:\n{combined}"
    );
}
