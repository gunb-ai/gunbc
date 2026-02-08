//! Graph builder for doc generation.
//!
//! Generates documentation artifacts from live code and test sources.

use crate::docgen::ops::{
    DocgenOp, AB_DOC_PATH, CLIPPY_CONFIG_PATH, CLIPPY_GENERATED_TESTS_PATH, CLIPPY_GRAPH_MOCK_PATH,
    CLIPPY_GRAPH_PATH, CLIPPY_LIB_PATH, CLIPPY_LINT_PATH, CLIPPY_OPS_PATH, CLIPPY_POLICY_PATH,
    GIST_CODEGEN_CLI_PATH, GIST_GENERATED_INTEGRATION_TESTS_PATH,
    GIST_GENERATED_TESTS_SNAPSHOT_PATH, GIST_GRAPH_MOCK_PATH,
};
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    add_content_upsert_chain, add_transport_triplet, build::*, BuilderError, Dag, DagBuilder, Node,
    Value,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv, PrepareFileReadOp, PrepareFileWriteOp};
use std::collections::HashMap;

/// Union type for docgen graph operations.
#[derive(Debug, Clone)]
pub enum DocgenGraphOp {
    /// Docgen-specific pure operations.
    Docgen(DocgenOp),
    /// Filesystem environment (resource acquisition).
    FsEnv(FsEnv),
    /// Prepare file read (pure).
    PrepareFileRead(PrepareFileReadOp),
    /// Prepare file write (pure).
    PrepareFileWrite(PrepareFileWriteOp),
    /// Blob operations (compare content - pure).
    Blob(BlobOps),
    /// Transport operations (boundary - actual I/O).
    Transport(TransportOps),
}

impl Executable for DocgenGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DocgenGraphOp::Docgen(op) => op.execute(inputs),
            DocgenGraphOp::FsEnv(op) => op.execute(inputs),
            DocgenGraphOp::PrepareFileRead(op) => op.execute(inputs),
            DocgenGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            DocgenGraphOp::Blob(op) => op.execute(inputs),
            DocgenGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

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

fn add_docgen_read_triplet(
    builder: &mut DagBuilder<DocgenGraphOp>,
    fs_env: &gunbc_ir::builder::NodeRef<DocgenGraphOp>,
    target: &DocgenReadTarget,
) -> Result<gunbc_ir::builder::NodeRef<DocgenGraphOp>, BuilderError> {
    let triplet = add_transport_triplet(
        builder,
        target.name,
        vec![],
        vec![resource("fs", "FilesystemHandle", AccessMode::Read)],
        vec![port("content", "String")],
        DocgenGraphOp::Docgen(DocgenOp::PrepareFileRead {
            path: target.path.to_string(),
        }),
        DocgenGraphOp::Docgen(DocgenOp::ParseFileContent {
            path: target.path.to_string(),
            allow_missing: target.allow_missing,
        }),
        DocgenGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    builder.add_edge(fs_env.out("fs:write"), triplet.execute.in_port("res:fs"))?;

    Ok(triplet.parse)
}

/// Build the docgen graph.
///
/// One content-upsert chain:
/// - docs/ab-writing-workflows.md (handwritten template + generated sections)
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "docgen",
    builder = "build_docgen_graph().unwrap()"
)]
pub fn build_docgen_graph() -> Result<Dag<DocgenGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs:write", "FilesystemHandle")],
        DocgenGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
    ))?;

    let mut read_nodes: HashMap<&'static str, gunbc_ir::builder::NodeRef<DocgenGraphOp>> =
        HashMap::new();
    for target in DOCGEN_READ_TARGETS {
        let parse = add_docgen_read_triplet(&mut builder, &fs_env, target)?;
        read_nodes.insert(target.input_port, parse);
    }

    let anchor = read_nodes
        .values()
        .next()
        .expect("docgen requires read inputs")
        .clone();

    let render_inputs: Vec<_> = DOCGEN_READ_TARGETS
        .iter()
        .map(|target| port(target.input_port, "String"))
        .collect();

    // Generate main doc (with generated sections)
    let render_ab_doc = builder.add_node_after(
        Node::opaque(
            "render_ab_workflows_doc",
            render_inputs,
            vec![scalar("content", "String"), scalar("path", "String")],
            DocgenGraphOp::Docgen(DocgenOp::RenderAbWorkflowsDoc),
        ),
        &anchor,
    )?;

    for target in DOCGEN_READ_TARGETS {
        let parse = read_nodes
            .get(target.input_port)
            .expect("docgen read target missing parse node");
        builder.add_edge(
            parse.out("content"),
            render_ab_doc.in_port(target.input_port),
        )?;
    }

    let doc_read = resource(
        "fs:docs/ab-writing-workflows.md",
        "FilesystemHandle",
        AccessMode::Read,
    );
    let doc_write = resource(
        "fs:docs/ab-writing-workflows.md",
        "FilesystemHandle",
        AccessMode::Write,
    );
    let chain_ab_doc = add_content_upsert_chain(
        &mut builder,
        "ab_workflows_doc",
        &render_ab_doc,
        "content",
        vec![doc_read],
        vec![doc_write],
        DocgenGraphOp::PrepareFileRead(PrepareFileReadOp),
        DocgenGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        DocgenGraphOp::Blob(BlobOps::CompareContent),
        DocgenGraphOp::Transport(TransportOps::Execute),
    )?;

    builder.add_edge(
        render_ab_doc.out("path"),
        chain_ab_doc.prepare_read.in_port("path"),
    )?;
    builder.add_edge(
        render_ab_doc.out("path"),
        chain_ab_doc.prepare_write.in_port("path"),
    )?;

    builder.add_edge(
        fs_env.out("fs:write"),
        chain_ab_doc
            .execute_read
            .in_port("res:fs:docs/ab-writing-workflows.md"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        chain_ab_doc
            .execute_write
            .in_port("res:fs:docs/ab-writing-workflows.md"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::detect_boundaries;

    #[test]
    fn test_transport_boundaries_present() {
        let dag = build_docgen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);
        assert!(boundaries.is_boundary_node(&"execute_ab_workflows_doc_transport".into()));
    }
}
