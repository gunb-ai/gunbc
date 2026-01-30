//! Category SubDags: Group shared traits for language/format families.
//!
//! - [`TuringComplete`]: Programming languages (Rust, Python, TypeScript)
//! - [`ConfigFormat`]: Configuration file formats (Makefile, gitignore, YAML)

mod turing;
mod config;

pub use turing::build_turing_complete_subdag;
pub use config::build_config_format_subdag;
