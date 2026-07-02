// SCAFFOLD — Concern B split from extdeps_shape_transport_policy_project.rs (lens E).
// Dissolution: migrate extdeps_external_authority.dag to a host-surface builtin that reads
// module structure directly, replacing these Rust projectors with a single structured-facts
// builtin (parallel to extdeps_shape_transport_policy_facts_for_qualified_name). Tracked as
// a separate migration lane from Concern A; gunbc#5364 successor.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::v1_compiler_emit_core_support::is_data_def_item;
use crate::v1_std_core::{
    authored_name_at, field_init_node_name_at, field_init_node_value, ExprData, LiteralValue, Node,
};

fn literal_string_value(node: &Rc<Node>) -> Option<String> {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => Some(value.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn record_field_value(
    record: &Rc<Node>,
    field_name: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<Node>> {
    if !matches!(record.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        return None;
    }
    for field_init in record.children.iter() {
        let name = field_init_node_name_at(field_init.clone(), source_indices.clone());
        if name == field_name {
            return Some(field_init_node_value(field_init.clone()));
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExternalAuthorityAnchorProjection {
    Absent,
    Present {
        scheme_identity: String,
        locator: String,
    },
}

fn uri_record_from_anchor_body(
    body: &Rc<Node>,
    variant: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<Node>> {
    match variant {
        "ExternalAuthority" | "StableAuthority" | "ExternalUri" => {
            record_field_value(body, "uri", source_indices)
        }
        _ => None,
    }
}

fn scheme_identity_from_value_node(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices.clone(), node.clone())
}

fn read_external_authority_anchor_from_items(
    items: &Rc<Vec<Rc<Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> ExternalAuthorityAnchorProjection {
    for item in items.iter() {
        if !is_data_def_item(item.clone()) || item.name != "extdeps_external_authority_anchor" {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            return ExternalAuthorityAnchorProjection::Absent;
        };
        let variant = authored_name_at(source_indices.clone(), body.clone());
        let Some(uri_node) = uri_record_from_anchor_body(body, variant.as_str(), source_indices)
        else {
            return ExternalAuthorityAnchorProjection::Absent;
        };
        let scheme = record_field_value(&uri_node, "scheme", source_indices)
            .map(|n| scheme_identity_from_value_node(&n, source_indices))
            .unwrap_or_default();
        let locator = record_field_value(&uri_node, "locator", source_indices)
            .and_then(|n| literal_string_value(&n))
            .unwrap_or_default();
        if scheme.is_empty() {
            return ExternalAuthorityAnchorProjection::Absent;
        }
        return ExternalAuthorityAnchorProjection::Present {
            scheme_identity: scheme,
            locator,
        };
    }
    ExternalAuthorityAnchorProjection::Absent
}

fn project_external_authority_anchor(module_path: &str) -> ExternalAuthorityAnchorProjection {
    let path = crate::cli_run::source_path_for_module_path(module_path.to_string());
    let (items, source_indices) = crate::cli_run::parse_extdeps_module_items(&path);
    read_external_authority_anchor_from_items(&items, &source_indices)
}

pub fn external_authority_anchor_kind_for_module_path(module_path: String) -> String {
    match project_external_authority_anchor(&module_path) {
        ExternalAuthorityAnchorProjection::Absent => "absent".to_string(),
        ExternalAuthorityAnchorProjection::Present { .. } => "present".to_string(),
    }
}

pub fn external_authority_scheme_identity_for_module_path(module_path: String) -> String {
    match project_external_authority_anchor(&module_path) {
        ExternalAuthorityAnchorProjection::Present {
            scheme_identity, ..
        } => scheme_identity,
        _ => String::new(),
    }
}

pub fn external_authority_anchor_kind_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
) -> String {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    external_authority_anchor_kind_for_module_path(module_path)
}

pub fn external_authority_scheme_identity_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
) -> String {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    external_authority_scheme_identity_for_module_path(module_path)
}

pub fn external_authority_locator_for_module_path(module_path: String) -> String {
    match project_external_authority_anchor(&module_path) {
        ExternalAuthorityAnchorProjection::Present { locator, .. } => locator,
        ExternalAuthorityAnchorProjection::Absent => String::new(),
    }
}

pub fn external_authority_locator_for_qualified_name(qn: &crate::v1_interpreter::Value) -> String {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    external_authority_locator_for_module_path(module_path)
}

pub fn derived_extdeps_module_paths() -> Vec<String> {
    let index = crate::cli_run::build_module_path_index_from_witness_roots();
    let mut paths: Vec<String> = index
        .keys()
        .filter(|k| k.starts_with("extdeps."))
        .cloned()
        .collect();
    paths.sort();
    paths
}

pub fn qualified_name_value_from_module_path(
    ctx: &crate::v1_interpreter::InterpContext,
    module_path: &str,
) -> crate::v1_interpreter::Value {
    crate::cli_run::qualified_name_value_from_dotted_string(ctx, module_path)
}

pub fn derived_extdeps_modules_value(
    ctx: &crate::v1_interpreter::InterpContext,
) -> crate::v1_interpreter::Value {
    use crate::v1_interpreter::list_value;
    let items: Vec<_> = derived_extdeps_module_paths()
        .iter()
        .map(|p| qualified_name_value_from_module_path(ctx, p))
        .collect();
    list_value(items)
}

pub fn backfill_pending_entries_value(
    ctx: &crate::v1_interpreter::InterpContext,
) -> crate::v1_interpreter::Value {
    use crate::v1_interpreter::list_value;
    let mut paths: Vec<String> = backfill_pending_module_paths().iter().cloned().collect();
    paths.sort();
    let items: Vec<_> = paths
        .iter()
        .map(|p| qualified_name_value_from_module_path(ctx, p))
        .collect();
    list_value(items)
}

fn backfill_pending_module_paths() -> &'static std::collections::HashSet<String> {
    use std::collections::HashSet;
    use std::sync::OnceLock;
    static PATHS: OnceLock<HashSet<String>> = OnceLock::new();
    PATHS.get_or_init(|| {
        let ws = crate::cli_run::workspace_root();
        let path = ws.join("dag/extdeps/external_authority_backfill_pending.txt");
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read backfill_pending snapshot {:?}: {e}", path));
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect()
    })
}

pub fn is_backfill_pending_for_module_path(module_path: &str) -> bool {
    backfill_pending_module_paths().contains(module_path)
}

pub fn is_backfill_pending_for_qualified_name(qn: &crate::v1_interpreter::Value) -> bool {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    is_backfill_pending_for_module_path(&module_path)
}

fn machinery_exempt_module_paths() -> &'static [&'static str] {
    &["extdeps.uri", "extdeps.external_authority"]
}

fn clean_tree_roster_exclusion_paths() -> &'static [&'static str] {
    &[
        "extdeps.fixture.external_authority_bogus_scheme",
        "extdeps.fixture.external_authority_missing",
        "extdeps.fixture.external_authority_clean_https_no_anchor",
        "extdeps.fixture.external_authority_file_anchor",
    ]
}

pub fn is_machinery_exempt_for_module_path(module_path: &str) -> bool {
    machinery_exempt_module_paths().contains(&module_path)
}

pub fn is_machinery_exempt_for_qualified_name(qn: &crate::v1_interpreter::Value) -> bool {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    is_machinery_exempt_for_module_path(&module_path)
}

pub fn is_clean_tree_roster_excluded_for_module_path(module_path: &str) -> bool {
    if module_path.starts_with("extdeps.fixture.") {
        return true;
    }
    if module_path.ends_with(".mock_corpus") {
        return true;
    }
    clean_tree_roster_exclusion_paths().contains(&module_path)
}

pub fn is_clean_tree_roster_excluded_for_qualified_name(qn: &crate::v1_interpreter::Value) -> bool {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    is_clean_tree_roster_excluded_for_module_path(&module_path)
}

fn live_violation_module_paths() -> Vec<String> {
    let backfill = backfill_pending_module_paths();
    let mut violations = Vec::new();
    for path in derived_extdeps_module_paths() {
        if is_clean_tree_roster_excluded_for_module_path(&path) {
            continue;
        }
        if is_machinery_exempt_for_module_path(&path) || backfill.contains(&path) {
            continue;
        }
        match project_external_authority_anchor(&path) {
            ExternalAuthorityAnchorProjection::Absent => violations.push(format!("missing:{path}")),
            ExternalAuthorityAnchorProjection::Present {
                scheme_identity, ..
            } if scheme_identity != "Http" && scheme_identity != "Https" => {
                violations.push(format!("non_external:{path}:{scheme_identity}"))
            }
            _ => {}
        }
    }
    violations
}

pub fn external_authority_live_clean_tree_holds() -> bool {
    live_violation_module_paths().is_empty()
}

pub fn external_authority_live_roster_module_count() -> i64 {
    derived_extdeps_module_paths()
        .into_iter()
        .filter(|path| !is_clean_tree_roster_excluded_for_module_path(path))
        .count() as i64
}

fn anchor_present_in_any_source_root(module_path: &str) -> bool {
    let ws = crate::cli_run::workspace_root();
    for root in crate::cli_run::default_source_roots() {
        let root_path = std::path::PathBuf::from(&root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        crate::cli_run::collect_dag_files_tolerant(&root_path, &mut files);
        for file in files {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            let declares = content.lines().find_map(|l| {
                l.trim()
                    .strip_prefix("module ")
                    .map(|m| m.trim().to_string())
            });
            if declares.as_deref() != Some(module_path) {
                continue;
            }
            let rel = file
                .strip_prefix(&ws)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| file.to_string_lossy().into_owned());
            let (items, source_indices) = crate::cli_run::parse_extdeps_module_items(&rel);
            if matches!(
                read_external_authority_anchor_from_items(&items, &source_indices),
                ExternalAuthorityAnchorProjection::Present { .. }
            ) {
                return true;
            }
        }
    }
    false
}

fn shadow_plant_paired_extdeps_module_path(module_path: &str) -> Option<String> {
    module_path
        .strip_prefix("test.fixture.")
        .map(|leaf| format!("extdeps.fixture.{leaf}"))
}

pub fn external_authority_anchor_shadow_masked_for_module_path(module_path: String) -> bool {
    match project_external_authority_anchor(&module_path) {
        ExternalAuthorityAnchorProjection::Present { .. } => false,
        ExternalAuthorityAnchorProjection::Absent => {
            if anchor_present_in_any_source_root(&module_path) {
                return true;
            }
            if let Some(extdeps_path) = shadow_plant_paired_extdeps_module_path(&module_path) {
                return anchor_present_in_any_source_root(&extdeps_path);
            }
            false
        }
    }
}

pub fn external_authority_anchor_shadow_masked_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
) -> bool {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    external_authority_anchor_shadow_masked_for_module_path(module_path)
}

pub fn external_authority_live_shadow_mask_holds() -> bool {
    for path in derived_extdeps_module_paths() {
        if is_clean_tree_roster_excluded_for_module_path(&path)
            || is_machinery_exempt_for_module_path(&path)
        {
            continue;
        }
        if external_authority_anchor_shadow_masked_for_module_path(path) {
            return false;
        }
    }
    true
}
