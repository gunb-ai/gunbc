use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::emit::{emit, emit_module, EmitTarget};
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
    assert!(
        compile.success(),
        "rustc failed on emitted source:\n{rendered}"
    );

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
    let rendered = emit(&dag, EmitTarget::Go).expect("emits go").text;
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

// The unused_parameters lens uses recursive helpers (e.g. walk_steps,
// expand_frontier_list, behavior_result_port) which v3 lowers to
// `Behavior::Loop`. emit_go does not yet emit Loop — it now
// fail-closes instead of silently rendering the loop body's result
// port. Re-enable when emit_go gains Loop emission (Lane 1e
// consolidation handles this via spec-driven walker dispatch).
#[test]
#[ignore = "blocked on emit_go Behavior::Loop support; previously passed via silent loop-body collapse"]
fn emit_go_lens_unused_parameters_module() {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compiled lens source");
    let rendered = emit_module(&dag, EmitTarget::Go)
        .expect("emits go module")
        .text;

    assert!(rendered.contains("package emitted"), "got: {rendered}");
    assert!(
        rendered.contains("type UnusedParameter struct"),
        "got: {rendered}"
    );
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
        return;
    };
    assert_eq!(rust, go, "Rust and Go outputs diverged");
}

/// E-5 / Lane 1 Stage 1c PR 2 pilot — unused match-arm payload
/// bindings are elided under `go_clean_emission.pattern_bindings =
/// EmitUnderscoreWhenUnused`. Before the pilot the Go emitter
/// rendered `case Boxed: value := v._0; return 0`, which Go rejects
/// with `value declared but not used`. After the pilot the arm
/// renders `case Boxed: return 0`, and because no arm binds through
/// `v` anymore the switch header drops `v :=` to keep `v` from
/// becoming the new unused local.
#[test]
fn emit_go_unused_payload_binding_is_elided() {
    let source = "\
type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(value) => 0, Empty => 1 }
let zero: Int = 0
";
    let dag = compile_to_dag(source, "clean_emission.v3").expect("compiles");
    let rendered = emit(&dag, EmitTarget::Go).expect("emits go").text;
    assert!(
        !rendered.contains("value := v._0"),
        "expected unused payload binding to be elided, got: {rendered}"
    );
    assert!(
        !rendered.contains("value := v"),
        "expected unused payload binding to be elided, got: {rendered}"
    );
    assert!(
        rendered.contains("case Boxed:"),
        "expected the Boxed arm to still be present, got: {rendered}"
    );
    assert!(
        rendered.contains("case Empty:"),
        "expected the Empty arm to still be present, got: {rendered}"
    );
    assert!(
        rendered.contains("switch any(") && !rendered.contains("switch v := any("),
        "expected the type-switch header to drop `v :=` when no arm binds, got: {rendered}"
    );
}

/// E-5 / Lane 1 Stage 1c PR 2 — when at least one arm consumes its
/// payload, the emitter must keep emitting `v :=` and route reads
/// through the type-switch witness. Companion regression test to
/// `emit_go_unused_payload_binding_is_elided`; proves elision is
/// keyed on port liveness per arm, not blanket suppression.
#[test]
fn emit_go_used_payload_binding_is_preserved() {
    let source = "\
type BoxedInt = Boxed(Int) | Empty
fn unwrap_or_zero(b: BoxedInt) -> Int = match b { Boxed(value) => value, Empty => 0 }
let zero: Int = 0
";
    let dag = compile_to_dag(source, "clean_emission.v3").expect("compiles");
    let rendered = emit(&dag, EmitTarget::Go).expect("emits go").text;
    assert!(
        rendered.contains("case Boxed: return v._0"),
        "used positional payload must still read from the type-switch witness, got: {rendered}"
    );
    assert!(
        rendered.contains("switch v := any("),
        "expected `v :=` to remain when any arm binds, got: {rendered}"
    );
}

#[test]
fn emit_go_named_single_field_payload_uses_the_variant_value() {
    let source = "\
type Point { x: Int y: Int }
type Wrapped = Wrap { inner: Point } | Empty
fn unwrap_or_zero(w: Wrapped) -> Int = match w { Wrap(payload) => payload.inner.x, Empty => 0 }
let zero: Int = 0
";
    let dag = compile_to_dag(source, "variant_payload_named_single.v3").expect("compiles");
    let rendered = emit(&dag, EmitTarget::Go).expect("emits go").text;
    assert!(
        rendered.contains("case Wrap: return ((v).inner).x"),
        "named single-field payload access must project from the variant value, got: {rendered}"
    );
    assert!(
        !rendered.contains("payload := v.inner"),
        "named single-field payload must not rebind to the bare field value, got: {rendered}"
    );
}

/// E-5 / Lane 1 Stage 1c PR 2 roundtrip — emitted Go with an unused
/// payload binding compiles and runs under `go run`. Proves the
/// pilot fires end-to-end against Go's unused-local compile error,
/// not just that the emission string looks right.
///
/// Gated behind `#[ignore]` like the other `go_*` roundtrips — CI
/// sandboxes don't always carry a Go toolchain. Run locally:
///
///     cargo test -p v3-compiler --test m1_3_emit_go_test \
///         emit_go_unused_payload_binding_compiles \
///         -- --ignored --nocapture
#[test]
#[ignore]
fn emit_go_unused_payload_binding_compiles() {
    let source = "\
type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(value) => 0, Empty => 1 }
let result: Int = ignore_payload(Empty)
";
    let Some(stdout) = go_stdout(source) else {
        return;
    };
    assert_eq!(stdout, "1");
}

/// E-5 / Lane 1 Stage 1c PR 4 — emitted Go passes
/// `go_clean_emission.post_emit_verifier` as invoked through the
/// shared harness. The verifier (gofmt -l + RequireEmptyStdout)
/// enforces format-cleanliness by construction — gofmt lists
/// ill-formatted files on stdout while exiting 0, so the output
/// policy is the load-bearing verdict channel, not the exit code.
///
/// Gated behind `#[ignore]` like the other `go_*` roundtrips — CI
/// sandboxes don't always carry a Go toolchain. Run locally:
///
///     cargo test -p v3-compiler --test m1_3_emit_go_test \
///         go_pilot_source_passes_post_emit_verifier_harness \
///         -- --ignored --nocapture
#[test]
#[ignore]
fn go_pilot_source_passes_post_emit_verifier_harness() {
    use v3_compiler::post_emit_verifier::{parse_post_emit_verifier, run_post_emit_verifier};
    let go_available = Command::new("gofmt")
        .arg("-h")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|status| status.code().is_some());
    if !go_available {
        return;
    }
    let source = "\
type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(value) => 0, Empty => 1 }
let result: Int = ignore_payload(Empty)
";
    let dag = compile_to_dag(source, "go_post_emit_verifier.v3").expect("source compiles");
    let spec = dag
        .go_clean_emission_spec()
        .expect("go_clean_emission cached");
    let binding = parse_post_emit_verifier(&dag, spec).expect("parse contract");
    let rendered = emit(&dag, EmitTarget::Go).expect("emits go").text;
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.go");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write go source");
    run_post_emit_verifier(&binding, &src_path)
        .expect("go post_emit_verifier rejected pilot source — E-5 contract regression");
}

#[test]
fn emit_go_wrapper_matches_shared_entrypoint() {
    let program_source = "\
fn double(x: Int) -> Int = x + x
let result: Int = double(21)
";
    let program_dag =
        compile_to_dag(program_source, "emit_go_wrapper_program_parity.v3").expect("compiles");
    let shared = emit(&program_dag, EmitTarget::Go)
        .expect("shared emit")
        .text;
    let wrapper = emit_go(&program_dag).expect("wrapper emit");
    assert_eq!(shared, wrapper, "emit_go wrapper drifted from emit::emit");

    let module_source = "\
fn double(x: Int) -> Int = x + x
";
    let module_dag =
        compile_to_dag(module_source, "emit_go_wrapper_module_parity.v3").expect("compiles");
    let shared_module = emit_module(&module_dag, EmitTarget::Go)
        .expect("shared module emit")
        .text;
    let wrapper_module = emit_go_module(&module_dag).expect("wrapper module emit");
    assert_eq!(
        shared_module, wrapper_module,
        "emit_go_module wrapper drifted from emit::emit_module"
    );
}
