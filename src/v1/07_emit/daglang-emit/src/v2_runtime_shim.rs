//! Rust source text for the `v2_rt` runtime shim module.
//!
//! When the v2 compiler crate is emitted, this module is written verbatim
//! as `src/v2_rt.rs` so that intrinsic calls (`char_at`, `string_length`,
//! `substring`, `lookup`, `concat`) resolve to real Rust functions.

/// The Rust source code for the v2 runtime shim module.
pub const V2_RUNTIME_SOURCE: &str = r#"//! Runtime shims for v2 compiler intrinsic operations.

use std::collections::HashMap;

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
"#;
