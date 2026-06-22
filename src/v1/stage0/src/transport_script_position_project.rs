use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::v1_compiler_infer_items::item_kind;
use crate::v1_compiler_infer_items::ItemKind;
use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{
    arg_name_at, arg_value, authored_name_at, block_stmts, build_newline_index,
    expr_method_name_at, expr_var_name_at, field_access_base, field_access_field_at,
    let_binding_name_at, let_value, method_arg_nodes, method_receiver, ExprData, LiteralValue,
    Node,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportScriptArgShape {
    ComputedApplication = 0,
    BareStringLiteral = 1,
    LetBoundStringLiteral = 2,
    StringInterpLiteralsOnly = 3,
}

impl TransportScriptArgShape {
    pub fn is_literal_blob(self) -> bool {
        matches!(
            self,
            Self::BareStringLiteral | Self::LetBoundStringLiteral | Self::StringInterpLiteralsOnly
        )
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn resolve_dag_path(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_file() {
        return candidate.to_path_buf();
    }
    let rooted = workspace_root().join(path);
    if rooted.is_file() {
        return rooted;
    }
    panic!("transport_script_position projection: file not found: {path}");
}

fn parse_module_items(
    path: &str,
) -> (
    Rc<Vec<Rc<Node>>>,
    Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    let resolved = resolve_dag_path(path);
    let path_str = resolved.to_string_lossy();
    let content = std::fs::read_to_string(&resolved).unwrap_or_else(|e| {
        panic!("transport_script_position projection: failed to read {path_str}: {e}")
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
            "transport_script_position projection: parse error in {path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .expect("transport_script_position projection: missing module");
    (module.children.clone(), source_indices)
}

fn literal_string_value(node: &Rc<Node>) -> bool {
    matches!(
        node.expr_data.as_ref(),
        ExprData::ExprLiteral {
            value: lit,
            ..
        } if matches!(lit.as_ref(), LiteralValue::LitStr { .. })
    )
}

fn classify_script_arg(
    node: &Rc<Node>,
    let_literal_bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> TransportScriptArgShape {
    if literal_string_value(node) {
        return TransportScriptArgShape::BareStringLiteral;
    }
    match node.expr_data.as_ref() {
        ExprData::ExprStringInterp => {
            for child in node.children.iter() {
                match child.expr_data.as_ref() {
                    ExprData::ExprLiteral { value, .. } => {
                        if !matches!(value.as_ref(), LiteralValue::LitStr { .. }) {
                            return TransportScriptArgShape::ComputedApplication;
                        }
                    }
                    ExprData::ExprVar { .. } => {
                        let name = expr_var_name_at(child.clone(), source_indices.clone());
                        if !let_literal_bindings.get(&name).copied().unwrap_or(false) {
                            return TransportScriptArgShape::ComputedApplication;
                        }
                    }
                    _ => return TransportScriptArgShape::ComputedApplication,
                }
            }
            TransportScriptArgShape::StringInterpLiteralsOnly
        }
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), source_indices.clone());
            if let_literal_bindings.get(&name).copied().unwrap_or(false) {
                TransportScriptArgShape::LetBoundStringLiteral
            } else {
                TransportScriptArgShape::ComputedApplication
            }
        }
        _ => TransportScriptArgShape::ComputedApplication,
    }
}

fn is_shell_exec_run(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    match node.expr_data.as_ref() {
        ExprData::ExprMethodCall { .. } => {
            if expr_method_name_at(node.clone(), source_indices.clone()) != "Run" {
                return false;
            }
            let recv = method_receiver(node.clone());
            match recv.expr_data.as_ref() {
                ExprData::ExprFieldAccess { .. } => {
                    if field_access_field_at(recv.clone(), source_indices.clone()) != "Exec" {
                        return false;
                    }
                    let base = field_access_base(recv.clone());
                    match base.expr_data.as_ref() {
                        ExprData::ExprVar { .. } => {
                            expr_var_name_at(base.clone(), source_indices.clone()) == "shell"
                        }
                        _ => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn script_arg_node(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<Node>> {
    for arg in method_arg_nodes(node.clone()).iter() {
        if arg_name_at(arg.clone(), source_indices.clone()).as_deref() == Some("script") {
            return Some(arg_value(arg.clone()));
        }
    }
    None
}

fn binding_is_literal_shaped(
    node: &Rc<Node>,
    bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    classify_script_arg(node, bindings, source_indices).is_literal_blob()
}

fn collect_let_bindings_in_block(
    block: &Rc<Node>,
    bindings: &mut HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    for stmt in block_stmts(block.clone()).iter() {
        match stmt.expr_data.as_ref() {
            ExprData::ExprLet { .. } => {
                let name = let_binding_name_at(stmt.clone(), source_indices.clone());
                let val = let_value(stmt.clone());
                let literal_shaped = binding_is_literal_shaped(&val, bindings, source_indices);
                bindings.insert(name, literal_shaped);
            }
            _ => walk_expr(stmt, bindings, source_indices, &mut |_| {}),
        }
    }
}

fn walk_expr(
    node: &Rc<Node>,
    let_bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    on_run: &mut dyn FnMut(TransportScriptArgShape),
) {
    if is_shell_exec_run(node, source_indices) {
        if let Some(script_node) = script_arg_node(node, source_indices) {
            on_run(classify_script_arg(
                &script_node,
                let_bindings,
                source_indices,
            ));
        }
    }
    for child in node.children.iter() {
        walk_expr(child, let_bindings, source_indices, on_run);
    }
}

fn violation_count_for_function(
    body: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> i64 {
    let mut bindings = HashMap::new();
    if let ExprData::ExprBlock { .. } = body.expr_data.as_ref() {
        collect_let_bindings_in_block(body, &mut bindings, source_indices);
    }
    let mut count = 0i64;
    walk_expr(body, &bindings, source_indices, &mut |shape| {
        if shape.is_literal_blob() {
            count += 1;
        }
    });
    count
}

pub fn transport_script_literal_violation_count_for_path(path: String) -> i64 {
    let (items, source_indices) = parse_module_items(&path);
    let mut total = 0i64;
    for item in items.iter() {
        let kind = item_kind(item.clone());
        if !matches!(kind, ItemKind::FuncItem | ItemKind::FnItem) {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            continue;
        };
        total += violation_count_for_function(body, &source_indices);
    }
    total
}
