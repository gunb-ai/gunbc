//! `tests/integration.rs` wiring scanner (Band-C cementing dispatch).
//!
//! Lives in the compiler crate so `test_runner` can validate
//! `#[path = "integration/cementing/<stem>.rs"]` + `mod <stem>;` pairings without
//! duplicating the scanner across test binaries.

/// `true` when `bytes[i..]` opens a `b"…"`, `br#"…"#`, `r#"…"#`, or `r"…"` literal in
/// Rust source. The wiring scanner does not model these — fail loud if they appear
/// in `Code` so Band-C ratchets cannot silently mis-scan `tests/integration.rs`.
fn code_opens_raw_or_byte_string_literal(bytes: &[u8], i: usize) -> bool {
    match bytes.get(i) {
        Some(b'b') => {
            if bytes.get(i + 1) == Some(&b'"') {
                return true;
            }
            if bytes.get(i + 1) == Some(&b'r') {
                let mut j = i + 2;
                while bytes.get(j) == Some(&b'#') {
                    j += 1;
                }
                return bytes.get(j) == Some(&b'"');
            }
            false
        }
        Some(b'r') => {
            let mut j = i + 1;
            while bytes.get(j) == Some(&b'#') {
                j += 1;
            }
            bytes.get(j) == Some(&b'"')
        }
        _ => false,
    }
}

/// Byte index after the last byte of a Rust escape sequence starting at `backslash`
/// (`bytes[backslash] == b'\\'`). Matches suffix forms used in character and byte
/// literals (`\n`, `\t`, `\xNN`, `\u{…}`, …). Unicode escapes use **only** the braced
/// `\u{…}` form in modern Rust; there is no separate `\uXXXX` tail to scan.
fn integration_rs_escape_sequence_end(bytes: &[u8], backslash: usize) -> usize {
    debug_assert_eq!(bytes.get(backslash), Some(&b'\\'));
    let i = backslash + 1;
    if i >= bytes.len() {
        return bytes.len();
    }
    match bytes[i] {
        b'x' => (backslash + 4).min(bytes.len()),
        b'u' if bytes.get(i + 1) == Some(&b'{') => {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b'}' {
                j += 1;
            }
            (j + 1).min(bytes.len())
        }
        _ => (backslash + 2).min(bytes.len()),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IntegrationRsScan {
    Code,
    LineComment,
    BlockComment(u32),
    String,
    /// Inside `b'…'` after consuming the `b'` prefix.
    ByteCharLiteral,
    /// Inside `'…'` after consuming the opening `'`.
    CharLiteral,
}

fn integration_rs_panic_on_raw_string(bytes: &[u8], i: usize) {
    if code_opens_raw_or_byte_string_literal(bytes, i) {
        panic!(
            "integration_rs wiring scan: raw/byte string literal at byte offset {i} \
             (e.g. r\"…\", r#\"…\"#, b\"…\", br#\"…\"#). Extend IntegrationRsScan before scanning this file."
        );
    }
}

/// Byte offsets of `needle` in `integration_rs` that occur in **Code** (outside
/// `//`, `/* */`, `"…"` strings, `b'…'` byte literals, and `'…'` character literals).
/// Lifetimes and labels (`'a`, `'static`) are skipped without entering string/literal
/// states. See [`integration_rs_active_line_contains`].
fn integration_rs_code_substring_positions(integration_rs: &str, needle: &str) -> Vec<usize> {
    let mut hits = Vec::new();
    if needle.is_empty() {
        hits.push(0);
        return hits;
    }
    let bytes = integration_rs.as_bytes();
    let mut i = 0usize;
    let mut state = IntegrationRsScan::Code;

    while i < bytes.len() {
        match state {
            IntegrationRsScan::Code => {
                integration_rs_panic_on_raw_string(bytes, i);
                if bytes[i] == b'/' && i + 1 < bytes.len() {
                    if bytes[i + 1] == b'/' {
                        state = IntegrationRsScan::LineComment;
                        i += 2;
                        continue;
                    }
                    if bytes[i + 1] == b'*' {
                        state = IntegrationRsScan::BlockComment(1);
                        i += 2;
                        continue;
                    }
                }
                if bytes.get(i) == Some(&b'b') && bytes.get(i + 1) == Some(&b'\'') {
                    state = IntegrationRsScan::ByteCharLiteral;
                    i += 2;
                    continue;
                }
                if bytes[i] == b'\'' {
                    if bytes.get(i + 1) == Some(&b'\\') {
                        state = IntegrationRsScan::CharLiteral;
                        i += 1;
                        continue;
                    }
                    if let Some(rest) = integration_rs.get(i + 1..) {
                        let mut chars = rest.chars();
                        if let Some(c) = chars.next() {
                            let after_char = i + 1 + c.len_utf8();
                            if bytes.get(after_char) == Some(&b'\'') {
                                i = after_char + 1;
                                continue;
                            }
                        }
                    }
                    i += 1;
                    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                    {
                        i += 1;
                    }
                    continue;
                }
                if bytes[i] == b'"' {
                    state = IntegrationRsScan::String;
                    i += 1;
                    continue;
                }
                if integration_rs[i..].starts_with(needle) {
                    hits.push(i);
                }
                i += 1;
            }
            IntegrationRsScan::LineComment => {
                if bytes[i] == b'\n' {
                    state = IntegrationRsScan::Code;
                }
                i += 1;
            }
            IntegrationRsScan::BlockComment(depth) => {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    state = IntegrationRsScan::BlockComment(depth + 1);
                    i += 2;
                    continue;
                }
                if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    let d = depth - 1;
                    i += 2;
                    state = if d == 0 {
                        IntegrationRsScan::Code
                    } else {
                        IntegrationRsScan::BlockComment(d)
                    };
                    continue;
                }
                i += 1;
            }
            IntegrationRsScan::String => match bytes[i] {
                b'\\' => {
                    i = (i + 2).min(bytes.len());
                }
                b'"' => {
                    i += 1;
                    state = IntegrationRsScan::Code;
                }
                _ => {
                    i += 1;
                }
            },
            IntegrationRsScan::ByteCharLiteral => {
                if bytes.get(i) == Some(&b'\\') {
                    i = integration_rs_escape_sequence_end(bytes, i);
                } else {
                    i += 1;
                }
                if bytes.get(i) == Some(&b'\'') {
                    i += 1;
                }
                state = IntegrationRsScan::Code;
            }
            IntegrationRsScan::CharLiteral => {
                if bytes.get(i) == Some(&b'\\') {
                    i = integration_rs_escape_sequence_end(bytes, i);
                } else if let Some(rest) = integration_rs.get(i..) {
                    let mut ci = rest.chars();
                    if let Some(c) = ci.next() {
                        i += c.len_utf8();
                    } else {
                        state = IntegrationRsScan::Code;
                        continue;
                    }
                } else {
                    state = IntegrationRsScan::Code;
                    continue;
                }
                if bytes.get(i) == Some(&b'\'') {
                    i += 1;
                }
                state = IntegrationRsScan::Code;
            }
        }
    }
    hits
}

/// Consume one outer `#[ … ]` attribute (handles quoted `"` regions and nested `[` `]`).
fn skip_rust_attribute(s: &str) -> Option<&str> {
    if !s.starts_with("#[") {
        return None;
    }
    let bytes = s.as_bytes();
    let mut i = 1usize;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escape {
                escape = false;
                i += 1;
                continue;
            }
            if b == b'\\' {
                escape = true;
                i += 1;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => {
                in_string = true;
                i += 1;
            }
            b'[' => {
                depth += 1;
                i += 1;
            }
            b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
                if depth == 0 {
                    return Some(&s[i..]);
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

/// After a `#[path = "integration/cementing/<stem>.rs"]` attribute in Code, the next
/// Rust item must be `mod <stem>;` (optionally separated by whitespace, newlines, or
/// additional `#[…]` attributes).
fn mod_decl_follows_cementing_path_tail(mut tail: &str, stem: &str) -> bool {
    let prefix = format!("mod {stem}");
    loop {
        tail = tail.trim_start();
        if tail.is_empty() {
            return false;
        }
        if tail.starts_with("#[") {
            let Some(rest) = skip_rust_attribute(tail) else {
                return false;
            };
            tail = rest;
            continue;
        }
        if !tail.starts_with(&prefix) {
            return false;
        }
        let after_prefix = &tail[prefix.len()..];
        if after_prefix
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            // e.g. `mod stem` must not match `mod stem_extra`.
            return false;
        }
        let after = after_prefix.trim_start();
        return after.starts_with(';');
    }
}

/// `true` when `integration_rs` contains `#[path = "integration/cementing/<stem>.rs"]`
/// in **Code** and that attribute is immediately followed (per
/// [`mod_decl_follows_cementing_path_tail`]) by `mod <stem>;`.
///
/// This is stricter than substring checks for `#[path]` and `mod` separately — it
/// rejects a mismatched `#[path]` on one module paired with a stray `mod <stem>;`
/// elsewhere (`TESTING.md` Band-C dispatch).
pub fn integration_rs_cementing_path_attr_binds_mod_stem(integration_rs: &str, stem: &str) -> bool {
    let path_attr = format!(r#"#[path = "integration/cementing/{stem}.rs"]"#);
    for pos in integration_rs_code_substring_positions(integration_rs, &path_attr) {
        let Some(tail) = integration_rs.get(pos + path_attr.len()..) else {
            continue;
        };
        if mod_decl_follows_cementing_path_tail(tail, stem) {
            return true;
        }
    }
    false
}

/// True when `needle` appears in `integration_rs` **outside** Rust line comments
/// (`//` … including `///` / `//!`), **nested** block comments (`/* … */`), normal
/// `"…"` string literals, `b'…'` byte character literals, and `'…'` character
/// literals (excluding lifetimes / labels such as `'a` or `'static`).
///
/// Band-C wiring ratchets use this so `#[path = …]` / `mod …;` matches cannot
/// false-green on commented-out or string-embedded copies. Needles are ASCII
/// (`#[path`, `mod foo`); the scan is byte-oriented on UTF-8 boundaries.
///
/// **Not handled:** raw strings (`r#"…"#`), byte strings (`b"…"`, `br#"…"#`). If
/// those appear in scanned source, extend `IntegrationRsScan` **or** the scan will
/// **panic** when the raw/byte-string opener probe fires in `Code` (loud failure vs
/// a silent false green).
pub fn integration_rs_active_line_contains(integration_rs: &str, needle: &str) -> bool {
    !integration_rs_code_substring_positions(integration_rs, needle).is_empty()
}
