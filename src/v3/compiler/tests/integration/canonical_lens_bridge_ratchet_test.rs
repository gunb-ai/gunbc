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

/// Strip Rust comments from `src` so the bridge ratchets anchor on
/// live syntax only — historical or example text in docstrings must
/// not mask bridge removal or fabricate bridge growth.
///
/// Handles enough Rust lexical forms to keep comments inside literals
/// from being treated as comments, and to keep lifetime ticks from
/// being treated as char-literal openers:
///
/// - Line comments (`// …\n`) and block comments (`/* … */`, **nested**
///   per Rust 2021 spec).
/// - Normal/byte strings (`"…"`, `b"…"`) with `\\` escapes.
/// - Raw strings (`r"…"`, `r#"…"#`, `r##"…"##`, …) and raw byte strings
///   (`br"…"`, `br#"…"#`, …) — escapes do not process; closer matches
///   the opener's `#` count.
/// - Char literals (`'a'`, `'\n'`) and byte char literals (`b'a'`)
///   distinguished from lifetime ticks (`'a`, `'static`, `'_`) by
///   bounded lookahead for a closing `'`.
fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let n = bytes.len();
    while i < n {
        let b = bytes[i];
        // Raw string: `r"…"` / `r#"…"#` / `br"…"` / `br#"…"#` (any
        // number of `#`). Escapes do not process; closer matches the
        // opener's `#` count.
        if let Some(open_len) = match_raw_string_open(bytes, i) {
            let pounds = open_len - 2 - if bytes[i] == b'b' { 1 } else { 0 };
            // open_len = (b? r # … " ); start of body is i + open_len.
            let body_start = i + open_len;
            let close = find_raw_string_close(bytes, body_start, pounds);
            let end = close + 1 + pounds;
            out.push_str(&src[i..end.min(n)]);
            i = end.min(n);
            continue;
        }
        // Normal/byte string literal: optional `b` prefix, opens with `"`.
        if b == b'"' || (b == b'b' && i + 1 < n && bytes[i + 1] == b'"') {
            if b == b'b' {
                out.push('b');
                i += 1;
            }
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
        // Char / byte-char literal vs lifetime: `'a` / `'static` are
        // lifetimes (no closing quote); `'a'` / `'\n'` / `b'a'` are char
        // literals. Look ahead to distinguish — if a closing `'` appears
        // within ~6 bytes (max payload incl. byte prefix: `b'\xNN'`),
        // treat as a char literal and skip through it. Otherwise treat
        // the `'` as a lifetime tick and copy a single byte.
        if b == b'\''
            || (b == b'b' && i + 1 < n && bytes[i + 1] == b'\'' && {
                // Don't grab `b'<lifetime>` as char literal — but `b<ident>`
                // is just `b` followed by punctuation; not a normal pattern.
                // Apply the same lookahead test to the `'` after the `b`.
                let test_start = i + 2;
                let test_end = (i + 7).min(n);
                let mut found = false;
                let mut k = test_start;
                while k < test_end {
                    if bytes[k] == b'\\' && k + 1 < n {
                        k += 2;
                        continue;
                    }
                    if bytes[k] == b'\'' {
                        found = true;
                        break;
                    }
                    k += 1;
                }
                found
            })
        {
            let tick_at = if b == b'b' {
                out.push('b');
                i += 1;
                i
            } else {
                i
            };
            let lookahead_start = tick_at + 1;
            let lookahead_end = (tick_at + 6).min(n);
            let mut closing: Option<usize> = None;
            let mut j = lookahead_start;
            while j < lookahead_end {
                if bytes[j] == b'\\' && j + 1 < n {
                    j += 2;
                    continue;
                }
                if bytes[j] == b'\'' {
                    closing = Some(j);
                    break;
                }
                j += 1;
            }
            if let Some(close) = closing {
                out.push_str(&src[tick_at..=close]);
                i = close + 1;
                continue;
            }
            out.push('\'');
            i = tick_at + 1;
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
        // Block comment, nested per Rust 2021.
        if b == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            i += 2;
            let mut depth: usize = 1;
            while i + 1 < n && depth > 0 {
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

/// Detect the opening of a raw / raw-byte string at `i`. Returns the
/// length of the opener (`r"…`, `r#"…`, `br#"…`, etc.) including the
/// final `"`. Returns `None` if no raw-string opener starts at `i`.
fn match_raw_string_open(bytes: &[u8], i: usize) -> Option<usize> {
    let n = bytes.len();
    let mut p = i;
    if p >= n {
        return None;
    }
    if bytes[p] == b'b' {
        p += 1;
    }
    if p >= n || bytes[p] != b'r' {
        return None;
    }
    p += 1;
    while p < n && bytes[p] == b'#' {
        p += 1;
    }
    if p < n && bytes[p] == b'"' {
        Some(p - i + 1)
    } else {
        None
    }
}

/// Find the closing `"` of a raw string starting at `body_start`, where
/// `pounds` is the opener's `#` count. The closer is the first `"`
/// followed by exactly `pounds` `#` characters. Returns the position of
/// the closing `"`. If unterminated, returns the end of input minus the
/// pound padding (best-effort).
fn find_raw_string_close(bytes: &[u8], body_start: usize, pounds: usize) -> usize {
    let n = bytes.len();
    let mut i = body_start;
    while i < n {
        if bytes[i] == b'"' {
            let mut k = 0;
            while k < pounds && i + 1 + k < n && bytes[i + 1 + k] == b'#' {
                k += 1;
            }
            if k == pounds {
                return i;
            }
        }
        i += 1;
    }
    n.saturating_sub(pounds + 1)
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
const EXPECTED_NAME_EQ_DISPATCH_ARMS: usize = 0;
const EXPECTED_GENERIC_NAME_LOOKUPS: usize = 0;

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
fn strip_comments_distinguishes_lifetimes_from_char_literals() {
    // Lifetimes (`'a`, `'static`, `'_`) must NOT be treated as opening a
    // char literal — otherwise comments after them are masked from the
    // ratchet (cursor BLOCKING on PR #1183).
    let src = r#"
        fn f<'a>(x: &'a str) {} // marker_after_lifetime
        fn g(x: &'static str) {} // marker_after_static
        let c = 'x'; // marker_after_char_literal
        let escaped = '\n'; // marker_after_escape
    "#;
    let stripped = strip_comments(src);
    // All four trailing comments must be stripped. None of the markers
    // should survive in the stripped output.
    assert!(
        !stripped.contains("marker_after_lifetime"),
        "lifetime `'a` masked the line comment that follows it: {stripped}"
    );
    assert!(
        !stripped.contains("marker_after_static"),
        "lifetime `'static` masked the line comment that follows it: {stripped}"
    );
    assert!(!stripped.contains("marker_after_char_literal"));
    assert!(!stripped.contains("marker_after_escape"));
    // Live syntax preserved.
    assert!(stripped.contains("&'a str"));
    assert!(stripped.contains("&'static str"));
    assert!(stripped.contains("'x'"));
}

#[test]
fn strip_comments_handles_raw_strings() {
    // Raw strings (r"…", r#"…"#, br#"…"#) must NOT have their bodies
    // interpreted as code, so embedded // and /* */ inside the raw-string
    // body are preserved verbatim. Closer must match the opener's # count.
    let src = r####"
        let a = r"a // not a comment, b /* still raw */ c";
        let b = r#"hash one " in middle"#;
        let c = r##"two hashes "# still inside"##;
        let d = br#"byte raw"#;
        // marker_real_comment
    "####;
    let stripped = strip_comments(src);
    // Real line comment stripped.
    assert!(!stripped.contains("marker_real_comment"));
    // Raw-string bodies preserved verbatim.
    assert!(stripped.contains("a // not a comment"));
    assert!(stripped.contains("b /* still raw */ c"));
    assert!(stripped.contains("hash one \" in middle"));
    assert!(stripped.contains("two hashes \"# still inside"));
    assert!(stripped.contains("byte raw"));
}

#[test]
fn strip_comments_handles_nested_block_comments() {
    let src = "code_a /* outer /* inner */ still_outer */ code_b // marker\n";
    let stripped = strip_comments(src);
    assert!(stripped.contains("code_a"));
    assert!(stripped.contains("code_b"));
    assert!(!stripped.contains("inner"));
    assert!(!stripped.contains("still_outer"));
    assert!(!stripped.contains("marker"));
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
