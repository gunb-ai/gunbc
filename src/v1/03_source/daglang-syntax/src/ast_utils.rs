use crate::ast::{Expr, ForBody, Stmt, TypeExpr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExprIdentity(pub usize);

/// Returns true if the type expression is optional (`T?`), looking through refinement wrappers.
pub fn is_type_expr_optional(expr: &TypeExpr) -> bool {
    match expr {
        TypeExpr::Optional(_) => true,
        TypeExpr::Refined(inner, _) => is_type_expr_optional(inner),
        _ => false,
    }
}

/// Returns true if the inner (unwrapped) type has the given name.
/// Sees through `Refined` and `Optional` wrappers.
pub fn is_named_type(expr: &TypeExpr, name: &str) -> bool {
    match expr {
        TypeExpr::Named(n) => n == name,
        TypeExpr::Refined(inner, _) | TypeExpr::Optional(inner) => is_named_type(inner, name),
        _ => false,
    }
}

/// Returns true if the type is `Secret` (possibly refined/optional).
pub fn is_secret_type(expr: &TypeExpr) -> bool {
    is_named_type(expr, "Secret")
}

/// Returns true if the type is `Bool` or `bool` (possibly refined/optional).
pub fn is_bool_type(expr: &TypeExpr) -> bool {
    match expr {
        TypeExpr::Named(n) => n == "Bool" || n == "bool",
        TypeExpr::Refined(inner, _) | TypeExpr::Optional(inner) => is_bool_type(inner),
        _ => false,
    }
}

/// Returns true if the type is `List<...>` (possibly refined/optional).
pub fn is_list_type(expr: &TypeExpr) -> bool {
    match expr {
        TypeExpr::Generic(name, _) => name == "List",
        TypeExpr::Refined(inner, _) | TypeExpr::Optional(inner) => is_list_type(inner),
        _ => false,
    }
}

/// Returns true if the type is `Map<String, String>` (possibly refined/optional).
pub fn is_map_string_string(expr: &TypeExpr) -> bool {
    match expr {
        TypeExpr::Generic(name, args) => {
            name == "Map"
                && args.len() == 2
                && matches!(&args[0], TypeExpr::Named(a) if a == "String")
                && matches!(&args[1], TypeExpr::Named(b) if b == "String")
        }
        TypeExpr::Refined(inner, _) | TypeExpr::Optional(inner) => is_map_string_string(inner),
        _ => false,
    }
}

/// Returns true if the type is a function type `fn(...)` (possibly refined/optional).
pub fn is_function_type(expr: &TypeExpr) -> bool {
    match expr {
        TypeExpr::Function(_, _) => true,
        TypeExpr::Refined(inner, _) | TypeExpr::Optional(inner) => is_function_type(inner),
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
        TypeExpr::AssociatedOutput(base) => format!("{base}.Output"),
        TypeExpr::Function(params, output) => format!(
            "fn({}) -> {}",
            params
                .iter()
                .map(type_expr_to_string)
                .collect::<Vec<_>>()
                .join(", "),
            type_expr_to_string(output)
        ),
        TypeExpr::Optional(inner) => format!("{}?", type_expr_to_string(inner)),
        TypeExpr::Refined(inner, _) => type_expr_to_string(inner),
        TypeExpr::Record(fields) => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|f| format!("{}: {}", f.name, type_expr_to_string(&f.ty)))
                .collect();
            format!("{{{}}}", field_strs.join(", "))
        }
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
        TypeExpr::AssociatedOutput(base) => format!("{base}.Output"),
        TypeExpr::Function(_, _) => "fn".to_string(),
        TypeExpr::Optional(inner) | TypeExpr::Refined(inner, _) => resource_type_name(inner),
        TypeExpr::Record(_) => "Record".to_string(),
    }
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
        Expr::BinOp(lhs, _, rhs) => {
            walk_expr(lhs, visitor);
            walk_expr(rhs, visitor);
        }
        Expr::UnaryOp(_, inner) | Expr::Lambda(_, inner) | Expr::After(inner, _) => {
            walk_expr(inner, visitor)
        }
        Expr::For(_, iterable, _, body) => {
            walk_expr(iterable, visitor);
            match body {
                ForBody::Expr(expr) => walk_expr(expr, visitor),
                ForBody::Block(stmts) => walk_stmts(stmts, visitor),
            }
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
        Expr::Block(stmts) => walk_stmts(stmts, visitor),
        Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

pub fn walk_stmts_with_expr_identities(
    stmts: &[Stmt],
    visitor: &mut impl FnMut(ExprIdentity, &Expr),
) {
    let mut next_identity = 0usize;
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                walk_expr_with_identity(expr, &mut next_identity, visitor);
            }
            Stmt::Node(ns) => {
                walk_expr_with_identity(&ns.expr, &mut next_identity, visitor);
                if let Some(guard) = &ns.when_guard {
                    walk_expr_with_identity(guard, &mut next_identity, visitor);
                }
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    walk_expr_with_identity(expr, &mut next_identity, visitor);
                }
            }
        }
    }
}

fn walk_expr_with_identity(
    expr: &Expr,
    next_identity: &mut usize,
    visitor: &mut impl FnMut(ExprIdentity, &Expr),
) {
    let expr_identity = ExprIdentity(*next_identity);
    *next_identity += 1;
    visitor(expr_identity, expr);
    match expr {
        Expr::Call(_, args) | Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                walk_expr_with_identity(arg, next_identity, visitor);
            }
        }
        Expr::FieldAccess(base, _) => walk_expr_with_identity(base, next_identity, visitor),
        Expr::BinOp(lhs, _, rhs) => {
            walk_expr_with_identity(lhs, next_identity, visitor);
            walk_expr_with_identity(rhs, next_identity, visitor);
        }
        Expr::UnaryOp(_, inner) | Expr::Lambda(_, inner) | Expr::After(inner, _) => {
            walk_expr_with_identity(inner, next_identity, visitor)
        }
        Expr::For(_, iterable, _, body) => {
            walk_expr_with_identity(iterable, next_identity, visitor);
            match body {
                ForBody::Expr(expr) => walk_expr_with_identity(expr, next_identity, visitor),
                ForBody::Block(stmts) => {
                    walk_stmts_with_expr_identities_in_place(stmts, next_identity, visitor)
                }
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let crate::ast::StringPart::Expr(inner) = part {
                    walk_expr_with_identity(inner, next_identity, visitor);
                }
            }
        }
        Expr::Record(_, fields) | Expr::Return(fields) => {
            for (_, value) in fields {
                walk_expr_with_identity(value, next_identity, visitor);
            }
        }
        Expr::Match(scrutinee, arms) => {
            walk_expr_with_identity(scrutinee, next_identity, visitor);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    walk_expr_with_identity(guard, next_identity, visitor);
                }
                walk_expr_with_identity(&arm.body, next_identity, visitor);
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            walk_expr_with_identity(cond, next_identity, visitor);
            walk_expr_with_identity(then_expr, next_identity, visitor);
            if let Some(otherwise) = else_expr {
                walk_expr_with_identity(otherwise, next_identity, visitor);
            }
        }
        Expr::List(items) => {
            for item in items {
                walk_expr_with_identity(item, next_identity, visitor);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                walk_expr_with_identity(key, next_identity, visitor);
                walk_expr_with_identity(value, next_identity, visitor);
            }
        }
        Expr::Guarded(inner, guard) => {
            walk_expr_with_identity(inner, next_identity, visitor);
            walk_expr_with_identity(guard, next_identity, visitor);
        }
        Expr::Block(stmts) => {
            walk_stmts_with_expr_identities_in_place(stmts, next_identity, visitor);
        }
        Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

fn walk_stmts_with_expr_identities_in_place(
    stmts: &[Stmt],
    next_identity: &mut usize,
    visitor: &mut impl FnMut(ExprIdentity, &Expr),
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                walk_expr_with_identity(expr, next_identity, visitor);
            }
            Stmt::Node(ns) => {
                walk_expr_with_identity(&ns.expr, next_identity, visitor);
                if let Some(guard) = &ns.when_guard {
                    walk_expr_with_identity(guard, next_identity, visitor);
                }
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    walk_expr_with_identity(expr, next_identity, visitor);
                }
            }
        }
    }
}
