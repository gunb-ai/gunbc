//! Test generation from proof obligations.
//!
//! Generates test code from DAG structures using the
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

pub mod analyze;
pub mod cardinality;
pub mod codegen;
pub mod obligation;
pub mod render;
pub mod render_python;
pub mod render_rust;
pub mod render_ts;


pub use analyze::{analyze_dag, analyze_dag_with_obligations, DagAnalysis};
pub use codegen::{TestConfig, TestGenerator};
pub use obligation::{
    collect_obligations, DischargeStatus, EntailmentStatus, Obligation, ObligationSet,
    ObligationSource, ObligationStats, ProofObligation,
};
