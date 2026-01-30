//! Cargo/Rust toolchain tool DAG.
//!
//! This crate provides operations for working with the Rust toolchain.
//! Other tool DAGs (like Clippy) can use CargoOp to verify cargo is
//! available before running their commands.
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_cargo::CargoOp;
//! use gunbc_exec::Executable;
//!
//! // Check if cargo is available
//! let result = CargoOp::CheckInstalled.execute(HashMap::new())?;
//! let cargo_exists = result.get("exists").and_then(|v| v.as_bool()).unwrap_or(false);
//!
//! if !cargo_exists {
//!     return Err("Cargo not installed. Visit https://rustup.rs to install.".into());
//! }
//!
//! // Run cargo build
//! let build_result = CargoOp::build(["--release"]).execute(HashMap::new())?;
//! ```
//!
//! # Design
//!
//! Unlike other tools, Rust/Cargo installation typically requires user
//! interaction (rustup.rs) or is pre-installed. This DAG focuses on:
//!
//! 1. **Verification**: Check if cargo/rustup is installed
//! 2. **Clear errors**: Provide guidance if not installed
//! 3. **Execution**: Run cargo commands

pub mod ops;

pub use ops::CargoOp;
