//! Graph builder for the deps tool.

use crate::ops::DepsOp;
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, WorkflowSignature,
};

/// Get the declared signature for the deps workflow.
///
/// This defines the expected inputs and outputs of the workflow.
/// Validation ensures the DAG matches this interface.
///
/// Boundary outputs include:
/// - From load_manifest: dep_count, dep_names (manifest_path is connected downstream)
/// - From generate_scripts: already_installed, needs_install, platform (install_script is connected)
/// - From execute_installs: executed, script
pub fn deps_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints - unconnected input ports)
        .with_input("manifest_path", "String", Cardinality::ZeroOrOne)
        // Outputs from load_manifest (boundary outputs)
        .with_output("dep_count", "Int", Cardinality::One)
        .with_output("dep_names", "StrList", Cardinality::ZeroOrMore)
        // Outputs from generate_scripts (boundary outputs)
        .with_output("already_installed", "StrList", Cardinality::ZeroOrMore)
        .with_output("needs_install", "StrList", Cardinality::ZeroOrMore)
        .with_output("platform", "String", Cardinality::One)
        // Outputs from execute_installs (boundary outputs)
        .with_output("executed", "Bool", Cardinality::One)
        .with_output("script", "String", Cardinality::One)
}

/// Build the deps graph using the generational DagBuilder.
///
/// Pipeline:
/// ```text
/// LoadManifest -> GenerateScripts -> ExecuteInstalls
///                                          ↓
///                                     (boundary)
/// ```
///
/// # Port Cardinalities
///
/// - `manifest_path`: ZeroOrOne (optional, defaults to "deps.toml")
/// - `dep_count`: One (scalar integer)
/// - `dep_names`: ZeroOrMore (list of dependency names, may be empty)
/// - `install_script`: One (generated script)
/// - `already_installed`, `needs_install`: ZeroOrMore (lists of dep names)
/// - `platform`: One (detected platform)
/// - `executed`: One (boolean flag)
/// - `script`: One (executed script)
///
/// # Benefits of DagBuilder
///
/// - Cycles are prevented by construction (generational tracking)
/// - Type and cardinality mismatches are caught at edge creation
/// - Signature validation ensures interface stability
///
/// # Future: UpsertBuilder Pattern with LoopBuilder
///
/// Each dependency installation could be modeled as an upsert:
/// - Check: Verify if dependency is installed
/// - Create: Install the dependency if missing
/// - Resolve: Verify installation succeeded
///
/// This requires combining UpsertBuilder with LoopBuilder:
/// ```text
/// LoadManifest -> LoopBuilder(per dep: UpsertBuilder) -> Aggregate Results
/// ```
///
/// The current implementation handles this in GenerateScripts which processes
/// all dependencies together. A refactored version would:
/// 1. Add DepsOp variants: CheckInstalled { name }, Install { name }, Verify { name }
/// 2. Use LoopBuilder to iterate over dependencies from manifest
/// 3. Each iteration applies UpsertBuilder for that dependency
///
/// This would enable per-dependency dry-run testing and fine-grained control.
pub fn build_deps_graph() -> Result<Dag<DepsOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: LoadManifest (root node - generation 0)
    // Input: optional manifest path
    // Output: dependency metadata
    let load_manifest = builder.add_root_node(Node::opaque(
        "load_manifest",
        vec![optional("manifest_path", "String")],
        vec![
            scalar("dep_count", "Int"),
            list("dep_names", "StrList"),
            scalar("manifest_path", "String"),
        ],
        DepsOp::LoadManifest,
    ))?;

    // Node: GenerateScripts (generation 1)
    // Input: manifest path
    // Output: scripts and dependency status
    let generate_scripts = builder.add_node_after(
        Node::opaque(
            "generate_scripts",
            vec![scalar("manifest_path", "String")],
            vec![
                scalar("install_script", "String"),
                list("already_installed", "StrList"),
                list("needs_install", "StrList"),
                scalar("platform", "String"),
            ],
            DepsOp::GenerateScripts,
        ),
        &load_manifest,
    )?;

    // Node: ExecuteInstalls (generation 2 - BOUNDARY)
    // Input: install script
    // Output: execution results
    let execute_installs = builder.add_node_after(
        Node::opaque(
            "execute_installs",
            vec![scalar("install_script", "String")],
            vec![scalar("executed", "Bool"), scalar("script", "String")],
            DepsOp::ExecuteInstalls,
        ),
        &generate_scripts,
    )?;

    // Wire up the pipeline (validated at edge creation)
    builder.add_edge(
        load_manifest.out("manifest_path"),
        generate_scripts.in_port("manifest_path"),
    )?;
    builder.add_edge(
        generate_scripts.out("install_script"),
        execute_installs.in_port("install_script"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_deps_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 2);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_deps_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(boundaries.is_boundary_node(&"execute_installs".into()));
    }

    #[test]
    fn test_graph_has_entrypoint() {
        let dag = build_deps_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // manifest_path is an entrypoint
        assert!(entrypoints.is_entrypoint_port(&"load_manifest".into(), &"manifest_path".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_deps_graph().expect("graph should build");
        let sig = deps_signature();
        
        // Validate declared signature matches inferred
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_deps_graph().expect("graph should build");
        let inferred = infer_signature(&dag);
        
        // Should have one input (manifest_path) 
        assert_eq!(inferred.inputs.len(), 1);
        assert_eq!(inferred.inputs[0].name.0, "manifest_path");
        
        // Should have 7 boundary outputs total:
        // - load_manifest: dep_count, dep_names
        // - generate_scripts: already_installed, needs_install, platform
        // - execute_installs: executed, script
        assert_eq!(inferred.outputs.len(), 7);
        let output_names: Vec<_> = inferred.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"dep_count"));
        assert!(output_names.contains(&"dep_names"));
        assert!(output_names.contains(&"already_installed"));
        assert!(output_names.contains(&"needs_install"));
        assert!(output_names.contains(&"platform"));
        assert!(output_names.contains(&"executed"));
        assert!(output_names.contains(&"script"));
    }
}
