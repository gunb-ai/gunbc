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
//! bridge from lowered `.dag` yet: `sub_charclass_in_std_unicode_gate` locks the
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
pub enum TokenizerCharClass {
    Whitespace,
    Digit,
    IdentStart,
    IdentContinue,
}

#[inline]
pub fn byte_matches(byte: u8, class: TokenizerCharClass) -> bool {
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
        TokenizerCharClass::IdentContinue => {
            byte_matches(byte, TokenizerCharClass::Digit)
                || (65..=90).contains(&cp)
                || (97..=122).contains(&cp)
                || cp == 95
        }
    }
}
