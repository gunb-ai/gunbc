//! Tokenizer for the v3 surface grammar.
//!
//! Authority: `src/v3/compiler/tokenize.dag`.
//! The Rust projection is `tokenize_generated.rs`, produced by
//! `cargo run -p v3-compiler --bin regen_tokenize` — do not hand-edit the generated file.
//!
//! Keywords, punctuation classification, and token kinds are declared in `tokenize.dag`.
//! `<`/`>` tokenize as comparison operators; the parser disambiguates them as
//! type-parameter delimiters by context (M1_DESIGN.md §8.8).

#[allow(
    dead_code,
    unused_imports,
    unused_parens,
    unused_variables,
    clippy::clone_on_copy,
    clippy::collapsible_else_if
)]
mod generated {
    use crate::diagnostics::{Diagnostic, SourceSpan};

    include!("tokenize_generated.rs");
}

pub use generated::{tokenize, Token, TokenKind};
