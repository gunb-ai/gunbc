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
