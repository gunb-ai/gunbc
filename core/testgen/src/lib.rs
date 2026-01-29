//! gunbc-testgen: Test generation for gunbc DAGs.
//!
//! This crate generates test code from DAG structures:
//! - Boundary tests: verify world-write boundaries can be mocked
//! - Composition tests: verify edge types are compatible
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_testgen::{TestGenerator, TestConfig};
//!
//! let generator = TestGenerator::new(&dag)
//!     .with_config(TestConfig::default());
//!
//! let code = generator.generate_test_module("gist_tests");
//! ```

pub mod codegen;
pub mod analyze;

pub use analyze::{DagAnalysis, analyze_dag};
pub use codegen::{TestGenerator, TestConfig};
