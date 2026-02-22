//! Graph builder for the deps tool.
//!
//! Two graphs are provided:
//!
//! ## Install Graph (`build_deps_graph`)
//! All I/O happens through explicit `TransportOps::Execute` nodes:
//! - LoadManifest: PrepareLoadManifest -> Execute -> ParseManifest
//! - ExecuteInstalls: PrepareExecuteInstalls -> Execute -> ParseExecuteResult
//!
//! ## Generation Graph (`build_deps_generate_graph`)
//! Generates deps.toml from the tool registry:
//! - LoadToolRegistry -> RenderDepsToml -> PrepareFileWrite -> ExecuteTransport

use crate::env::PlatformEnv;
use crate::ops::DepsOp;
use gunbc_exec::DynOp;
use gunbc_ir::{
    add_transport_triplet_named_with_passthrough, build::*, BuilderError, Cardinality, Dag,
    DagBuilder, Node, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv, PrepareFileWriteOp};

/// Runtime op type for deps graphs.
pub type DepsGraphOp = DynOp;

/// Get the declared signature for the deps workflow.
pub fn deps_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs
        .with_input("manifest_path", "FilePath", Cardinality::ONE)
        // Outputs - boundary outputs from terminal nodes
        .with_output("dep_count", "Int", Cardinality::ONE)
        .with_output("dep_names", "StringList", Cardinality::ZERO_OR_MORE)
        .with_output("manifest_path", "FilePath", Cardinality::ONE)
        .with_output("already_installed", "StringList", Cardinality::ZERO_OR_MORE)
        .with_output("needs_install", "StringList", Cardinality::ZERO_OR_MORE)
        .with_output("platform", "String", Cardinality::ONE)
        .with_output("executed", "Bool", Cardinality::ONE)
        .with_output("success", "Bool", Cardinality::ONE)
        .with_output("script", "String", Cardinality::ONE)
        .with_output("stdout", "String", Cardinality::ONE)
        .with_output("stderr", "String", Cardinality::ONE)
}

/// Build the deps graph with explicit transport nodes.
///
/// Pipeline:
/// ```text
/// PrepareLoadManifest -> Execute -> ParseManifest -> GenerateScripts -> PrepareExecuteInstalls -> Execute -> ParseExecuteResult
///                          ↑                                                                        ↑
///                      (boundary)                                                               (boundary)
/// ```
pub fn build_deps_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Environment: Platform
    // ========================================================================

    let platform_env = builder.add_root_node(Node::opaque(
        "platform_env",
        vec![],
        vec![port("platform", "Platform")],
        DynOp::new(PlatformEnv),
    ))?;

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        DynOp::new(FsEnv::new(filename::Scope::Write)),
    ))?;

    // ========================================================================
    // LoadManifest chain: PrepareLoadManifest -> Execute -> ParseManifest
    // ========================================================================

    let load_manifest = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "load_manifest",
        "prepare_load_manifest",
        "execute_load_manifest",
        "parse_manifest",
        vec![port("manifest_path", "FilePath")],
        vec![resource(
            "file:deps.toml",
            "FilesystemHandle",
            AccessMode::Read,
        )],
        vec![port("manifest_path", "FilePath")],
        vec![
            scalar("dep_count", "Int"),
            list("dep_names", "StringList"),
            scalar("manifest_path", "FilePath"),
            scalar("manifest_content", "String"), // Pass content to GenerateScripts
        ],
        DynOp::new(DepsOp::PrepareLoadManifest),
        DynOp::new(DepsOp::ParseManifest),
        DynOp::new(TransportOps::Execute),
        Some(&fs_env),
    )?;

    // ========================================================================
    // GenerateScripts (PURE domain logic)
    // Now receives manifest_content instead of loading from file
    // ========================================================================

    let generate_scripts = builder.add_node_after(
        Node::opaque(
            "generate_scripts",
            vec![
                scalar("manifest_content", "String"), // Receives content, not path
                resource("platform", "Platform", AccessMode::Read), // Platform acquired at boundary
            ],
            vec![
                scalar("install_script", "String"),
                list("already_installed", "StringList"),
                list("needs_install", "StringList"),
                scalar("platform", "String"),
            ],
            DynOp::new(DepsOp::GenerateScripts),
        ),
        &load_manifest,
    )?;

    // ========================================================================
    // ExecuteInstalls chain: PrepareExecuteInstalls -> Execute -> ParseExecuteResult
    // ========================================================================

    // Node: PrepareExecuteInstalls (PURE)
    let execute_installs = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "execute_installs",
        "prepare_execute_installs",
        "execute_installs",
        "parse_execute_result",
        vec![scalar("install_script", "String")],
        vec![resource("file", "FilesystemHandle", AccessMode::Write)],
        vec![scalar("script", "String")],
        vec![
            scalar("executed", "Bool"),
            scalar("success", "Bool"),
            scalar("script", "String"),
            scalar("stdout", "String"),
            scalar("stderr", "String"),
        ],
        DynOp::new(DepsOp::PrepareExecuteInstalls),
        DynOp::new(DepsOp::ParseExecuteResult),
        DynOp::new(TransportOps::Execute),
        Some(&generate_scripts),
    )?;

    // ========================================================================
    // Wire up the pipeline
    // ========================================================================

    // LoadManifest chain
    // To GenerateScripts (now receives manifest_content instead of path)
    builder.add_edge(
        load_manifest.out("manifest_content"),
        generate_scripts.in_port("manifest_content"),
    )?;
    builder.add_edge(
        platform_env.out("platform"),
        generate_scripts.in_port("res:platform"),
    )?;

    // ExecuteInstalls chain
    builder.add_edge(
        generate_scripts.out("install_script"),
        execute_installs.in_port("install_script"),
    )?;

    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        load_manifest.in_port("res:file:deps.toml"),
    )?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        execute_installs.in_port("res:file"),
    )?;

    let dag = builder.build();
    if let Some(unwired) = gunbc_ir::validate_resource_wiring(&dag).first() {
        return Err(BuilderError::UnwiredResourceInput {
            node: unwired.node.clone(),
            port: unwired.port.clone(),
        });
    }
    Ok(dag)
}

// ============================================================================
// deps.toml Generation Graph
// ============================================================================

/// Get the declared signature for the deps generate workflow.
pub fn deps_generate_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("path", "FilePath", Cardinality::ONE)
        // Outputs from execute_transport (boundary)
        .with_output("response", "TransportResponse", Cardinality::ONE)
        .with_output("written_path", "FilePath", Cardinality::ONE)
        .with_output("content", "String", Cardinality::ONE)
        // Informational outputs from load_tool_registry
        .with_output("tool_count", "Int", Cardinality::ONE)
        .with_output("tool_names", "String", Cardinality::ONE_OR_MORE)
}

/// Build the deps generate graph.
///
/// Pipeline (follows makegen pattern):
/// ```text
/// LoadToolRegistry -> RenderDepsToml -> PrepareFileWrite -> ExecuteTransport
///      (deps)           (deps)           (primitive)         (transport)
///                                           PURE              BOUNDARY
/// ```
///
/// This graph generates deps.toml from the tool registry, owning the file's
/// generation in the same way makegen owns Makefile generation.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "deps-generate",
    builder = "build_deps_generate_graph().unwrap()"
)]
pub fn build_deps_generate_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        DynOp::new(FsEnv::new(filename::Scope::Write)),
    ))?;

    // Node: LoadToolRegistry (deps-specific) - generation 0
    // No inputs (uses default registry)
    // Outputs: tool metadata (informational)
    let load_registry = builder.add_root_node(Node::opaque(
        "load_tool_registry",
        vec![],
        vec![
            scalar("tool_count", "Int"),
            non_empty_list("tool_names", "NonEmptyStringList"),
        ],
        DynOp::new(DepsOp::LoadToolRegistry),
    ))?;

    // Node: RenderDepsToml (deps-specific) - generation 1
    // No inputs (uses registry directly internally)
    // Output: generated deps.toml content
    let render_deps_toml = builder.add_node_after(
        Node::opaque(
            "render_deps_toml",
            vec![],
            vec![scalar("deps_toml_content", "NonEmptyString")],
            DynOp::new(DepsOp::RenderDepsToml),
        ),
        &load_registry,
    )?;

    // Node: PrepareFileWrite (primitive) - generation 2
    // Prepares the TransportRequest for file write
    let prepare_write = builder.add_node_after(
        Node::opaque(
            "prepare_file_write",
            vec![
                scalar("content", "NonEmptyString"),
                port("path", "FilePath"),
            ],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(PrepareFileWriteOp),
        ),
        &render_deps_toml,
    )?;

    // Node: ExecuteTransport (transport boundary) - generation 3
    // Actually writes the file
    let execute_transport = builder.add_node_after(
        Node::opaque(
            "execute_transport",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("file:deps.toml", "FilesystemHandle", AccessMode::Write),
            ],
            vec![
                port("response", "TransportResponse"),
                port("written_path", "FilePath"),
                port("content", "String"),
            ],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_write,
    )?;

    // ========================================================================
    // Wire up the pipeline
    // ========================================================================

    // RenderDepsToml -> PrepareFileWrite
    builder.add_edge(
        render_deps_toml.out("deps_toml_content"),
        prepare_write.in_port("content"),
    )?;

    // PrepareFileWrite -> ExecuteTransport
    builder.add_edge(
        prepare_write.out("request"),
        execute_transport.in_port("request"),
    )?;
    builder.add_edge(prepare_write.out("skip"), execute_transport.in_port("skip"))?;

    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        execute_transport.in_port("res:file:deps.toml"),
    )?;

    let dag = builder.build();
    if let Some(unwired) = gunbc_ir::validate_resource_wiring(&dag).first() {
        return Err(BuilderError::UnwiredResourceInput {
            node: unwired.node.clone(),
            port: unwired.port.clone(),
        });
    }
    Ok(dag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_deps_graph().expect("graph should build");
        // Top-level nodes: env nodes, SubDag wrappers, and pure nodes
        let expected_nodes = [
            "platform_env",
            "fs_env",
            "load_manifest",
            "generate_scripts",
            "execute_installs",
        ];

        for node_id in expected_nodes {
            assert!(
                dag.get_node(&node_id.into()).is_some(),
                "missing node: {}",
                node_id
            );
        }
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_deps_graph().expect("graph should build");

        // Verify SubDag transport nodes exist
        assert!(dag.get_node(&"load_manifest".into()).is_some());
        assert!(dag.get_node(&"execute_installs".into()).is_some());
    }

    #[test]
    fn test_graph_has_entrypoint() {
        let dag = build_deps_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // manifest_path is an entrypoint on the load_manifest SubDag
        assert!(entrypoints.is_entrypoint_port(&"load_manifest".into(), &"manifest_path".into()));
    }

    #[test]
    fn test_env_nodes_not_boundaries() {
        let dag = build_deps_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Environment nodes provide resources to other nodes — not boundaries
        assert!(!boundaries.is_boundary_node(&"platform_env".into()));
        assert!(!boundaries.is_boundary_node(&"fs_env".into()));
    }
}
