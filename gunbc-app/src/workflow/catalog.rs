//! DSL-backed workflow catalog + derivation helpers.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::OnceLock;

use daglang_driver::compile_data_from_module;
use gunbc_ir::{Dag, Edge, Node, Port, WorkspaceLayout};
use serde::Deserialize;

use super::capabilities::{
    CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
};
use gunbc_workflow::{
    claim_handle_type_id, ProcessUnitRef, ProcessUnitRegistry, ProcessUnitSpec, UnitClaim,
};
use gunbc_workflow::{
    required_input_contract, required_output_contract, ReportSpec, WorkflowOp, WorkflowSpec,
    WorkflowUnit,
};

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WorkflowVariantDef {
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub file: String,
    pub pipeline: String,
    pub mode: Option<String>,
    pub namespace: String,
    pub is_tool: bool,
}

static WORKFLOW_VARIANTS: OnceLock<Result<Vec<WorkflowVariantDef>, String>> = OnceLock::new();
static DSL_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn dsl_root() -> &'static PathBuf {
    DSL_ROOT.get_or_init(|| {
        let layout = WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout for workflow catalog");
        layout.workspace_root.join("dsl")
    })
}

fn workflow_variants() -> Result<&'static [WorkflowVariantDef], String> {
    WORKFLOW_VARIANTS
        .get_or_init(load_workflow_variants_from_dsl)
        .as_ref()
        .map(|variants| variants.as_slice())
        .map_err(|error| error.clone())
}

fn load_workflow_variants_from_dsl() -> Result<Vec<WorkflowVariantDef>, String> {
    let output = compile_data_from_module(dsl_root(), "config/workflow_catalog.dag")
        .map_err(|e| format!("config/workflow_catalog.dag compilation failed: {e}"))?;
    let value = output
        .data_values
        .get("workflow_variants")
        .ok_or_else(|| {
            "config/workflow_catalog.dag must declare `workflow_variants` data".to_string()
        })?;
    serde_json::from_value(value.clone())
        .map_err(|e| format!("workflow_variants deserialization failed: {e}"))
}

#[derive(Debug, Clone)]
struct StageTemplate {
    name: String,
    after: Vec<String>,
    modes: BTreeSet<String>,
    claims: Vec<UnitClaim>,
}

#[derive(Debug, Clone)]
struct WorkflowTemplate {
    pipeline_name: String,
    stages: Vec<StageTemplate>,
}

pub(super) fn all_tool_workflow_names() -> Result<Vec<&'static str>, String> {
    Ok(workflow_variants()?
        .iter()
        .filter(|variant| variant.is_tool)
        .map(|variant| variant.canonical_name.as_str())
        .collect())
}

pub(super) fn all_known_workflow_names() -> Result<Vec<&'static str>, String> {
    Ok(workflow_variants()?
        .iter()
        .map(|variant| variant.canonical_name.as_str())
        .collect())
}

pub(super) fn resolve_workflow_variant(
    name: &str,
) -> Result<Option<&'static WorkflowVariantDef>, String> {
    let normalized = name.replace('_', "-");
    Ok(workflow_variants()?.iter().find(|variant| {
        variant.canonical_name == name
            || variant.canonical_name == normalized
            || variant
                .aliases
                .iter()
                .any(|alias| alias == name || alias == &normalized)
    }))
}

pub(super) fn build_workflow_spec(
    name: &str,
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let known_workflows = all_known_workflow_names()?;
    let variant = resolve_workflow_variant(name)?.ok_or_else(|| {
        format!(
            "unknown workflow '{}': expected one of {}",
            name,
            known_workflows.join(", ")
        )
    })?;

    let templates = load_workflow_templates()?;
    let template = templates
        .get(&variant.file)
        .ok_or_else(|| format!("missing workflow template for file '{}'", variant.file))?;

    if template.pipeline_name != variant.pipeline {
        return Err(format!(
            "workflow file '{}' defines pipeline '{}', expected '{}'",
            variant.file, template.pipeline_name, variant.pipeline
        ));
    }

    let active_stages: Vec<&StageTemplate> = template
        .stages
        .iter()
        .filter(|stage| stage_is_enabled(stage, variant.mode.as_deref()))
        .collect();

    if active_stages.is_empty() {
        return Err(format!(
            "workflow '{}' has no active stages for mode {:?}",
            variant.canonical_name, variant.mode
        ));
    }

    let included_names = active_stages
        .iter()
        .map(|stage| stage.name.as_str())
        .collect::<BTreeSet<_>>();

    let mut dag: Dag<WorkflowUnit> = Dag::new();

    for stage in &active_stages {
        let node_id = format!("{}.{}", variant.namespace, stage.name);
        let node = if stage.name == "report" {
            Node::opaque(
                node_id.clone(),
                required_input_contract(),
                required_output_contract(),
                WorkflowUnit::new(WorkflowOp::Report(ReportSpec::new(node_id))),
            )
        } else {
            let process_ref = process_ref_for_stage(variant, &stage.name);
            let process_spec = registry.get(&process_ref).ok_or_else(|| {
                format!(
                    "missing process unit registry entry for {}::{}",
                    process_ref.process_id.0, process_ref.unit_id.0
                )
            })?;

            let mut inputs = required_input_contract();
            for claim in &process_spec.required_claims {
                inputs.push(Port::resource(
                    claim.claim_id.as_resource_name(),
                    claim_handle_type_id(&claim.claim_id),
                    claim.access_mode,
                ));
            }

            Node::opaque(
                node_id,
                inputs,
                required_output_contract(),
                WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(process_ref)),
            )
        };

        dag.add_node(node);
    }

    for stage in &active_stages {
        for dependency in &stage.after {
            if !included_names.contains(dependency.as_str()) {
                continue;
            }
            dag.add_edge(Edge::control(
                format!("{}.{}", variant.namespace, dependency),
                "commit",
                format!("{}.{}", variant.namespace, stage.name),
                "after",
            ));
        }
    }

    Ok(WorkflowSpec::new(variant.canonical_name.as_str(), dag, 1))
}

pub(super) fn build_process_unit_registry() -> Result<ProcessUnitRegistry, String> {
    let templates = load_workflow_templates()?;
    let mut registry = ProcessUnitRegistry::new();

    let mut compilation_claims: Option<Vec<UnitClaim>> = None;
    let mut codegen_claims: Option<Vec<UnitClaim>> = None;

    for variant in workflow_variants()? {
        let template = templates
            .get(&variant.file)
            .ok_or_else(|| format!("missing workflow template for file '{}'", variant.file))?;

        if template.pipeline_name != variant.pipeline {
            return Err(format!(
                "workflow file '{}' defines pipeline '{}', expected '{}'",
                variant.file, template.pipeline_name, variant.pipeline
            ));
        }

        for stage in template
            .stages
            .iter()
            .filter(|stage| stage_is_enabled(stage, variant.mode.as_deref()))
        {
            if stage.name == "report" {
                continue;
            }

            if stage.name == "compilation_ensure" {
                if !stage.claims.is_empty() {
                    merge_capability_claims(
                        &mut compilation_claims,
                        &stage.claims,
                        "compilation_ensure",
                    )?;
                }
                continue;
            }

            if stage.name == "codegen_ensure" {
                if !stage.claims.is_empty() {
                    merge_capability_claims(&mut codegen_claims, &stage.claims, "codegen_ensure")?;
                }
                continue;
            }

            let process_ref = process_ref_for_stage(variant, &stage.name);
            let claims = if stage.claims.is_empty() {
                default_stage_claims(&stage.name)
            } else {
                stage.claims.clone()
            };
            let spec = ProcessUnitSpec::new(process_ref.clone(), 1, claims);
            if let Some(existing) = registry.get(&process_ref) {
                if existing.required_claims != spec.required_claims {
                    return Err(format!(
                        "conflicting derived claims for {}::{}",
                        process_ref.process_id.0, process_ref.unit_id.0
                    ));
                }
                continue;
            }
            registry.register(spec);
        }
    }

    registry.register(ProcessUnitSpec::new(
        compilation_ref(),
        1,
        compilation_claims.unwrap_or_else(default_compilation_claims),
    ));
    registry.register(ProcessUnitSpec::new(
        codegen_ref(),
        1,
        codegen_claims.unwrap_or_else(default_codegen_claims),
    ));

    Ok(registry)
}

fn merge_capability_claims(
    slot: &mut Option<Vec<UnitClaim>>,
    claims: &[UnitClaim],
    capability: &str,
) -> Result<(), String> {
    match slot {
        None => {
            *slot = Some(claims.to_vec());
            Ok(())
        }
        Some(existing) if existing == claims => Ok(()),
        Some(_) => Err(format!(
            "conflicting claims derived for capability stage '{}'",
            capability
        )),
    }
}

fn process_ref_for_stage(variant: &WorkflowVariantDef, stage_name: &str) -> ProcessUnitRef {
    match stage_name {
        "compilation_ensure" => compilation_ref(),
        "codegen_ensure" => codegen_ref(),
        _ => {
            let node_id = format!("{}.{}", variant.namespace, stage_name);
            ProcessUnitRef::new(variant.namespace.as_str(), node_id)
        }
    }
}

fn stage_is_enabled(stage: &StageTemplate, mode: Option<&str>) -> bool {
    if stage.modes.is_empty() {
        return true;
    }
    match mode {
        Some(mode_name) => stage.modes.contains(mode_name),
        None => false,
    }
}

fn load_workflow_templates() -> Result<HashMap<String, WorkflowTemplate>, String> {
    let mut templates = HashMap::new();
    for variant in workflow_variants()? {
        if templates.contains_key(&variant.file) {
            continue;
        }
        let module_path = format!("workflows/{}", variant.file);
        let output = compile_data_from_module(dsl_root(), &module_path)
            .map_err(|e| format!("workflow file '{}' compilation failed: {e}", variant.file))?;

        let stages = output
            .pipelines
            .get(&variant.pipeline)
            .ok_or_else(|| {
                format!(
                    "workflow file '{}' does not define pipeline '{}'",
                    variant.file, variant.pipeline
                )
            })?
            .iter()
            .map(|stage| StageTemplate {
                name: stage.name.clone(),
                after: stage.after.clone(),
                modes: stage.modes.clone(),
                claims: Vec::new(),
            })
            .collect();

        templates.insert(
            variant.file.clone(),
            WorkflowTemplate {
                pipeline_name: variant.pipeline.clone(),
                stages,
            },
        );
    }
    Ok(templates)
}

/// Default claims for well-known stages whose resource usage is known
/// structurally but not yet expressible in DSL stage bodies.
fn default_stage_claims(stage_name: &str) -> Vec<UnitClaim> {
    match stage_name {
        // cargo build writes to target/
        "build_compile" => vec![
            UnitClaim::write("file:target"),
            UnitClaim::read("tool:cargo"),
        ],
        // cargo test reads target/ and test artifacts
        "test_run" => vec![
            UnitClaim::read("file:target"),
            UnitClaim::read("tool:cargo"),
        ],
        // clippy reads source + target
        "clippy_run" => vec![
            UnitClaim::read("file:target"),
            UnitClaim::read("tool:cargo"),
        ],
        _ => vec![],
    }
}

fn compilation_ref() -> ProcessUnitRef {
    ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT)
}

fn codegen_ref() -> ProcessUnitRef {
    ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT)
}

fn default_compilation_claims() -> Vec<UnitClaim> {
    vec![
        UnitClaim::write("file:target"),
        UnitClaim::read("tool:cargo"),
    ]
}

fn default_codegen_claims() -> Vec<UnitClaim> {
    vec![UnitClaim::write("file:generated:cli")]
}

/// Default registry for WF1/WF2 planner bootstrap.
///
/// Derived from DSL workflow catalog.
pub fn default_process_unit_registry() -> Result<ProcessUnitRegistry, String> {
    build_process_unit_registry()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_extraction_includes_mode_literals() {
        let output =
            compile_data_from_module(dsl_root(), "workflows/gist.dag").expect("compile gist.dag");
        let stages = output.pipelines.values().next().expect("at least one pipeline");
        let gated: Vec<_> = stages.iter().filter(|s| !s.modes.is_empty()).collect();
        assert!(!gated.is_empty(), "gist.dag should have mode-gated stages");
        // Verify specific mode extraction
        let list_files = stages.iter().find(|s| s.name == "list_files").unwrap();
        assert!(list_files.modes.contains("gist"));
        assert!(list_files.modes.contains("gist-snapshot"));
    }

    #[test]
    fn process_registry_derivation_includes_core_workflows() {
        let registry = build_process_unit_registry().expect("derive registry");
        assert!(registry.contains(&ProcessUnitRef::new("ci", "ci.codegen")));
        assert!(registry.contains(&ProcessUnitRef::new("test_all", "test_all.codegen")));
        assert!(registry.contains(&ProcessUnitRef::new("sdlc", "sdlc.worker")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.gist_create")));
        assert!(registry.contains(&compilation_ref()));
        assert!(registry.contains(&codegen_ref()));
    }

    #[test]
    fn resolve_workflow_variant_supports_sdlc_aliases() {
        let direct = resolve_workflow_variant("sdlc")
            .expect("resolve sdlc")
            .expect("sdlc variant");
        assert_eq!(direct.canonical_name, "sdlc");

        let alias = resolve_workflow_variant("sdlc_worker")
            .expect("resolve sdlc_worker alias")
            .expect("sdlc_worker variant");
        assert_eq!(alias.canonical_name, "sdlc");

        let hyphen_alias = resolve_workflow_variant("sdlc-worker")
            .expect("resolve sdlc-worker alias")
            .expect("sdlc-worker variant");
        assert_eq!(hyphen_alias.canonical_name, "sdlc");
    }
}
