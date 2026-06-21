//! Structural projection for `v2.lens.medium_structure_containment`.
//!
//! The `MediumStructureLeak` generalization of `RealizationVocabularyLeak` (DESIGN §5;
//! `docs/plans/emission-ingestion-inverse.md` §5.1). The §5 guard catches a module
//! *importing* a target AST it shouldn't; this catches the sin one rung earlier — a medium
//! whose structure has leaked out of a `Node` into a raw string. Two faces of one leak:
//!
//!   (a) emit-side    — a string LITERAL whose text contains a grounded medium syntax
//!                      marker (the GHA expression delimiter `${{`, cited from GitHub
//!                      Actions' "Evaluate expressions" spec), authored OUTSIDE the
//!                      realization edge. This reads AUTHORING source (the `.dag` the
//!                      developer wrote) for a grounded token — NOT a re-ingest of emitted
//!                      output (that would wall the realization edge it protects).
//!   (b1) check-side  — a string-op call (`string_contains`/`starts_with`/…) whose RECEIVER
//!                      (the `s:` subject, always the first arg) traces — directly or through
//!                      at most one intra-function `let` hop — to a CALL of a grounded emit
//!                      fn (`serialize_*`/`expected_ci_yml`/`serialize_expression`/…). The
//!                      receiver is identified as an emitted-medium value by call-target
//!                      PROVENANCE, NEVER by inspecting the string's contents — else the
//!                      detector would become the ad-hoc re-ingest it forbids. (The heavier
//!                      `(b2)` — receiver whose declared TYPE is `Medium<R>` — needs resolver
//!                      type info and is a named follow-on, not built here.)
//!
//! Policy — the grounded marker set, the emit-fn roster, the string-op set, and the scan
//! roots — is PASSED IN from the `.dag` single authority; this host does the mechanical
//! `Node` walk only. Realization-edge and exception-roster filtering is the `.dag` lens's
//! job (mirrors `layering_imports_project`: the host enumerates candidate facts, the lens
//! applies the predicate). The output is sorted, so a Wet run is byte-identical.
//!
//! dissolve-on: v2 self-host folds this projection over its own parsed module `Node` set;
//!   and per-medium, the leak dissolves once the medium is a `Medium<R>` `Node` emitted via
//!   grammar rows (then a check is a model walk or unwritable — the ① wall).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::cli_run::collect_dag_files_tolerant;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{
    arg_value, block_stmts, build_newline_index, expr_call_func_at, expr_var_name_at,
    let_binding_name_at, let_value, ExprData, LiteralValue, NewlineIndex, Node,
};

pub const FACE_INLINE_SYNTAX_LITERAL: &str = "InlineSyntaxLiteral";
pub const FACE_EMITTED_MEDIUM_STRING_OP: &str = "EmittedMediumStringOp";

/// One projected leak-candidate fact. `face` is the variant name of `MediumLeakFace`;
/// `detail` carries the grounded marker (emit-side) or `op@emit_fn` (check-side) so the
/// `.dag` lens can map it to a census medium and render the violation.
pub struct MediumStructureLeakRaw {
    pub path: String,
    pub face: &'static str,
    pub detail: String,
}

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

fn rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Parse one `.dag` file into its top-level items. Returns `None` on read/parse error
/// (a malformed sibling file must not abort a tree scan — fail-soft per file, like
/// `collect_dag_files_tolerant`).
fn parse_file(path: &Path) -> Option<(Rc<Vec<Rc<Node>>>, SourceIndices)> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    let tokens = tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices: SourceIndices = Rc::new(indices);
    let result = parse(tokens, source_indices.clone());
    if result.error.is_some() {
        return None;
    }
    let module = result.module.as_ref()?;
    Some((module.children.clone(), source_indices))
}

/// Function bodies (check-side: receiver-provenance needs per-function `let` scope).
fn function_bodies(items: &Rc<Vec<Rc<Node>>>) -> Vec<Rc<Node>> {
    let mut bodies = Vec::new();
    for item in items.iter() {
        if !matches!(
            item_kind(item.clone()),
            ItemKind::FuncItem | ItemKind::FnItem
        ) {
            continue;
        }
        if let Some(body) = item.body.as_ref() {
            bodies.push(body.clone());
        }
    }
    bodies
}

/// Every item value `Node` (emit-side: a `${{` literal can sit in a fn body OR a `data` decl
/// value — both are stored in `item.body`, NOT `item.children`).
fn item_value_bodies(items: &Rc<Vec<Rc<Node>>>) -> Vec<Rc<Node>> {
    items.iter().filter_map(|item| item.body.clone()).collect()
}

// ── detector (a): emit-side marker-in-literal ──────────────────────────────────

fn string_literal_text(node: &Rc<Node>) -> Option<String> {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value: s, .. } => Some(s.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn emit_side_walk(
    node: &Rc<Node>,
    markers: &[String],
    path: &str,
    out: &mut Vec<MediumStructureLeakRaw>,
) {
    if let Some(text) = string_literal_text(node) {
        for marker in markers {
            if !marker.is_empty() && text.contains(marker.as_str()) {
                out.push(MediumStructureLeakRaw {
                    path: path.to_string(),
                    face: FACE_INLINE_SYNTAX_LITERAL,
                    detail: marker.clone(),
                });
            }
        }
    }
    for child in node.children.iter() {
        emit_side_walk(child, markers, path, out);
    }
}

// ── detector (b1): check-side string-op over an emitted-medium receiver ──────────

/// Collect the `let` name → bound-value `Node` map for one block (intra-function scope,
/// one level — nested blocks are walked for their own lets by the caller's recursion).
fn collect_let_values(
    block: &Rc<Node>,
    bindings: &mut HashMap<String, Rc<Node>>,
    si: &SourceIndices,
) {
    for stmt in block_stmts(block.clone()).iter() {
        if let ExprData::ExprLet { .. } = stmt.expr_data.as_ref() {
            let name = let_binding_name_at(stmt.clone(), si.clone());
            bindings.insert(name, let_value(stmt.clone()));
        }
    }
}

/// The grounded-emit-fn name a receiver `Node` traces to: a direct emit-fn call, or a
/// var let-bound (≤1 hop) to an emit-fn call. `None` ⇒ not an emitted-medium value (so
/// NOT a leak — provenance gate, not a content guess).
fn receiver_emit_fn(
    receiver: &Rc<Node>,
    bindings: &HashMap<String, Rc<Node>>,
    emit_fns: &[String],
    si: &SourceIndices,
) -> Option<String> {
    if let ExprData::ExprCall { .. } = receiver.expr_data.as_ref() {
        let callee = expr_call_func_at(receiver.clone(), si.clone());
        if emit_fns.iter().any(|f| f == &callee) {
            return Some(callee);
        }
    }
    if let ExprData::ExprVar { .. } = receiver.expr_data.as_ref() {
        let name = expr_var_name_at(receiver.clone(), si.clone());
        if let Some(bound) = bindings.get(&name) {
            if let ExprData::ExprCall { .. } = bound.expr_data.as_ref() {
                let callee = expr_call_func_at(bound.clone(), si.clone());
                if emit_fns.iter().any(|f| f == &callee) {
                    return Some(callee);
                }
            }
        }
    }
    None
}

fn check_side_walk(
    node: &Rc<Node>,
    bindings: &HashMap<String, Rc<Node>>,
    emit_fns: &[String],
    string_ops: &[String],
    path: &str,
    si: &SourceIndices,
    out: &mut Vec<MediumStructureLeakRaw>,
) {
    if let ExprData::ExprCall { .. } = node.expr_data.as_ref() {
        let callee = expr_call_func_at(node.clone(), si.clone());
        if string_ops.iter().any(|o| o == &callee) {
            // Subject (`s:`) is the first arg of every std/text string op.
            if let Some(first) = node.children.first() {
                let receiver = arg_value(first.clone());
                if let Some(emit_fn) = receiver_emit_fn(&receiver, bindings, emit_fns, si) {
                    out.push(MediumStructureLeakRaw {
                        path: path.to_string(),
                        face: FACE_EMITTED_MEDIUM_STRING_OP,
                        detail: format!("{callee}@{emit_fn}"),
                    });
                }
            }
        }
    }
    for child in node.children.iter() {
        check_side_walk(child, bindings, emit_fns, string_ops, path, si, out);
    }
}

fn check_side_scan_function(
    body: &Rc<Node>,
    emit_fns: &[String],
    string_ops: &[String],
    path: &str,
    si: &SourceIndices,
    out: &mut Vec<MediumStructureLeakRaw>,
) {
    let mut bindings: HashMap<String, Rc<Node>> = HashMap::new();
    if let ExprData::ExprBlock { .. } = body.expr_data.as_ref() {
        collect_let_values(body, &mut bindings, si);
    }
    check_side_walk(body, &bindings, emit_fns, string_ops, path, si, out);
}

// ── tree scan (text-prefiltered: a marker/op absent from the raw text cannot be in the
//    parsed Node, so the prefilter never drops a real leak — it only skips clean files) ──

fn any_present(text: &str, needles: &[String]) -> bool {
    needles
        .iter()
        .any(|n| !n.is_empty() && text.contains(n.as_str()))
}

fn dag_files_sorted(root: &str) -> Vec<PathBuf> {
    let root_path = Path::new(root);
    if !root_path.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = Vec::new();
    collect_dag_files_tolerant(root_path, &mut files);
    files.sort();
    files
}

/// Project `MediumStructureLeak` candidate facts:
///   - emit-side over `emit_roots` (markers in string literals);
///   - check-side over `check_roots` (string ops over emitted-medium receivers).
/// All policy is supplied by the caller (the `.dag` single authority).
pub fn medium_structure_leak_facts(
    emit_roots: &[String],
    check_roots: &[String],
    markers: &[String],
    emit_fns: &[String],
    string_ops: &[String],
) -> Vec<MediumStructureLeakRaw> {
    let mut out: Vec<MediumStructureLeakRaw> = Vec::new();

    for root in emit_roots {
        for file in dag_files_sorted(root) {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            if !any_present(&content, markers) {
                continue;
            }
            if let Some((items, _si)) = parse_file(&file) {
                let rel = rel_path(&file);
                for body in item_value_bodies(&items) {
                    emit_side_walk(&body, markers, &rel, &mut out);
                }
            }
        }
    }

    for root in check_roots {
        for file in dag_files_sorted(root) {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            if !(any_present(&content, string_ops) && any_present(&content, emit_fns)) {
                continue;
            }
            if let Some((items, si)) = parse_file(&file) {
                let rel = rel_path(&file);
                for body in function_bodies(&items) {
                    check_side_scan_function(&body, emit_fns, string_ops, &rel, &si, &mut out);
                }
            }
        }
    }

    out.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.face.cmp(b.face))
            .then(a.detail.cmp(&b.detail))
    });
    out
}
