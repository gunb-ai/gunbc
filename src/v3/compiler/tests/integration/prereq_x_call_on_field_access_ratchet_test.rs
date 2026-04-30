//! Prereq-X (call-on-field-access) blocker ratchet.
//!
//! Records the exact parser-grammar gap that blocks `fold_lens<C>` consumer
//! wiring (Prereq-3b dispatch). The audit at
//! `docs/design-prereq-x-ho-field-call.md` (PR #1264) decomposes this
//! into X1 (call-on-field-access), X2 (call-on-Var if not subsumed by X1),
//! and X3 (explicit block expressions inside `=` bodies).
//!
//! Each assertion below pins the *diagnostic shape* of a present gap. When
//! the implementation lane lands, these tests flip to red and the lane
//! owner is expected to retire each fixture (or re-shape it to a positive
//! parse) at the same time as the parser/lowerer change. No hand-Rust
//! scaffolding for `fold_lens<C>` ships here — the gap is structural; the
//! only honest deliverable until X1/X3 land is this ratchet plus the audit.

use v3_compiler::{parse_for_test, tokenize_for_test};

/// Control: the `type Wrapper { f: fn(Int) -> Int }` declaration on its
/// own parses cleanly. Confirms the X1/X3 fixtures' parse failures
/// originate at the `w.f(x)` / `{ let g = w.f; g(x) }` call site, not at
/// the type declaration.
#[test]
fn control_arrow_typed_field_decl_parses() {
    let src = "type Wrapper { f: fn(Int) -> Int }\n";
    let tokens = tokenize_for_test(src, "control.v3").expect("tokenize");
    parse_for_test(&tokens, "control.v3").expect(
        "Arrow-typed field declaration must parse cleanly so X1/X3 isolate the call-site gap.",
    );
}

/// X1: direct call-on-field-access in fn body. The `lens.read(d, b)`
/// dispatch shape used by `fold_lens<C>` reduces to exactly this.
#[test]
fn x1_direct_field_call_blocked() {
    let src = r#"
type Wrapper { f: fn(Int) -> Int }

fn invoke(w: Wrapper, x: Int) -> Int = w.f(x)
"#;
    let tokens = tokenize_for_test(src, "x1.v3").expect("tokenize");
    let err = parse_for_test(&tokens, "x1.v3").err().expect(
        "Prereq-X1 still blocks `w.f(x)` — if this test panics, the parser was extended; retire this ratchet.",
    );
    assert!(
        err.message().contains("LParen"),
        "X1 diagnostic shape changed; verify against #1264 audit. Got: {}",
        err.message()
    );
}

/// X3: brace-bodied block expression inside `=` body, with a `let` head.
/// Required to factor `let g = w.f; g(x)` out of a SingleRoot fold.
#[test]
fn x3_brace_block_with_let_head_blocked() {
    let src = r#"
type Wrapper { f: fn(Int) -> Int }

fn invoke(w: Wrapper, x: Int) -> Int = { let g = w.f; g(x) }
"#;
    let tokens = tokenize_for_test(src, "x3.v3").expect("tokenize");
    let err = parse_for_test(&tokens, "x3.v3").err().expect(
        "Prereq-X3 still blocks `= { let ...; ... }` — if this test panics, retire this ratchet.",
    );
    assert!(
        err.message().contains("LParen") || err.message().contains("KwLet"),
        "X3 diagnostic shape changed; verify against #1264 audit. Got: {}",
        err.message()
    );
}
