//! Tool registry macros (deprecated).
//!
//! The `#[tool_target]` proc macro has been replaced by DSL structural
//! entrypoint inference. See `gunbc-dag/src/dsl_registry.rs`.
//!
//! This crate is retained as an empty shell. It will be removed entirely
//! once downstream references in documentation are cleaned up.

use proc_macro::TokenStream;

/// Deprecated: no-op. Entrypoints are now inferred structurally from DSL `.dag` files.
#[proc_macro_attribute]
pub fn tool_target(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
