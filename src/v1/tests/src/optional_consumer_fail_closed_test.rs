//! Residual first-Option family (Route-A): a genuinely OPTIONAL value (`-> T?`, e.g.
//! the modeled `first` which returns `OptionalOf<T>`) flowing into a NON-optional
//! consumer parameter is the model hole the renderer fix could not absorb (the
//! signature now renders `Option<T>` faithfully, so the mismatch `expected Rc<T>,
//! found Option<Rc<T>>` is exposed at the call site, not hidden).
//!
//! The emitter resolves this by construction at exactly those sites — and ONLY those
//! sites — with a §5 FAIL-CLOSED unwrap: a located `.expect("fail-closed: ...")` that
//! names the consumer (param index + callee) so an empty Optional at runtime aborts
//! loudly with a diagnostic, rather than fabricating a value. It is explicitly NOT
//! `unwrap_or_default` / `unwrap_or_else(<fabricated>)` (that would be a §5 fail-OPEN:
//! a silent wrong answer). The unwrap is type-DERIVED: emitted iff the declared
//! parameter is required AND the argument's resolved type is optional.
//!
//! Two teeth:
//!   - Part A (emit construction): a required-param consumer fed an optional arg emits
//!     the located `.expect("fail-closed:`; a control whose arg is non-optional does not.
//!   - Part B (discriminating EXECUTION): the empty case actually fails closed — the
//!     emitted unwrap shape, run on `None`, PANICS, whereas the rejected fail-open shape
//!     (`unwrap_or_default`) silently returns a fabricated default. This is the §5
//!     distinction made executable, not asserted.

use crate::helpers::compile_dag_target;
use v1_compiler::v1_compiler_artifact::RenderTarget;

fn emit(source: &str) -> String {
    compile_dag_target(source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

// === Part B: the empty case actually FAILS CLOSED (execution) ===
// Mirrors the emitted unwrap shape exactly: an optional produced by `first` over an
// empty collection, unwrapped at a required consumer slot. The emitted code uses
// `.expect("fail-closed: ...")`; on `None` that PANICS. The §5-WRONG alternative we
// reject (`unwrap_or_default`) would instead fabricate a default and NOT panic — the
// control below proves the two diverge precisely on the empty input.
#[test]
fn empty_optional_unwrap_panics_not_fabricates() {
    let empty: Vec<i64> = vec![];

    // The fail-CLOSED shape the emitter produces: located `.expect` on the optional.
    let panicked = std::panic::catch_unwind(|| {
        let optional: Option<i64> = empty.first().cloned();
        optional.expect(
            "fail-closed: an optional value flowed into non-optional parameter 0 of consume (empty Optional at runtime)",
        )
    })
    .is_err();
    assert!(
        panicked,
        "an empty Optional at a required slot must FAIL CLOSED (panic), not fabricate a value"
    );

    // The fail-OPEN shape we explicitly reject: unwrap_or_default fabricates 0 silently.
    let fabricated: i64 = empty.first().cloned().unwrap_or_default();
    assert_eq!(
        fabricated, 0,
        "control: unwrap_or_default would silently fabricate 0 on empty — the §5 fail-OPEN path the emitter must NOT take"
    );
}

// === Part A: the emitter emits the located fail-closed unwrap, type-derived ===
// `maybe` returns `Int?` (declared `CardOptional`); fed into `consume`'s required
// `x: Int` slot it is exactly the optional-into-required model hole. The control feeds
// the same required slot a non-optional literal, proving the unwrap is type-DERIVED
// (param required AND arg resolved-type optional), not blanket.
const CONSUME_AND_MAYBE: &str = "module failclosed.fixture\n\nfn maybe(flag: Bool) -> Int? {\n  if flag { Present { value: 1 } } else { none }\n}\n\nfn consume(x: Int) -> Int {\n  x\n}\n";

// The emitted `drive` fn body (from `fn drive` to the next top-level `pub fn`), so the
// assertions see only the call site under test, not unrelated runtime modules (which
// legitimately use `unwrap_or_default` for their own reasons).
fn drive_fn(emitted: &str) -> String {
    let start = emitted
        .find("fn drive")
        .unwrap_or_else(|| panic!("`drive` was not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest[8..]
        .find("\npub fn ")
        .map(|i| i + 8)
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn required_param_optional_arg_emits_located_fail_closed_unwrap() {
    let source = format!(
        "{CONSUME_AND_MAYBE}\nfn drive(flag: Bool) -> Int {{\n  consume(x: maybe(flag: flag))\n}}\n"
    );
    let emitted = emit(&source);
    let drive = drive_fn(&emitted);
    assert!(
        drive.contains(".expect(\"fail-closed:"),
        "an optional arg (`maybe(..) -> Int?`) into a required param (`consume(x: Int)`) must emit a located fail-closed `.expect`, got:\n{drive}"
    );
    assert!(
        !drive.contains("unwrap_or_default") && !drive.contains("unwrap_or_else"),
        "the fail-closed unwrap must NOT fabricate (no unwrap_or_default/unwrap_or_else):\n{drive}"
    );
}

#[test]
fn required_param_nonoptional_arg_stays_bare() {
    // Discriminating control: a non-optional arg into the same required param must NOT
    // get the unwrap — the construction is type-derived, not blanket.
    let source = format!("{CONSUME_AND_MAYBE}\nfn drive(y: Int) -> Int {{\n  consume(x: y)\n}}\n");
    let emitted = emit(&source);
    let drive = drive_fn(&emitted);
    assert!(
        !drive.contains(".expect(\"fail-closed:"),
        "a non-optional arg must NOT receive a fail-closed unwrap, got:\n{drive}"
    );
}
