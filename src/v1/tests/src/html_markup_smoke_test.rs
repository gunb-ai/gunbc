use crate::helpers::workspace_root;

pub fn assert_html_markup_smoke_executes() {
    let ws = workspace_root();
    let gunbc = ws.join("target/release/gunbc");
    assert!(
        gunbc.is_file(),
        "release gunbc binary required for html_markup_smoke (built in ci_floor / ci_floor_parity)"
    );

    let entry = ws.join("dsl/examples/html_markup_smoke/html_markup_smoke.dag");
    let std_root = ws.join("dsl/std");
    let smoke_root = ws.join("dsl/examples/html_markup_smoke");

    let output = std::process::Command::new(&gunbc)
        .arg("run")
        .arg("--source-root")
        .arg(smoke_root)
        .arg("--source-root")
        .arg(std_root)
        .arg("--source-root")
        .arg(extdeps_root)
        .arg("--entry")
        .arg(&entry)
        .arg("--function")
        .arg("main")
        .current_dir(&ws)
        .output()
        .expect("spawn gunbc run for html_markup_smoke");

    if output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    panic!(
        "html_markup_smoke func main failed (exit {:?}).\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        output.status.code()
    );
}

#[test]
fn html_markup_smoke_runs_green() {
    let ws = workspace_root();
    let gunbc = ws.join("target/release/gunbc");
    if !gunbc.is_file() {
        eprintln!(
            "skipping html_markup_smoke_runs_green: release binary not found (run `cargo build --release -p v1-compiler` first)"
        );
        return;
    }
    assert_html_markup_smoke_executes();
}
