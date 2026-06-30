// Two distinct concerns hosted here (DESIGN §3 — distinct authorities, split migration track):
//
// CONCERN A — shape/transport/policy scan (dead_param, policy_leak, transport_fusion, argv
// projection): authority = `src/v2/lens/extdeps_shape_transport_policy.dag`; floor coverage =
// `src/v2/test/claim/extdeps_shape_transport_policy/**` (extensive, on auto-discovery floor).
// DISSOLUTION (same Chunk-D class as inert_carrier / non_fold_residue): when the interpreter
// consumes .dag lens tables directly instead of calling host-fed text scans — gunbc#5364. Until
// then the host functions below are the only callers of the .dag lens builtin seam.
//
// CONCERN B — external_authority anchor checking (anchor_kind/scheme/locator, clean_tree_holds,
// roster counts, shadow-mask): authority = `dsl/extdeps/external_authority` + transport gate.
// DISSOLUTION: separate migration to the external_authority host surface (not cli_run.rs). The
// two concerns migrate on DIFFERENT schedules; track them independently.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::v1_compiler_emit::effective_operation_transport;
use crate::v1_compiler_emit_core_support::is_data_def_item;
use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{
    build_newline_index, field_init_node_name_at, field_init_node_value, is_rest_transport,
    param_node_name_at, transport_request_body, ExprData, LiteralValue, Node,
};

fn resolve_extdeps_path(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_file() {
        return candidate.to_path_buf();
    }
    let rooted = crate::cli_run::workspace_root().join(path);
    if rooted.is_file() {
        return rooted;
    }
    panic!("extdeps shape/transport/policy projection: file not found: {path}");
}

fn argv_expr_token(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => value.clone(),
            other => format!("{other:?}"),
        },
        ExprData::ExprVar { .. } => {
            let name = crate::v1_std_core::expr_var_name_at(node.clone(), source_indices.clone());
            if name.is_empty() {
                node.name.clone()
            } else {
                format!("{{{name}}}")
            }
        }
        ExprData::ExprStringInterp => node
            .children
            .iter()
            .map(|child| match child.expr_data.as_ref() {
                ExprData::ExprLiteral { value } => match value.as_ref() {
                    LiteralValue::LitStr { value } => value.clone(),
                    _ => String::new(),
                },
                ExprData::ExprVar { .. } => {
                    let name =
                        crate::v1_std_core::expr_var_name_at(child.clone(), source_indices.clone());
                    if name.is_empty() {
                        child.name.clone()
                    } else {
                        format!("{{{name}}}")
                    }
                }
                _ => String::new(),
            })
            .collect(),
        _ => String::new(),
    }
}

fn argv_token_references_param(token: &str, param_name: &str) -> bool {
    token == param_name || token.contains(&format!("{{{param_name}}}"))
}

fn input_param_is_transport_bound(param_name: &str) -> bool {
    param_name == "env"
}

fn dead_param_count_for_operation_node(
    op_node: &Rc<Node>,
    transport: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> i64 {
    let eff = effective_operation_transport(op_node.clone(), transport.clone());
    let argv_tokens: Vec<String> = eff
        .children
        .iter()
        .map(|arg| argv_expr_token(arg, source_indices))
        .collect();

    let mut count = 0i64;
    for param in op_node.params.iter() {
        let name = param_node_name_at(param.clone(), source_indices.clone());
        if input_param_is_transport_bound(&name) {
            continue;
        }
        if argv_tokens
            .iter()
            .any(|token| argv_token_references_param(token, &name))
        {
            continue;
        }
        count += 1;
    }
    count
}

fn parse_module_items(
    path: &str,
) -> (
    Rc<Vec<Rc<Node>>>,
    Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    let resolved = resolve_extdeps_path(path);
    let path_str = resolved.to_string_lossy();
    let content = std::fs::read_to_string(&resolved).unwrap_or_else(|e| {
        panic!("extdeps shape/transport/policy projection: failed to read {path_str}: {e}")
    });
    let filename = resolved
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let tokens = tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.to_string(), source_index);
    let source_indices = Rc::new(source_indices);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "extdeps shape/transport/policy projection: parse error in {path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .expect("extdeps shape/transport/policy projection: missing module");
    (module.children.clone(), source_indices)
}

pub fn shell_argv_nodes_for_operation(
    path: String,
    service: String,
    operation: String,
) -> (
    Rc<Vec<Rc<Node>>>,
    Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    let (items, source_indices) = parse_module_items(&path);
    for item in items.iter() {
        if item.name != service {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name != operation {
                continue;
            }
            let eff = effective_operation_transport(op.clone(), fallback_transport.clone());
            return (eff.children.clone(), source_indices);
        }
    }
    panic!("shell argv projection: operation {service}.{operation} not found in {path}");
}

pub fn dead_param_count_for_operation(path: String, service: String, operation: String) -> i64 {
    let (items, source_indices) = parse_module_items(&path);
    for item in items.iter() {
        if item.name != service {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name != operation {
                continue;
            }
            return dead_param_count_for_operation_node(op, &fallback_transport, &source_indices);
        }
    }
    panic!("extdeps argv projection: operation {service}.{operation} not found in {path}");
}

pub fn dead_param_count_for_path(path: String) -> i64 {
    let (items, source_indices) = parse_module_items(&path);
    let mut total = 0i64;
    for item in items.iter() {
        if item.name.is_empty() || item.children.is_empty() {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name.is_empty() {
                continue;
            }
            total += dead_param_count_for_operation_node(op, &fallback_transport, &source_indices);
        }
    }
    total
}

fn literal_string_value(node: &Rc<Node>) -> Option<String> {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => Some(value.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn argv_token_is_consumer_policy_literal(token: &str) -> bool {
    if token.contains('{') && token.contains('}') {
        return false;
    }
    if token == "diff" || token == "--name-only" || token == "git" || token == "..." {
        return false;
    }
    token == "--all-targets"
        || token == "--all"
        || token == "--check"
        || token == "--workspace"
        || token == "--no-deps"
        || token == "-D"
        || token == "warnings"
        || token.contains("origin/")
        || token.contains("...")
}

fn data_literal_is_embedded_policy_literal(value: &str) -> bool {
    if value.contains(' ') {
        true
    } else {
        argv_token_is_consumer_policy_literal(value)
    }
}

fn embedded_policy_literal_count_for_data_node(
    data_node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> i64 {
    let Some(body) = data_node.body.as_ref() else {
        return 0;
    };
    if !matches!(body.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        return 0;
    }
    let mut count = 0i64;
    for field_init in body.children.iter() {
        let _field_name = field_init_node_name_at(field_init.clone(), source_indices.clone());
        let value_node = field_init_node_value(field_init.clone());
        if let Some(literal) = literal_string_value(&value_node) {
            if data_literal_is_embedded_policy_literal(&literal) {
                count += 1;
            }
        }
    }
    count
}

pub fn embedded_policy_literal_count_for_path(path: String) -> i64 {
    let (items, source_indices) = parse_module_items(&path);
    let mut total = 0i64;
    for item in items.iter() {
        if is_data_def_item(item.clone()) && !item.name.is_empty() {
            total += embedded_policy_literal_count_for_data_node(item, &source_indices);
        }
    }
    total
}

pub fn qualified_name_resolves_in_derived_module_set(qn: &crate::v1_interpreter::Value) -> bool {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    !module_path.is_empty()
        && crate::cli_run::build_module_path_index_from_witness_roots().contains_key(&module_path)
}

pub fn dead_param_count_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
    service: String,
    operation: String,
) -> i64 {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    dead_param_count_for_module_path(module_path, service, operation)
}

pub fn embedded_policy_literal_count_for_qualified_name(qn: &crate::v1_interpreter::Value) -> i64 {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    embedded_policy_literal_count_for_module_path(module_path)
}

fn module_source_nickname_literal_count_in_node(
    node: &Rc<Node>,
    real_paths: &HashSet<String>,
) -> i64 {
    let mut count = 0i64;
    if let Some(lit) = literal_string_value(node) {
        if real_paths.contains(&lit) {
            count += 1;
        }
    }
    if let Some(body) = node.body.as_ref() {
        count += module_source_nickname_literal_count_in_node(body, real_paths);
    }
    for child in node.children.iter() {
        count += module_source_nickname_literal_count_in_node(child, real_paths);
    }
    for param in node.params.iter() {
        count += module_source_nickname_literal_count_in_node(param, real_paths);
    }
    if let Some(type_annotation) = node.type_annotation.as_ref() {
        count += module_source_nickname_literal_count_in_node(type_annotation, real_paths);
    }
    count
}

pub fn module_source_nickname_literal_count_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
) -> i64 {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    let path = source_path_for_module_path(&module_path);
    let index = crate::cli_run::build_module_path_index_from_witness_roots();
    let real_paths: HashSet<String> = index.into_values().collect();
    let (items, _) = parse_module_items(&path);
    let mut total = 0i64;
    for item in items.iter() {
        total += module_source_nickname_literal_count_in_node(item, &real_paths);
    }
    total
}

pub fn policy_leak_count_for_qualified_name(qn: &crate::v1_interpreter::Value) -> i64 {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    policy_leak_count_for_module_path(module_path)
}

pub fn transport_fusion_fork_count_for_qualified_name(qn: &crate::v1_interpreter::Value) -> i64 {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    transport_fusion_fork_count_for_module_path(module_path)
}

pub fn gist_create_declares_filename_input_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
) -> bool {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    gist_create_declares_filename_input(module_path)
}

fn source_path_for_module_path(module_path: &str) -> String {
    crate::cli_run::source_path_for_module_path(module_path.to_string())
}

pub fn dead_param_count_for_module_path(
    module_path: String,
    service: String,
    operation: String,
) -> i64 {
    dead_param_count_for_operation(
        source_path_for_module_path(&module_path),
        service,
        operation,
    )
}

pub fn dead_param_count_for_module_path_file(module_path: String) -> i64 {
    dead_param_count_for_path(source_path_for_module_path(&module_path))
}

pub fn embedded_policy_literal_count_for_module_path(module_path: String) -> i64 {
    embedded_policy_literal_count_for_path(source_path_for_module_path(&module_path))
}

fn policy_leak_count_for_parsed_module(
    items: &Rc<Vec<Rc<Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> i64 {
    let mut count = 0i64;
    for item in items.iter() {
        if item.name.is_empty() || item.children.is_empty() {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name.is_empty() {
                continue;
            }
            let eff = effective_operation_transport(op.clone(), fallback_transport.clone());
            for arg in eff.children.iter() {
                let token = argv_expr_token(arg, source_indices);
                if argv_token_is_consumer_policy_literal(&token) {
                    count += 1;
                }
            }
        }
    }
    count
}

pub fn policy_leak_count_for_module_path(module_path: String) -> i64 {
    let path = source_path_for_module_path(&module_path);
    let (items, source_indices) = parse_module_items(&path);
    policy_leak_count_for_parsed_module(&items, &source_indices)
}

fn transport_fusion_fork_count_for_parsed_module(items: &Rc<Vec<Rc<Node>>>) -> i64 {
    let mut service_names: Vec<String> = Vec::new();
    for item in items.iter() {
        if item.name.is_empty() || item.children.is_empty() {
            continue;
        }
        service_names.push(item.name.clone());
    }
    let has_oauth_google = service_names.iter().any(|s| s == "oauth2.Google");
    let has_shell_oauth = service_names.iter().any(|s| s == "shell.OAuth2");
    if has_oauth_google && has_shell_oauth {
        1
    } else {
        0
    }
}

pub fn transport_fusion_fork_count_for_module_path(module_path: String) -> i64 {
    let path = source_path_for_module_path(&module_path);
    let (items, _) = parse_module_items(&path);
    transport_fusion_fork_count_for_parsed_module(&items)
}

pub fn gist_create_declares_filename_input(module_path: String) -> bool {
    let path = source_path_for_module_path(&module_path);
    let (items, source_indices) = parse_module_items(&path);
    gist_create_declares_filename_input_for_parsed_module(&items, &source_indices)
}

fn gist_create_declares_filename_input_for_parsed_module(
    items: &Rc<Vec<Rc<Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    for item in items.iter() {
        if item.name != "github.Gist" {
            continue;
        }
        for op in item.children.iter() {
            if op.name != "Create" {
                continue;
            }
            for param in op.params.iter() {
                let name = param_node_name_at(param.clone(), source_indices.clone());
                if name == "filename" {
                    return true;
                }
            }
        }
    }
    false
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

fn map_literal_keys_use_filename_placeholder(
    map_node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    if !matches!(map_node.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        return false;
    }
    if map_node.children.is_empty() {
        return false;
    }
    for entry in map_node.children.iter() {
        let key = field_init_node_name_at(entry.clone(), source_indices.clone());
        if !argv_token_references_param(&key, "filename") {
            return false;
        }
    }
    true
}

pub fn gist_create_files_keyed_by_filename_placeholder(module_path: String) -> bool {
    let path = source_path_for_module_path(&module_path);
    let (items, source_indices) = parse_module_items(&path);
    gist_create_files_keyed_by_filename_placeholder_for_parsed_module(&items, &source_indices)
}

fn gist_create_files_keyed_by_filename_placeholder_for_parsed_module(
    items: &Rc<Vec<Rc<Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    for item in items.iter() {
        if item.name != "github.Gist" {
            continue;
        }
        for op in item.children.iter() {
            if op.name != "Create" {
                continue;
            }
            let Some(transport) = op.transport.as_ref() else {
                return false;
            };
            if !is_rest_transport(transport.clone(), source_indices.clone()) {
                return false;
            }
            let Some(body) = transport_request_body(transport.clone(), source_indices.clone())
            else {
                return false;
            };
            let Some(files) = record_field_value(&body, "files", source_indices) else {
                return false;
            };
            return map_literal_keys_use_filename_placeholder(&files, source_indices);
        }
    }
    false
}

pub fn gist_create_files_keyed_by_filename_placeholder_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
) -> bool {
    let module_path = crate::cli_run::qualified_name_value_to_module_path(qn);
    gist_create_files_keyed_by_filename_placeholder(module_path)
}

// --- CONCERN B: external_authority anchor checking (see file header) ---

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
    crate::v1_std_core::authored_name_at(source_indices.clone(), node.clone())
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
        let variant = crate::v1_std_core::authored_name_at(source_indices.clone(), body.clone());
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
    let path = source_path_for_module_path(module_path);
    let (items, source_indices) = parse_module_items(&path);
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
        let path = ws.join("dsl/extdeps/external_authority_backfill_pending.txt");
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
            let (items, source_indices) = parse_module_items(&rel);
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

/// Cross-tree shadow plants live under `test.fixture.*` in `src/v2/` while the paired
/// anchor-bearing copy stays at `extdeps.fixture.*` in `dsl/`. Distinct module paths
/// avoid two-root compile panics; pairing restores shadow-mask detection.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_domain_module_source_nickname_literal_count_is_zero_after_qn_migration() {
        let path = crate::cli_run::source_path_for_module_path(
            "v2.test.extdeps_shape_transport_policy.coverage_domain_equivalence".to_string(),
        );
        let index = crate::cli_run::build_module_path_index_from_witness_roots();
        let real_paths: HashSet<String> = index.into_values().collect();
        let (items, _) = parse_module_items(&path);
        let mut total = 0i64;
        for item in items.iter() {
            total += module_source_nickname_literal_count_in_node(item, &real_paths);
        }
        assert_eq!(
            total, 0,
            "expected no nickname literals after migration, got {total}"
        );
    }

    #[test]
    fn local_red_module_source_nickname_literal_count_is_positive() {
        let path = crate::cli_run::source_path_for_module_path(
            "v2.test.extdeps_shape_transport_policy.lens_unit.module_source_nickname_literal_local_red"
                .to_string(),
        );
        let index = crate::cli_run::build_module_path_index_from_witness_roots();
        let real_paths: HashSet<String> = index.into_values().collect();
        let (items, _) = parse_module_items(&path);
        let mut total = 0i64;
        for item in items.iter() {
            total += module_source_nickname_literal_count_in_node(item, &real_paths);
        }
        assert!(
            total > 0,
            "expected nickname literals in local_red fixture, got {total}"
        );
    }

    #[test]
    fn gist_create_hardcoded_snapshot_md_red_under_perturbation() {
        let ws = crate::cli_run::workspace_root();
        let path = ws.join("target/test_gist_perturb_snapshot_md.dag");
        let content = r#"module extdeps.github.gists

service github.Gist {
  operation Create {
    input { description: String, filename: String, content: String, auth_token: Secret }
    transport rest {
      method: POST,
      path: "/gists",
      body: {
        description: description,
        public: false,
        files: { "snapshot.md": { content: content } }
      }
    }
  }
}
"#;
        std::fs::create_dir_all(ws.join("target")).ok();
        std::fs::write(&path, content).expect("write perturb fixture");
        let rel = path
            .strip_prefix(&ws)
            .expect("under workspace")
            .to_string_lossy()
            .replace('\\', "/");
        let (items, source_indices) = parse_module_items(&rel);
        assert!(
            !gist_create_files_keyed_by_filename_placeholder_for_parsed_module(
                &items,
                &source_indices
            ),
            "hardcoded snapshot.md must not pass files_keyed_by_filename projection"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn embedded_policy_predicate_discriminates_benign_env_var() {
        assert!(
            !data_literal_is_embedded_policy_literal("GOOGLE_APPLICATION_CREDENTIALS"),
            "adc_env_var benign control must not flag"
        );
        assert!(
            data_literal_is_embedded_policy_literal("gcloud auth print-access-token"),
            "fallback_auth_command policy literal must flag"
        );
    }

    #[test]
    fn external_authority_self_anchor_projects_file_with_fragment() {
        assert_eq!(
            external_authority_anchor_kind_for_module_path(
                "extdeps.external_authority".to_string()
            ),
            "present"
        );
        assert_eq!(
            external_authority_scheme_identity_for_module_path(
                "extdeps.external_authority".to_string()
            ),
            "File"
        );
        match project_external_authority_anchor("extdeps.external_authority") {
            ExternalAuthorityAnchorProjection::Present { locator, .. } => {
                assert_eq!(locator, "DESIGN.md#3-single-authority");
            }
            other => panic!("expected present file anchor, got {other:?}"),
        }
    }

    #[test]
    fn external_authority_bogus_scheme_fixture_projects_present_gopher() {
        let path = crate::cli_run::source_path_for_module_path(
            "extdeps.fixture.external_authority_bogus_scheme".to_string(),
        );
        let (items, source_indices) = parse_module_items(&path);
        let proj = read_external_authority_anchor_from_items(&items, &source_indices);
        assert_eq!(
            proj,
            ExternalAuthorityAnchorProjection::Present {
                scheme_identity: "Gopher".to_string(),
                locator: "example.invalid/spec".to_string(),
            }
        );
    }

    #[test]
    fn derived_extdeps_module_paths_covers_live_corpus() {
        let paths = derived_extdeps_module_paths();
        assert!(
            paths.len() > 150,
            "expected live extdeps roster, got {}",
            paths.len()
        );
        assert!(paths.iter().any(|p| p == "extdeps.external_authority"));
        assert!(paths.iter().any(|p| p == "extdeps.uri"));
        assert!(paths.iter().any(|p| p == "extdeps.git"));
    }

    #[test]
    fn machinery_exempt_and_roster_exclusion_paths_are_fixed() {
        assert!(is_machinery_exempt_for_module_path("extdeps.uri"));
        assert!(is_machinery_exempt_for_module_path(
            "extdeps.external_authority"
        ));
        assert!(is_clean_tree_roster_excluded_for_module_path(
            "extdeps.fixture.external_authority_bogus_scheme"
        ));
    }

    #[test]
    fn mock_corpus_excluded_but_real_sibling_still_in_roster() {
        assert!(
            is_clean_tree_roster_excluded_for_module_path("extdeps.git.mock_corpus"),
            "a *.mock_corpus hermetic replay fixture must be excluded from the anchor roster"
        );
        assert!(
            is_clean_tree_roster_excluded_for_module_path("extdeps.github.mock_corpus"),
            "*.mock_corpus exclusion must hold across services"
        );
        assert!(
            !is_clean_tree_roster_excluded_for_module_path("extdeps.git.git"),
            "the real sibling that the mock replays must stay in the roster (exclusion is not fail-open)"
        );
        assert!(
            !is_clean_tree_roster_excluded_for_module_path("extdeps.cloud.cloud"),
            "a real external-dependency module must stay in the roster"
        );
    }

    #[test]
    fn external_authority_shadow_mask_detector_has_teeth() {
        assert!(
            external_authority_anchor_shadow_masked_for_module_path(
                "test.fixture.external_authority_shadow_masked".to_string()
            ),
            "the two-tree shadow plant (anchor in dsl extdeps.fixture copy, none in index-resolved \
             src/v2 test.fixture copy) must read as masked -- proves the detector is not inert"
        );
        assert!(
            external_authority_live_shadow_mask_holds(),
            "no live extdeps module may carry an anchor only in a non-index-resolved shadow copy"
        );
    }

    #[test]
    fn external_authority_clean_https_fixture_projects_https_locator() {
        assert_eq!(
            external_authority_anchor_kind_for_module_path(
                "extdeps.fixture.external_authority_clean_https".to_string()
            ),
            "present"
        );
        assert_eq!(
            external_authority_scheme_identity_for_module_path(
                "extdeps.fixture.external_authority_clean_https".to_string()
            ),
            "Https"
        );
        assert_eq!(
            external_authority_locator_for_module_path(
                "extdeps.fixture.external_authority_clean_https".to_string()
            ),
            "yaml.org/spec/1.2.2/"
        );
    }
}
