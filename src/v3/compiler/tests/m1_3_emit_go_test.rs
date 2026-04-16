use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_go::{emit_go, emit_go_module};
use v3_compiler::emit_rust::emit_rust;

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("unused_parameters.dag")
}

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read unused_parameters.dag")
}

fn next_roundtrip_dir() -> PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_emit_go_roundtrip_{}_{}",
        std::process::id(),
        id
    ))
}

fn rust_stdout(source: &str) -> String {
    let dag = compile_to_dag(source, "parity.v3").expect("compiles");
    let rendered = emit_rust(&dag).expect("emits rust");
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write rust source");

    let compile = Command::new("rustc")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc");
    assert!(compile.success(), "rustc failed on emitted source:\n{rendered}");

    let run = Command::new(&bin_path).output().expect("run rust binary");
    assert!(run.status.success(), "compiled rust binary failed");
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn go_stdout(source: &str) -> Option<String> {
    let go_available = Command::new("go")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|status| status.success());
    if !go_available {
        return None;
    }

    let dag = compile_to_dag(source, "parity.v3").expect("compiles");
    let rendered = emit_go(&dag).expect("emits go");
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.go");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write go source");

    let run = Command::new("go")
        .arg("run")
        .arg(&src_path)
        .current_dir(&tmp_dir)
        .output()
        .expect("invoke go");
    assert!(
        run.status.success(),
        "go run failed on emitted source:\n{}\nstdout:\n{}\nstderr:\n{}",
        rendered,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    Some(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

#[test]
fn emit_go_lens_unused_parameters_module() {
    let dag = compile_to_dag(
        &lens_source(),
        lens_path().to_string_lossy().as_ref(),
    )
    .expect("compiled lens source");
    let rendered = emit_go_module(&dag).expect("emits go module");

    assert!(rendered.contains("package emitted"), "got: {rendered}");
    assert!(rendered.contains("type UnusedParameter struct"), "got: {rendered}");
    assert!(rendered.contains("func check("), "got: {rendered}");
    assert!(rendered.contains("switch v := any("), "got: {rendered}");
    assert!(
        !rendered.contains(".clone("),
        "GC target should not render Rust clone calls: {rendered}"
    );
}

#[test]
fn emit_go_and_rust_programs_are_behaviorally_equivalent_when_go_is_available() {
    let source = "\
fn double(x: Int) -> Int = x + x
let result: Int = if double(20) == 40 then 7 else 9
";
    let rust = rust_stdout(source);
    let Some(go) = go_stdout(source) else {
        eprintln!("skipping Go parity roundtrip: `go` toolchain not available");
        return;
    };
    assert_eq!(rust, go, "Rust and Go outputs diverged");
}
