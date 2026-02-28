use crate::ast::{Expr, Stmt, TypeExpr};

/// Returns true if the type expression is optional (`T?`), looking through refinement wrappers.
pub fn is_type_expr_optional(expr: &TypeExpr) -> bool {
    match expr {
        TypeExpr::Optional(_) => true,
        TypeExpr::Refined(inner, _) => is_type_expr_optional(inner),
        _ => false,
    }
}

pub fn type_expr_to_string(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Generic(name, args) => format!(
            "{name}<{}>",
            args.iter()
                .map(type_expr_to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpr::Optional(inner) => format!("{}?", type_expr_to_string(inner)),
        TypeExpr::Refined(inner, _) => type_expr_to_string(inner),
        TypeExpr::Record(_) => "Record".to_string(),
    }
}

pub fn canonical_resource_type_name(name: &str) -> String {
    let base_without_config = name.split('(').next().unwrap_or(name).trim();
    let base_without_annotations = base_without_config
        .split_whitespace()
        .next()
        .unwrap_or(base_without_config);
    // Strip generic parameters (e.g., `List<String>` → `List`).
    base_without_annotations
        .split('<')
        .next()
        .unwrap_or(base_without_annotations)
        .trim()
        .to_string()
}

pub fn resource_type_name(resource_type: &TypeExpr) -> String {
    match resource_type {
        TypeExpr::Named(name) | TypeExpr::Generic(name, _) => canonical_resource_type_name(name),
        TypeExpr::Optional(inner) | TypeExpr::Refined(inner, _) => resource_type_name(inner),
        TypeExpr::Record(_) => "Record".to_string(),
    }
}

/// Built-in pipe methods resolved by the evaluator, not as callable targets.
///
/// Single authoritative registry. To add a new pipe method:
/// 1. Add a variant here
/// 2. Add the string match in `from_str`
/// 3. Implement the evaluation in `eval.rs`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeMethod {
    // Collection → Collection
    Map,
    Filter,
    FilterMap,
    FlatMap,
    SortBy,
    Append,
    // Collection → Scalar
    Fold,
    Join,
    Count,
    Sum,
    First,
    Last,
    MaxBy,
    Any,
    All,
    Contains,
    // String methods
    StartsWith,
    EndsWith,
    Repeat,
    ReplaceSection,
    Chars,
    // Conversion methods
    ToBytes,
    ToJson,
    Hash,
}

impl PipeMethod {
    /// Parse a method name string into a PipeMethod, if it is a known built-in.
    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "map" => Some(Self::Map),
            "filter" => Some(Self::Filter),
            "filter_map" => Some(Self::FilterMap),
            "flat_map" => Some(Self::FlatMap),
            "sort_by" => Some(Self::SortBy),
            "append" => Some(Self::Append),
            "fold" => Some(Self::Fold),
            "join" => Some(Self::Join),
            "count" => Some(Self::Count),
            "sum" => Some(Self::Sum),
            "first" => Some(Self::First),
            "last" => Some(Self::Last),
            "max_by" => Some(Self::MaxBy),
            "any" => Some(Self::Any),
            "all" => Some(Self::All),
            "contains" => Some(Self::Contains),
            "starts_with" => Some(Self::StartsWith),
            "ends_with" => Some(Self::EndsWith),
            "repeat" => Some(Self::Repeat),
            "replace_section" => Some(Self::ReplaceSection),
            "chars" => Some(Self::Chars),
            "to_bytes" => Some(Self::ToBytes),
            "to_json" => Some(Self::ToJson),
            "hash" => Some(Self::Hash),
            _ => None,
        }
    }
}

pub fn should_track_call_name(name: &str) -> bool {
    !matches!(name, "<expr>" | "as" | "with" | "fn") && PipeMethod::from_str(name).is_none()
}

pub fn service_call_lookup_keys(call_path: &[String]) -> Option<[String; 3]> {
    if call_path.len() < 2 {
        return None;
    }
    let operation = call_path.last()?;
    let service_name = call_path[..call_path.len() - 1].join(".");
    let short_service = call_path[call_path.len() - 2].clone();
    Some([
        format!("{service_name}.{operation}"),
        format!("{short_service}.{operation}"),
        call_path.join("."),
    ])
}

pub fn walk_stmts(stmts: &[Stmt], visitor: &mut impl FnMut(&Expr)) {
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                walk_expr(expr, visitor);
            }
            Stmt::Node(ns) => {
                walk_expr(&ns.expr, visitor);
                if let Some(guard) = &ns.when_guard {
                    walk_expr(guard, visitor);
                }
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    walk_expr(expr, visitor);
                }
            }
        }
    }
}

pub fn walk_expr(expr: &Expr, visitor: &mut impl FnMut(&Expr)) {
    visitor(expr);
    match expr {
        Expr::Call(_, args) | Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                walk_expr(arg, visitor);
            }
        }
        Expr::FieldAccess(base, _) => walk_expr(base, visitor),
        Expr::BinOp(lhs, _, rhs) | Expr::Pipe(lhs, rhs) => {
            walk_expr(lhs, visitor);
            walk_expr(rhs, visitor);
        }
        Expr::UnaryOp(_, inner) | Expr::Lambda(_, inner) | Expr::After(inner, _) => {
            walk_expr(inner, visitor)
        }
        Expr::For(_, iterable, _, body) => {
            walk_expr(iterable, visitor);
            walk_expr(body, visitor);
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let crate::ast::StringPart::Expr(inner) = part {
                    walk_expr(inner, visitor);
                }
            }
        }
        Expr::Record(_, fields) | Expr::Return(fields) => {
            for (_, value) in fields {
                walk_expr(value, visitor);
            }
        }
        Expr::Match(scrutinee, arms) => {
            walk_expr(scrutinee, visitor);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    walk_expr(guard, visitor);
                }
                walk_expr(&arm.body, visitor);
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            walk_expr(cond, visitor);
            walk_expr(then_expr, visitor);
            if let Some(otherwise) = else_expr {
                walk_expr(otherwise, visitor);
            }
        }
        Expr::List(items) => {
            for item in items {
                walk_expr(item, visitor);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                walk_expr(key, visitor);
                walk_expr(value, visitor);
            }
        }
        Expr::Guarded(inner, guard) => {
            walk_expr(inner, visitor);
            walk_expr(guard, visitor);
        }
        Expr::Literal(_) | Expr::Ident(_) => {}
    }
}
