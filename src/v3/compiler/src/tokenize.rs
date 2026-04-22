//! Tokenizer for the v3 surface grammar.
//!
//! Authority: `src/v3/compiler/tokenize.dag` for scanner behavior,
//! `src/v3/std/tokenize.dag` for shared token types.
//! The Rust projection is `tokenize_generated.rs`, produced by
//! `cargo run -p v3-compiler --bin regen_tokenize` — do not hand-edit the generated file.
//!
//! Tokenizer-local punctuation and scanner controls are declared in
//! `tokenize.dag`; the shared `Token` / `TokenKind` family lives in
//! `src/v3/std/tokenize.dag`. Dedicated keywords and shared-operator
//! punctuation derive from `dsl/extdeps/languages/dag/syntax.dag` during
//! `regen_tokenize`.
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
    include!("tokenize_generated.rs");
}

pub use generated::{tokenize, Token, TokenKind};
