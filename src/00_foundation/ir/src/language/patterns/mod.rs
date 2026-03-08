//! Pattern SubDags: Foundation patterns that other SubDags compose.
//!
//! These are the lowest-level building blocks:
//! - [`Regex`]: Pattern matching and validation
//! - [`GlobPatterns`]: File matching (composes Regex)
//! - [`VariableSyntax`]: Template expansion

pub mod glob;
pub mod regex;
pub mod variable;

pub use glob::build_glob_subdag;
pub use regex::build_regex_subdag;
pub use variable::build_variable_syntax_subdag;
