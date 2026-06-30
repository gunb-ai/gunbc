// Locked carrier shape for whole-tree declaration facts (neat-fox-279 / #5966 follow-up).
//
// `decl_facts(roots) -> List<DeclFact>` where each row is `{qualified_name, name, kind, node}`.
// STUB: parse-only walk via `parse_dag_file` — same substrate the additive host builtin will use.
// DISSOLUTION: swap `decl_facts_parse_only` body for the real host builtin when the additive
// follow-up merges; projection logic over `DeclFact` rows stays unchanged.

use std::path::PathBuf;
use std::rc::Rc;

use crate::cli_run::collect_dag_files_tolerant;
use crate::corpus_lex::{is_test_dag, repo_rel};
use crate::medium_structure_project::parse_dag_file;
use crate::module_path_index::workspace_root;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_std_core::{authored_name_at, Node};

#[derive(Debug, Clone)]
pub struct DeclFact {
    pub qualified_name: String,
    pub name: String,
    pub kind: ItemKind,
    pub node: Rc<Node>,
    /// Repo-relative file path (discriminator for site keys matching roster / audit TSV).
    pub rel_path: String,
}

fn extract_module_path(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return Some(trimmed["module ".len()..].trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

pub fn extract_module_path_from_content(content: &str) -> Option<String> {
    extract_module_path(content)
}

fn logical_qualified_name(module_name: &str, name: &str) -> String {
    let logical = module_name.strip_prefix("v2.").unwrap_or(module_name);
    if logical.is_empty() {
        name.to_string()
    } else {
        format!("{logical}.{name}")
    }
}

pub fn logical_qualified_name_from_module(module_name: &str, name: &str) -> String {
    logical_qualified_name(module_name, name)
}

fn corpus_dag_files(roots: &[String]) -> Vec<PathBuf> {
    let ws = workspace_root();
    let mut files = Vec::new();
    for root in roots {
        let root_path = ws.join(root);
        if root_path.is_dir() {
            collect_dag_files_tolerant(&root_path, &mut files);
        }
    }
    files.sort();
    files
}

pub fn corpus_dag_files_for_roots(roots: &[String]) -> Vec<PathBuf> {
    corpus_dag_files(roots)
}

/// Parse-only stub for the locked `decl_facts(roots)` primitive.
///
/// NOTE: materializes one row per declaration — use the streaming audit walk for
/// whole-corpus host builtins (witness eval); reserve this for consumers that need
/// the full carrier list.
pub fn decl_facts_parse_only(roots: &[String]) -> Vec<DeclFact> {
    let mut out = Vec::new();
    for file in corpus_dag_files(roots) {
        let rel = repo_rel(&file);
        if is_test_dag(&rel) {
            continue;
        }
        let content = std::fs::read_to_string(&file).ok();
        let module_path = content
            .as_ref()
            .and_then(|c| extract_module_path(c))
            .unwrap_or_default();
        let Some(parsed) = parse_dag_file(&file) else {
            continue;
        };
        let si = parsed.source_indices;
        for item in parsed.items.iter() {
            let name = authored_name_at(si.clone(), item.clone());
            if name.is_empty() {
                continue;
            }
            let kind = item_kind(item.clone());
            let qualified_name = logical_qualified_name(&module_path, &name);
            out.push(DeclFact {
                qualified_name,
                name,
                kind,
                node: item.clone(),
                rel_path: rel.clone(),
            });
        }
    }
    out.sort_by(|a, b| {
        (&a.rel_path, &a.name, format!("{:?}", a.kind))
            .cmp(&(&b.rel_path, &b.name, format!("{:?}", b.kind)))
    });
    out
}

pub fn decl_facts_is_fn_like(kind: ItemKind) -> bool {
    matches!(kind, ItemKind::FnItem | ItemKind::FuncItem)
}

pub fn decl_facts_fn_items<'a>(facts: &'a [DeclFact]) -> impl Iterator<Item = &'a DeclFact> {
    facts.iter().filter(|f| decl_facts_is_fn_like(f.kind))
}

pub fn decl_facts_count(roots: &[String]) -> usize {
    decl_facts_parse_only(roots).len()
}

pub fn decl_facts_fn_item_count(roots: &[String]) -> usize {
    decl_facts_parse_only(roots)
        .iter()
        .filter(|f| decl_facts_is_fn_like(f.kind))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_path_index::witness_layer_roots;

    #[test]
    fn decl_facts_stub_nonempty_over_witness_roots() {
        let facts = decl_facts_parse_only(&witness_layer_roots());
        assert!(facts.len() > 500, "expected whole-tree decl rows; got {}", facts.len());
        assert!(
            decl_facts_fn_items(&facts).count() > 100,
            "expected fn/func decl rows"
        );
    }

    #[test]
    fn decl_facts_rows_carry_locked_shape_fields() {
        let facts = decl_facts_parse_only(&witness_layer_roots());
        let sample = facts
            .iter()
            .find(|f| f.name == "eval_bind_node_eval")
            .expect("expected eval_bind_node_eval in corpus");
        assert!(sample.qualified_name.contains("eval_bind_node_eval"));
        assert!(decl_facts_is_fn_like(sample.kind));
        assert!(sample.node.body.is_some());
    }
}
