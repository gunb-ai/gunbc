// Pure-emitter regression pin for the multi-field struct-variant
// match fix in `emit_rust.rs` (see `destructured_field_alias` and
// `render_path_body`'s `field_overrides` population).
//
// Before the fix, pattern-matching on any `TypeConnective` struct
// variant with more than one field (`Arrow`, `Cardinality`,
// `Instantiation`) emitted `a @ TypeConnective::Arrow { _, _, _ }`
// and then routed downstream `a.body` as `(a).body` — a field
// access on the enum type, which Rust rejects.
//
// This test drives `emit_rust_module` on a minimal `.dag` that
// matches on `TypeConnective::Arrow` and reads `.body`, then
// asserts the emitted Rust uses the aliased-field pattern
// (`body: __a_body`) and the aliased reference (`__a_body`) in
// place of the broken `(a).body`. Guards the fix even if
// `lens_structural_resolution.dag` is later modified or deleted.

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

const PROBE_DAG: &str = "\
module test.emit.multi_field_struct_variant

import std.substrate { Declaration, TypeConnective, ArrowBody }

fn is_arrow_pending(decl: Declaration) -> Bool =
  match decl.connective {
    Atom(payload) => false
    Conj(c) => false
    Disj(d) => false
    Arrow(a) => is_pending(a.body)
    Cardinality(c) => false
    Instantiation(i) => false
  }

fn is_pending(body: ArrowBody) -> Bool =
  match body {
    UserDefined(n) => false
    ExternalRealization(e) => false
    Pending => true
    NoBody => false
    Unparsed(s) => false
  }
";

fn emit(module_source: &str) -> String {
    let dag = compile_to_dag(module_source, "probe.dag").expect("probe .dag compiles");
    assert!(
        dag.diagnostics().is_empty(),
        "probe .dag must compile cleanly, got {:?}",
        dag.diagnostics()
    );
    emit_rust_module(&dag).expect("emit Rust module")
}

#[test]
fn multi_field_struct_variant_match_emits_aliased_field_destructure() {
    // The Arrow match arm should bind `body` to `__a_body` (the
    // aliased destructure) — not leave it as a wildcard and then
    // fall through to the broken `(a).body` access.
    let rust = emit(PROBE_DAG);
    assert!(
        rust.contains("body: __a_body"),
        "expected `body: __a_body` destructure in emitted Rust; got:\n{rust}"
    );
    // Pre-fix form was `body: _,` or `body: _ }` — a literal
    // wildcard rather than an aliased binding. Check both variants
    // explicitly; the positive `body: __a_body` assertion above is
    // the primary signal, but these catch a pattern regression
    // that happens to also keep the aliased binding somewhere
    // downstream.
    assert!(
        !rust.contains("body: _,") && !rust.contains("body: _ }"),
        "wildcard `body: _` would drop the binding the fix routes through; got:\n{rust}"
    );
}

#[test]
fn multi_field_struct_variant_arm_body_uses_aliased_reference() {
    // `a.body` in the .dag must translate to `__a_body` (the
    // aliased local) in the emitted arm body, NOT to `(a).body`
    // (the pre-fix form that does not compile because
    // `&TypeConnective` has no `body` field).
    let rust = emit(PROBE_DAG);
    assert!(
        rust.contains("__a_body"),
        "emitted arm body must reference the aliased `__a_body`; got:\n{rust}"
    );
    assert!(
        !rust.contains("(a).body") && !rust.contains("((a).body)"),
        "pre-fix `(a).body` access must not appear in emitted Rust; got:\n{rust}"
    );
}

#[test]
fn multi_field_struct_variant_emitted_rust_is_valid_syntax() {
    // Pipe the emitted code through rustfmt; rustfmt only succeeds
    // on syntactically valid Rust, so a failure here means the
    // pattern emitter is producing a token stream rustc can't
    // parse. Headed-off at the pattern layer rather than waiting
    // for a downstream type-check failure in a consumer crate.
    use std::io::Write;
    use std::process::{Command, Stdio};

    let rust = emit(PROBE_DAG);
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn rustfmt");
    child
        .stdin
        .as_mut()
        .expect("rustfmt stdin")
        .write_all(rust.as_bytes())
        .expect("write to rustfmt");
    let output = child.wait_with_output().expect("wait rustfmt");
    assert!(
        output.status.success(),
        "emitted Rust did not parse through rustfmt; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
