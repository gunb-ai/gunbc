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

/// Extract the character at a given position as a single-char string.
/// O(1) for ASCII (all .dag source); falls back to O(pos) for multi-byte UTF-8.
pub fn char_at(s: impl AsRef<str>, pos: i64) -> String {
    let bytes = s.as_ref().as_bytes();
    let pos = pos as usize;
    if pos < bytes.len() && bytes[pos] < 128 {
        // ASCII fast path: O(1) indexed access
        return String::from(bytes[pos] as char);
    }
    // Non-ASCII fallback
    s.as_ref().chars().nth(pos).map(|c| c.to_string()).unwrap_or_default()
}

/// Return the number of characters in a string.
/// O(1) for ASCII (byte length == char count); O(n) for multi-byte UTF-8.
pub fn string_length(s: impl AsRef<str>) -> i64 {
    let s = s.as_ref();
    if s.is_ascii() {
        s.len() as i64
    } else {
        s.chars().count() as i64
    }
}

/// Extract a substring by character indices [start, end).
/// O(end - start) for ASCII; O(start + len) for multi-byte UTF-8.
pub fn substring(s: impl AsRef<str>, start: i64, end: i64) -> String {
    let s = s.as_ref();
    let start = start.max(0) as usize;
    let end = end.max(0) as usize;
    if s.is_ascii() && end <= s.len() {
        // ASCII fast path: byte slicing is O(end - start)
        return s[start..end.min(s.len())].to_string();
    }
    s.chars().skip(start).take((end - start).max(0)).collect()
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

/// Scan while a predicate holds, returning the end position.
/// `pred` receives a single-character string and returns true to continue.
/// O(k) where k = number of chars satisfying pred.
pub fn scan_while(s: impl AsRef<str>, start: i64, pred: impl Fn(&str) -> bool) -> i64 {
    let bytes = s.as_ref().as_bytes();
    let mut pos = start.max(0) as usize;
    if s.as_ref().is_ascii() {
        while pos < bytes.len() && pred(&String::from(bytes[pos] as char)) {
            pos += 1;
        }
    } else {
        let chars: Vec<char> = s.as_ref().chars().collect();
        while pos < chars.len() && pred(&chars[pos].to_string()) {
            pos += 1;
        }
    }
    pos as i64
}

/// Skip horizontal whitespace (spaces and tabs), returning the new position.
/// O(k) where k = whitespace run length.
pub fn skip_horizontal_ws(s: impl AsRef<str>, start: i64) -> i64 {
    let bytes = s.as_ref().as_bytes();
    let mut pos = start.max(0) as usize;
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos as i64
}

/// Scan to end of line, returning the position of the newline (or end of string).
/// O(k) where k = distance to newline.
pub fn scan_to_eol(s: impl AsRef<str>, start: i64) -> i64 {
    let bytes = s.as_ref().as_bytes();
    let start = start.max(0) as usize;
    for i in start..bytes.len() {
        if bytes[i] == b'\n' {
            return i as i64;
        }
    }
    bytes.len() as i64
}

/// Scan to end of a string literal, handling escape sequences.
/// Returns the position after the closing quote.
/// O(k) where k = string literal length.
pub fn scan_string_end(s: impl AsRef<str>, start: i64) -> i64 {
    let bytes = s.as_ref().as_bytes();
    let mut pos = start.max(0) as usize;
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
    bytes.len() as i64
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
