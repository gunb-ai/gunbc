//! Comment-aware source slicing for grep-style ratchets.
//!
//! Preserves string/char/raw-string literals so `//` inside `"…"` is not treated
//! as a line comment — matches the lexical discipline from
//! `canonical_lens_bridge_ratchet_test` (PR #1183 follow-up).

/// Strip Rust comments from `src` so ratchets anchor on live syntax only.
///
/// See module docs for handled lexical forms.
pub fn strip_rust_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let n = bytes.len();
    while i < n {
        let b = bytes[i];
        if let Some(open_len) = match_raw_string_open(bytes, i) {
            let pounds = open_len - 2 - if bytes[i] == b'b' { 1 } else { 0 };
            let body_start = i + open_len;
            let close = find_raw_string_close(bytes, body_start, pounds);
            let end = close + 1 + pounds;
            out.push_str(&src[i..end.min(n)]);
            i = end.min(n);
            continue;
        }
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
        if b == b'\''
            || (b == b'b' && i + 1 < n && bytes[i + 1] == b'\'' && {
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
        if b == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            i += 2;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
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

#[cfg(test)]
mod tests {
    use super::strip_rust_comments;

    #[test]
    fn strip_comments_removes_line_and_block_comments_but_preserves_strings() {
        let src = r#"
        // pub const R1_CANONICAL_FOO_LENS: &str = include_str!("...");
        /* lens_decl.name.as_deref() == Some("ghost") */
        let s = "// not a comment // still in string";
        pub const REAL: &str = include_str!("real.dag");
    "#;
        let stripped = strip_rust_comments(src);
        assert!(!stripped.contains("R1_CANONICAL_FOO_LENS"));
        assert!(!stripped.contains("ghost"));
        assert!(stripped.contains("// not a comment // still in string"));
        assert!(stripped.contains("pub const REAL"));
    }

    #[test]
    fn strip_comments_distinguishes_lifetimes_from_char_literals() {
        let src = r#"
        fn f<'a>(x: &'a str) {} // marker_after_lifetime
        fn g(x: &'static str) {} // marker_after_static
        let c = 'x'; // marker_after_char_literal
        let escaped = '\n'; // marker_after_escape
    "#;
        let stripped = strip_rust_comments(src);
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
        assert!(stripped.contains("&'a str"));
        assert!(stripped.contains("&'static str"));
        assert!(stripped.contains("'x'"));
    }

    #[test]
    fn strip_comments_handles_raw_strings() {
        let src = r####"
        let a = r"a // not a comment, b /* still raw */ c";
        let b = r#"hash one " in middle"#;
        let c = r##"two hashes "# still inside"##;
        let d = br#"byte raw"#;
        // marker_real_comment
    "####;
        let stripped = strip_rust_comments(src);
        assert!(!stripped.contains("marker_real_comment"));
        assert!(stripped.contains("a // not a comment"));
        assert!(stripped.contains("b /* still raw */ c"));
        assert!(stripped.contains("hash one \" in middle"));
        assert!(stripped.contains("two hashes \"# still inside"));
        assert!(stripped.contains("byte raw"));
    }

    #[test]
    fn strip_comments_handles_nested_block_comments() {
        let src = "code_a /* outer /* inner */ still_outer */ code_b // marker\n";
        let stripped = strip_rust_comments(src);
        assert!(stripped.contains("code_a"));
        assert!(stripped.contains("code_b"));
        assert!(!stripped.contains("inner"));
        assert!(!stripped.contains("still_outer"));
        assert!(!stripped.contains("marker"));
    }
}
