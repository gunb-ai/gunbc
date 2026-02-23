//! DSL-backed workflow catalog + derivation helpers.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use daglang_syntax::{ast::Item, parser};
use gunbc_ir::{AccessMode, Dag, Edge, Node, Port};

use super::capabilities::{
    CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
};
use super::process_registry::{
    claim_handle_type_id, ClaimId, ProcessUnitRef, ProcessUnitRegistry, ProcessUnitSpec, UnitClaim,
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
        aliases: &[],
        file: "gist.dag",
        pipeline: "gist",
        mode: Some("gist"),
        namespace: "gist",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "gist-snapshot",
        aliases: &["gist_snapshot"],
        file: "gist.dag",
        pipeline: "gist",
        mode: Some("gist-snapshot"),
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
        canonical_name: "dag-viz",
        aliases: &["dag_viz"],
        file: "dag_viz.dag",
        pipeline: "dag_viz",
        mode: Some("dag-viz"),
        namespace: "dag_viz",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "dag-viz-diff",
        aliases: &["dag_viz_diff"],
        file: "dag_viz.dag",
        pipeline: "dag_viz",
        mode: Some("dag-viz-diff"),
        namespace: "dag_viz",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "dag-viz-recent",
        aliases: &["dag_viz_recent"],
        file: "dag_viz.dag",
        pipeline: "dag_viz",
        mode: Some("dag-viz-recent"),
        namespace: "dag_viz",
        is_tool: true,
    },
    WorkflowVariantDef {
        canonical_name: "dag-snapshot",
        aliases: &["dag_snapshot"],
        file: "dag_viz.dag",
        pipeline: "dag_viz",
        mode: Some("dag-snapshot"),
        namespace: "dag_snapshot",
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
    WorkflowVariantDef {
        canonical_name: "sdlc",
        aliases: &[],
        file: "sdlc.dag",
        pipeline: "sdlc",
        mode: None,
        namespace: "sdlc",
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

#[derive(Debug, Clone)]
struct StageSection {
    attrs: String,
    body: String,
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

    let mut pipeline_stages: Option<Vec<(String, Vec<String>)>> = None;
    for item in parsed.items {
        if let Item::PipelineDef(def) = item.node {
            if def.name == pipeline_name {
                pipeline_stages = Some(
                    def.stages
                        .into_iter()
                        .map(|stage| (stage.name, stage.after))
                        .collect(),
                );
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

    let stage_names = stage_defs
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let sections = extract_stage_sections(&source, &stage_names)?;

    if stage_defs.len() != sections.len() {
        return Err(format!(
            "stage section mismatch while parsing '{}': {} defs vs {} sections",
            file,
            stage_defs.len(),
            sections.len()
        ));
    }

    let stages = stage_defs
        .into_iter()
        .zip(sections)
        .map(|((name, after), section)| StageTemplate {
            name,
            after,
            modes: parse_stage_modes(&section.attrs),
            claims: parse_stage_claims(&section.body),
        })
        .collect();

    Ok(WorkflowTemplate {
        pipeline_name: pipeline_name.to_string(),
        stages,
    })
}

fn extract_stage_sections(
    source: &str,
    stage_names: &[String],
) -> Result<Vec<StageSection>, String> {
    let mut sections = Vec::new();
    let mut cursor = 0usize;

    for stage_name in stage_names {
        let stage_start = find_stage_start(source, stage_name, cursor).ok_or_else(|| {
            format!(
                "could not locate `stage {}` in workflow source while deriving claims",
                stage_name
            )
        })?;

        let mut i = stage_start + "stage ".len() + stage_name.len();
        i = skip_ascii_whitespace(source, i);

        let mut attrs = String::new();
        if source.as_bytes().get(i) == Some(&b'[') {
            let (attr_inner, next) = extract_delimited_block(source, i, b'[', b']')?;
            attrs = attr_inner;
            i = skip_ascii_whitespace(source, next);
        }

        if source.as_bytes().get(i) != Some(&b'{') {
            return Err(format!(
                "expected '{{' after stage '{}' header while deriving claims",
                stage_name
            ));
        }

        let (body, next) = extract_delimited_block(source, i, b'{', b'}')?;
        sections.push(StageSection { attrs, body });
        cursor = next;
    }

    Ok(sections)
}

fn find_stage_start(source: &str, stage_name: &str, start: usize) -> Option<usize> {
    let target = format!("stage {}", stage_name);
    let mut cursor = start;
    while cursor < source.len() {
        let line_end = source[cursor..]
            .find('\n')
            .map(|offset| cursor + offset)
            .unwrap_or(source.len());
        let line = &source[cursor..line_end];
        let trimmed = line.trim_start();
        if trimmed.starts_with(&target) {
            let leading_ws = line.len().saturating_sub(trimmed.len());
            return Some(cursor + leading_ws);
        }
        if line_end == source.len() {
            break;
        }
        cursor = line_end + 1;
    }
    None
}

fn skip_ascii_whitespace(source: &str, mut index: usize) -> usize {
    while let Some(byte) = source.as_bytes().get(index) {
        if byte.is_ascii_whitespace() {
            index += 1;
        } else {
            break;
        }
    }
    index
}

fn extract_delimited_block(
    source: &str,
    start: usize,
    open: u8,
    close: u8,
) -> Result<(String, usize), String> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&open) {
        return Err(format!(
            "expected '{}' at byte offset {} while parsing workflow source",
            open as char, start
        ));
    }

    let mut depth = 0usize;
    let mut i = start;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(byte) = bytes.get(i) {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if *byte == b'"' {
            in_string = true;
            i += 1;
            continue;
        }

        if *byte == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while let Some(next) = bytes.get(i) {
                if *next == b'\n' {
                    break;
                }
                i += 1;
            }
            continue;
        }

        if *byte == open {
            depth += 1;
        } else if *byte == close {
            if depth == 0 {
                return Err("workflow parser encountered unmatched closing delimiter".to_string());
            }
            depth -= 1;
            if depth == 0 {
                let inner = source[start + 1..i].to_string();
                return Ok((inner, i + 1));
            }
        }

        i += 1;
    }

    Err("workflow parser reached EOF before closing delimiter".to_string())
}

fn parse_stage_modes(attrs: &str) -> BTreeSet<String> {
    if !attrs.contains("when") {
        return BTreeSet::new();
    }

    let mut modes = BTreeSet::new();
    let bytes = attrs.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i <= bytes.len() {
                let value = attrs[start..i].to_string();
                if !value.is_empty() {
                    modes.insert(value);
                }
            }
        }
        i += 1;
    }

    modes
}

fn parse_stage_claims(body: &str) -> Vec<UnitClaim> {
    let mut claims = Vec::new();

    for raw_line in body.lines() {
        let line = raw_line.trim();
        if !line.starts_with('@') {
            continue;
        }

        let Some(open_idx) = line.find('(') else {
            continue;
        };
        let Some(close_idx) = line.rfind(')') else {
            continue;
        };
        if close_idx <= open_idx {
            continue;
        }

        let name = line[1..open_idx].trim();
        let args = split_annotation_args(&line[open_idx + 1..close_idx]);
        let Some(claim) = claim_from_annotation(name, &args) else {
            continue;
        };
        claims.push(claim);
    }

    claims.sort_by(|left, right| {
        left.claim_id.cmp(&right.claim_id).then_with(|| {
            access_mode_rank(left.access_mode).cmp(&access_mode_rank(right.access_mode))
        })
    });
    claims.dedup();
    claims
}

fn split_annotation_args(raw: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for ch in raw.chars() {
        if in_string {
            current.push(ch);
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                current.push(ch);
            }
            ',' => {
                let value = current.trim();
                if !value.is_empty() {
                    args.push(value.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let value = current.trim();
    if !value.is_empty() {
        args.push(value.to_string());
    }

    args
}

fn claim_from_annotation(name: &str, args: &[String]) -> Option<UnitClaim> {
    let normalized = name.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "file" | "tool" | "ledger" | "network" | "credential" => {
            if args.len() < 2 {
                return None;
            }
            let mode = parse_access_mode(&args[0])?;
            let target = unquote(&args[1]);
            let claim_id = compose_claim_id(&normalized, &target);
            Some(UnitClaim::new(ClaimId::new(claim_id), mode))
        }
        "claim" => {
            if args.len() < 2 {
                return None;
            }
            let mode = parse_access_mode(&args[0])?;
            let claim_id = unquote(&args[1]);
            Some(UnitClaim::new(ClaimId::new(claim_id), mode))
        }
        _ => None,
    }
}

fn parse_access_mode(raw: &str) -> Option<AccessMode> {
    let normalized = raw.trim().trim_matches('"').to_ascii_uppercase();
    match normalized.as_str() {
        "READ" => Some(AccessMode::Read),
        "WRITE" => Some(AccessMode::Write),
        "EXCLUSIVE" => Some(AccessMode::Exclusive),
        _ => None,
    }
}

fn compose_claim_id(kind: &str, target: &str) -> String {
    if target.starts_with(&format!("{kind}:")) {
        target.to_string()
    } else {
        format!("{kind}:{target}")
    }
}

fn unquote(raw: &str) -> String {
    raw.trim().trim_matches('"').to_string()
}

fn access_mode_rank(mode: AccessMode) -> u8 {
    match mode {
        AccessMode::Read => 0,
        AccessMode::Write => 1,
        AccessMode::Exclusive => 2,
    }
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
    fn claim_parser_handles_file_and_network_annotations() {
        let body = r#"
            @file(WRITE, "workspace")
            @network(READ, "github")
        "#;
        let claims = parse_stage_claims(body);
        assert_eq!(claims.len(), 2);
        assert!(claims
            .iter()
            .any(|claim| claim.claim_id.0 == "file:workspace"
                && claim.access_mode == AccessMode::Write));
        assert!(claims
            .iter()
            .any(|claim| claim.claim_id.0 == "network:github"
                && claim.access_mode == AccessMode::Read));
    }

    #[test]
    fn stage_mode_parser_extracts_mode_literals() {
        let attrs = r#"
            after codegen_ensure,
            when mode == "gist" || mode == "gist-recent"
        "#;
        let modes = parse_stage_modes(attrs);
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
