//! **Layer:** integration
//!
//! T-Sub `sub_charclass_in_std_unicode` (ROADMAP.md:358): `CharClass` lives in
//! `dsl/std/unicode.dag`; the SG-1 tokenizer must not embed hidden
//! `is_ascii_*` host predicates in generated Rust — it consumes the same ASCII
//! scalar classification structurally via `tokenize_char_class` (until M1(2.8)
//! allows `CharClass` `data` rows in `tokenize.dag`).

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
