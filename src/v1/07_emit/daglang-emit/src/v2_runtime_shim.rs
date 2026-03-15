//! Rust source text for the `v2_rt` runtime shim module.
//!
//! When the v2 compiler crate is emitted, this module is written verbatim
//! as `src/v2_rt.rs` so that intrinsic calls (`char_at`, `string_length`,
//! `substring`, `lookup`, `concat`) resolve to real Rust functions.

/// The Rust source code for the v2 runtime shim module.
pub const V2_RUNTIME_SOURCE: &str = r#"//! Runtime shims for v2 compiler intrinsic operations.

use std::collections::HashMap;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Concat trait — resolves the DAG `concat(a, b)` intrinsic for all sequence
// types. The DAG language uses a single `concat` function for both string
// and list concatenation. Rust's type system dispatches to the right impl.
// ---------------------------------------------------------------------------

pub trait Concat {
    fn concat(self, other: Self) -> Self;
}

impl Concat for String {
    fn concat(self, other: String) -> String {
        let mut s = self;
        s.push_str(&other);
        s
    }
}

impl<T> Concat for Vec<T> {
    fn concat(mut self, other: Vec<T>) -> Vec<T> {
        self.extend(other);
        self
    }
}

impl<T: Clone> Concat for Rc<Vec<T>> {
    fn concat(self, other: Self) -> Self {
        let mut v = Rc::try_unwrap(self).unwrap_or_else(|rc| (*rc).clone());
        v.extend(Rc::try_unwrap(other).unwrap_or_else(|rc| (*rc).clone()));
        Rc::new(v)
    }
}

/// Concatenate two sequences (strings or lists).
/// The DAG `concat(a, b)` intrinsic compiles to this.
pub fn concat<T: Concat>(a: T, b: T) -> T {
    a.concat(b)
}

// ---------------------------------------------------------------------------
// String operations
// ---------------------------------------------------------------------------

/// Extract the character at a given position as a single-char string.
pub fn char_at(s: impl AsRef<str>, pos: i64) -> String {
    s.as_ref()
        .chars()
        .nth(pos as usize)
        .map(|c| c.to_string())
        .unwrap_or_default()
}

/// Return the number of characters in a string.
pub fn string_length(s: impl AsRef<str>) -> i64 {
    s.as_ref().chars().count() as i64
}

/// Extract a substring by character indices [start, end).
pub fn substring(s: impl AsRef<str>, start: i64, end: i64) -> String {
    s.as_ref()
        .chars()
        .skip(start as usize)
        .take((end - start).max(0) as usize)
        .collect()
}

// ---------------------------------------------------------------------------
// Collection operations
// ---------------------------------------------------------------------------

/// Look up a key in a HashMap, returning a clone of the value if found.
pub fn lookup<V: Clone>(table: &HashMap<String, V>, key: impl AsRef<str>) -> Option<V> {
    table.get(key.as_ref()).cloned()
}

/// Concatenate two lists. Kept for backward compatibility with code that
/// calls `list_concat` directly; new code should use `concat()` instead.
pub fn list_concat<T>(mut a: Vec<T>, b: Vec<T>) -> Vec<T> {
    a.extend(b);
    a
}

/// String equality comparison (works with both &str and String).
pub fn str_eq(a: impl AsRef<str>, b: impl AsRef<str>) -> bool {
    a.as_ref() == b.as_ref()
}

// ---------------------------------------------------------------------------
// Scanner operations — used by the v2 tokenizer
// ---------------------------------------------------------------------------

/// Scan while a predicate holds, returning the end position.
/// `pred` receives a single-character string and returns true to continue.
pub fn scan_while(s: impl AsRef<str>, start: i64, pred: impl Fn(String) -> bool) -> i64 {
    let chars: Vec<char> = s.as_ref().chars().collect();
    let mut pos = start.max(0) as usize;
    while pos < chars.len() && pred(chars[pos].to_string()) {
        pos += 1;
    }
    pos as i64
}

/// Skip horizontal whitespace (spaces and tabs), returning the new position.
pub fn skip_horizontal_ws(s: impl AsRef<str>, start: i64) -> i64 {
    let chars: Vec<char> = s.as_ref().chars().collect();
    let mut pos = start.max(0) as usize;
    while pos < chars.len() && (chars[pos] == ' ' || chars[pos] == '\t') {
        pos += 1;
    }
    pos as i64
}

/// Scan to end of line, returning the position of the newline (or end of string).
pub fn scan_to_eol(s: impl AsRef<str>, start: i64) -> i64 {
    let chars: Vec<char> = s.as_ref().chars().collect();
    let start = start.max(0) as usize;
    for (i, &ch) in chars.iter().enumerate().skip(start) {
        if ch == '\n' {
            return i as i64;
        }
    }
    chars.len() as i64
}

/// Scan to end of a string literal, handling escape sequences.
/// Returns the position after the closing quote.
pub fn scan_string_end(s: impl AsRef<str>, start: i64) -> i64 {
    let chars: Vec<char> = s.as_ref().chars().collect();
    let mut pos = start.max(0) as usize;
    while pos < chars.len() {
        if chars[pos] == '\\' {
            if pos + 1 < chars.len() {
                pos += 2;
            } else {
                return chars.len() as i64;
            }
        } else if chars[pos] == '"' {
            return (pos + 1) as i64;
        } else {
            pos += 1;
        }
    }
    chars.len() as i64
}

/// Return the Unicode code point of a character (given as a single-char string).
pub fn code_point(c: impl AsRef<str>) -> i64 {
    c.as_ref().chars().next().map(|ch| ch as i64).unwrap_or(0)
}

/// Convert a Unicode code point to a single-character string.
pub fn from_code_point(cp: i64) -> String {
    char::from_u32(cp as u32)
        .map(|c| c.to_string())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Filesystem operations — stub for Filesystem.read service call
// ---------------------------------------------------------------------------

/// Result of a filesystem read operation.
#[derive(Debug, Clone)]
pub struct FilesystemReadResult {
    pub content: String,
}

/// Read a file's text content. Stub for the Filesystem.read service call.
pub fn filesystem_read(path: String) -> FilesystemReadResult {
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
    FilesystemReadResult { content }
}
"#;
