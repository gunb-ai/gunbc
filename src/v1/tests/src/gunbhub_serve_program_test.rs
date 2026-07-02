use crate::helpers::workspace_root;

#[test]
fn gunbhub_serve_program_runs_green() {
    let ws = workspace_root();
    let gunbc = ws.join("target/release/gunbc");
    assert!(
        gunbc.is_file(),
        "release gunbc binary required for gunbhub_serve_program (built in ci_floor / ci_floor_parity)"
    );

    let entry = ws.join("dag/examples/gunbhub_serve_program/gunbhub_serve_program.dag");
    let program_root = ws.join("dag/examples/gunbhub_serve_program");
    let std_root = ws.join("dag/std");
    let gunbc_root = ws.join("dag/gunbc");
    let extdeps_root = ws.join("dag/extdeps");

    let output = std::process::Command::new(&gunbc)
        .arg("run")
        .arg("--source-root")
        .arg(program_root)
        .arg("--source-root")
        .arg(std_root)
        .arg("--source-root")
        .arg(gunbc_root)
        .arg("--source-root")
        .arg(extdeps_root)
        .arg("--entry")
        .arg(&entry)
        .arg("--function")
        .arg("main")
        .current_dir(&ws)
        .output()
        .expect("spawn gunbc run for gunbhub_serve_program");

    if output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    panic!(
        "gunbhub_serve_program func main failed (exit {:?}).\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        output.status.code()
    );
}
