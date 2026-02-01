//! gunbc-testgen: Test generation from proof obligations.
//!
//! This crate generates test code from DAG structures using the
//! **proof obligation** model:
//!
//! 1. Analyze the DAG to collect proof obligations
//! 2. Discharge obligations that are statically proven
//! 3. Generate tests only for undischarged obligations
//!
//! This ensures tests are **non-tautological**: we never test what
//! the compiler already proves (type compatibility, cardinality
//! satisfaction, acyclicity).
//!
//! # Obligation Buckets
//!
//! | Bucket | What It Proves |
//! |--------|---------------|
//! | **A** | Execution semantics (DryRun, interception, determinism) |
//! | **B** | Contract obligations (L3 entailment, node compliance) |
//! | **C** | Scenario coverage (success/failure paths, guard branches) |
//! | **D** | Resource hygiene (connectivity, ownership, conflicts) |
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_testgen::{TestGenerator, TestConfig};
//!
//! let generator = TestGenerator::new(&dag)
//!     .with_config(TestConfig::default());
//!
//! let code = generator.generate_test_module("gist_tests", "build_gist_graph()");
//! ```

pub mod analyze;
pub mod codegen;
pub mod obligation;

pub use analyze::{analyze_dag, analyze_dag_with_obligations, DagAnalysis};
pub use codegen::{TestConfig, TestGenerator};
pub use obligation::{
    collect_obligations, DischargeStatus, EntailmentStatus, Obligation, ObligationSet,
    ObligationSource, ObligationStats, ProofObligation,
};
