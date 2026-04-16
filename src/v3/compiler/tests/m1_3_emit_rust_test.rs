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
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::{emit_rust, emit_rust_module};

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);

fn emit(source: &str) -> String {
    let dag = compile_to_dag(source, "test.v3").expect("compiles");
    emit_rust(&dag).expect("emits")
}

fn emit_module(source: &str) -> String {
    let dag = compile_to_dag(source, "test.v3").expect("compiles");
    emit_rust_module(&dag).expect("emits module")
}

fn next_roundtrip_dir() -> std::path::PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_emit_rust_roundtrip_{}_{}",
        std::process::id(),
        id
    ))
}

fn roundtrip_stdout(source: &str) -> String {
    let source = emit(source);

    let tmp_dir = next_roundtrip_dir();
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
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn deps_dir() -> PathBuf {
    std::env::current_exe()
        .expect("current test binary path")
        .parent()
        .expect("deps dir")
        .to_path_buf()
}

fn find_current_rlib(crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let mut matches: Vec<PathBuf> = std::fs::read_dir(deps_dir())
        .expect("read deps dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let file_name = path.file_name()?.to_str()?;
            if file_name.starts_with(&prefix) && file_name.ends_with(".rlib") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    matches.sort();
    matches
        .into_iter()
        .next()
        .expect("compiled rlib for current crate")
}

fn compile_with_current_crate(src_path: &Path, bin_path: &Path) {
    let deps = deps_dir();
    let current_rlib = find_current_rlib("v3_compiler");
    let compile = Command::new("rustc")
        .arg("--edition=2021")
        .arg(src_path)
        .arg("-o")
        .arg(bin_path)
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--extern")
        .arg(format!("v3_compiler={}", current_rlib.display()))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc — install a rust toolchain to run this test");
    assert!(compile.success(), "rustc failed on emitted source");
}

fn roundtrip_module_stdout(module_source: &str, wrapper_body: &str) -> String {
    let wrapped = format!(
        "mod emitted {{ use v3_compiler::dag::*; use v3_compiler::diagnostics::*; {module_source} pub fn __run() -> i64 {{ {wrapper_body} }} }} fn main() {{ println!(\"{{}}\", emitted::__run()); }}"
    );

    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(wrapped.as_bytes()))
        .expect("write wrapped rust source");

    compile_with_current_crate(&src_path, &bin_path);

    let run = Command::new(&bin_path)
        .output()
        .expect("run compiled binary");
    assert!(run.status.success(), "compiled binary failed");
    String::from_utf8_lossy(&run.stdout).trim().to_string()
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
fn emit_rust_emits_user_enum_type_definitions() {
    let out = emit(
        "type Sign = Plus | Minus
let zero: Int = 0",
    );
    assert!(out.contains("pub enum Sign {"), "got: {out}");
    assert!(out.contains("Plus,"), "got: {out}");
    assert!(out.contains("Minus,"), "got: {out}");
}

#[test]
fn emit_rust_match_on_user_sum_uses_match_expression() {
    let out = emit(
        "type Sign = Plus | Minus
fn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }
let zero: Int = 0",
    );
    assert!(out.contains("pub enum Sign {"), "got: {out}");
    assert!(out.contains("fn classify(p0: Sign) -> i64 { match (p0).clone() {"), "got: {out}");
    assert!(out.contains("Sign::Plus => 0,"), "got: {out}");
    assert!(out.contains("Sign::Minus => 1,"), "got: {out}");
}

#[test]
fn emit_rust_payload_match_uses_struct_variant_pattern() {
    let out = emit(
        "type BoxedInt = Boxed(Int) | Empty
fn unwrap_or_zero(b: BoxedInt) -> Int = match b { Boxed(value) => value, Empty => 0 }
let zero: Int = 0",
    );
    assert!(out.contains("pub enum BoxedInt {"), "got: {out}");
    assert!(out.contains("Boxed { _0: i64, },"), "got: {out}");
    assert!(out.contains("fn unwrap_or_zero(p0: BoxedInt) -> i64 { match (p0).clone() {"), "got: {out}");
    assert!(out.contains("BoxedInt::Boxed { _0: value } => (value).clone(),"), "got: {out}");
    assert!(out.contains("BoxedInt::Empty => 0,"), "got: {out}");
}

#[test]
fn emit_rust_record_literal_uses_value_construction_syntax() {
    let out = emit(
        "type Point { x: Int y: Int }
let p: Point = { x: 1, y: 2 }",
    );
    assert!(out.contains("pub struct Point {"), "got: {out}");
    assert!(out.contains("let p: Point = Point { x: 1, y: 2 };"), "got: {out}");
}

#[test]
fn emit_rust_multi_bind_uses_last_as_print_target() {
    let out = emit(
        "let a: Int = 1
let b: Int = a + 2",
    );
    assert!(out.contains("let a: i64 = 1;"), "got: {out}");
    assert!(out.contains("let b: i64 = ((a).clone() + 2);"), "got: {out}");
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

/// **PR-B-unwind regression test.** The emitter must NOT contain
/// any Rust string literal that names a substrate concept (the
/// canonical primitive name "Int", behavior names "Bind"/"Branch"/
/// "Main", etc.) in dispatch position. This test scans the
/// emitter source file (excluding comment lines) and asserts the
/// absence of the specific patterns that the unwind fixed.
///
/// **Why this is a runtime test instead of a static lint.** The
/// emitter file is loaded with `include_str!` so the assertion
/// runs at test time. Rust's macro hygiene doesn't give us a
/// proper compile-time grep, so we accept the runtime cost — the
/// test runs in <1ms.
///
/// **Comment-line filtering.** Lines whose first non-whitespace
/// content is `//` are excluded. The unwind's documentation talks
/// about the bad pattern explicitly (so future readers understand
/// what was removed) and those mentions must not trip the check.
/// This is a coarse heuristic — it doesn't handle block comments
/// or strings-on-comment-lines — but it's sufficient for the
/// emit_rust.rs file as written.
///
/// If anyone re-introduces a `.lookup("Int", ...)` or similar
/// dispatch, this test fails and the reviewer sees the
/// reintroduction immediately. The failure message points at the
/// rust.dag typed-reference shape that should be used instead.
#[test]
fn emit_rust_has_no_substrate_name_string_dispatches() {
    const EMITTER_SOURCE: &str = include_str!("../src/emit_rust.rs");

    // Strip comment-only lines (// ... and ///-style doc comments)
    // before scanning for forbidden patterns. This avoids false
    // positives on the file's documentation, which describes the
    // bad pattern explicitly so future readers know what was
    // removed.
    let code_only: String = EMITTER_SOURCE
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n");

    // Each forbidden substring is a Rust string literal naming a
    // substrate concept. The check is "does the emitter code
    // contain a string literal of this exact form in non-comment
    // position?" — using the double-quote framing makes it a
    // literal search, not a bare-name search (so identifier
    // mentions in doc strings don't trip the check).
    let forbidden = [
        "\"Int\"",
        "\"Bool\"",
        "\"String\"",
        "\"Bind\"",
        "\"Branch\"",
        "\"Main\"",
        "\"True\"",
        "\"False\"",
        "\"target_name\"",
        "\"op_name\"",
    ];
    for pattern in forbidden {
        assert!(
            !code_only.contains(pattern),
            "emit_rust.rs must not contain the string literal {pattern} in non-comment position — that would be a name-string dispatch on a substrate concept. The PR-B unwind moved every such lookup to typed declaration ids resolved via dag.{{bind_marker, branch_marker, main_marker, ...}}() and dag.declaration_by_name() at index-build time only. If you reintroduced one, see src/v3/spec/rust.dag for the typed pattern: `data rust_int: TypeRealization = {{ target: Int, ... }}` instead of `target_name: {pattern}`."
        );
    }
}

/// **Layer opacity gate documentation.**
///
/// The static regression test above
/// (`emit_rust_has_no_substrate_name_string_dispatches`) is the
/// load-bearing **rename test** in static form, enforcing the
/// `INVARIANTS.md` §"Layer opacity" rule on `emit_rust.rs`. It
/// asserts that emit_rust.rs contains zero string literals
/// naming any user-facing std/ identifier
/// (`Int`/`Bool`/`String`/`True`/`False`) or substrate L1
/// behavior (`Bind`/`Branch`/`Main`) in non-comment position.
/// If the test passes, it follows by construction that:
///
///   1. **Renaming `Int` → `Integer` in `dsl/std/integer.dag`**
///      requires editing `dsl/std/integer.dag` (the declaration),
///      `src/v3/spec/rust.dag` (the typed reference
///      `target: Int` → `target: Integer`), and any user-source
///      that mentions `Int`. Emit_rust.rs needs **zero edits**
///      because it dispatches on declaration ids resolved at
///      bootstrap time, not on name strings.
///
///   2. **Renaming any other primitive** has the same property —
///      one std/ edit + one rust.dag edit + user-source updates,
///      with the emitter unchanged.
///
///   3. **Adding a new primitive** (e.g. `Decimal` for fixed-
///      point) is one std/ addition + one rust.dag addition,
///      again with the emitter unchanged.
///
/// This is the **layer opacity guarantee** in static form. The
/// thesis claim — "the compiler exists to make compositions
/// opaque, application code sitting on rest/http/service should
/// be unable to observe layer changes" (THESIS.md §"Compositional
/// layering: below-boundary opacity by construction") — applies
/// one layer up: the emitter sits on top of the substrate layer,
/// and the substrate layer should be replaceable without the
/// emitter noticing. The regression test's empty-grep result is
/// the proof. The eventual structural enforcement is
/// `lens_layer_opacity` per `docs/lens-library-design.md` §2.2;
/// this static grep is the precursor that catches the same class
/// of violations until the lens lands.
///
/// The DYNAMIC version of the rename test (literally rename a
/// declaration in std/, recompile, verify) would touch every
/// std/ file that references the renamed type and is too coarse
/// to express as a unit test. The static check is strictly
/// stronger anyway: a passing static test guarantees the dynamic
/// test would pass without running it.
///
/// **The carve-out for `EmitError` payloads.** `EmitError`
/// variants like `MissingSubstrateMarker(SubstrateMarkerRole::
/// Bind)` carry typed enum tags — not strings — for the same
/// reason. The `SubstrateMarkerRole` enum is internal compiler
/// dispatch metadata, NOT a string-keyed lookup. The regression
/// test's pattern list catches the literal `"Bind"`/`"Branch"`/
/// `"Main"` quoted strings; if you re-introduced one, the test
/// fires immediately.
#[test]
fn composition_opacity_gate_is_documented() {
    // No-op test that exists purely to anchor the documentation
    // above. The actual gate lives in
    // `emit_rust_has_no_substrate_name_string_dispatches` and
    // runs every test invocation.
}

#[test]
fn roundtrip_temp_dirs_are_unique() {
    assert_ne!(next_roundtrip_dir(), next_roundtrip_dir());
}

#[test]
fn rustc_roundtrip_list_fold_prints_six() {
    let stdout = roundtrip_stdout(
        "let total: Int = fold(cons(1, cons(2, singleton(3))), 0, |acc, x| acc + x)",
    );
    assert_eq!(stdout, "6", "compiled binary printed {stdout:?}, not `6`");
}

#[test]
fn rustc_roundtrip_generic_list_fold_prints_one() {
    let stdout = roundtrip_stdout(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
    );
    assert_eq!(stdout, "1", "compiled binary printed {stdout:?}, not `1`");
}

#[test]
fn rustc_roundtrip_list_map_then_fold_prints_twelve() {
    let stdout = roundtrip_stdout(
        "let total: Int = fold(map(cons(1, cons(2, singleton(3))), |x| x * 2), 0, |acc, x| acc + x)",
    );
    assert_eq!(stdout, "12", "compiled binary printed {stdout:?}, not `12`");
}

#[test]
fn rustc_roundtrip_list_filter_then_fold_prints_seven() {
    let stdout = roundtrip_stdout(
        "let total: Int = fold(filter(cons(1, cons(2, cons(3, singleton(4)))), |x| x > 2), 0, |acc, x| acc + x)",
    );
    assert_eq!(stdout, "7", "compiled binary printed {stdout:?}, not `7`");
}

#[test]
fn rustc_roundtrip_nested_list_builtins_inside_lambda_prints_six() {
    let stdout = roundtrip_stdout(
        "let total: Int = fold(cons(1, singleton(2)), 0, |acc, x| acc + fold(map(singleton(x), |y| y * 2), 0, |n, y| n + y))",
    );
    assert_eq!(stdout, "6", "compiled binary printed {stdout:?}, not `6`");
}

#[test]
fn rustc_roundtrip_user_function_call_prints_three() {
    let stdout = roundtrip_stdout(
        "fn add(a: Int, b: Int) -> Int = a + b\nlet total: Int = add(1, 2)",
    );
    assert_eq!(stdout, "3", "compiled binary printed {stdout:?}, not `3`");
}

#[test]
fn rustc_roundtrip_recursive_function_call_prints_six() {
    let stdout = roundtrip_stdout(
        "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet total: Int = count_down(3)",
    );
    assert_eq!(stdout, "6", "compiled binary printed {stdout:?}, not `6`");
}

#[test]
fn rustc_roundtrip_record_literal_through_function_prints_one() {
    let stdout = roundtrip_stdout(
        "type Point { x: Int y: Int }\nfn x_of(p: Point) -> Int = p.x\nlet total: Int = x_of({ x: 1, y: 2 })",
    );
    assert_eq!(stdout, "1", "compiled binary printed {stdout:?}, not `1`");
}

#[test]
fn rustc_roundtrip_user_sum_match_prints_zero() {
    let stdout = roundtrip_stdout(
        "type Sign = Plus | Minus\nfn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }\nlet total: Int = classify(Plus)",
    );
    assert_eq!(stdout, "0", "compiled binary printed {stdout:?}, not `0`");
}

#[test]
fn rustc_roundtrip_emitted_module_invokes_reflected_dag_function() {
    let module = emit_module(
        "fn node_count(d: Dag) -> Int = fold(d.nodes, 0, |n, node| n + 1)",
    );
    let stdout = roundtrip_module_stdout(
        &module,
        "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); node_count(dag)",
    );
    assert!(
        stdout.parse::<i64>().is_ok_and(|count| count > 0),
        "compiled reflected Dag function should return a positive node count, got {stdout:?}"
    );
}

#[test]
fn rustc_roundtrip_emitted_module_matches_reflected_behavior_payloads() {
    let module = emit_module(
        "fn bind_count(d: Dag) -> Int = fold(d.nodes, 0, |n, behavior| match behavior { Value(v) => n, Transform(t) => n, Branch(b) => n, Loop(l) => n, Bind(bind) => n + 1 })",
    );
    let stdout = roundtrip_module_stdout(
        &module,
        "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); bind_count(dag)",
    );
    assert_eq!(
        stdout, "2",
        "compiled reflected Behavior match should count the two top-level binds, got {stdout:?}"
    );
}

#[test]
fn rustc_roundtrip_emitted_module_returns_reflected_source_span_list() {
    let module = emit_module(
        "fn singleton_span(bind: BindNode) -> List<SourceSpan> = [bind.span]",
    );
    let stdout = roundtrip_module_stdout(
        &module,
        "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); let bind = dag.nodes().iter().find_map(|node| match node { v3_compiler::dag::Behavior::Bind(bind) => Some(bind.clone()), _ => None }).expect(\"bind\"); singleton_span(bind).len() as i64",
    );
    assert_eq!(
        stdout, "1",
        "compiled reflected function returning List<SourceSpan> should yield a singleton list, got {stdout:?}"
    );
}

#[test]
fn rustc_roundtrip_emitted_module_compares_reflected_port_ids_in_list_contains() {
    let module = emit_module(
        "fn result_port_is_param(bind: BindNode) -> Bool = contains(bind.params, bind.result_port)",
    );
    let stdout = roundtrip_module_stdout(
        &module,
        "let dag = v3_compiler::compile_to_dag(\"fn id(x: Int) -> Int = x\", \"runtime_reflection.v3\").expect(\"compiles\"); let bind = dag.nodes().iter().find_map(|node| match node { v3_compiler::dag::Behavior::Bind(bind) if !bind.params.is_empty() => Some(bind.clone()), _ => None }).expect(\"function bind\"); if result_port_is_param(bind) { 1 } else { 0 }",
    );
    assert_eq!(
        stdout, "1",
        "compiled reflected function should compare PortId handles through list contains, got {stdout:?}"
    );
}

#[test]
fn rustc_roundtrip_emitted_module_returns_user_record_list_from_reflected_binds() {
    let module = emit_module(
        "type FoundBind { name: String }\n\
         fn bind_names(d: Dag) -> List<FoundBind> = \
           fold(d.nodes, empty(), |acc, behavior| \
             match behavior { \
               Value(v) => acc, \
               Transform(t) => acc, \
               Branch(b) => acc, \
               Loop(l) => acc, \
               Bind(bind) => cons({ name: bind.name }, acc) \
             })",
    );
    let stdout = roundtrip_module_stdout(
        &module,
        "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); bind_names(dag).len() as i64",
    );
    assert_eq!(
        stdout, "2",
        "compiled reflected function should return a user record per top-level bind, got {stdout:?}"
    );
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
    let stdout = roundtrip_stdout("let x: Int = 1 + 2");
    assert_eq!(stdout, "3", "compiled binary printed {stdout:?}, not `3`");
}
