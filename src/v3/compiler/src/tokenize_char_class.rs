//! ASCII-oriented lexical classes for the SG-1 tokenizer.
//!
//! Authority: `dsl/std/unicode.dag` — `CharClass` + `char_in_class` define the
//! same ASCII scalar predicates. This module is the Rust projection the
//! `regen_tokenize` binary emits into `tokenize_generated.rs` because M1(2.8)
//! still treats list / sum-variant `data` bodies in `tokenize.dag` as opaque
//! (`DOWNSTREAM_REQUIREMENTS.md` class-5 gap #3), so the scanner cannot yet
//! read `CharClass` rows structurally from that authority file.
//!
//! `byte_matches` is **hand-synced** with `char_in_class` in `dsl/std/unicode.dag`
//! on code points U+0000–U+007F (same Int-range semantics). There is no runtime
//! bridge from lowered `.dag` yet: `#[cfg(test)]` checks in this file lock the
//! mirror against Rust’s historical `u8::is_ascii_*` scanner contract, not an
//! automated proof against evaluated `char_in_class`. Follow-up once
//! `char_in_class` is executable from the compiler test harness: assert parity
//! on 0..=127 directly against the `.dag` definition and delete redundant prose.
//!
//! **Lane framing:** tokenizer-side interim only — not structural consumption of
//! `CharClass` from lowered `tokenize.dag` (see M1(2.8) class-5 gap #3). Remove
//! this module when `regen_tokenize` can read class predicates from `.dag`.

/// Mirrors `std.unicode::CharClass` variant names for generated call sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenizerCharClass {
    Whitespace,
    Digit,
    IdentStart,
    IdentContinue,
}

#[inline]
pub(crate) fn byte_matches(byte: u8, class: TokenizerCharClass) -> bool {
    let cp = byte as i64;
    match class {
        // Match Rust `u8::is_ascii_whitespace` (excludes vertical tab U+000B).
        TokenizerCharClass::Whitespace => {
            matches!(byte, b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
        }
        TokenizerCharClass::Digit => (48..=57).contains(&cp),
        TokenizerCharClass::IdentStart => {
            (65..=90).contains(&cp) || (97..=122).contains(&cp) || cp == 95
        }
        // Same predicate as `std.unicode::char_in_class` / `IdentContinue`: digit ∪ ident-start.
        TokenizerCharClass::IdentContinue => {
            byte_matches(byte, TokenizerCharClass::Digit)
                || byte_matches(byte, TokenizerCharClass::IdentStart)
        }
    }
}

#[cfg(test)]
mod sub_charclass_in_std_unicode_gate {
    //! **Layer:** unit (TESTING.md) — T-Sub `sub_charclass_in_std_unicode` tokenizer
    //! half / bounded interim (ROADMAP.md:358). Ratchets `std.unicode` + generated
    //! tokenizer wiring + `byte_matches` vs `u8::is_ascii_*` on 0..=127.
    //!
    //! **Sync boundary:** parity here is `byte_matches` vs `u8::is_ascii_*`, not
    //! evaluated `char_in_class` from `.dag`; keep `tokenize_char_class.rs` and
    //! `unicode.dag` predicates edited together until an interpreter-backed check exists.
    //! Substring anchors below only catch gross drift (missing sum, restored host
    //! `is_ascii_*`, accidental reintroduction of `char_in_class` self-call on the
    //! same `c` in `IdentContinue`); they do not prove arithmetic matches `.dag`.

    use super::{byte_matches, TokenizerCharClass};

    const UNICODE_DAG: &str = include_str!("../../../../dsl/std/unicode.dag");
    const TOKENIZE_GENERATED: &str = include_str!("tokenize_generated.rs");

    #[test]
    fn unicode_dag_defines_char_class() {
        assert!(
            UNICODE_DAG.contains("type CharClass")
                && UNICODE_DAG.contains("fn char_in_class")
                && UNICODE_DAG.contains("Whitespace")
                && UNICODE_DAG.contains("IdentContinue"),
            "expected `CharClass` sum + `char_in_class` predicate in `dsl/std/unicode.dag` authority"
        );
        assert!(
            UNICODE_DAG.contains("IdentContinue => (cp >= 48 && cp <= 57)"),
            "expected `IdentContinue` to inline the digit range (no `char_in_class` recursion on the same `c`)"
        );
        assert!(
            !UNICODE_DAG.contains("char_in_class(c: c, class: Digit)"),
            "`IdentContinue` must not recurse into `char_in_class` with the same `c` (CX / bounded recursion)"
        );
    }

    #[test]
    fn generated_tokenizer_avoids_ascii_host_predicates() {
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
    fn byte_matches_locks_ascii_scanner_semantics() {
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
}
