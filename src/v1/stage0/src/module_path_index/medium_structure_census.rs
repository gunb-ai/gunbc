use im_rc::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::cli_run::collect_dag_files_tolerant;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{
    arg_value, build_newline_index, expr_call_func_at, expr_var_name_at, let_binding_name_at,
    let_value, ExprData, LiteralValue, NewlineIndex, Node,
};

pub const FACE_INLINE_SYNTAX_LITERAL: &str = "InlineSyntaxLiteral";
pub const FACE_EMITTED_MEDIUM_STRING_OP: &str = "EmittedMediumStringOp";

pub struct MediumStructureLeakRaw {
    pub path: String,
    pub face: &'static str,
    pub detail: String,
}

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

fn rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Parse-only module items for one `.dag` file (no resolve). Shared substrate for
/// `decl_facts(roots)` (#5966) and emit-only corpus audits.
pub struct ParsedDagFile {
    pub items: Rc<Vec<Rc<Node>>>,
    pub source_indices: SourceIndices,
}

pub fn parse_dag_file(path: &Path) -> Option<ParsedDagFile> {
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
    Some(ParsedDagFile {
        items: module.children.clone(),
        source_indices,
    })
}

pub fn parse_file(path: &Path) -> Option<(Rc<Vec<Rc<Node>>>, SourceIndices)> {
    parse_dag_file(path).map(|parsed| (parsed.items, parsed.source_indices))
}

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

fn item_value_bodies(items: &Rc<Vec<Rc<Node>>>) -> Vec<Rc<Node>> {
    items.iter().filter_map(|item| item.body.clone()).collect()
}

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

fn collect_let_values(
    body: &Rc<Node>,
    bindings: &mut HashMap<String, Rc<Node>>,
    si: &SourceIndices,
) {
    if let ExprData::ExprLet { .. } = body.expr_data.as_ref() {
        let name = let_binding_name_at(body.clone(), si.clone());
        bindings.insert(name, let_value(body.clone()));
    }
    for child in body.children.iter() {
        collect_let_values(child, bindings, si);
    }
}

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
    collect_let_values(body, &mut bindings, si);
    check_side_walk(body, &bindings, emit_fns, string_ops, path, si, out);
}

fn is_string_literal_arg(
    val: &Rc<Node>,
    bindings: &HashMap<String, Rc<Node>>,
    si: &SourceIndices,
) -> bool {
    if string_literal_text(val).is_some() {
        return true;
    }
    if let ExprData::ExprVar { .. } = val.expr_data.as_ref() {
        let name = expr_var_name_at(val.clone(), si.clone());
        if let Some(bound) = bindings.get(&name) {
            return string_literal_text(bound).is_some();
        }
    }
    false
}

fn emit_side_provenance_walk(
    node: &Rc<Node>,
    bindings: &HashMap<String, Rc<Node>>,
    emit_fns: &[String],
    path: &str,
    si: &SourceIndices,
    out: &mut Vec<MediumStructureLeakRaw>,
) {
    if let ExprData::ExprCall { .. } = node.expr_data.as_ref() {
        let callee = expr_call_func_at(node.clone(), si.clone());
        if emit_fns.iter().any(|f| f == &callee) {
            let has_str_arg = node
                .children
                .iter()
                .any(|arg| is_string_literal_arg(&arg_value(arg.clone()), bindings, si));
            if has_str_arg {
                out.push(MediumStructureLeakRaw {
                    path: path.to_string(),
                    face: FACE_INLINE_SYNTAX_LITERAL,
                    detail: format!("provenance@{callee}"),
                });
            }
        }
    }
    for child in node.children.iter() {
        emit_side_provenance_walk(child, bindings, emit_fns, path, si, out);
    }
}

fn emit_side_provenance_scan_function(
    body: &Rc<Node>,
    emit_fns: &[String],
    path: &str,
    si: &SourceIndices,
    out: &mut Vec<MediumStructureLeakRaw>,
) {
    let mut bindings: HashMap<String, Rc<Node>> = HashMap::new();
    collect_let_values(body, &mut bindings, si);
    emit_side_provenance_walk(body, &bindings, emit_fns, path, si, out);
}

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

    // Emit-side provenance scan (fail-closed: string literal as arg to an emit fn)
    for root in emit_roots {
        for file in dag_files_sorted(root) {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            if !any_present(&content, emit_fns) {
                continue;
            }
            if let Some((items, si)) = parse_file(&file) {
                let rel = rel_path(&file);
                for body in function_bodies(&items) {
                    emit_side_provenance_scan_function(&body, emit_fns, &rel, &si, &mut out);
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
