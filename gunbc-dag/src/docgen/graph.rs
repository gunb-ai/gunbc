//! DSL-backed graph builder for doc generation.

use crate::docgen::ops::{
    AB_DOC_PATH, CLIPPY_CONFIG_PATH, CLIPPY_GENERATED_TESTS_PATH, CLIPPY_GRAPH_MOCK_PATH,
    CLIPPY_GRAPH_PATH, CLIPPY_LIB_PATH, CLIPPY_LINT_PATH, CLIPPY_OPS_PATH, CLIPPY_POLICY_PATH,
    GIST_CODEGEN_CLI_PATH, GIST_GENERATED_INTEGRATION_TESTS_PATH,
    GIST_GENERATED_TESTS_SNAPSHOT_PATH, GIST_GRAPH_MOCK_PATH,
};
use crate::dsl_builder::build_docgen_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

/// Runtime op type for docgen graphs.
pub type DocgenGraphOp = DynOp;

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
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "clippy_generated_tests",
        path: CLIPPY_GENERATED_TESTS_PATH,
        input_port: "clippy_generated_tests",
        allow_missing: false,
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
    DocgenReadTarget {
        name: "gist_graph_mock",
        path: GIST_GRAPH_MOCK_PATH,
        input_port: "gist_graph_mock",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "gist_generated_tests_snapshot",
        path: GIST_GENERATED_TESTS_SNAPSHOT_PATH,
        input_port: "gist_generated_tests_snapshot",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "gist_generated_integration_tests",
        path: GIST_GENERATED_INTEGRATION_TESTS_PATH,
        input_port: "gist_generated_integration_tests",
        allow_missing: false,
    },
    DocgenReadTarget {
        name: "gist_codegen_cli",
        path: GIST_CODEGEN_CLI_PATH,
        input_port: "gist_codegen_cli",
        allow_missing: true,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_docgen_graph_from_dsl() {
        let dag = build_docgen_graph().expect("docgen DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
