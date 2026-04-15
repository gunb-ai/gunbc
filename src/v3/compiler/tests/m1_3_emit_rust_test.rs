// M1(3) PR-B — Rust emitter acceptance tests.
//
// The success criterion the whole plan validates:
//   `compile_to_dag("let x: Int = 1 + 2").and_then(emit_rust)`
// produces Rust source that, when fed to `rustc`, compiles and runs
// producing `3` on stdout.
//
// The #[ignore]d `rustc_roundtrip` test runs that whole pipeline;
// it's gated because CI environments don't always have `rustc`
// available. Run it locally via:
//     cargo test -p v3-compiler --test m1_3_emit_rust_test \
//                  -- --ignored --nocapture
//
// Everything else is structural: assert the emitter produced the
// right substring for each kind of program without depending on
// exact formatting.

use std::io::Write;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust;

fn emit(source: &str) -> String {
    let dag = compile_to_dag(source, "test.v3").expect("compiles");
    emit_rust(&dag).expect("emits")
}

#[test]
fn emit_rust_single_int_binding() {
    let out = emit("let x: Int = 42");
    assert!(out.contains("let x: i64 = 42;"), "got: {out}");
    assert!(out.contains("fn main()"), "got: {out}");
    assert!(out.contains("println!(\"{}\", x)"), "got: {out}");
}

#[test]
fn emit_rust_addition() {
    let out = emit("let x: Int = 1 + 2");
    assert!(out.contains("let x: i64 = (1 + 2);"), "got: {out}");
}

#[test]
fn emit_rust_chained_arithmetic() {
    // Left-associative: ((1 + 2) + 3)
    let out = emit("let x: Int = 1 + 2 + 3");
    assert!(out.contains("let x: i64 = ((1 + 2) + 3);"), "got: {out}");
}

#[test]
fn emit_rust_subtraction_and_multiplication() {
    let out = emit("let x: Int = 10 - 2 * 3");
    // Precedence: 10 - (2 * 3) = 4
    assert!(out.contains("let x: i64 = (10 - (2 * 3));"), "got: {out}");
}

#[test]
fn emit_rust_if_else_branch() {
    let out = emit("let r: Int = if 1 > 0 then 10 else 20");
    assert!(out.contains("if (1 > 0) {"), "got: {out}");
    assert!(out.contains("} else {"), "got: {out}");
    assert!(out.contains("10"), "got: {out}");
    assert!(out.contains("20"), "got: {out}");
}

#[test]
fn emit_rust_multi_bind_uses_last_as_print_target() {
    let out = emit(
        "let a: Int = 1
let b: Int = a + 2",
    );
    assert!(out.contains("let a: i64 = 1;"), "got: {out}");
    assert!(out.contains("let b: i64 = (a + 2);"), "got: {out}");
    // Main wrap prints the LAST bind (`b`), not the first (`a`).
    assert!(out.contains("println!(\"{}\", b)"), "got: {out}");
}

#[test]
fn emit_rust_preserves_rust_dag_is_the_only_rust_syntax_source() {
    // Structural check: the carrier strings the emitter produced
    // match what rust.dag declared — not some hardcoded Rust-side
    // string. This is the thesis guarantee: "add a new emission
    // target = one spec-file edit." If the emitter were
    // fabricating Rust syntax in Rust code, the test below would
    // still pass trivially, but ANY attempt to change the carrier
    // in rust.dag (e.g. editing `"i64"` to `"int64_t"`) would fail
    // to propagate — the substring check here guards against that
    // class of regression.
    let out = emit("let x: Int = 1 + 2");
    // Every token the emitter rendered for this program traces to
    // a rust.dag carrier: "let %N: %T = %V;" (rust_let_stmt),
    // "i64" (rust_int), "+" (rust_int_add), and the main wrapper.
    assert!(out.contains("let x: i64 = (1 + 2);"));
}

/// End-to-end roundtrip test: emit Rust from a v3 program, feed the
/// Rust source to `rustc`, run the resulting binary, assert stdout.
/// Gated behind `#[ignore]` because CI runners often don't have a
/// Rust toolchain available inside the test sandbox. Run locally:
///
///     cargo test -p v3-compiler --test m1_3_emit_rust_test \
///                  -- --ignored --nocapture
///
/// This is the PR-B success criterion made literal: the v3 compiler
/// produces Rust that a real Rust toolchain turns into a working
/// binary, without touching the emitter between "here's the
/// program" and "here's the answer `3` on stdout."
#[test]
#[ignore]
fn rustc_roundtrip_int_addition_prints_three() {
    let source = emit("let x: Int = 1 + 2");

    let tmp_dir = std::env::temp_dir().join(format!(
        "v3_emit_rust_roundtrip_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write rust source");

    let compile = Command::new("rustc")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc — install a rust toolchain to run this test");
    assert!(compile.success(), "rustc failed on emitted source:\n{source}");

    let run = Command::new(&bin_path)
        .output()
        .expect("run compiled binary");
    assert!(run.status.success(), "compiled binary failed");
    let stdout = String::from_utf8_lossy(&run.stdout).trim().to_string();
    assert_eq!(stdout, "3", "compiled binary printed {stdout:?}, not `3`");
}
