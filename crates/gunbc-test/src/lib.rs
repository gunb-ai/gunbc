//! gunbc-test: Test infrastructure for gunbc DAGs.
//!
//! This crate provides:
//! - [`MockOp`]: A mock operation for testing DAGs without real implementations
//! - [`ScriptedDagBuilder`]: Build DAGs with scripted mock behaviors
//! - [`assert_boundary_mockable`]: Verify that a DAG's boundaries can be mocked
//! - [`assert_types_compatible`]: Verify edge type compatibility
//! - [`Mockable`]: Trait for operations to provide test fixtures
//!
//! # Boundary Tests
//!
//! Every DAG should be testable in dry-run mode. Boundary tests verify that
//! all world-write boundaries can be intercepted with mock values.
//!
//! # Composition Tests
//!
//! Edges between nodes must have compatible types. Composition tests verify
//! that the output type of one node matches the input type of the connected node.
//!
//! # Mockable Trait
//!
//! Operations can implement the [`Mockable`] trait to provide:
//! - Default mock outputs for dry-run testing
//! - Cardinality test inputs (empty, one, many) for edge case testing
//! - Error cases for failure testing
//!
//! This enables automatic test generation that provides real signal.

pub mod boundary;
pub mod composition;
pub mod mock;
pub mod mockable;

pub use boundary::{assert_boundary_mockable, default_mocks, mocks_with_values, BoundaryTestResult};
pub use composition::{assert_types_compatible, TypeCompatibility};
pub use mock::{MockBehavior, MockOp, ScriptedDagBuilder};
pub use mockable::{
    CardinalityTestInput, ErrorTestCase, ExpectedBehavior, Mockable,
};
