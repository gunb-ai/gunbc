//! **Layer:** integration
//!
//! T-Sub `sub_charclass_in_std_unicode` — **tokenizer half / bounded interim**
//! (ROADMAP.md:358). This gate ratchets: (1) `CharClass` + `char_in_class` exist
//! in `dsl/std/unicode.dag`; (2) generated `tokenize_generated.rs` routes ASCII
//! scanner classes through `tokenize_char_class` (no hidden `is_ascii_*` in
//! emitted Rust); (3) the mirror matches Rust’s ASCII helpers on 0..=127.
//!
//! **Out of scope for this gate:** `syntax.dag` / `std.syntax` operator-symbol
//! and keyword-map retagging; structural `CharClass` `data` in `tokenize.dag`
//! (blocked on M1(2.8) class-5 gap #3 — list / sum-variant literals in `data`
//! bodies). Full lane closure = those follow-ups + deleting the Rust mirror once
//! the compiler can lower class rows from `.dag`.
//!
//! **Sync boundary (API review):** parity here is `byte_matches` vs `u8::is_ascii_*`,
//! not evaluated `char_in_class` from `.dag`; keep `tokenize_char_class.rs` and
//! `unicode.dag` predicates edited together until an interpreter-backed check exists.

use v3_compiler::tokenize_char_class::{byte_matches, TokenizerCharClass};

const UNICODE_DAG: &str = include_str!("../../../../../dsl/std/unicode.dag");
const TOKENIZE_GENERATED: &str = include_str!("../../src/tokenize_generated.rs");

#[test]
fn sub_charclass_in_std_unicode_gate_unicode_dag_defines_char_class() {
    assert!(
        UNICODE_DAG.contains("type CharClass")
            && UNICODE_DAG.contains("fn char_in_class")
            && UNICODE_DAG.contains("Whitespace")
            && UNICODE_DAG.contains("IdentContinue"),
        "expected `CharClass` sum + `char_in_class` predicate in `dsl/std/unicode.dag` authority"
    );
}

#[test]
fn sub_charclass_in_std_unicode_gate_generated_tokenizer_avoids_ascii_host_predicates() {
    assert!(
        !TOKENIZE_GENERATED.contains("is_ascii_whitespace")
            && !TOKENIZE_GENERATED.contains("is_ascii_digit")
            && !TOKENIZE_GENERATED.contains("is_ascii_alphabetic")
            && !TOKENIZE_GENERATED.contains("is_ascii_alphanumeric"),
        "tokenize_generated.rs should route ASCII classes through `byte_matches` / `TokenizerCharClass`, \
         not std-lib `is_ascii_*` helpers"
    );
    assert!(
        TOKENIZE_GENERATED.contains("byte_matches")
            && TOKENIZE_GENERATED.contains("TokenizerCharClass::Whitespace")
            && TOKENIZE_GENERATED.contains("TokenizerCharClass::Digit")
            && TOKENIZE_GENERATED.contains("TokenizerCharClass::IdentStart")
            && TOKENIZE_GENERATED.contains("TokenizerCharClass::IdentContinue"),
        "expected structural CharClass projection wired into generated tokenizer"
    );
}

#[test]
fn sub_charclass_in_std_unicode_gate_byte_matches_locks_ascii_scanner_semantics() {
    for byte in 0u8..=127 {
        let b = byte;
        assert_eq!(
            byte_matches(b, TokenizerCharClass::Whitespace),
            b.is_ascii_whitespace(),
            "Whitespace mismatch at {byte}"
        );
        assert_eq!(
            byte_matches(b, TokenizerCharClass::Digit),
            b.is_ascii_digit(),
            "Digit mismatch at {byte}"
        );
        assert_eq!(
            byte_matches(b, TokenizerCharClass::IdentStart),
            b.is_ascii_alphabetic() || b == b'_',
            "IdentStart mismatch at {byte}"
        );
        assert_eq!(
            byte_matches(b, TokenizerCharClass::IdentContinue),
            b.is_ascii_alphanumeric() || b == b'_',
            "IdentContinue mismatch at {byte}"
        );
    }
}
