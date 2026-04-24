//! **Layer:** boundary (TESTING.md § test layers — class-5 rustc roundtrip).

// M1(3) PR-B — Rust emitter acceptance tests.
//
// The success criterion the whole plan validates:
//   `compile_to_dag("let x: Int = 1 + 2").and_then(emit_rust)`
// produces Rust source that, when fed to `rustc`, compiles and runs
// producing `3` on stdout.
//
// The #[ignore]d `rustc_roundtrip` tests run that whole pipeline;
// they're gated because CI environments don't always have `rustc`
// available. Run locally via the consolidated integration binary, e.g.:
//     cargo test -p v3-compiler --test integration -- --ignored --nocapture
//
// Everything else is structural: assert the emitter produced the
// right substring for each kind of program without depending on
// exact formatting.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::emit::{emit as shared_emit, emit_module as shared_emit_module, EmitTarget};
use v3_compiler::emit_rust::{emit_rust, emit_rust_module};
use v3_compiler::test_runner::TestClaimValue;

use crate::common::determinism_fixtures::PROGRAM_FIXTURES;
use crate::common::{HarnessLinkMode, RustcHarness};

static HARNESS: OnceLock<RustcHarness> = OnceLock::new();
fn harness() -> &'static RustcHarness {
    HARNESS.get_or_init(|| RustcHarness::new("emit_rust"))
}

fn emit(source: &str) -> String {
    let dag = compile_to_dag(source, "test.v3").expect("compiles");
    emit_rust(&dag).expect("emits")
}

fn emit_module(source: &str) -> String {
    let dag = compile_to_dag(source, "test.v3").expect("compiles");
    emit_rust_module(&dag).expect("emits module")
}

fn r1_gate_claim_source(claim_name: &str) -> String {
    let gate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r1_gates.dag");
    let gate_source = std::fs::read_to_string(&gate_path)
        .unwrap_or_else(|err| panic!("read {gate_path:?}: {err}"));
    let gate_dag = compile_to_dag(&gate_source, "src/v3/compiler/tests/fixtures/r1_gates.dag")
        .expect("r1_gates.dag compiles");
    let claim = gate_dag
        .declaration_by_name(claim_name)
        .unwrap_or_else(|| panic!("claim `{claim_name}` not found"));
    TestClaimValue::from_declaration(claim)
        .unwrap_or_else(|reason| panic!("claim `{claim_name}` should lower structurally: {reason}"))
        .source
}

#[test]
fn emit_rust_bool_logical_ops_use_spec_carriers() {
    let src = "let x: Bool = true && false || true\n";
    let out = emit(src);
    assert!(
        out.contains("&&") && out.contains("||"),
        "expected Rust `&&` / `||` from Bool OperatorRealization rows; got:\n{out}"
    );
}

#[test]
fn emit_rust_bool_alias_logical_ops_use_base_operator_realization() {
    // Bool literals type as kernel `Bool`, so exercise `&&` / `||` on `MyBool`
    // parameters (operand ports carry the alias id while spec rows key `Bool`).
    let src = "\
type MyBool = Bool
fn f(a: MyBool, b: MyBool) -> MyBool = a && b
fn g(a: MyBool, b: MyBool) -> MyBool = a || b
";
    let out = emit_module(src);
    assert!(
        out.contains("&&") && out.contains("||"),
        "expected alias-of-Bool to reuse Bool OperatorRealization carriers; got:\n{out}"
    );
}

#[test]
fn emit_rust_wrappers_match_shared_entrypoint() {
    let program_source = "\
fn double(x: Int) -> Int = x + x
let result: Int = double(21)
";
    let program_dag =
        compile_to_dag(program_source, "emit_rust_wrapper_program_parity.v3").expect("compiles");
    let shared = shared_emit(&program_dag, EmitTarget::Rust)
        .expect("shared emit")
        .text;
    let wrapper = emit_rust(&program_dag).expect("wrapper emit");
    assert_eq!(shared, wrapper, "emit_rust wrapper drifted from emit::emit");

    let module_source = "\
fn double(x: Int) -> Int = x + x
";
    let module_dag =
        compile_to_dag(module_source, "emit_rust_wrapper_module_parity.v3").expect("compiles");
    let shared_module = shared_emit_module(&module_dag, EmitTarget::Rust)
        .expect("shared module emit")
        .text;
    let wrapper_module = emit_rust_module(&module_dag).expect("wrapper module emit");
    assert_eq!(
        shared_module, wrapper_module,
        "emit_rust_module wrapper drifted from emit::emit_module"
    );
}

fn roundtrip_stdout(source: &str) -> String {
    let source = emit(source);

    let tmp_dir = harness().next_child_dir();
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write rust source");

    let compile = Command::new("rustc")
        // See common::RustcHarness::compile: strip RUSTC_BOOTSTRAP so the ratchet
        // CI step's libtest unlock does not leak into child rustc invocations.
        .env_remove("RUSTC_BOOTSTRAP")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc — install a rust toolchain to run this test");
    assert!(
        compile.success(),
        "rustc failed on emitted source:\n{source}"
    );

    let run = Command::new(&bin_path)
        .output()
        .expect("run compiled binary");
    assert!(run.status.success(), "compiled binary failed");
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

static PROGRAM_HARNESS_BIN: OnceLock<PathBuf> = OnceLock::new();

fn build_program_harness() -> PathBuf {
    let mut body = String::new();
    for fixture in PROGRAM_FIXTURES {
        let emitted = emit(fixture.source);
        // emit_rust produces `fn main() { ... }`; wrapping in a submodule
        // would leave the nested main private. Promote to `pub fn main`
        // so the outer dispatcher can invoke it.
        let emitted_pub_main = emitted.replace("fn main()", "pub fn main()");
        body.push_str(&format!(
            "#[allow(warnings, clippy::all)] pub mod {name} {{ {emitted} }}\n",
            name = fixture.name,
            emitted = emitted_pub_main,
        ));
    }
    body.push_str(
        "fn main() { \
           let name = std::env::args().nth(1).expect(\"program fixture name\"); \
           match name.as_str() { \
        ",
    );
    for fixture in PROGRAM_FIXTURES {
        body.push_str(&format!("\"{0}\" => {0}::main(), ", fixture.name));
    }
    body.push_str(
        "other => panic!(\"unknown program fixture: {other}\"), \
         } \
         }\n",
    );

    // Group 1 emissions are self-contained Rust (no v3_compiler deps),
    // so the batched harness compiles with plain rustc. If one fixture's
    // emission fails to compile, rustc surfaces the file + line — the
    // submodule name narrows attribution.
    harness().compile(&body, "main_bin", HarnessLinkMode::Standalone)
}

fn program_harness_bin() -> &'static Path {
    PROGRAM_HARNESS_BIN
        .get_or_init(build_program_harness)
        .as_path()
}

fn run_program(name: &str) -> String {
    RustcHarness::run(program_harness_bin(), &[name])
}

fn program_expected(name: &str) -> &'static str {
    PROGRAM_FIXTURES
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no PROGRAM_FIXTURES entry for {name:?}"))
        .expected_stdout
}

/// Expected output shape for a reflected-module roundtrip fixture.
enum ReflectedExpected {
    /// Exact stdout string (trimmed).
    Exact(&'static str),
    /// Any positive integer (used for `node_count`, whose exact value is not pinned).
    PositiveInt,
}

/// Descriptor for one reflected-module rustc roundtrip fixture. Each
/// descriptor becomes a submodule in the batched harness; tests
/// dispatch by `name` at runtime.
///
/// Previously each fixture compiled its own rustc binary, paying a
/// fresh linker + codegen cost per test (~3-5s on CI cold cache).
/// Batching all fixtures into one compilation amortizes that cost.
struct ReflectedFixture {
    name: &'static str,
    module_source: &'static str,
    wrapper_body: &'static str,
    expected_stdout: ReflectedExpected,
}

const REFLECTED_FIXTURES: &[ReflectedFixture] = &[
    ReflectedFixture {
        name: "node_count",
        module_source: "fn node_count(d: Dag) -> Int = fold(d.nodes, 0, |n, node| n + 1)",
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); node_count(&dag)",
        expected_stdout: ReflectedExpected::PositiveInt,
    },
    ReflectedFixture {
        name: "bind_count",
        module_source: "fn bind_count(d: Dag) -> Int = fold(d.nodes, 0, |n, behavior| match behavior { Value(v) => n, Transform(t) => n, Branch(b) => n, Loop(l) => n, Bind(bind) => n + 1 })",
        // Subtract the bootstrap baseline so the test still pins the
        // user-program bind count after Lane 2 Stage 2d added
        // `src/v3/std/algebra.dag` + `dimensions.dag`, whose lowered
        // bodies contribute their own Bind nodes to `d.nodes`.
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); let baseline = v3_compiler::dag::Dag::new(); bind_count(&dag) - bind_count(&baseline)",
        expected_stdout: ReflectedExpected::Exact("2"),
    },
    ReflectedFixture {
        name: "singleton_span",
        module_source: "fn singleton_span(bind: BindNode) -> List<SourceSpan> = [bind.span]",
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); let bind = dag.nodes().iter().find_map(|node| match node { v3_compiler::dag::Behavior::Bind(bind) => Some(bind.clone()), _ => None }).expect(\"bind\"); singleton_span(&bind).len() as i64",
        expected_stdout: ReflectedExpected::Exact("1"),
    },
    ReflectedFixture {
        name: "result_port_is_param",
        module_source: "fn result_port_is_param(bind: BindNode) -> Bool = contains(bind.params, bind.result_port)",
        // Filter by source file so the bind picked is the user's `id`
        // — after Lane 2 Stage 2d landed std-module binds with
        // non-empty params, the raw `params.is_empty()` filter was
        // no longer specific enough to isolate user code.
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"fn id(x: Int) -> Int = x\", \"runtime_reflection.v3\").expect(\"compiles\"); let bind = dag.nodes().iter().find_map(|node| match node { v3_compiler::dag::Behavior::Bind(bind) if !bind.params.is_empty() && bind.span.file == \"runtime_reflection.v3\" => Some(bind.clone()), _ => None }).expect(\"function bind\"); if result_port_is_param(&bind) { 1 } else { 0 }",
        expected_stdout: ReflectedExpected::Exact("1"),
    },
    ReflectedFixture {
        name: "bind_names",
        module_source: "type FoundBind { name: String }\n\
             fn bind_names(d: Dag) -> List<FoundBind> = \
               fold(d.nodes, empty(), |acc, behavior| \
                 match behavior { \
                   Value(v) => acc, \
                   Transform(t) => acc, \
                   Branch(b) => acc, \
                   Loop(l) => acc, \
                   Bind(bind) => cons({ name: bind.name }, acc) \
                 })",
        // Same bootstrap-baseline subtraction as `bind_count` — the
        // test pins the two user-program binds (`x`, `y`) even
        // though `bind_names` also materializes a record per std-
        // module bind in `d.nodes`.
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); let baseline = v3_compiler::dag::Dag::new(); (bind_names(&dag).len() as i64) - (bind_names(&baseline).len() as i64)",
        expected_stdout: ReflectedExpected::Exact("2"),
    },
];

/// Lazily-initialized path to the batched reflected-module harness.
/// All five fixtures compile into one binary on first access; each
/// subsequent `run_reflected(name)` call dispatches via argv.
static REFLECTED_HARNESS_BIN: OnceLock<PathBuf> = OnceLock::new();

fn build_reflected_harness() -> PathBuf {
    let mut body = String::new();
    for fixture in REFLECTED_FIXTURES {
        let module = emit_module(fixture.module_source);
        body.push_str(&format!(
            "#[allow(warnings, clippy::all)] \
             pub mod {name} {{ \
               use v3_compiler::dag::*; \
               use v3_compiler::diagnostics::*; \
               {module} \
               pub fn run() -> i64 {{ {wrapper} }} \
             }}\n",
            name = fixture.name,
            module = module,
            wrapper = fixture.wrapper_body,
        ));
    }
    body.push_str(
        "fn main() { \
           let name = std::env::args().nth(1).expect(\"test name arg\"); \
           let value: i64 = match name.as_str() { \
        ",
    );
    for fixture in REFLECTED_FIXTURES {
        body.push_str(&format!("\"{0}\" => {0}::run(), ", fixture.name));
    }
    body.push_str(
        "other => panic!(\"unknown reflected harness test: {other}\"), \
         }; \
         println!(\"{value}\"); \
         }\n",
    );

    harness().compile(&body, "reflected_bin", HarnessLinkMode::WithV3Compiler)
}

fn reflected_harness_bin() -> &'static Path {
    REFLECTED_HARNESS_BIN
        .get_or_init(build_reflected_harness)
        .as_path()
}

fn run_reflected(name: &str) -> String {
    RustcHarness::run(reflected_harness_bin(), &[name])
}

fn reflected_expected(name: &str) -> &'static ReflectedExpected {
    &REFLECTED_FIXTURES
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no REFLECTED_FIXTURES entry for {name:?}"))
        .expected_stdout
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
    assert!(
        out.contains("fn classify(p0: &Sign) -> i64 { match p0 {"),
        "got: {out}"
    );
    assert!(out.contains("Sign::Plus => 0,"), "got: {out}");
    assert!(out.contains("Sign::Minus => 1,"), "got: {out}");
}

#[test]
fn emit_rust_module_match_on_imported_workflow_effect_sum() {
    let src = "\
module emit_match_workflow_effect_smoke
import std.effects { WorkflowEffect }
import std.list { length }
fn classify_wf(w: WorkflowEffect) -> Int = match w {
  LinearEffect { ops: ops } => length(ops)
  BranchEffect { arms: arms } => 1
  LoopEffect { body: body } => 2
  ParallelEffect { branches: branches } => 3
}
";
    let out = emit_module(src);
    assert!(
        out.contains("fn classify_wf") && out.contains("WorkflowEffect::LinearEffect"),
        "expected emitted lens-style module to lower `match` on imported \
         user-defined `WorkflowEffect` with qualified enum patterns; got:\n{out}"
    );
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
    assert!(
        out.contains("fn unwrap_or_zero(p0: &BoxedInt) -> i64 { match p0 {"),
        "got: {out}"
    );
    assert!(
        out.contains("BoxedInt::Boxed { _0: value } => (*(value)),"),
        "got: {out}"
    );
    assert!(out.contains("BoxedInt::Empty => 0,"), "got: {out}");
}

/// Day-1 T-Sub receipt gate for `sub_match_over_user_sum`.
///
/// Audit result: the first-class implementation path already exists on `main`.
/// The parser accepts source-local `type Choice = ...` sums, lowering carries
/// them as `Disj` + `Branch`, inference resolves arm patterns, and Rust emit
/// renders the general enum-pattern `match` path. The surrounding tests already
/// cover string-level Rust receipts for no-payload sums, payload sums, and
/// imported sums.
///
/// This gate intentionally adds no implementation. Its narrower job is to keep
/// the named R1/T-Sub surface live as an unignored, end-to-end pipeline receipt:
/// parse -> lower -> infer -> Rust emit -> rustc link -> runtime execution.
#[test]
fn sub_match_over_user_sum_links_and_runs() {
    let source = r1_gate_claim_source("sub_match_over_user_sum");
    assert_eq!(roundtrip_stdout(&source), "true");
}

#[test]
fn emit_rust_named_single_field_payload_routes_field_access_through_binding() {
    let out = emit(
        "type Point { x: Int y: Int }
type Wrapped = Wrap { inner: Point } | Empty
fn unwrap_or_zero(w: Wrapped) -> Int = match w { Wrap(payload) => payload.inner.x, Empty => 0 }
let zero: Int = 0",
    );
    assert!(
        out.contains("Wrapped::Wrap { inner: payload } => (payload).x,"),
        "expected named single-field payload access to route through the bound field, got: {out}"
    );
    assert!(
        !out.contains("(payload).inner"),
        "named single-field payload access must not project through a synthetic whole-payload binding, got: {out}"
    );
}

/// E-5 / Lane 1 Stage 1c pilot — unused match-arm payload bindings
/// render as `_` under `rust_clean_emission.pattern_bindings =
/// EmitUnderscoreWhenUnused`. Before the pilot the emitter rendered
/// `Boxed { _0: value } => 0`, which fired `unused_variables` under
/// `rustc -D warnings`; after the pilot it renders `Boxed { _0: _ }
/// => 0`, which passes the invariant by construction.
#[test]
fn emit_rust_unused_payload_binding_renders_as_underscore() {
    let out = emit(
        "type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(value) => 0, Empty => 1 }
let zero: Int = 0",
    );
    assert!(
        out.contains("BoxedInt::Boxed { _0: _ } => 0,"),
        "expected unused payload to render as `_`, got: {out}"
    );
    assert!(
        !out.contains("BoxedInt::Boxed { _0: value }"),
        "unused binding leaked the identifier name, got: {out}"
    );
    assert!(out.contains("BoxedInt::Empty => 1,"), "got: {out}");
}

/// Multi-field payloads still need a full variant pattern for type
/// shape, but an unused whole-payload binding must not be rendered
/// as `_ @ Variant { ... }` because Rust only permits identifiers on
/// the left side of `@`.
#[test]
fn emit_rust_unused_multi_field_payload_binding_avoids_wildcard_alias() {
    let out = emit(
        "type IntList = Empty | Cons { head: Int, tail: IntList }
fn ignore_payload(list: IntList) -> Int = match list { Empty => 0, Cons(payload) => 1 }
let zero: Int = 0",
    );
    assert!(
        out.contains("IntList::Cons { head: _, tail: _ } => 1,"),
        "expected unused multi-field payload to render as a plain variant pattern, got: {out}"
    );
    assert!(
        !out.contains("_ @ IntList::Cons"),
        "unused multi-field payload rendered an invalid wildcard alias, got: {out}"
    );
}

/// E-5 / Lane 1 Stage 1c pilot (rustc roundtrip) — emitted code with
/// an unused match-arm payload binding passes `rustc -D
/// unused_variables`. Proves the rule fires end-to-end, not just
/// that the emission string looks right.
///
/// Gated behind `#[ignore]` for the same reason as the other
/// `rustc_roundtrip_*` tests — CI sandboxes don't always carry a
/// toolchain. Run locally:
///
///     cargo test -p v3-compiler --test integration \
///         emit_rust_unused_payload_binding_passes_deny_unused \
///         -- --ignored --nocapture
/// E-5 / Lane 1 Stage 1c PR 4 — the pilot Rust source passes
/// `rust_clean_emission.post_emit_verifier` as invoked through the
/// shared harness. The harness reads command / args / syntax_only /
/// expected_exit_code / output_policy from the contract; the test
/// does not hardcode `rustc -D warnings`. Adding a verifier flag
/// (e.g. `--edition=2024`) means editing `spec/rust.dag` — the
/// harness picks it up automatically.
///
/// Gated behind `#[ignore]` like `emit_rust_unused_payload_binding_passes_deny_unused`
/// above — CI sandboxes don't always carry rustc. Run locally:
///
///     cargo test -p v3-compiler --test integration \
///         rust_pilot_source_passes_post_emit_verifier_harness \
///         -- --ignored --nocapture
#[test]
#[ignore]
fn rust_pilot_source_passes_post_emit_verifier_harness() {
    use v3_compiler::post_emit_verifier::{parse_post_emit_verifier, run_post_emit_verifier};
    // `ignore_payload` has to be called so rustc's dead_code
    // (implied by -D warnings) does not fire on it. The pilot rule
    // under test is `pattern_bindings = EmitUnderscoreWhenUnused`
    // — the unused `value` inside the match arm; the let-binding
    // below wires the fn into `main` so the rest of the emitted
    // module stays warning-clean.
    let source = "type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(value) => 0, Empty => 1 }
let result: Int = ignore_payload(Empty)";
    let dag =
        v3_compiler::compile_to_dag(source, "rust_post_emit_verifier.v3").expect("source compiles");
    let spec = dag
        .rust_clean_emission_spec()
        .expect("rust_clean_emission cached");
    let binding = parse_post_emit_verifier(&dag, spec).expect("parse contract");
    let rendered = v3_compiler::emit_rust::emit_rust(&dag).expect("emits rust");
    let tmp_dir = harness().next_child_dir();
    let src_path = tmp_dir.join("main.rs");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write rust source");
    run_post_emit_verifier(&binding, &src_path)
        .expect("rust post_emit_verifier rejected pilot source — E-5 contract regression");
}

#[test]
#[ignore]
fn emit_rust_unused_payload_binding_passes_deny_unused() {
    let source = emit(
        "type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(value) => 0, Empty => 1 }
let zero: Int = 0",
    );
    let wrapped = format!("#![deny(unused_variables)]\n{source}");
    let tmp_dir = harness().next_child_dir();
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("deny_unused_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(wrapped.as_bytes()))
        .expect("write rust source");
    let status = Command::new("rustc")
        // See common::RustcHarness::compile: strip RUSTC_BOOTSTRAP so the ratchet
        // CI step's libtest unlock does not leak into child rustc invocations.
        .env_remove("RUSTC_BOOTSTRAP")
        .arg("--edition=2021")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc");
    assert!(
        status.success(),
        "emitted code tripped #[deny(unused_variables)] — E-5 pilot regression"
    );
}

#[test]
fn emit_rust_record_literal_uses_value_construction_syntax() {
    let out = emit(
        "type Point { x: Int y: Int }
let p: Point = { x: 1, y: 2 }",
    );
    assert!(out.contains("pub struct Point {"), "got: {out}");
    assert!(
        out.contains("let p: Point = Point { x: 1, y: 2 };"),
        "got: {out}"
    );
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
    const EMITTER_SOURCE: &str = include_str!("../../src/emit_rust.rs");

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
        "\"List\"",
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
    assert_ne!(harness().next_child_dir(), harness().next_child_dir());
}

// The nine tests below dispatch into one batched rustc-roundtrip
// program harness. Fixture sources live in `PROGRAM_FIXTURES`; the
// harness compiles on first access and is reused across tests.
//
// These run unconditionally (no `#[ignore]`), giving per-fixture test
// names in cargo output. `emit_rust_fixtures_rustc_green` (below) is the
// `#[ignore]`d full-matrix gate that sweeps the same fixtures in one shot;
// both coexist because the per-fixture names are useful for triage.

#[test]
fn rustc_roundtrip_list_fold_prints_six() {
    let name = "list_fold_six";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_generic_list_fold_prints_one() {
    let name = "generic_list_fold_one";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_list_map_then_fold_prints_twelve() {
    let name = "list_map_then_fold_twelve";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_list_filter_then_fold_prints_seven() {
    let name = "list_filter_then_fold_seven";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_nested_list_builtins_inside_lambda_prints_six() {
    let name = "nested_list_builtins_inside_lambda_six";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_user_function_call_prints_three() {
    let name = "user_function_call_three";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_recursive_function_call_prints_six() {
    let name = "recursive_function_call_six";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_record_literal_through_function_prints_one() {
    let name = "record_literal_through_function_one";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

#[test]
fn rustc_roundtrip_user_sum_match_prints_zero() {
    let name = "user_sum_match_zero";
    let stdout = run_program(name);
    let expected = program_expected(name);
    assert_eq!(
        stdout, expected,
        "compiled binary printed {stdout:?}, not {expected:?}"
    );
}

// The five reflected-module tests below share one batched harness
// built lazily by `reflected_harness_bin()`. Each test invokes the
// same binary with a different argv[1]; fixture source lives in
// `REFLECTED_FIXTURES`.

#[test]
fn rustc_roundtrip_emitted_module_invokes_reflected_dag_function() {
    let name = "node_count";
    let stdout = run_reflected(name);
    match reflected_expected(name) {
        ReflectedExpected::PositiveInt => assert!(
            stdout.parse::<i64>().is_ok_and(|n| n > 0),
            "compiled reflected Dag function should return a positive node count, got {stdout:?}"
        ),
        ReflectedExpected::Exact(expected) => assert_eq!(stdout, *expected),
    }
}

#[test]
fn rustc_roundtrip_emitted_module_matches_reflected_behavior_payloads() {
    let name = "bind_count";
    let stdout = run_reflected(name);
    let ReflectedExpected::Exact(expected) = reflected_expected(name) else {
        panic!("bind_count expected Exact")
    };
    assert_eq!(
        stdout, *expected,
        "compiled reflected Behavior match should count the two top-level binds, got {stdout:?}"
    );
}

#[test]
fn rustc_roundtrip_emitted_module_returns_reflected_source_span_list() {
    let name = "singleton_span";
    let stdout = run_reflected(name);
    let ReflectedExpected::Exact(expected) = reflected_expected(name) else {
        panic!("singleton_span expected Exact")
    };
    assert_eq!(
        stdout, *expected,
        "compiled reflected function returning List<SourceSpan> should yield a singleton list, got {stdout:?}"
    );
}

#[test]
fn rustc_roundtrip_emitted_module_compares_reflected_port_ids_in_list_contains() {
    let name = "result_port_is_param";
    let stdout = run_reflected(name);
    let ReflectedExpected::Exact(expected) = reflected_expected(name) else {
        panic!("result_port_is_param expected Exact")
    };
    assert_eq!(
        stdout, *expected,
        "compiled reflected function should compare PortId handles through list contains, got {stdout:?}"
    );
}

#[test]
fn rustc_roundtrip_emitted_module_returns_user_record_list_from_reflected_binds() {
    let name = "bind_names";
    let stdout = run_reflected(name);
    let ReflectedExpected::Exact(expected) = reflected_expected(name) else {
        panic!("bind_names expected Exact")
    };
    assert_eq!(
        stdout, *expected,
        "compiled reflected function should return a user record per top-level bind, got {stdout:?}"
    );
}

/// End-to-end roundtrip test: emit Rust from a v3 program, feed the
/// Rust source to `rustc`, run the resulting binary, assert stdout.
/// Gated behind `#[ignore]` because CI runners often don't have a
/// Rust toolchain available inside the test sandbox. Run locally:
///
///     cargo test -p v3-compiler --test integration \
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

/// Gate test: the full Rust fixture matrix (all 9 program fixtures and
/// all 5 reflected-module fixtures) compiles and produces the expected
/// output under a real `rustc`. Passes when every individual
/// `rustc_roundtrip_*` test would pass.
///
/// Gated behind `#[ignore]` because CI runners may not have `rustc`.
/// The individual `rustc_roundtrip_*` tests above are unignored (they
/// run unconditionally); this gate sweeps them in one shot for a fast
/// go/no-go signal. Run locally:
///
///     cargo test -p v3-compiler --test integration \
///         emit_rust_fixtures_rustc_green -- --ignored --nocapture
#[test]
#[ignore]
fn emit_rust_fixtures_rustc_green() {
    let mut failures: Vec<String> = Vec::new();

    for fixture in PROGRAM_FIXTURES {
        let stdout = run_program(fixture.name);
        if stdout != fixture.expected_stdout {
            failures.push(format!(
                "program {:?}: expected {:?}, got {stdout:?}",
                fixture.name, fixture.expected_stdout,
            ));
        }
    }

    for fixture in REFLECTED_FIXTURES {
        let stdout = run_reflected(fixture.name);
        let (ok, label) = match &fixture.expected_stdout {
            ReflectedExpected::Exact(expected) => (stdout == *expected, format!("{expected:?}")),
            ReflectedExpected::PositiveInt => (
                stdout.parse::<i64>().is_ok_and(|n| n > 0),
                "positive integer".to_owned(),
            ),
        };
        if !ok {
            failures.push(format!(
                "reflected {:?}: expected {label}, got {stdout:?}",
                fixture.name,
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "emit_rust_fixtures_rustc_green: {} fixture(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
