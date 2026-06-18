//! Node-tree projection for `v2.lens.extdeps_shape_transport_policy`.
//! Parses extdeps `.dag` modules: dead input params per operation + embedded policy literals in data rows.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::v1_compiler_emit::effective_operation_transport;
use crate::v1_compiler_emit_core_support::is_data_def_item;
use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{
    build_newline_index, field_init_node_name_at, field_init_node_value, param_node_name_at,
    ExprData, LiteralValue, Node,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn resolve_extdeps_path(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_file() {
        return candidate.to_path_buf();
    }
    let rooted = workspace_root().join(path);
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

// dissolve-on: v2.lens.extdeps_shape_transport_policy.extdeps_input_param_is_transport_bound
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

/// Count dead input params for one `(service, operation)` in a parsed extdeps file.
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

/// Whole-file dead-param count across all service operations with shell argv.
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

// dissolve-on: v2.lens.extdeps_shape_transport_policy.extdeps_argv_token_is_consumer_policy_literal
fn argv_token_is_consumer_policy_literal(token: &str) -> bool {
    if token.contains('{') && token.contains('}') {
        return false;
    }
    if token == "diff"
        || token == "--name-only"
        || token == "git"
        || token == "--no-deps"
        || token == "..."
    {
        return false;
    }
    token == "--all-targets"
        || token == "--all"
        || token == "--check"
        || token == "--workspace"
        || token == "-D"
        || token == "warnings"
        || token.contains("origin/")
        || token.contains("...")
}

// dissolve-on: v2.lens.extdeps_shape_transport_policy.extdeps_data_literal_is_embedded_policy_literal
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

/// Count embedded transport/policy literals across all `data` rows in an extdeps file.
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

/// QualifiedName is in the derived module set (build_module_index); path never surfaces to .dag.
pub fn qualified_name_resolves_in_derived_module_set(qn: &crate::v1_interpreter::Value) -> bool {
    let module_path = crate::module_path_index::qualified_name_value_to_module_path(qn);
    !module_path.is_empty()
        && crate::module_path_index::build_module_path_index().contains_key(&module_path)
}

pub fn dead_param_count_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
    service: String,
    operation: String,
) -> i64 {
    let module_path = crate::module_path_index::qualified_name_value_to_module_path(qn);
    dead_param_count_for_module_path(module_path, service, operation)
}

pub fn embedded_policy_literal_count_for_qualified_name(qn: &crate::v1_interpreter::Value) -> i64 {
    let module_path = crate::module_path_index::qualified_name_value_to_module_path(qn);
    embedded_policy_literal_count_for_module_path(module_path)
}

pub fn policy_leak_count_for_qualified_name(qn: &crate::v1_interpreter::Value) -> i64 {
    let module_path = crate::module_path_index::qualified_name_value_to_module_path(qn);
    policy_leak_count_for_module_path(module_path)
}

pub fn transport_fusion_fork_count_for_qualified_name(qn: &crate::v1_interpreter::Value) -> i64 {
    let module_path = crate::module_path_index::qualified_name_value_to_module_path(qn);
    transport_fusion_fork_count_for_module_path(module_path)
}

pub fn gist_create_declares_filename_input_for_qualified_name(
    qn: &crate::v1_interpreter::Value,
) -> bool {
    let module_path = crate::module_path_index::qualified_name_value_to_module_path(qn);
    gist_create_declares_filename_input(module_path)
}

fn source_path_for_module_path(module_path: &str) -> String {
    crate::module_path_index::source_path_for_module_path(module_path.to_string())
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

/// Structural argv projection — consumer-policy literals in operation argv.
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

/// Structural service-decl projection — transport-fusion fork across handlers.
pub fn transport_fusion_fork_count_for_module_path(module_path: String) -> i64 {
    let path = source_path_for_module_path(&module_path);
    let (items, _) = parse_module_items(&path);
    transport_fusion_fork_count_for_parsed_module(&items)
}

/// Gist Create op declares `filename` input (structural — not a filepath nickname check).
pub fn gist_create_declares_filename_input(module_path: String) -> bool {
    let path = source_path_for_module_path(&module_path);
    let (items, source_indices) = parse_module_items(&path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_clippy_dead_param_defused_after_list_argv_splice() {
        assert_eq!(
            dead_param_count_for_module_path(
                "extdeps.cargo_build".to_string(),
                "cargo.Build".to_string(),
                "Clippy".to_string(),
            ),
            0
        );
    }

    #[test]
    fn cargo_fmt_dead_param_defused_on_live_tree() {
        assert_eq!(
            dead_param_count_for_module_path(
                "extdeps.cargo_build".to_string(),
                "cargo.Build".to_string(),
                "Fmt".to_string(),
            ),
            0
        );
    }

    #[test]
    fn cargo_doc_dead_param_defused_on_live_tree() {
        assert_eq!(
            dead_param_count_for_module_path(
                "extdeps.cargo_build".to_string(),
                "cargo.Build".to_string(),
                "Doc".to_string(),
            ),
            0
        );
    }

    #[test]
    fn git_policy_leak_defused_by_module_path() {
        assert_eq!(
            policy_leak_count_for_module_path("extdeps.git".to_string()),
            0
        );
    }

    #[test]
    fn cargo_build_policy_leak_defused_by_module_path() {
        assert_eq!(
            policy_leak_count_for_module_path("extdeps.cargo_build".to_string()),
            0
        );
    }

    #[test]
    fn gcp_oauth_fusion_defused_by_module_path() {
        assert_eq!(
            transport_fusion_fork_count_for_module_path("extdeps.cloud.gcp.gcp".to_string()),
            0
        );
    }
}
