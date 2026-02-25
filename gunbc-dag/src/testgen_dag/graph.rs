//! Graph builder for the testgen DAG.
//!
//! Builds a dynamic DAG with N parallel upsert chains, one per testgen target.

use crate::testgen_dag::dag_test_discovery::discover_compilable_modules;
use crate::testgen_dag::ops::TestgenOp;
use crate::{add_fs_env_root_node, wire_fs_env_write_edges};
use gunbc_exec::{DynOp, Executable};
use gunbc_ir::{add_content_upsert_chain, build::*, BuilderError, Dag, DagBuilder, Node};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{PrepareFileReadOp, PrepareFileWriteOp};
use gunbc_testgen_registry::DagSpecDef;
use std::path::Path;

/// Runtime op type for testgen graphs.
pub type TestgenGraphOp = DynOp;

fn dyn_op<T>(op: T) -> TestgenGraphOp
where
    T: Executable + Send + Sync + 'static,
{
    DynOp::new(op)
}

/// Build the testgen graph from discovered DAG specs.
pub fn build_testgen_graph(
    targets: &[&DagSpecDef],
    output_dir: &Path,
) -> Result<Dag<TestgenGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    let fs_env = add_fs_env_root_node(&mut builder, dyn_op)?;

    for target in targets {
        let config = target.to_def();
        let name = config.name.clone();

        add_upsert_chain(
            &mut builder,
            &fs_env,
            &name,
            dyn_op(TestgenOp::Generate {
                name: name.to_string(),
                target_def: config,
                generate_fn: target.generate,
            }),
        )?;
    }

    let _ = output_dir;

    Ok(builder.build())
}

/// Build a testgen graph from auto-discovered compilable .dag modules.
///
/// Scans `dsl/` for all .dag files with `func` items and creates a content
/// upsert chain for each. Each chain auto-generates tests from DAG types +
/// structure (zero manual input).
pub fn build_testgen_graph_auto() -> Result<Dag<TestgenGraphOp>, BuilderError> {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .map_err(|e| BuilderError::InternalInvariant(format!("workspace layout: {e}")))?;

    let dsl_root = layout.workspace_root.join("dsl");
    let output_dir = layout.workspace_root.join("gunbc-dag").join("src");
    let modules = discover_compilable_modules(&dsl_root);

    // PT-6: Discover available profiles for per-profile live test generation.
    let profiles = super::profile_discovery::discover_profiles(&dsl_root);

    let mut builder = DagBuilder::new();
    let fs_env = add_fs_env_root_node(&mut builder, dyn_op)?;

    for module in &modules {
        // IS-3: No longer skip modules that require --profile.
        // Stub transport allows compilation without a profile for DryRun testing.

        let safe_name = module.module_name.replace('.', "-");
        let output_path = format!(
            "{}/generated_tests_{}.rs",
            output_dir.display(),
            module.module_name.replace('.', "_"),
        );

        // PT-6: Populate per-profile live test configs for modules that
        // import interfaces. Only profiles binding at least one imported
        // interface are included (scoping per PT-4).
        let live_profile_tests = if module.interface_imports.is_empty() {
            Vec::new()
        } else {
            super::profile_discovery::profiles_for_module(&profiles, &module.interface_imports)
                .into_iter()
                .map(|profile| gunbc_codegen::registry::LiveProfileTestConfig {
                    profile_name: profile.name.clone(),
                    test_class: profile.test_class,
                    fermi_cost: gunbc_test::FermiCost::M,
                    required_env: Vec::new(),
                    required_any_of: Vec::new(),
                    dag_builder_call: format!(
                        "crate::dsl_builder::build_dsl_graph_with_profile(\"{}\", \"{}\").expect(\"profile graph should build\")",
                        module.dsl_path, profile.name,
                    ),
                })
                .collect()
        };

        add_upsert_chain(
            &mut builder,
            &fs_env,
            &safe_name,
            dyn_op(TestgenOp::AutoGenerate {
                dsl_path: module.dsl_path.clone(),
                module_name: module.module_name.clone(),
                output_path,
                live_profile_tests,
            }),
        )?;
    }

    Ok(builder.build())
}

/// Build a testgen graph for testing with hardcoded mock targets.
pub fn build_testgen_graph_for_test() -> Result<Dag<TestgenGraphOp>, BuilderError> {
    use gunbc_codegen::TestgenTargetDef;

    fn mock_generate(def: &TestgenTargetDef) -> String {
        format!(
            "// Generated tests for {}\n#[cfg(test)]\nmod {} {{}}\n",
            def.name, def.module_name
        )
    }

    let targets = [
        (
            "mock-alpha",
            "mock_alpha/generated_tests.rs",
            "mock_alpha_generated_tests",
        ),
        (
            "mock-beta",
            "mock_beta/generated_tests.rs",
            "mock_beta_generated_tests",
        ),
    ];

    let mut builder = DagBuilder::new();
    let fs_env = add_fs_env_root_node(&mut builder, dyn_op)?;

    for (name, output_path, module_name) in &targets {
        let def = TestgenTargetDef::new(*name, *output_path, *module_name);

        add_upsert_chain(
            &mut builder,
            &fs_env,
            name,
            dyn_op(TestgenOp::Generate {
                name: name.to_string(),
                target_def: def,
                generate_fn: mock_generate,
            }),
        )?;
    }

    Ok(builder.build())
}

fn add_upsert_chain(
    builder: &mut DagBuilder<TestgenGraphOp>,
    fs_env: &gunbc_ir::builder::NodeRef<TestgenGraphOp>,
    name: &str,
    generate_op: TestgenGraphOp,
) -> Result<(), BuilderError> {
    let gen_id = format!("generate_{name}");

    let generate = builder.add_root_node(Node::opaque(
        gen_id.as_str(),
        vec![],
        vec![
            port("content", "NonEmptyString"),
            port("path", "String"),
        ],
        generate_op,
    ))?;

    let read_res = resource("file", "FilesystemHandle", AccessMode::Read);
    let write_res = resource("file", "FilesystemHandle", AccessMode::Write);
    let chain = add_content_upsert_chain(
        builder,
        name,
        &generate,
        "content",
        vec![read_res],
        vec![write_res],
        dyn_op(PrepareFileReadOp),
        dyn_op(PrepareFileWriteOp),
        dyn_op(BlobOps::CompareContent),
        dyn_op(TransportOps::Execute),
    )?;

    // Wire the output path from generate into the upsert chain's read/write prepare nodes.
    builder.add_edge(generate.out("path"), chain.prepare_read.in_port("path"))?;
    builder.add_edge(generate.out("path"), chain.prepare_write.in_port("path"))?;

    wire_fs_env_write_edges(
        builder,
        fs_env,
        vec![
            chain.execute_read.in_port("res:file"),
            chain.execute_write.in_port("res:file"),
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_testgen_graph_for_test().expect("graph should build");

        assert!(dag.get_node(&"execute_read_mock-alpha".into()).is_some());
        assert!(dag
            .get_node(&"execute_mock-alpha_transport".into())
            .is_some());
        assert!(dag.get_node(&"execute_read_mock-beta".into()).is_some());
        assert!(dag
            .get_node(&"execute_mock-beta_transport".into())
            .is_some());
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_testgen_graph_for_test().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints
            .is_entrypoint_port(&"compare_mock-alpha_content".into(), &"check_mode".into()));
        assert!(entrypoints
            .is_entrypoint_port(&"compare_mock-beta_content".into(), &"check_mode".into()));
        // path ports are now internally wired from the generate node, not entrypoints.
        assert!(!entrypoints.is_entrypoint_port(&"prepare_read_mock-alpha".into(), &"path".into()));
        assert!(!entrypoints.is_entrypoint_port(&"prepare_read_mock-beta".into(), &"path".into()));
    }

    #[test]
    fn test_pure_nodes_not_boundaries() {
        let dag = build_testgen_graph_for_test().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"generate_mock-alpha".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_mock-alpha".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_write_mock-alpha".into()));
    }

    #[test]
    fn test_auto_graph_discovers_modules() {
        let dag = build_testgen_graph_auto().expect("auto graph should build");

        // Should have nodes for multiple modules (all compilable .dag files)
        let generate_nodes: Vec<_> = dag
            .nodes
            .iter()
            .filter(|n| n.id.0.starts_with("generate_"))
            .collect();

        // At least the 14 tools should produce generate nodes
        assert!(
            generate_nodes.len() >= 14,
            "expected >= 14 generate nodes, found {}",
            generate_nodes.len()
        );

        // Spot-check: makegen should have a generate + upsert chain
        assert!(
            dag.get_node(&"generate_tools-makegen".into()).is_some(),
            "missing generate node for tools-makegen"
        );
        assert!(
            dag.get_node(&"execute_read_tools-makegen".into()).is_some(),
            "missing read transport for tools-makegen"
        );
        assert!(
            dag.get_node(&"execute_tools-makegen_transport".into())
                .is_some(),
            "missing write transport for tools-makegen"
        );
    }
}
