use std::process::Command;

#[test]
fn shipped_build_command_reaches_the_generated_dispatcher() {
    let output = Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .args(["--dry-run", "build", "gunbc"])
        .output()
        .expect("the shipped gunbc binary must execute");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stderr).expect("the refusal must be UTF-8"),
        "REFUSED: --dry-run cannot execute a bootstrap successor operation\n"
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn bootstrap_bindings_require_receipt_readback_after_a_successful_evaluation() {
    let fixture =
        std::env::temp_dir().join(format!("gunbc-bootstrap-receipts-{}", std::process::id()));
    std::fs::create_dir(&fixture).expect("fresh checkout fixture");
    for directory in [".git", "dag/gunbc/devboot", "dag/gunbc/fleet", "src/v2"] {
        std::fs::create_dir_all(fixture.join(directory)).unwrap();
    }
    let build_entry = "dag/gunbc/devboot/build.dag";
    let converge_entry = "dag/gunbc/fleet/fleet_converge_plan_cli.dag";
    let exit = "type ProcessExit = ExitSuccess | ExitFailure { code: Int, reason: String }";
    std::fs::write(
        fixture.join(build_entry),
        format!("module gunbc.devboot.build\n{exit}\nfn gunbc_build_cli_entry(program: String, receipt_path: String) -> ProcessExit {{ ExitSuccess }}\n"),
    ).unwrap();
    std::fs::write(
        fixture.join(converge_entry),
        format!("module gunbc.fleet.fleet_converge_plan_cli\n{exit}\nfn fleet_converge_operator_cli_entry(host: String, mode: String, scope: String, plan_run_id: String, plan_hash: String, target: String, candidate_branch: String, receipt_path: String) -> ProcessExit {{ ExitSuccess }}\n"),
    ).unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_gunbc"))
            .current_dir(&fixture)
            .env_remove("GUNBC_WORKSPACE_ROOT")
            .args(args)
            .output()
            .expect("run the shipped dispatcher in the fixture")
    };
    let direct = run(&[
        "run",
        "--source-root",
        "dag",
        "--source-root",
        "src/v2",
        "--entry",
        build_entry,
        "--function",
        "gunbc_build_cli_entry",
        "--arg",
        "program=fixture",
        "--arg",
        "receipt_path=unused",
    ]);
    let build = run(&["build", "fixture"]);
    let converge = run(&["converge", "--host", "fixture", "--mode", "plan"]);
    std::fs::remove_dir_all(fixture).unwrap();

    assert!(direct.status.success(), "{direct:?}");
    for (verb, output) in [("build", build), ("converge", converge)] {
        assert_eq!(output.status.code(), Some(1), "{verb}: {output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(&format!(
                "REFUSED: bootstrap operation receipt `.gunbc-{verb}-receipts/receipt-"
            )),
            "{stderr}"
        );
        assert!(stderr.contains("unreadable"), "{stderr}");
        assert!(output.stdout.is_empty(), "{verb}: {output:?}");
    }
}
