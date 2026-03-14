use super::*;

use daglang_emit::rust_exec_runtime::{
    emit_exec_runtime_with_config, EmitConfig as ExecRuntimeEmitConfig,
};

/// Run the compile pipeline stages in order over prepared inputs.
///
/// The stage runner itself does not perform discovery, load profile files, or
/// reread source text for receipts. It simply applies the compiler stages in
/// order to the provided inputs.
pub fn run_compile_pipeline(
    prepared: PreparedCompileContext,
    options: CompileOptions,
) -> Result<CompileOutput, CompileError> {
    let PreparedCompileContext {
        module_graph,
        callable_scope,
        entry_module_name,
        target_module_name,
        source_digest,
        exec_runtime_emit_config,
    } = prepared;

    let typed = typecheck_module_graph_located(
        &module_graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .map_err(|errors| {
        CompileError::Diagnostics(typecheck_diagnostics_located(errors, &module_graph))
    })?;
    let extern_assets = collect_extern_assets(&typed);
    let dsl_registry = typed.dsl_type_registry();
    let process_env_resolver = |name: &str| -> Option<String> { std::env::var(name).ok() };
    let lower_output = lower_to_output_with_config(
        &typed,
        &LoweringConfig {
            callable_modules: callable_scope.as_ref(),
            emit_collection_nodes: options.emit_collection_nodes,
            active_profile: options.profile.as_deref(),
            entry_module: entry_module_name.as_deref(),
            type_registry: Some(dsl_registry),
            env_resolver: &process_env_resolver,
            ..Default::default()
        },
    )
    .map_err(CompileError::from)?;
    let structural_primitive_wiring_errors =
        validate_structural_primitive_input_wiring(&lower_output.dag);
    if !structural_primitive_wiring_errors.is_empty() {
        return Err(CompileError::from(structural_primitive_wiring_errors));
    }
    let daglang_lower::LowerOutput {
        dag: lowered_dag,
        output_paths,
        inferred_entrypoints,
    } = lower_output;
    let verified_dag = VerifiedDag::verify(lowered_dag).map_err(|errors| {
        CompileError::Diagnostics(super::verification_diagnostics_with_sources(
            errors,
            &module_graph,
        ))
    })?;

    let derived = derive_artifacts(&verified_dag).map_err(CompileError::Derive)?;

    let target = options.target;
    let layer = options.layer;
    let mut emitted = emit_with_options(
        &verified_dag,
        &derived,
        options,
        target_module_name.as_deref(),
        &extern_assets,
        &exec_runtime_emit_config,
    )?;
    let emit_manifest_path = append_emit_manifest(&mut emitted, target, layer)?;

    let receipt = source_digest
        .as_deref()
        .map(|digest| compute_receipt(&verified_dag, &emitted, &emit_manifest_path, digest))
        .transpose()?;

    Ok(CompileOutput {
        lowered_dag: verified_dag,
        derived,
        emitted,
        emit_manifest_path,
        output_paths,
        pipeline_params: typed.pipeline_params().to_vec(),
        inferred_entrypoints,
        dsl_type_registry: typed.dsl_type_registry().clone(),
        receipt,
    })
}

fn primitive_requires_strict_input_wiring(kind: &daglang_lower::PrimitiveOpKind) -> bool {
    matches!(kind, daglang_lower::PrimitiveOpKind::GetField { .. }) || kind.is_structural()
}

/// Unified verification for lowered DAGs (CP-41).
///
/// Combines two checks into a single validation step:
/// 1. Generic IR verification (SubDag interfaces, resource wiring, fingerprints, required inputs)
/// 2. Structural primitive input wiring (LoweredOp-specific: GetField + structural ops)
///
/// The structural check always runs and feeds the shared verification failure path.
pub(crate) fn validate_structural_primitive_input_wiring(
    dag: &Dag<LoweredOp>,
) -> Vec<gunbc_ir::VerifyError> {
    let mut errors = Vec::new();
    validate_structural_primitive_input_wiring_recursive(dag, &mut errors);
    errors
}

fn validate_structural_primitive_input_wiring_recursive(
    dag: &Dag<LoweredOp>,
    errors: &mut Vec<gunbc_ir::VerifyError>,
) {
    let connected_inputs: HashSet<(&str, &str)> = dag
        .edges
        .iter()
        .filter(|edge| edge.kind.carries_data())
        .map(|edge| (edge.to_node.0.as_str(), edge.to_port.0.as_str()))
        .collect();

    for node in &dag.nodes {
        match &node.body {
            gunbc_ir::NodeBody::Opaque(LoweredOp::Primitive { kind, .. })
                if primitive_requires_strict_input_wiring(kind) =>
            {
                for port in &node.inputs {
                    if port.name.is_resource()
                        || port.name.is_tool()
                        || port.name.is_internal()
                        || port.cardinality.min == 0
                    {
                        continue;
                    }
                    if !connected_inputs.contains(&(node.id.0.as_str(), port.name.0.as_str())) {
                        errors.push(gunbc_ir::VerifyError::UnwiredInput(
                            gunbc_ir::UnwiredInputError {
                                node_id: node.id.0.clone(),
                                node_name: node.id.0.clone(),
                                port_name: port.name.0.clone(),
                                origin: node.origin.clone(),
                            },
                        ));
                    }
                }
            }
            gunbc_ir::NodeBody::SubDag(inner, _) => {
                validate_structural_primitive_input_wiring_recursive(inner, errors);
            }
            gunbc_ir::NodeBody::Opaque(_) => {}
        }
    }
}

/// Collect extern asset declarations from the typed project.
fn collect_extern_assets(typed: &TypedProject<'_>) -> BTreeSet<ProgramSymbolId> {
    let mut assets = BTreeSet::new();
    for module in typed.modules() {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            if let Item::ExternAssetDecl(def) = &item.node {
                assets.insert(ProgramSymbolId::from_parts(&module_name, &def.name));
            }
        }
    }
    assets
}

fn emit_with_options(
    dag: &Dag<LoweredOp>,
    derived: &DerivedArtifacts,
    options: CompileOptions,
    target_module_name: Option<&str>,
    extern_assets: &BTreeSet<ProgramSymbolId>,
    exec_runtime_emit_config: &ExecRuntimeEmitConfig,
) -> Result<EmissionBundle, CompileError> {
    match (options.target, options.layer) {
        (CodegenTarget::Rust, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_rust_bundle(&reachable, derived).map_err(CompileError::Emit)
        }
        (CodegenTarget::Go, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_go_bundle(&reachable, derived, extern_assets, &options.embedded_data)
                .map_err(CompileError::Emit)
        }
        (CodegenTarget::C, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_c_bundle(&reachable, derived, extern_assets, &options.embedded_data)
                .map_err(CompileError::Emit)
        }
        (CodegenTarget::Mips, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_mips_bundle(&reachable, derived, extern_assets, &options.embedded_data)
                .map_err(CompileError::Emit)
        }
        (CodegenTarget::Rust, CodegenLayer::ExecRuntime) => {
            let module_name = target_module_name
                .or_else(|| {
                    derived
                        .tool_metadata
                        .modules
                        .first()
                        .map(|module| module.module.as_str())
                })
                .unwrap_or("daglang.generated");
            let files = emit_exec_runtime_with_config(dag, module_name, exec_runtime_emit_config)
                .map_err(|error| CompileError::Message(format!("exec-runtime emit: {error}")))?;
            let callable_count = dag.nodes.len();
            let pipeline_count = dag
                .nodes
                .iter()
                .filter(|node| {
                    matches!(
                        &node.body,
                        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline { .. })
                    )
                })
                .count();
            Ok(EmissionBundle {
                backend: "rust-exec-runtime".to_string(),
                files,
                summary: EmissionSummary {
                    module_count: derived.tool_metadata.modules.len(),
                    callable_count,
                    pipeline_count,
                },
            })
        }
        (target, CodegenLayer::ExecRuntime) => Err(CompileError::from(format!(
            "unsupported compile target/layer combination: --target {target} --layer 1; layer 1 currently supports only --target rust"
        ))),
    }
}
