//! DSL-backed workflow catalog + derivation helpers.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use daglang_syntax::{
    ast::{Expr, Item, Literal, Stmt},
    parser,
};
use gunbc_ir::{Dag, Edge, Node, Port};

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

#[derive(Debug, Clone, Copy)]
pub(super) struct WorkflowVariantDef {
    pub canonical_name: &'static str,
    pub aliases: &'static [&'static str],
    pub file: &'static str,
    pub pipeline: &'static str,
    pub mode: Option<&'static str>,
    pub namespace: &'static str,
    pub is_tool: bool,
}

const WORKFLOW_VARIANTS: &[WorkflowVariantDef] = &[
    // Core planner workflows
    WorkflowVariantDef {
        canonical_name: "ci",
        aliases: &[],
        file: "ci.dag",
        pipeline: "ci",
        mode: None,
        namespace: "ci",
        is_tool: false,
    },
    WorkflowVariantDef {
        canonical_name: "test-all",
        aliases: &["test_all"],
        file: "test_all.dag",
        pipeline: "test_all",
        mode: None,
        namespace: "test_all",
        is_tool: false,
    },
    // Tool workflow variants
    WorkflowVariantDef {
        canonical_name: "gist",
        aliases: &["gist_snapshot", "gist-snapshot"],
        file: "gist.dag",
        pipeline: "gist",
        mode: Some("gist"),
        namespace: "gist",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "gist-diff",
        aliases: &["gist_diff"],
        file: "gist.dag",
        pipeline: "gist",
        mode: Some("gist-diff"),
        namespace: "gist",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "gist-recent",
        aliases: &["gist_recent"],
        file: "gist.dag",
        pipeline: "gist",
        mode: Some("gist-recent"),
        namespace: "gist",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "bootstrap",
        aliases: &[],
        file: "bootstrap.dag",
        pipeline: "bootstrap",
        mode: None,
        namespace: "bootstrap",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "makegen",
        aliases: &[],
        file: "makegen.dag",
        pipeline: "makegen",
        mode: None,
        namespace: "makegen",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "pragma",
        aliases: &[],
        file: "pragma.dag",
        pipeline: "pragma",
        mode: None,
        namespace: "pragma",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "deps",
        aliases: &[],
        file: "deps.dag",
        pipeline: "deps",
        mode: None,
        namespace: "deps",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "build-all",
        aliases: &["build_all"],
        file: "build_all.dag",
        pipeline: "build_all",
        mode: None,
        namespace: "build_all",
        is_tool: true,
    },
];

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
    WORKFLOW_VARIANTS
        .iter()
        .filter(|variant| variant.is_tool)
        .map(|variant| variant.canonical_name)
        .collect()
}

pub(super) fn all_known_workflow_names() -> Vec<&'static str> {
    WORKFLOW_VARIANTS
        .iter()
        .map(|variant| variant.canonical_name)
        .collect()
}

pub(super) fn resolve_workflow_variant(name: &str) -> Option<&'static WorkflowVariantDef> {
    WORKFLOW_VARIANTS
        .iter()
        .find(|variant| variant.canonical_name == name || variant.aliases.contains(&name))
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
        .get(variant.file)
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
        .filter(|stage| stage_is_enabled(stage, variant.mode))
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

    Ok(WorkflowSpec::new(variant.canonical_name, dag, 1))
}

pub(super) fn build_process_unit_registry() -> Result<ProcessUnitRegistry, String> {
    let templates = load_workflow_templates()?;
    let mut registry = ProcessUnitRegistry::new();

    let mut compilation_claims: Option<Vec<UnitClaim>> = None;
    let mut codegen_claims: Option<Vec<UnitClaim>> = None;

    for variant in WORKFLOW_VARIANTS {
        let template = templates
            .get(variant.file)
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
            .filter(|stage| stage_is_enabled(stage, variant.mode))
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
            let spec = ProcessUnitSpec::new(process_ref.clone(), 1, stage.claims.clone());
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
            ProcessUnitRef::new(variant.namespace, node_id)
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
fn load_workflow_templates() -> Result<HashMap<&'static str, WorkflowTemplate>, String> {
    let mut templates = HashMap::new();
    for variant in WORKFLOW_VARIANTS {
        if templates.contains_key(variant.file) {
            continue;
        }
        templates.insert(
            variant.file,
            parse_workflow_template(variant.file, variant.pipeline)?,
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
    Vec::new()
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
