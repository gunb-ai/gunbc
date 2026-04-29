//! Ratchet for the canonical lens-name dispatch bridge in `test_runner.rs`.
//!
//! Pins the current bridge surface so any growth fails CI, and any reduction
//! can lower the pin in the same PR. Per `feedback_ratchet_only_down`: never
//! raise these constants.
//!
//! The bridge is described in `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`
//! (gate `bridge_canonical_lens_name_dispatch_retired`, R3 T-Bridge-Retirement
//! distributed bridge #3). Full retirement is gated on PB-Runtime
//! interpreter-as-data OR a typed lens-registry carrier substrate-introduction;
//! both are outside this PR's scope.
//!
//! Three categories are pinned:
//!   A. `include_str!("…/lenses/<name>.dag")` calls.
//!   B. `lens_decl.name.as_deref() == Some("<name>")` string-name dispatch arms.
//!   C. Generic `lens_decl.name.as_deref()` name-keyed lookups (ones not
//!      already counted in B).

use std::fs;
use std::path::PathBuf;

fn test_runner_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_runner.rs");
    fs::read_to_string(&path).expect("read test_runner.rs source")
}

/// Strip Rust line (`//`) and block (`/* … */`) comments from `src` so
/// the bridge ratchets anchor on live syntax only — historical or
/// example text in docstrings must not mask bridge removal or fabricate
/// bridge growth. String literals are preserved verbatim (the `//`
/// inside a string is not a comment).
fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let n = bytes.len();
    while i < n {
        let b = bytes[i];
        // String literal: copy through, honoring `\"` escapes.
        if b == b'"' {
            out.push('"');
            i += 1;
            while i < n {
                let c = bytes[i];
                out.push(c as char);
                i += 1;
                if c == b'\\' && i < n {
                    out.push(bytes[i] as char);
                    i += 1;
                } else if c == b'"' {
                    break;
                }
            }
            continue;
        }
        // Char literal: minimal handling; copy through to closing quote.
        if b == b'\'' && i + 1 < n {
            out.push('\'');
            i += 1;
            while i < n {
                let c = bytes[i];
                out.push(c as char);
                i += 1;
                if c == b'\\' && i < n {
                    out.push(bytes[i] as char);
                    i += 1;
                } else if c == b'\'' {
                    break;
                }
            }
            continue;
        }
        // Line comment.
        if b == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            i += 2;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Block comment.
        if b == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(n);
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

/// Category A: canonical-lens `include_str!` bridge constants in
/// `test_runner.rs`. Counts pub-const definitions of the shape
/// `pub const R1_CANONICAL_<NAME>_LENS: &str = include_str!` — any new
/// canonical-lens `include_str!` const with this naming is caught by the
/// ratchet, not just the two current names. Lines/blocks of comment text
/// are stripped before scanning so docstring examples can't fabricate or
/// mask growth.
///
/// Other `include_str!`-of-`/lenses/` cases (e.g. `INFER_HELPERS_SOURCE`)
/// are not part of this bridge surface — they are not consumed via
/// name-dispatch from a `lens_decl.name == …` arm and do not match the
/// `R1_CANONICAL_*_LENS` naming.
fn count_canonical_lens_include_str_bridges(src: &str) -> usize {
    // Anchor on live syntax: scan tokens between "pub const " and "=".
    let mut n = 0;
    let mut rest = src;
    let needle = "pub const ";
    while let Some(idx) = rest.find(needle) {
        let after = &rest[idx + needle.len()..];
        // Bound the inspection window to the const's signature/initializer.
        let window_end = after.find(';').map(|p| p + 1).unwrap_or(after.len());
        let window = &after[..window_end];
        // Match the canonical-lens const naming + include_str! initializer.
        if let Some(eq_pos) = window.find('=') {
            let (name_part, init_part) = window.split_at(eq_pos);
            let name = name_part.trim().trim_end_matches(": &str").trim();
            if name.starts_with("R1_CANONICAL_")
                && name.ends_with("_LENS")
                && init_part.contains("include_str!")
            {
                n += 1;
            }
        }
        rest = after;
    }
    n
}

/// Category B: `lens_decl.name.as_deref() == Some("...")` string-name
/// dispatch arms (the literal-equality comparisons).
fn count_lens_name_eq_dispatches(src: &str) -> usize {
    let mut n = 0;
    let needle = "lens_decl.name.as_deref()";
    let mut rest = src;
    while let Some(idx) = rest.find(needle) {
        let after = &rest[idx + needle.len()..];
        // Count only the equality-to-Some("...") pattern, not the generic
        // `if let Some(name) = lens_decl.name.as_deref()` form.
        let trimmed = after.trim_start();
        if trimmed.starts_with("== Some(\"") {
            n += 1;
        }
        rest = after;
    }
    n
}

/// Category C: generic `if let Some(name) = lens_decl.name.as_deref()`
/// name-keyed lookups.
fn count_lens_name_lookups(src: &str) -> usize {
    src.matches("if let Some(name) = lens_decl.name.as_deref()")
        .count()
}

const EXPECTED_INCLUDE_STR_LENS_BYTES: usize = 2;
const EXPECTED_NAME_EQ_DISPATCH_ARMS: usize = 2;
const EXPECTED_GENERIC_NAME_LOOKUPS: usize = 2;

#[test]
fn canonical_lens_include_str_bridges_pinned() {
    let src = strip_comments(&test_runner_source());
    let n = count_canonical_lens_include_str_bridges(&src);
    assert_eq!(
        n, EXPECTED_INCLUDE_STR_LENS_BYTES,
        "test_runner.rs canonical-lens `include_str!` count drifted: \
         expected {EXPECTED_INCLUDE_STR_LENS_BYTES}, found {n}. \
         If this is a *reduction* (bridge retired), lower the constant in \
         this file in the same PR. If this is *growth*, that is forbidden \
         per the disposition in \
         docs/briefs/r2-pb-canonical-lens-bridge-disposition.md — split \
         a structural lens-registry carrier brief instead of adding another \
         `include_str!` of canonical lens bytes."
    );
}

#[test]
fn canonical_lens_name_dispatch_arms_pinned() {
    let src = strip_comments(&test_runner_source());
    let n = count_lens_name_eq_dispatches(&src);
    assert_eq!(
        n, EXPECTED_NAME_EQ_DISPATCH_ARMS,
        "test_runner.rs `lens_decl.name.as_deref() == Some(\"…\")` arm \
         count drifted: expected {EXPECTED_NAME_EQ_DISPATCH_ARMS}, found \
         {n}. If this is a *reduction*, lower the constant. If this is \
         *growth*, that is forbidden per the disposition — name-keyed \
         dispatch on lens identity must be retired via PB-Runtime \
         interpreter-as-data or a typed lens-registry carrier, not \
         extended."
    );
}

#[test]
fn canonical_lens_generic_name_lookups_pinned() {
    let src = strip_comments(&test_runner_source());
    let n = count_lens_name_lookups(&src);
    assert_eq!(
        n, EXPECTED_GENERIC_NAME_LOOKUPS,
        "test_runner.rs generic `lens_decl.name.as_deref()` name-keyed \
         lookup count drifted: expected {EXPECTED_GENERIC_NAME_LOOKUPS}, \
         found {n}. Same rule as the equality-arm ratchet: ratchet only \
         down."
    );
}

#[test]
fn strip_comments_removes_line_and_block_comments_but_preserves_strings() {
    // Line comment containing a fake bridge marker must NOT be visible to
    // the ratchet predicates after stripping.
    let src = r#"
        // pub const R1_CANONICAL_FOO_LENS: &str = include_str!("...");
        /* lens_decl.name.as_deref() == Some("ghost") */
        let s = "// not a comment // still in string";
        pub const REAL: &str = include_str!("real.dag");
    "#;
    let stripped = strip_comments(src);
    assert!(!stripped.contains("R1_CANONICAL_FOO_LENS"));
    assert!(!stripped.contains("ghost"));
    assert!(stripped.contains("// not a comment // still in string"));
    assert!(stripped.contains("pub const REAL"));
}

#[test]
fn count_canonical_lens_catches_new_r1_canonical_const() {
    // A future bridge addition under a different name MUST be caught by
    // the ratchet — the prior exact-name predicate would have returned 2,
    // missing the third bridge.
    let src = r#"
pub const R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS: &str = include_str!("a.dag");
pub const R1_CANONICAL_COMPLEXITY_LENS: &str = include_str!("b.dag");
pub const R1_CANONICAL_FUTURE_BRIDGE_LENS: &str = include_str!("c.dag");
pub const NOT_A_LENS_BRIDGE: &str = include_str!("d.dag");
    "#;
    assert_eq!(count_canonical_lens_include_str_bridges(src), 3);
}
