//! Runtime shims for v2 compiler intrinsic operations.

use std::collections::HashMap;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Concat trait - resolves the DAG `concat(a, b)` intrinsic for all sequence
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
/// Positions are character indices, not byte offsets.
pub fn char_at(s: impl AsRef<str>, pos: i64) -> String {
    let s = s.as_ref();
    let pos = pos.max(0) as usize;
    if s.is_ascii() {
        return s
            .as_bytes()
            .get(pos)
            .map(|byte| char::from(*byte).to_string())
            .unwrap_or_default();
    }
    s.chars()
        .nth(pos)
        .map(|c| c.to_string())
        .unwrap_or_default()
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
    if s.is_ascii() {
        let start = start.min(s.len());
        let end = end.min(s.len());
        if end <= start {
            return String::new();
        }
        return s[start..end].to_string();
    }
    s.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
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
pub fn map_insert<V: Clone>(
    map: Rc<HashMap<String, V>>,
    key: String,
    value: V,
) -> Rc<HashMap<String, V>> {
    let mut m = Rc::try_unwrap(map).unwrap_or_else(|rc| (*rc).clone());
    m.insert(key, value);
    Rc::new(m)
}

/// Merge two Rc-wrapped HashMaps. Overlay entries overwrite base entries.
/// O(|overlay|) amortized.
pub fn map_merge<V: Clone>(
    base: Rc<HashMap<String, V>>,
    overlay: Rc<HashMap<String, V>>,
) -> Rc<HashMap<String, V>> {
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
// Scanner operations - used by the v2 tokenizer
// ---------------------------------------------------------------------------

/// Scan while a predicate holds, returning the end position.
/// `pred` receives a single-character string and returns true to continue.
/// O(k) where k = number of chars satisfying pred.
pub fn scan_while(s: impl AsRef<str>, start: i64, pred: impl Fn(&str) -> bool) -> i64 {
    let s = s.as_ref();
    let bytes = s.as_bytes();
    let mut pos = start.max(0) as usize;
    if s.is_ascii() {
        while pos < bytes.len() && pred(&char::from(bytes[pos]).to_string()) {
            pos += 1;
        }
    } else {
        let chars: Vec<char> = s.chars().collect();
        while pos < chars.len() && pred(&chars[pos].to_string()) {
            pos += 1;
        }
    }
    pos as i64
}

/// Skip horizontal whitespace (spaces and tabs), returning the new position.
/// Positions are character indices, not byte offsets.
pub fn skip_horizontal_ws(s: impl AsRef<str>, start: i64) -> i64 {
    let s = s.as_ref();
    let mut pos = start.max(0) as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        pos = pos.min(bytes.len());
        while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
            pos += 1;
        }
        return pos as i64;
    }

    let chars: Vec<char> = s.chars().collect();
    pos = pos.min(chars.len());
    while pos < chars.len() && (chars[pos] == ' ' || chars[pos] == '\t') {
        pos += 1;
    }
    pos as i64
}

/// Scan to end of line, returning the position of the newline (or end of string).
/// Positions are character indices, not byte offsets.
pub fn scan_to_eol(s: impl AsRef<str>, start: i64) -> i64 {
    let s = s.as_ref();
    let start = start.max(0) as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        let start = start.min(bytes.len());
        for (offset, byte) in bytes[start..].iter().enumerate() {
            if *byte == b'\n' {
                return (start + offset) as i64;
            }
        }
        return bytes.len() as i64;
    }

    let chars: Vec<char> = s.chars().collect();
    let start = start.min(chars.len());
    for (offset, ch) in chars[start..].iter().enumerate() {
        if *ch == '\n' {
            return (start + offset) as i64;
        }
    }
    chars.len() as i64
}

/// Scan to end of a string literal, handling escape sequences.
/// Returns the position after the closing quote.
/// Positions are character indices, not byte offsets.
pub fn scan_string_end(s: impl AsRef<str>, start: i64) -> i64 {
    let s = s.as_ref();
    let mut pos = start.max(0) as usize;
    if s.is_ascii() {
        let bytes = s.as_bytes();
        pos = pos.min(bytes.len());
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
    pos = pos.min(chars.len());
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
// Filesystem operations - stub for Filesystem.read service call
// ---------------------------------------------------------------------------

/// Result of a filesystem read operation.
#[derive(Debug, Clone)]
pub struct FilesystemReadResult {
    pub content: String,
}

/// Read a file's text content. Stub for the Filesystem.read service call.
pub fn filesystem_read(path: String) -> FilesystemReadResult {
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
    FilesystemReadResult { content }
}
