//! DSL-backed workflow catalog + derivation helpers.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use daglang_driver::{compile_from_context, DriverContext};
use daglang_syntax::{
    ast::{Expr, Item, Literal, Stmt},
    parser,
};
use gunbc_ir::{resource::AccessMode, Dag, Edge, Node, Port};
use serde::Deserialize;

use super::capabilities::{
    CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
};
use super::process_registry::{
    claim_handle_type_id, ProcessUnitRef, ProcessUnitRegistry, ProcessUnitSpec, UnitClaim,
};
use super::schema::{
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

#[derive(Debug, Clone, Deserialize)]
struct StageClaimDef {
    claim_id: String,
    access_mode: String,
}

#[derive(Debug, Clone, Deserialize)]
struct StageDefaultClaimsDef {
    stage_name: String,
    claims: Vec<StageClaimDef>,
}

#[derive(Debug, Clone)]
struct WorkflowCatalogData {
    workflow_variants: Vec<WorkflowVariantDef>,
    default_stage_claims: HashMap<String, Vec<UnitClaim>>,
}

static WORKFLOW_CATALOG_DATA: OnceLock<Result<WorkflowCatalogData, String>> = OnceLock::new();

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

#[allow(clippy::disallowed_methods)]
fn workflow_catalog_data() -> Result<&'static WorkflowCatalogData, String> {
    WORKFLOW_CATALOG_DATA
        .get_or_init(load_workflow_catalog_data)
        .as_ref()
        .map_err(|error| error.clone())
}

#[allow(clippy::disallowed_methods)]
fn load_workflow_catalog_data() -> Result<WorkflowCatalogData, String> {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let dag_file = dsl_root.join("config/workflow_catalog.dag");
    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(dag_file.clone()),
    };
    let output = compile_from_context(&context)
        .map_err(|error| format!("failed to compile {}: {error}", dag_file.display()))?;

    let variants_value = output
        .data_values
        .get("workflow_variants")
        .cloned()
        .ok_or_else(|| "missing data value 'workflow_variants'".to_string())?;
    let workflow_variants: Vec<WorkflowVariantDef> = serde_json::from_value(variants_value)
        .map_err(|error| format!("invalid workflow_variants data: {error}"))?;
    if workflow_variants.is_empty() {
        return Err("workflow catalog is empty".to_string());
    }

    let defaults_value = output
        .data_values
        .get("default_stage_claims")
        .cloned()
        .ok_or_else(|| "missing data value 'default_stage_claims'".to_string())?;
    let defaults: Vec<StageDefaultClaimsDef> = serde_json::from_value(defaults_value)
        .map_err(|error| format!("invalid default_stage_claims data: {error}"))?;

    let mut default_stage_claims = HashMap::new();
    for stage in defaults {
        if default_stage_claims.contains_key(&stage.stage_name) {
            return Err(format!(
                "duplicate default_stage_claims entry for stage '{}'",
                stage.stage_name
            ));
        }
        let claims = stage
            .claims
            .into_iter()
            .map(|claim| {
                Ok(UnitClaim::new(
                    claim.claim_id,
                    parse_access_mode(&claim.access_mode)?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        default_stage_claims.insert(stage.stage_name, claims);
    }

    Ok(WorkflowCatalogData {
        workflow_variants,
        default_stage_claims,
    })
}

fn parse_access_mode(mode: &str) -> Result<AccessMode, String> {
    match mode.to_ascii_lowercase().as_str() {
        "read" => Ok(AccessMode::Read),
        "write" => Ok(AccessMode::Write),
        "exclusive" => Ok(AccessMode::Exclusive),
        other => Err(format!(
            "unknown access mode '{other}' in workflow catalog; expected Read/Write/Exclusive"
        )),
    }
}

pub(super) fn all_tool_workflow_names() -> Vec<String> {
    workflow_catalog_data()
        .unwrap_or_else(|error| panic!("failed to load workflow catalog DSL data: {error}"))
        .workflow_variants
        .iter()
        .filter(|variant| variant.is_tool)
        .map(|variant| variant.canonical_name.clone())
        .collect()
}

pub(super) fn all_known_workflow_names() -> Vec<String> {
    workflow_catalog_data()
        .unwrap_or_else(|error| panic!("failed to load workflow catalog DSL data: {error}"))
        .workflow_variants
        .iter()
        .map(|variant| variant.canonical_name.clone())
        .collect()
}

pub(super) fn resolve_workflow_variant(name: &str) -> Option<WorkflowVariantDef> {
    let normalized = name.replace('_', "-");
    workflow_catalog_data()
        .ok()?
        .workflow_variants
        .iter()
        .find(|variant| {
            variant.canonical_name == name
                || variant.canonical_name == normalized
                || variant.aliases.iter().any(|alias| alias == name || alias == &normalized)
        })
        .cloned()
}

pub(super) fn build_workflow_spec(
    name: &str,
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let variant = resolve_workflow_variant(name).ok_or_else(|| {
        format!(
            "unknown workflow '{}': expected one of {}",
            name,
            all_known_workflow_names().join(", ")
        )
    })?;

    let templates = load_workflow_templates()?;
    let template = templates
        .get(variant.file.as_str())
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

    for variant in &workflow_catalog_data()?.workflow_variants {
        let template = templates
            .get(variant.file.as_str())
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

#[allow(clippy::disallowed_methods)]
fn load_workflow_templates() -> Result<HashMap<String, WorkflowTemplate>, String> {
    let mut templates = HashMap::new();
    for variant in &workflow_catalog_data()?.workflow_variants {
        if templates.contains_key(&variant.file) {
            continue;
        }
        templates.insert(
            variant.file.clone(),
            parse_workflow_template(&variant.file, &variant.pipeline)?,
        );
    }
    Ok(templates)
}

#[allow(clippy::disallowed_methods)]
fn parse_workflow_template(file: &str, pipeline_name: &str) -> Result<WorkflowTemplate, String> {
    let path = workflow_file_path(file);
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    let parsed = parser::parse_with_file_diagnostics(&path, &source).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.render())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let mut pipeline_stages = None;
    for item in parsed.items {
        if let Item::PipelineDef(def) = item.node {
            if def.name == pipeline_name {
                pipeline_stages = Some(def.stages);
                break;
            }
        }
    }

    let stage_defs = pipeline_stages.ok_or_else(|| {
        format!(
            "workflow file '{}' does not define pipeline '{}'",
            file, pipeline_name
        )
    })?;

    let stages = stage_defs
        .into_iter()
        .map(|stage| StageTemplate {
            name: stage.name,
            after: stage.after,
            modes: parse_stage_modes(stage.when.as_ref()),
            claims: parse_stage_claims(&stage.body.stmts),
        })
        .collect();

    Ok(WorkflowTemplate {
        pipeline_name: pipeline_name.to_string(),
        stages,
    })
}

fn parse_stage_modes(condition: Option<&Expr>) -> BTreeSet<String> {
    let mut modes = BTreeSet::new();
    let Some(condition) = condition else {
        return modes;
    };
    collect_mode_literals(condition, &mut modes);
    modes
}

fn collect_mode_literals(expr: &Expr, modes: &mut BTreeSet<String>) {
    match expr {
        Expr::BinOp(lhs, op, rhs) => match op {
            daglang_syntax::ast::BinOp::Eq => {
                if let Some(mode) = mode_literal_from_equality(lhs, rhs) {
                    modes.insert(mode);
                }
            }
            daglang_syntax::ast::BinOp::And | daglang_syntax::ast::BinOp::Or => {
                collect_mode_literals(lhs, modes);
                collect_mode_literals(rhs, modes);
            }
            _ => {}
        },
        Expr::Guarded(inner, guard) => {
            collect_mode_literals(inner, modes);
            collect_mode_literals(guard, modes);
        }
        Expr::After(inner, _) => collect_mode_literals(inner, modes),
        _ => {}
    }
}

fn mode_literal_from_equality(lhs: &Expr, rhs: &Expr) -> Option<String> {
    let lhs_is_mode = matches!(lhs, Expr::Ident(name) if name == "mode");
    let rhs_is_mode = matches!(rhs, Expr::Ident(name) if name == "mode");
    if lhs_is_mode {
        return literal_string(rhs);
    }
    if rhs_is_mode {
        return literal_string(lhs);
    }
    None
}

fn literal_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn parse_stage_claims(_stmts: &[Stmt]) -> Vec<UnitClaim> {
    // Annotations were removed from the AST; claims are no longer
    // extracted from inline `@file`/`@tool` annotations.
    // Known stages get default claims below via default_stage_claims().
    Vec::new()
}

/// Default claims for well-known stages whose resource usage is known
/// structurally but not yet expressible in DSL stage bodies.
fn default_stage_claims(stage_name: &str) -> Vec<UnitClaim> {
    workflow_catalog_data()
        .unwrap_or_else(|error| panic!("failed to load workflow catalog DSL data: {error}"))
        .default_stage_claims
        .get(stage_name)
        .cloned()
        .unwrap_or_default()
}

fn workflow_file_path(file: &str) -> PathBuf {
    workflows_root().join(file)
}

fn workflows_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../dsl/workflows")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_mode_parser_extracts_mode_literals() {
        let when = Expr::BinOp(
            Box::new(Expr::BinOp(
                Box::new(Expr::Ident("mode".to_string())),
                daglang_syntax::ast::BinOp::Eq,
                Box::new(Expr::Literal(Literal::String("gist".to_string()))),
            )),
            daglang_syntax::ast::BinOp::Or,
            Box::new(Expr::BinOp(
                Box::new(Expr::Ident("mode".to_string())),
                daglang_syntax::ast::BinOp::Eq,
                Box::new(Expr::Literal(Literal::String("gist-recent".to_string()))),
            )),
        );
        let modes = parse_stage_modes(Some(&when));
        assert!(modes.contains("gist"));
        assert!(modes.contains("gist-recent"));
    }

    #[test]
    fn process_registry_derivation_includes_core_workflows() {
        let registry = build_process_unit_registry().expect("derive registry");
        assert!(registry.contains(&ProcessUnitRef::new("ci", "ci.codegen")));
        assert!(registry.contains(&ProcessUnitRef::new("test_all", "test_all.codegen")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.gist_create")));
        assert!(registry.contains(&compilation_ref()));
        assert!(registry.contains(&codegen_ref()));
    }
}
