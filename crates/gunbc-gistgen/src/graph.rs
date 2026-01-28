use gunbc_ir::types::PatternDecision;
use gunbc_ir::transport::external_types;
use gunbc_ir::*;

use crate::generated;
use crate::ops::{GistgenCoreOp, GistgenOp};

/// Understanding mode determines which implementation of external boundaries to use.
///
/// Each mode selects a different SubDAG for external operations:
/// - `Real` - actual network/filesystem calls
/// - `Mock` - return canned responses, no external calls
/// - `Simulator` - more sophisticated simulation (e.g., record/replay)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnderstandingMode {
    /// Actually perform external operations (network, filesystem, etc.)
    #[default]
    Real,
    /// Mock all external operations - return canned responses.
    Mock,
    /// Simulate external operations with more fidelity than mock.
    Simulator,
}

/// Controls how gistgen forms the CreateRequest payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GistPayloadMode {
    /// Emit a single markdown file (snapshot.md).
    #[default]
    SingleFile,
    /// Emit a file map (multi-file gist).
    FileMap,
}

pub fn build_gistgen_dag(repo_path: &str, glob: &str, mode: UnderstandingMode) -> Dag<GistgenOp> {
    build_gistgen_dag_with_payload(repo_path, glob, mode, GistPayloadMode::SingleFile)
}

pub fn build_gistgen_dag_with_payload(
    repo_path: &str,
    glob: &str,
    mode: UnderstandingMode,
    payload: GistPayloadMode,
) -> Dag<GistgenOp> {
    // Auth SubDAG — built from generated builder (ports, edges, export_node correct by construction)
    let auth_node = generated::build_auth_subdag(
        GistgenOp::Core(GistgenCoreOp::AuthCheck),
        GistgenOp::Core(GistgenCoreOp::AuthCreate),
        GistgenOp::Core(GistgenCoreOp::AuthResolve),
    );

    let gist_subdag = match mode {
        UnderstandingMode::Real => gunbc_ir::transport::build_gist_real(GistgenOp::Gist),
        UnderstandingMode::Mock | UnderstandingMode::Simulator => {
            gunbc_ir::transport::build_gist_mock(GistgenOp::Gist)
        }
    };

    let gist_node = Node {
        id: NodeId("gist".into()),
        inputs: vec![
            port("request", "GitHub::Gist::CreateRequest"),
            port("token", "Secret"),
        ],
        outputs: vec![
            port("response", "GitHub::Gist::CreateResponse"),
            port("gist_url", "String"),
        ],
        body: NodeBody::SubDag(gist_subdag),
    };

    let mut nodes = vec![
        Node {
            id: NodeId("context".into()),
            inputs: vec![],
            outputs: vec![port("repo", "String"), port("selection_spec", "String")],
            body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::Context {
                repo_path: repo_path.into(),
                glob_pattern: glob.into(),
            })),
        },
        auth_node,
        Node {
            id: NodeId("enumerate_files".into()),
            inputs: vec![port("repo", "String")],
            outputs: vec![port("files", "StrList")],
            body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::EnumerateFiles)),
        },
        Node {
            id: NodeId("filter_files".into()),
            inputs: vec![port("files", "StrList"), port("selection_spec", "String")],
            outputs: vec![port("files", "StrList")],
            body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::FilterFiles)),
        },
        Node {
            id: NodeId("read_files".into()),
            inputs: vec![port("repo", "String"), port("files", "StrList")],
            outputs: vec![port("contents", "MapStrStr")],
            body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::ReadFiles)),
        },
    ];

    match payload {
        GistPayloadMode::SingleFile => {
            nodes.push(Node {
                id: NodeId("compose_snapshot".into()),
                inputs: vec![port("contents", "MapStrStr")],
                outputs: vec![port("snapshot", "String")],
                body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::ComposeSnapshot)),
            });
            nodes.push(Node {
                id: NodeId("wrap_single_gist_file".into()),
                inputs: vec![port("snapshot", "String")],
                outputs: vec![port("files", "MapStrStr")],
                body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::WrapSingleGistFile)),
            });
        }
        GistPayloadMode::FileMap => {
            nodes.push(Node {
                id: NodeId("compose_gist_files".into()),
                inputs: vec![port("contents", "MapStrStr")],
                outputs: vec![port("files", "MapStrStr")],
                body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::ComposeGistFiles)),
            });
        }
    }

    nodes.push(Node {
        id: NodeId("build_gist_request".into()),
        inputs: vec![port("files", "MapStrStr")],
        outputs: vec![port("request", "GitHub::Gist::CreateRequest")],
        body: NodeBody::Opaque(GistgenOp::Core(GistgenCoreOp::BuildGistCreateRequest)),
    });
    nodes.push(gist_node);

    let mut edges = vec![
        edge("context", "repo", "enumerate_files", "repo"),
        edge("context", "selection_spec", "filter_files", "selection_spec"),
        edge("enumerate_files", "files", "filter_files", "files"),
        edge("filter_files", "files", "read_files", "files"),
        edge("context", "repo", "read_files", "repo"),
        edge("auth", "token", "gist", "token"),
        edge("build_gist_request", "request", "gist", "request"),
    ];

    match payload {
        GistPayloadMode::SingleFile => {
            edges.push(edge("read_files", "contents", "compose_snapshot", "contents"));
            edges.push(edge("compose_snapshot", "snapshot", "wrap_single_gist_file", "snapshot"));
            edges.push(edge("wrap_single_gist_file", "files", "build_gist_request", "files"));
        }
        GistPayloadMode::FileMap => {
            edges.push(edge("read_files", "contents", "compose_gist_files", "contents"));
            edges.push(edge("compose_gist_files", "files", "build_gist_request", "files"));
        }
    }

    let metadata = DagMetadata {
        pattern_decisions: vec![
            PatternDecisionEntry {
                node: NodeId("auth".into()),
                pattern: "upsert".into(),
                decision: PatternDecision::Instantiated,
            },
            PatternDecisionEntry {
                node: NodeId("gistgen".into()),
                pattern: "upsert".into(),
                decision: PatternDecision::NotApplicable {
                    reason: "gistgen is an Emit tool, not Upsert".into(),
                },
            },
        ],
        export_node: None,
        boundary_declarations: vec![
            BoundaryDeclaration {
                node: NodeId("enumerate_files".into()),
                port: PortName("files".into()),
                external_type: external_types::git_repo(),
            },
            BoundaryDeclaration {
                node: NodeId("read_files".into()),
                port: PortName("contents".into()),
                external_type: external_types::fs_read(),
            },
        ],
    };

    Dag { nodes, edges, metadata }
}
