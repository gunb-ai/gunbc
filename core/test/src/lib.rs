//! gunbc-test: Test infrastructure for gunbc DAGs.
//!
//! This crate provides:
//! - [`MockOp`]: A mock operation for testing DAGs without real implementations
//! - [`ScriptedDagBuilder`]: Build DAGs with scripted mock behaviors
//! - [`assert_boundary_mockable`]: Verify that a DAG's boundaries can be mocked
//! - [`assert_types_compatible`]: Verify edge type compatibility
//! - [`Mockable`]: Trait for operations to provide test fixtures
//! - [`MockSpec`]: File-level mock specifications for chain validation
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
//! # Mock Specifications
//!
//! Mock specs are declared in adjacent files (e.g., `graph_mock.rs`) and define:
//! - What mock values boundary nodes provide
//! - What input constraints upstream must satisfy
//!
//! This enables chain validation: A's mock output must satisfy B's expected input.
//!
//! This enables automatic test generation that provides real signal.

#![deny(dead_code)]
pub mod boundary;
pub mod composition;
pub mod corpus;
pub mod fermi;
pub mod fidelity;
pub mod mock;
pub mod mock_requirements;
pub mod mock_spec;
pub mod mock_synthesis;
pub mod mockable;
pub mod simulator;
pub mod window;

pub use boundary::{
    assert_boundary_mockable, default_mocks, mocks_with_values, BoundaryTestResult,
};
pub use composition::{assert_types_compatible, TypeCompatibility};
pub use corpus::{
    is_redacted_type, normalize_value, CorpusExample, EdgeExample, Expectation, MockCorpus,
    NodeIdentity, Provenance, SeedKind,
};
pub use fermi::{
    guard, guard_test, guard_test_with_env, max_cost_from_env, FermiCost, TestClass, TestMeta,
};
pub use fidelity::{
    canonical_ladders, node_max_fidelity, FidelityLadder, FidelityLevel, FidelityRung,
};
pub use mock::{MockBehavior, MockOp, ScriptedDagBuilder};
pub use mock_requirements::{
    extract_mock_requirements, MissingSlot, MockIncompleteError, MockRequirements, MockSlot,
    MockSlotKind, MockTypeError,
};
pub use mock_spec::{
    assert_boundaries, assert_transport_mocks, validate_chain, BoundaryMock, ChainError,
    ChainValidationResult, ExpectedOutput, InputConstraint, InputExpectation, InputMock,
    LiveExpectedOutput, MockSpec, NodeExample, OutputMatcher, ResourceAcquireResult,
    ResourceBehavior, ResourceMocks, ResourceRefreshResult, ResourceSimulation, ResourceType,
    TransportMock,
};
pub use mock_synthesis::{synthesize_rest_response, MockProvider, MockResponseSynthesis};
pub use mockable::{CardinalityTestInput, ErrorTestCase, ExpectedBehavior, Mockable};
pub use simulator::{IoContract, Simulator};
pub use window::{
    apply_window_inputs, assert_chain_outputs, assert_window_outputs, window_subdag, Window,
    WindowError,
};
/// Assert that the typed MockSpec builder rejects an unknown slot.
///
/// This is the shared implementation for the `test_typed_builder_rejects_wrong_slot`
/// test that appears in every `graph_mock.rs`.
pub fn assert_typed_builder_rejects_invalid_slot<T: Clone>(dag: &gunbc_ir::Dag<T>, name: &str) {
    let reqs = extract_mock_requirements(dag, name);
    let result = reqs.boundary_str("nonexistent_node", "nonexistent_port", "value");
    assert!(
        result.is_err(),
        "expected typed builder to reject unknown slot for {name}"
    );
}
pub mod temp;
pub use temp::{unique_temp_dir, unique_temp_file};

pub mod json;
pub use json::ParseJsonOutput;
