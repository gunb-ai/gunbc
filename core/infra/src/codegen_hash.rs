//! Shared codegen input hash computation.
//!
//! This is the single canonical implementation of `compute_codegen_input_hash()`,
//! used by both `gunbc-codegen` (to record freshness) and `gunbc-dag` CI ops
//! (to verify freshness).

use crate::hash::{ContentHash, HashBuilder};
use std::io;

/// Glob patterns for codegen input source files.
pub const CODEGEN_GLOB_PATTERNS: &[&str] = &[
    "core/codegen/src/**/*.rs",
    "core/ir/src/**/*.rs",
];

/// Extra individual files that affect codegen output.
pub const CODEGEN_EXTRA_FILES: &[&str] = &[
    "core/codegen/Cargo.toml",
    "core/ir/Cargo.toml",
];

/// Compute the content hash for codegen inputs.
///
/// Returns `(hash, file_count)` where `file_count` is the total number of
/// input files hashed (for the mtime fast path).
///
/// This hashes the source files that affect codegen output:
/// - core/codegen/src/**/*.rs (codegen implementation)
/// - core/ir/src/**/*.rs (IR types used by codegen)
/// - Cargo.toml files for these crates
pub fn compute_codegen_input_hash() -> io::Result<(ContentHash, usize)> {
    let builder = HashBuilder::new();
    let mut file_count: usize = 0;

    // Hash codegen source files
    let (builder, codegen_count) = builder.update_glob(CODEGEN_GLOB_PATTERNS[0])?;
    file_count += codegen_count;

    // Hash IR source files (codegen depends on IR types)
    let (builder, ir_count) = builder.update_glob(CODEGEN_GLOB_PATTERNS[1])?;
    file_count += ir_count;

    // Hash relevant Cargo.toml files
    let builder = builder.update_file(CODEGEN_EXTRA_FILES[0])?;
    file_count += 1;
    let builder = builder.update_file(CODEGEN_EXTRA_FILES[1])?;
    file_count += 1;

    // Include Rust version as part of the hash
    let rust_version = std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let builder = builder.update_str(&rust_version);

    Ok((builder.finalize(), file_count))
}
