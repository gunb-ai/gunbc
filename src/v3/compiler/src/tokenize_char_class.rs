//! ASCII-oriented lexical classes for the SG-1 tokenizer.
//!
//! Authority: `dsl/std/unicode.dag` — `CharClass` + `char_in_class` define the
//! same ASCII scalar predicates. This module is the Rust projection the
//! `regen_tokenize` binary emits into `tokenize_generated.rs` because M1(2.8)
//! still treats list / sum-variant `data` bodies in `tokenize.dag` as opaque
//! (`DOWNSTREAM_REQUIREMENTS.md` class-5 gap #3), so the scanner cannot yet
//! read `CharClass` rows structurally from that authority file.
//!
//! Keep `byte_matches` aligned with `char_in_class` on code points U+0000–U+007F.

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
