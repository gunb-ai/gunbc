//! Receipt for the string-literal escape path (`process_escapes` / `scan_string_body`).
//!
//! Two halves, per the Lane A oracle in docs/plans/inner-cost-lanes-scoping.md.
//!
//! EQUIVALENCE (`escape_decode_table`, `unknown_escape_passthrough_is_retained`): the decode
//! is unchanged. These are green on both sides of the migration; on their own they prove
//! nothing about cost, because they are satisfied by changing nothing.
//!
//! SEPARATION (`escape_cost_is_linear_in_literal_length`): the discriminating half. It reds
//! against the pre-migration implementation, which walked a raw `String` by index — `char_at`
//! is `chars().nth(pos)` and `string_length` is `chars().count()` once any non-ASCII character
//! appears anywhere in the literal, so a single em dash flipped the whole scan quadratic.
//! Measured on the seed before the change: 2,769 chars 7.8ms -> 44,019 chars 584ms, i.e. ~3.9x
//! per doubling. The assertion is a RATIO rather than a wall-clock bound so it does not encode
//! this machine's speed.
//!
//! Both halves drive `tokenize`, the real consumer, rather than the escape helpers directly:
//! the helpers' signatures changed in the migration, and a receipt that could not run against
//! both sides could not have shown the separation.

use std::time::{Duration, Instant};
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::TokenShape;

/// Tokenize a single string literal and return the decoded token text.
fn decode_literal(body: &str) -> String {
    let toks = tokenize(
        format!("data x: String = \"{}\"", body),
        "escape_receipt.dag".to_string(),
    );
    let lit = toks
        .iter()
        .find(|t| t.shape == TokenShape::ShLitStr)
        .unwrap_or_else(|| panic!("no ShLitStr token for body {:?}", body));
    lit.text.clone()
}

#[test]
fn escape_decode_table() {
    // The recognized escape vocabulary, exactly as `process_escapes_loop` declares it.
    assert_eq!(
        decode_literal(r"a\nb"),
        "a\nb",
        "\\n must decode to line feed"
    );
    assert_eq!(decode_literal(r"a\tb"), "a\tb", "\\t must decode to tab");
    assert_eq!(
        decode_literal(r"a\\b"),
        r"a\b",
        "\\\\ must decode to one backslash"
    );
    assert_eq!(
        decode_literal(r"a\{b"),
        "a{b",
        "escaped open brace must decode literally, not interpolate"
    );
    assert_eq!(
        decode_literal(r"a\}b"),
        "a}b",
        "escaped close brace must decode literally"
    );

    // \xNN. This is the arm whose absence silently emitted six characters where an ANSI
    // escape belonged -- see `hex_escape_note` in src/v1/01_tokenize.dag.
    assert_eq!(
        decode_literal(r"\x1b[0m"),
        "\u{1b}[0m",
        "\\x1b must decode to ESC"
    );
    assert_eq!(decode_literal(r"\x41"), "A", "\\x41 must decode to 'A'");
    assert_eq!(decode_literal(r"\x00"), "\u{0}", "\\x00 must decode to NUL");
    assert_eq!(
        decode_literal(r"\x0d"),
        "\r",
        "\\x0d must decode to carriage return"
    );

    // Non-ASCII passes through untouched, and -- the point of the migration -- mixing it with
    // escapes decodes identically to the pure-ASCII case.
    assert_eq!(
        decode_literal("héllo—🟡"),
        "héllo—🟡",
        "non-ASCII must pass through"
    );
    assert_eq!(
        decode_literal(r"é\n—\x41🟡"),
        "é\n—A🟡",
        "escapes must decode the same way when non-ASCII shares the literal"
    );
}

/// The `\xNN` arm decodes only when BOTH digits are hex; otherwise it declines and the
/// backslash survives. This is the discriminating pair for the hex arm: an implementation
/// that fabricated a value for malformed input, or that dropped the backslash, fails here
/// while `escape_decode_table` alone would still pass.
#[test]
fn malformed_hex_escape_declines_rather_than_fabricating() {
    assert_eq!(
        decode_literal(r"\xzz"),
        r"\xzz",
        "non-hex digits must not decode"
    );
    assert_eq!(
        decode_literal(r"\x1z"),
        r"\x1z",
        "a single bad digit must not decode"
    );
    // Truncated at end of literal: no digits to read at all.
    assert_eq!(decode_literal(r"\x"), r"\x", "a bare \\x must not decode");
}

/// `\s` is not in the vocabulary and today resolves to backslash-s. That is a knowingly
/// RETAINED closed-vocabulary violation, not an accepted one -- `unknown_escape_passthrough_frontier`
/// in src/v1/01_tokenize.dag carries the census and the dissolution trigger. Pinned here so the
/// migration cannot change it silently; this assertion flips when that frontier closes.
#[test]
fn unknown_escape_passthrough_is_retained() {
    assert_eq!(decode_literal(r"a\sb"), r"a\sb");
    assert_eq!(decode_literal(r"\."), r"\.");
}

/// Best-of-`n` wall time for tokenizing one literal of `repeats` units. Minimum, not mean:
/// scheduler noise only ever adds time, so the minimum is the least noisy estimator here.
fn best_tokenize_time(repeats: usize, samples: usize) -> Duration {
    // Non-ASCII (é, —, 🟡, ß) mixed with both escape shapes. The non-ASCII is what put the
    // pre-migration implementation on its quadratic branch.
    let unit = "é\\n—ü\\x41🟡ß";
    let mut body = String::with_capacity(unit.len() * repeats);
    for _ in 0..repeats {
        body.push_str(unit);
    }
    let src = format!("data x: String = \"{}\"", body);

    let mut best = Duration::MAX;
    for _ in 0..samples {
        let s = src.clone();
        let t0 = Instant::now();
        let toks = tokenize(s, "escape_cost.dag".to_string());
        let dt = t0.elapsed();
        assert!(!toks.is_empty());
        best = best.min(dt);
    }
    best
}

#[test]
fn escape_cost_is_linear_in_literal_length() {
    let small = best_tokenize_time(1_000, 3);
    let large = best_tokenize_time(4_000, 3);

    // 4x the input. Linear => ~4x the time. Quadratic => ~16x.
    let ratio = large.as_secs_f64() / small.as_secs_f64();

    // Generous: the bound sits between the two regimes with room on both sides. The seed
    // before the migration measured 14.1x here; linear measures ~4x. Fixed per-call overhead
    // pulls the observed ratio DOWN, so noise pushes this assertion toward passing rather
    // than toward a false red.
    assert!(
        ratio < 8.0,
        "string-literal escape cost is superlinear: 1k units {:?}, 4k units {:?} (4x input, \
         {:.1}x time). A ratio near 16 means the scan is quadratic again -- most likely \
         something reintroduced raw-String indexing (char_at / string_length) into the \
         escape path instead of walking the pre-decoded code points.",
        small,
        large,
        ratio
    );
}
