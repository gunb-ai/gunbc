//! DSL-backed workflow catalog + derivation helpers.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use daglang_syntax::{
    ast::{Expr, Item, Literal, Stmt},
    parser,
};
use gunbc_ir::{Dag, Edge, Node, Port};

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

#[derive(Debug, Clone)]
pub(super) struct WorkflowVariantDef {
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub file: String,
    pub pipeline: String,
    pub mode: Option<String>,
    pub namespace: String,
    pub is_tool: bool,
}

static WORKFLOW_VARIANTS: OnceLock<Vec<WorkflowVariantDef>> = OnceLock::new();

fn workflow_variants() -> &'static [WorkflowVariantDef] {
    WORKFLOW_VARIANTS.get_or_init(|| {
        load_workflow_variants_from_dsl()
            .unwrap_or_else(|error| panic!("failed to load workflow catalog DSL data: {error}"))
    })
}

#[allow(clippy::disallowed_methods)]
fn load_workflow_variants_from_dsl() -> Result<Vec<WorkflowVariantDef>, String> {
    let path = workflow_catalog_file_path();
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let parsed = parser::parse_with_file_diagnostics(&path, &source).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.render())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let mut raw = None;
    for item in parsed.items {
        if let Item::DataDef(def) = item.node {
            if def.name == "workflow_variants" {
                raw = Some(def.value);
                break;
            }
        }
    }
    let raw = raw.ok_or_else(|| {
        format!(
            "workflow catalog '{}' missing `data workflow_variants` declaration",
            path.display()
        )
    })?;
    parse_workflow_variants_expr(&raw)
}

fn parse_workflow_variants_expr(expr: &Expr) -> Result<Vec<WorkflowVariantDef>, String> {
    let Expr::List(items) = expr else {
        return Err("workflow_variants must be a list of records".to_string());
    };
    let mut variants = Vec::with_capacity(items.len());
    for (idx, item) in items.iter().enumerate() {
        let Expr::Record(_, fields) = item else {
            return Err(format!(
                "workflow_variants[{idx}] must be a record, found {:?}",
                item
            ));
        };
        variants.push(parse_workflow_variant_record(fields, idx)?);
    }
    Ok(variants)
}

fn parse_workflow_variant_record(
    fields: &[(String, Expr)],
    idx: usize,
) -> Result<WorkflowVariantDef, String> {
    Ok(WorkflowVariantDef {
        canonical_name: expect_string_field(fields, "canonical_name", idx)?,
        aliases: expect_string_list_field(fields, "aliases", idx)?,
        file: expect_string_field(fields, "file", idx)?,
        pipeline: expect_string_field(fields, "pipeline", idx)?,
        mode: expect_optional_string_field(fields, "mode", idx)?,
        namespace: expect_string_field(fields, "namespace", idx)?,
        is_tool: expect_bool_field(fields, "is_tool", idx)?,
    })
}

fn expect_field<'a>(fields: &'a [(String, Expr)], name: &str, idx: usize) -> Result<&'a Expr, String> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("workflow_variants[{idx}] missing required field '{name}'"))
}

fn expect_string_field(fields: &[(String, Expr)], name: &str, idx: usize) -> Result<String, String> {
    match expect_field(fields, name, idx)? {
        Expr::Literal(Literal::String(value)) => Ok(value.clone()),
        other => Err(format!(
            "workflow_variants[{idx}].{name} must be String, found {:?}",
            other
        )),
    }
}

fn expect_optional_string_field(
    fields: &[(String, Expr)],
    name: &str,
    idx: usize,
) -> Result<Option<String>, String> {
    match expect_field(fields, name, idx)? {
        Expr::Literal(Literal::String(value)) => Ok(Some(value.clone())),
        Expr::Literal(Literal::None) => Ok(None),
        Expr::Ident(ref id) if id == "None" || id == "none" => Ok(None),
        other => Err(format!(
            "workflow_variants[{idx}].{name} must be String or None, found {:?}",
            other
        )),
    }
}

fn expect_string_list_field(
    fields: &[(String, Expr)],
    name: &str,
    idx: usize,
) -> Result<Vec<String>, String> {
    let Expr::List(items) = expect_field(fields, name, idx)? else {
        return Err(format!(
            "workflow_variants[{idx}].{name} must be List<String>"
        ));
    };
    let mut values = Vec::with_capacity(items.len());
    for (alias_idx, item) in items.iter().enumerate() {
        match item {
            Expr::Literal(Literal::String(value)) => values.push(value.clone()),
            other => {
                return Err(format!(
                    "workflow_variants[{idx}].{name}[{alias_idx}] must be String, found {:?}",
                    other
                ))
            }
        }
    }
    Ok(values)
}

fn expect_bool_field(fields: &[(String, Expr)], name: &str, idx: usize) -> Result<bool, String> {
    match expect_field(fields, name, idx)? {
        Expr::Literal(Literal::Bool(value)) => Ok(*value),
        other => Err(format!(
            "workflow_variants[{idx}].{name} must be Bool, found {:?}",
            other
        )),
    }
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

pub(super) fn all_tool_workflow_names() -> Vec<&'static str> {
    workflow_variants()
        .iter()
        .filter(|variant| variant.is_tool)
        .map(|variant| variant.canonical_name.as_str())
        .collect()
}

pub(super) fn all_known_workflow_names() -> Vec<&'static str> {
    workflow_variants()
        .iter()
        .map(|variant| variant.canonical_name.as_str())
        .collect()
}

pub(super) fn resolve_workflow_variant(name: &str) -> Option<&'static WorkflowVariantDef> {
    let normalized = name.replace('_', "-");
    workflow_variants().iter().find(|variant| {
        variant.canonical_name == name
            || variant.canonical_name == normalized
            || variant.aliases.iter().any(|alias| alias == name || alias == &normalized)
    })
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

    for variant in workflow_variants() {
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

#[allow(clippy::disallowed_methods)]
fn load_workflow_templates() -> Result<HashMap<String, WorkflowTemplate>, String> {
    let mut templates = HashMap::new();
    for variant in workflow_variants() {
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
    match stage_name {
        // cargo build writes to target/
        "build_compile" => vec![UnitClaim::write("file:target"), UnitClaim::read("tool:cargo")],
        // cargo test reads target/ and test artifacts
        "test_run" => vec![UnitClaim::read("file:target"), UnitClaim::read("tool:cargo")],
        // clippy reads source + target
        "clippy_run" => vec![UnitClaim::read("file:target"), UnitClaim::read("tool:cargo")],
        _ => vec![],
    }
}

fn workflow_file_path(file: &str) -> PathBuf {
    workflows_root().join(file)
}

fn workflow_catalog_file_path() -> PathBuf {
    configs_root().join("workflow_catalog.dag")
}

fn workflows_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../dsl/workflows")
}

fn configs_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../dsl/config")
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
/// Derived from DSL workflow catalog. Panics on derivation failure.
pub fn default_process_unit_registry() -> ProcessUnitRegistry {
    build_process_unit_registry().unwrap_or_else(|error| {
        panic!("failed to derive process unit registry from DSL workflows: {error}")
    })
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
