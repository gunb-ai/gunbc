//! Route-A final-6-to-zero: discriminating witnesses for the three emitter families
//! that cleared the residual E0308 tail (647 -> 0 E0308) in the faithful `--emit-fresh`
//! seed. Each fix is faithful CONSTRUCTION derived from the declared type, with a
//! negative control proving it is type-derived (not a blanket transform). NOTE: 0 E0308
//! is a MILESTONE, not cargo-green — 32 other-family errors (measure-tower, etc.) remain
//! pre-existing and are tracked separately.

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

// The emitted body of `fn <name>` (from `fn <name>` to the next top-level `pub fn`), so
// assertions see only the construct under test, not unrelated runtime modules.
fn fn_body(emitted: &str, name: &str) -> String {
    let needle = format!("fn {name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fn `{name}` not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest[needle.len()..]
        .find("\npub fn ")
        .map(|i| i + needle.len())
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

// ===================== Family &str: string-concat borrow seam =====================
// Rust's `String + ` requires `String + &str`; the emitter must borrow the RHS of a
// string `+`. The control proves it is gated on string-typed operands (numeric `+`
// is untouched), not a blanket borrow.

#[test]
fn string_concat_borrows_rhs() {
    let source =
        "module strconcat.fixture\n\nfn label(x: String) -> String {\n  \"p:\" + x\n}\n";
    let body = fn_body(&emit(source), "label");
    assert!(
        body.contains("+ &"),
        "a String + String concat must borrow the RHS (String + &str), got:\n{body}"
    );
}

#[test]
fn numeric_plus_not_borrowed() {
    // Control: numeric `+` must NOT borrow — the borrow is string-typed-derived.
    let source = "module numplus.fixture\n\nfn addi(a: Int, b: Int) -> Int {\n  a + b\n}\n";
    let body = fn_body(&emit(source), "addi");
    assert!(
        !body.contains("+ &"),
        "numeric `+` must not borrow its RHS, got:\n{body}"
    );
}

// ===================== Family Box/Rc: BoundedLattice bare top/bottom =====================
// `BoundedLattice<T>` has `top: T, bottom: T` (bare generic T), so the data-def emission
// must NOT `Box::new` them. The `meet`/`join` fn fields still get `Rc::new` (Rc<dyn Fn>) —
// proving the fix removed only the wrong Box-wrap, not all field wrapping.

#[test]
fn bounded_lattice_top_bottom_are_bare_not_boxed() {
    let source = concat!(
        "module blat.fixture\n",
        "import std.algebra { BoundedLattice }\n\n",
        "fn meetb(a: Bool, b: Bool) -> Bool {\n  a\n}\n\n",
        "fn joinb(a: Bool, b: Bool) -> Bool {\n  b\n}\n\n",
        "data blat: BoundedLattice<Bool> = {\n",
        "  meet: meetb\n  join: joinb\n  top: true\n  bottom: false\n}\n"
    );
    let emitted = emit(source);
    assert!(
        !emitted.contains("top: Box::new(") && !emitted.contains("bottom: Box::new("),
        "BoundedLattice top/bottom (bare T) must NOT be Box::new'd, got:\n{emitted}"
    );
    assert!(
        emitted.contains("meet: Rc::new("),
        "BoundedLattice meet (Rc<dyn Fn>) must still be Rc::new'd — fix removed only the Box, not fn-Rc:\n{emitted}"
    );
}

// ===================== Family fn-item: single-field record un-collapse =====================
// A single-field product whose field is fn-typed (Arrow) must un-collapse to
// `Rc::new(R { field: Rc::new(fn) })` — the bare collapse would drop the nominal type +
// the fn->Rc coercion (a fn-item where `Rc<dyn Fn>` is wanted). Three evidences:

#[test]
fn fn_field_single_record_uncollapses_with_rc_wrap() {
    // Effectiveness: `pick` is fn-typed and `{pick}` is a UNIQUE single-field set.
    let source = concat!(
        "module schemefx.fixture\n",
        "import std.algebra { Ordering }\n\n",
        "fn cmp(a: Int, b: Int) -> Ordering {\n  Equal\n}\n\n",
        "type Scheme {\n  pick: fn(Int, Int) -> Ordering\n}\n\n",
        "data s: Scheme = {\n  pick: cmp\n}\n"
    );
    let emitted = emit(source);
    assert!(
        emitted.contains("Scheme {") && emitted.contains("Rc::new(cmp)"),
        "a single fn-field record must un-collapse to `Scheme {{ pick: Rc::new(cmp) }}`, got:\n{emitted}"
    );
}

#[test]
fn plain_single_field_newtype_stays_collapsed() {
    // Transparency control: a plain (non-Arrow) single field must NOT un-collapse —
    // it stays byte-identical transparent (no `Tag {` wrapper emitted for the value).
    let source = "module plainnt.fixture\n\ntype Tag {\n  slot: Int\n}\n\ndata t: Tag = {\n  slot: 7\n}\n";
    let emitted = emit(source);
    let body = fn_body(&emitted, "t");
    assert!(
        !body.contains("Tag {"),
        "a plain single-field newtype must stay collapsed (transparent), not wrap in `Tag {{ }}`, got:\n{body}"
    );
}

#[test]
fn ambiguous_single_field_name_fails_closed_to_collapse() {
    // Fail-closed on ambiguity: two structs share the single field name `item`, so
    // find_unique_struct_name_by_fields([item]) is ambiguous (count != 1) -> None ->
    // the fix must FALL BACK to bare collapse and NEVER guess a nominal R. (It must not
    // emit `AlphaItem {` / `BetaItem {` for the wrong type.)
    let source = concat!(
        "module ambig.fixture\n\n",
        "type AlphaItem {\n  item: Int\n}\n\n",
        "type BetaItem {\n  item: Int\n}\n\n",
        "data a: AlphaItem = {\n  item: 1\n}\n"
    );
    let emitted = emit(source);
    let body = fn_body(&emitted, "a");
    assert!(
        !body.contains("AlphaItem {") && !body.contains("BetaItem {"),
        "an ambiguous single-field name must fail closed to bare collapse, never guess a nominal struct, got:\n{body}"
    );
}
