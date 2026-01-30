//! Language SubDags: Concrete language and format definitions.
//!
//! Each language/format is a SubDag that composes category and trait SubDags:
//! - [`Rust`]: Programming language (composes TuringComplete)
//! - [`Gitignore`]: File format (composes ConfigFormat, GlobPatterns, Regex)
//! - [`Makefile`]: Build format (composes ConfigFormat, VariableSyntax)

mod rust;
mod gitignore;
mod makefile;

pub use rust::{build_rust_subdag, rust_type, RustConfig, RUST};
pub use gitignore::{
    build_gitignore_subdag, GitignoreConfig, DEFAULT_GITIGNORE_FILENAME, GITIGNORE,
};
pub use makefile::{
    build_makefile_subdag, MakefileConfig, MakeTarget, DEFAULT_MAKEFILE_FILENAME, MAKEFILE,
};
