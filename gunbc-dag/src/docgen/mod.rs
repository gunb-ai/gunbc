//! gunbc-dag doc generation module.
//!
//! Generates documentation with live code excerpts and test indices.

use crate::dsl_builder::build_docgen_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

/// Runtime op type for docgen graphs.
pub type DocgenGraphOp = DynOp;

// Path constants (previously in ops.rs)
pub const AB_DOC_PATH: &str = "docs/ab-writing-workflows.md";
pub const CLIPPY_GRAPH_MOCK_PATH: &str = "lib/tools/clippy/src/graph_mock.rs";
pub const CLIPPY_GENERATED_TESTS_PATH: &str = "lib/tools/clippy/src/generated_tests.rs";
pub const CLIPPY_CONFIG_PATH: &str = "lib/tools/clippy/src/config.rs";
pub const CLIPPY_GRAPH_PATH: &str = "lib/tools/clippy/src/graph.rs";
pub const CLIPPY_LIB_PATH: &str = "lib/tools/clippy/src/lib.rs";
pub const CLIPPY_LINT_PATH: &str = "lib/tools/clippy/src/lint.rs";
pub const CLIPPY_OPS_PATH: &str = "lib/tools/clippy/src/ops.rs";
pub const CLIPPY_POLICY_PATH: &str = "lib/tools/clippy/src/policy.rs";

#[derive(Debug, Clone)]
pub struct DocgenReadTarget {
    pub name: &'static str,
    pub path: &'static str,
    pub input_port: &'static str,
    pub allow_missing: bool,
}

pub const DOCGEN_READ_TARGETS: &[DocgenReadTarget] = &[
    DocgenReadTarget {
        name: "ab_doc_template",
        path: AB_DOC_PATH,
        input_port: "template",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "clippy_graph_mock",
        path: CLIPPY_GRAPH_MOCK_PATH,
        input_port: "clippy_graph_mock",
        allow_missing: true,
    },
    DocgenReadTarget {
        name: "clippy_generated_tests",
        path: CLIPPY_GENERATED_TESTS_PATH,
        input_port: "clippy_generated_tests",
        allow_missing: true,
    },
    DocgenReadTarget {
        name: "clippy_config",
        path: CLIPPY_CONFIG_PATH,
        input_port: "clippy_config",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "clippy_graph",
        path: CLIPPY_GRAPH_PATH,
        input_port: "clippy_graph",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "clippy_lib",
        path: CLIPPY_LIB_PATH,
        input_port: "clippy_lib",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "clippy_lint",
        path: CLIPPY_LINT_PATH,
        input_port: "clippy_lint",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "clippy_ops",
        path: CLIPPY_OPS_PATH,
        input_port: "clippy_ops",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "clippy_policy",
        path: CLIPPY_POLICY_PATH,
        input_port: "clippy_policy",
        allow_missing: false,
    },
];

/// Build docgen graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "docgen",
    builder = "build_docgen_graph().unwrap()"
)]
pub fn build_docgen_graph() -> Result<Dag<DocgenGraphOp>, BuilderError> {
    build_docgen_graph_dsl()
}
