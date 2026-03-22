//! Rust source text for the `v2_rt` runtime shim module.
//!
//! When the v2 compiler crate is emitted, this module is written verbatim
//! as `src/v2_rt.rs` so that intrinsic calls (`char_at`, `string_length`,
//! `substring`, `string_contains`, `lookup`, `concat`) resolve to real Rust functions.

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

/// Extract the character at a given position (character index) as a single-char string.
/// O(1) for ASCII strings; O(pos) for non-ASCII.
/// Position is a character index, consistent with the evaluator's char-based semantics.
pub fn char_at(s: impl AsRef<str>, pos: i64) -> String {
    let s = s.as_ref();
    let pos = pos as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        if pos >= bytes.len() {
            return String::new();
        }
        return String::from(bytes[pos] as char);
    }
    s.chars().nth(pos).map(|ch| ch.to_string()).unwrap_or_default()
}

/// Return the character count of a string.
/// O(1) for ASCII strings; O(n) for non-ASCII.
/// Returns character count, consistent with the evaluator's char-based semantics.
pub fn string_length(s: impl AsRef<str>) -> i64 {
    let s = s.as_ref();
    if s.is_ascii() { s.len() as i64 } else { s.chars().count() as i64 }
}

/// Extract a substring by character positions [start, end).
/// O(end - start) for ASCII; O(n) for non-ASCII.
/// Positions are character indices, consistent with the evaluator's char-based semantics.
pub fn substring(s: impl AsRef<str>, start: i64, end: i64) -> String {
    let s = s.as_ref();
    let start = start.max(0) as usize;
    let end = end.max(0) as usize;
    if end <= start {
        return String::new();
    }
    if s.is_ascii() {
        let len = s.len();
        let start = start.min(len);
        let end = end.min(len);
        if start >= end {
            return String::new();
        }
        return s[start..end].to_string();
    }
    s.chars().skip(start).take(end - start).collect()
}

/// Check whether a string contains a given substring.
pub fn string_contains(s: impl AsRef<str>, substring: impl AsRef<str>) -> bool {
    s.as_ref().contains(substring.as_ref())
}

// ---------------------------------------------------------------------------
// Collection operations
// ---------------------------------------------------------------------------

/// Look up a key in a HashMap, returning a clone of the value if found.
pub fn lookup<V: Clone>(table: &HashMap<String, V>, key: impl AsRef<str>) -> Option<V> {
    table.get(key.as_ref()).cloned()
}

/// Build a HashMap from a list by extracting a string key from each element.
/// Duplicate keys: last writer wins (later elements overwrite earlier ones).
pub fn index_by<V, F: Fn(&V) -> String>(list: Rc<Vec<V>>, key_fn: F) -> Rc<HashMap<String, V>>
where
    V: Clone,
{
    let mut map = HashMap::new();
    for item in Rc::try_unwrap(list).unwrap_or_else(|rc| (*rc).clone()) {
        let key = key_fn(&item);
        map.insert(key, item);
    }
    Rc::new(map)
}

/// Create an empty Rc-wrapped HashMap.
pub fn to_string(value: i64) -> String {
    value.to_string()
}

pub fn empty_map<V>() -> Rc<HashMap<String, V>> {
    Rc::new(HashMap::new())
}

/// Insert a key-value pair into an Rc-wrapped HashMap.
/// O(1) amortized when refcount=1 (true in fold accumulators).
pub fn map_insert<V: Clone>(map: Rc<HashMap<String, V>>, key: String, value: V) -> Rc<HashMap<String, V>> {
    let mut m = Rc::try_unwrap(map).unwrap_or_else(|rc| (*rc).clone());
    m.insert(key, value);
    Rc::new(m)
}

/// Merge two Rc-wrapped HashMaps. Overlay entries overwrite base entries.
/// O(|overlay|) amortized.
pub fn map_merge<V: Clone>(base: Rc<HashMap<String, V>>, overlay: Rc<HashMap<String, V>>) -> Rc<HashMap<String, V>> {
    let mut m = Rc::try_unwrap(base).unwrap_or_else(|rc| (*rc).clone());
    m.extend(Rc::try_unwrap(overlay).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(m)
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

/// Scan while a predicate holds, returning the end position (character index).
/// `pred` receives a single-character `&str` and returns true to continue.
/// O(k) where k = number of chars satisfying pred.
/// Positions are character indices, consistent with the evaluator's char-based semantics.
pub fn scan_while(s: impl AsRef<str>, start: i64, pred: impl Fn(&str) -> bool) -> i64 {
    let s = s.as_ref();
    let start = start.max(0) as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        let mut pos = start.min(bytes.len());
        while pos < bytes.len() {
            if !pred(&s[pos..pos + 1]) {
                break;
            }
            pos += 1;
        }
        return pos as i64;
    }
    let mut pos = start;
    for ch in s.chars().skip(start) {
        if !pred(&ch.to_string()) {
            break;
        }
        pos += 1;
    }
    pos as i64
}

/// Skip horizontal whitespace (spaces and tabs), returning the new position.
/// O(k) where k = whitespace run length.
/// Position is a character index, consistent with char-based semantics.
pub fn skip_horizontal_ws(s: impl AsRef<str>, start: i64) -> i64 {
    let s = s.as_ref();
    let start = start.max(0) as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        let mut pos = start.min(bytes.len());
        while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
            pos += 1;
        }
        return pos as i64;
    }
    let mut pos = start;
    for ch in s.chars().skip(start) {
        if ch != ' ' && ch != '\t' {
            break;
        }
        pos += 1;
    }
    pos as i64
}

/// Scan to end of line, returning the position of the newline (or end of string).
/// O(k) where k = distance to newline.
/// Position is a character index, consistent with char-based semantics.
pub fn scan_to_eol(s: impl AsRef<str>, start: i64) -> i64 {
    let s = s.as_ref();
    let start = start.max(0) as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        for i in start..bytes.len() {
            if bytes[i] == b'\n' {
                return i as i64;
            }
        }
        return bytes.len() as i64;
    }
    let mut pos = start;
    for ch in s.chars().skip(start) {
        if ch == '\n' {
            return pos as i64;
        }
        pos += 1;
    }
    pos as i64
}

/// Scan to end of a string literal, handling escape sequences.
/// Returns the position after the closing quote.
/// O(k) where k = string literal length.
/// Position is a character index, consistent with char-based semantics.
pub fn scan_string_end(s: impl AsRef<str>, start: i64) -> i64 {
    let s = s.as_ref();
    let start = start.max(0) as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        let mut pos = start.min(bytes.len());
        while pos < bytes.len() {
            if bytes[pos] == b'\\' {
                if pos + 1 < bytes.len() {
                    pos += 2;
                } else {
                    return bytes.len() as i64;
                }
            } else if bytes[pos] == b'"' {
                return (pos + 1) as i64;
            } else {
                pos += 1;
            }
        }
        return bytes.len() as i64;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut pos = start.min(chars.len());
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

#[cfg(test)]
mod tests {
    use super::V2_RUNTIME_SOURCE;

    #[test]
    fn substring_clamps_bounds() {
        assert!(
            V2_RUNTIME_SOURCE.contains("if end <= start {")
                && V2_RUNTIME_SOURCE.contains("return String::new();")
                && V2_RUNTIME_SOURCE.contains("let start = start.min(len);")
                && V2_RUNTIME_SOURCE.contains("let end = end.min(len);")
                && V2_RUNTIME_SOURCE.contains("s[start..end].to_string()"),
            "runtime shim should clamp substring bounds and return empty when the clamped range is invalid"
        );
    }
}
