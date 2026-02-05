//! Category SubDags: Group shared traits for language/format families.
//!
//! - [`TuringComplete`]: Programming languages (Rust, Python, TypeScript)
//! - [`ConfigFormat`]: Configuration file formats (Makefile, gitignore, YAML)

mod config;
mod turing;

pub use config::build_config_format_subdag;
pub use turing::build_turing_complete_subdag;
