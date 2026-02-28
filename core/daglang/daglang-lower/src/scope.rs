//! Scoped IR: scope-aware analysis of callable bodies.
//!
//! Builds a `ScopedBody` from AST statements, preserving the scope tree
//! so that downstream transport planning can assign transport triplets
//! to the correct SubDag level.
//!
//! The core insight: service calls inside match/if branches must have
//! their transport triplets inside the branch SubDag, not at the
//! top level. The existing `collect_service_calls_from_stmts` flattens
//! all calls regardless of scope. This module preserves that structure.
//!
//! Note: Types in this module are actively being integrated into the main
//! lowering pipeline. They replace the ad-hoc IfBranchSite/MatchBranchSite
//! structs and detect_*_branches_in_stmts functions in lib.rs.

use daglang_syntax::ast::{Expr, Pattern, Stmt};
#[cfg(test)]
use daglang_syntax::ast::MatchArm;

/// A service call found in the AST, with its dot-separated path and args.
#[derive(Debug, Clone)]
pub(crate) struct ScopedServiceCall {
    /// The dot-separated path, e.g., `["gcp", "SecretManager", "AccessSecret"]`.
    pub path: Vec<String>,
    /// Argument expressions — only the label and ident/literal info needed.
    pub arg_labels: Vec<Option<String>>,
}

/// Reference to an expression — simplified for scope analysis.
///
/// We don't deeply decompose expressions; we only need to know:
/// - Is it a service call? (captured in ScopedItem::ServiceCall)
/// - Is it an ident or field access? (for dependency tracking)
/// - Everything else is opaque.
#[derive(Debug, Clone)]
pub(crate) enum ExprRef {
    /// A bare identifier reference: `x`.
    Ident(String),
    /// A literal value.
    Literal(crate::ServiceCallArgLiteral),
    /// Any other expression (not decomposed further for scope analysis).
    Opaque,
}

/// An item within a scope block — one logical unit of work.
#[derive(Debug, Clone)]
pub(crate) enum ScopedItem {
    /// A service call at this scope level.
    ServiceCall(ScopedServiceCall),

    /// A for-loop introducing a nested scope for its body.
    ForLoop {
        element_var: String,
        passthrough: Vec<String>,
        body: ScopedBody,
    },

    /// An if/else introducing two nested scopes.
    IfBranch {
        then_body: ScopedBody,
        else_body: Option<ScopedBody>,
    },

    /// A match expression introducing N nested scopes (one per arm).
    MatchBranch {
        arms: Vec<MatchArmScope>,
    },

    /// A function call (non-service). Not scope-introducing, but tracked
    /// so transport planning knows about fn-call-based service forwarding.
    FnCall {
        name: String,
    },

    /// A let binding or assignment. Not scope-introducing.
    Binding {
        name: String,
    },

    /// Anything else (pure expressions, returns, etc.).
    Other,
}

/// One arm of a match expression, with its own scope.
#[derive(Debug, Clone)]
pub(crate) struct MatchArmScope {
    /// The pattern label (variant name, literal, or wildcard).
    pub label: String,
    /// The body scope for this arm.
    pub body: ScopedBody,
}

/// A scoped body — a sequence of items at a given nesting level.
///
/// Represents the scope tree for one callable's body. Service calls
/// at this level are direct; service calls inside ForLoop/IfBranch/MatchBranch
/// items are nested in their respective child scopes.
#[derive(Debug, Clone)]
pub(crate) struct ScopedBody {
    pub items: Vec<ScopedItem>,
}

impl ScopedBody {
    /// Build a ScopedBody from AST statements.
    pub fn from_stmts(stmts: &[Stmt]) -> Self {
        let mut items = Vec::new();
        for stmt in stmts {
            collect_scoped_items_from_stmt(stmt, &mut items);
        }
        ScopedBody { items }
    }

    /// Collect all service call paths at this scope level only (not nested).
    pub fn direct_service_calls(&self) -> Vec<&ScopedServiceCall> {
        self.items
            .iter()
            .filter_map(|item| match item {
                ScopedItem::ServiceCall(call) => Some(call),
                _ => None,
            })
            .collect()
    }

    /// Collect all service call paths recursively (at all scope levels).
    pub fn all_service_calls(&self) -> Vec<&ScopedServiceCall> {
        let mut result = Vec::new();
        self.collect_all_service_calls(&mut result);
        result
    }

    fn collect_all_service_calls<'a>(&'a self, out: &mut Vec<&'a ScopedServiceCall>) {
        for item in &self.items {
            match item {
                ScopedItem::ServiceCall(call) => out.push(call),
                ScopedItem::ForLoop { body, .. } => body.collect_all_service_calls(out),
                ScopedItem::IfBranch {
                    then_body,
                    else_body,
                } => {
                    then_body.collect_all_service_calls(out);
                    if let Some(else_body) = else_body {
                        else_body.collect_all_service_calls(out);
                    }
                }
                ScopedItem::MatchBranch { arms } => {
                    for arm in arms {
                        arm.body.collect_all_service_calls(out);
                    }
                }
                ScopedItem::FnCall { .. }
                | ScopedItem::Binding { .. }
                | ScopedItem::Other => {}
            }
        }
    }

    /// Check if this scope or any nested scope contains service calls.
    pub fn has_service_calls(&self) -> bool {
        !self.all_service_calls().is_empty()
    }

    /// Recursively count total service calls across all scopes.
    pub fn total_service_call_count(&self) -> usize {
        self.all_service_calls().len()
    }
}

// ============================================================================
// AST → ScopedBody conversion
// ============================================================================

fn collect_scoped_items_from_stmt(stmt: &Stmt, items: &mut Vec<ScopedItem>) {
    match stmt {
        Stmt::Let(name, expr) => {
            // First process the expression for service calls / control flow
            collect_scoped_items_from_expr(expr, items);
            items.push(ScopedItem::Binding {
                name: name.clone(),
            });
        }
        Stmt::Assign(name, expr) => {
            collect_scoped_items_from_expr(expr, items);
            items.push(ScopedItem::Binding {
                name: name.clone(),
            });
        }
        Stmt::Expr(expr) => {
            collect_scoped_items_from_expr(expr, items);
        }
        Stmt::Return(fields) => {
            for (_, expr) in fields {
                collect_scoped_items_from_expr(expr, items);
            }
        }
        Stmt::Node(node_stmt) => {
            collect_scoped_items_from_expr(&node_stmt.expr, items);
        }
    }
}

fn collect_scoped_items_from_expr(expr: &Expr, items: &mut Vec<ScopedItem>) {
    match expr {
        // Service call — the key item we're tracking.
        Expr::ServiceCall(path, args) => {
            items.push(ScopedItem::ServiceCall(ScopedServiceCall {
                path: path.clone(),
                arg_labels: args.iter().map(|(label, _)| label.clone()).collect(),
            }));
        }

        // For-loop — scope-introducing.
        Expr::For(var, _iterable, passthrough, body) => {
            let body_scope = scope_from_expr(body);
            items.push(ScopedItem::ForLoop {
                element_var: var.clone(),
                passthrough: passthrough.clone(),
                body: body_scope,
            });
        }

        // If/else — scope-introducing.
        Expr::If(_, then_expr, else_expr) => {
            let then_body = scope_from_expr(then_expr);
            let else_body = else_expr.as_ref().map(|e| scope_from_expr(e));
            // Only emit an IfBranch if there are service calls in the branches.
            if then_body.has_service_calls()
                || else_body.as_ref().is_some_and(|b| b.has_service_calls())
            {
                items.push(ScopedItem::IfBranch {
                    then_body,
                    else_body,
                });
            } else {
                // No service calls inside — treat as opaque expression.
                // Still recurse to pick up any service calls in condition.
                items.push(ScopedItem::Other);
            }
        }

        // Match — scope-introducing.
        Expr::Match(_, arms) => {
            let arm_scopes: Vec<MatchArmScope> = arms
                .iter()
                .map(|arm| {
                    let label = pattern_label(&arm.pattern);
                    let body = scope_from_expr(&arm.body);
                    MatchArmScope { label, body }
                })
                .collect();
            let has_calls = arm_scopes.iter().any(|a| a.body.has_service_calls());
            if has_calls {
                items.push(ScopedItem::MatchBranch { arms: arm_scopes });
            } else {
                items.push(ScopedItem::Other);
            }
        }

        // Function call — tracked but not scope-introducing.
        Expr::Call(name, args) => {
            // Recurse into args to catch nested service calls.
            for (_, arg_expr) in args {
                collect_scoped_items_from_expr(arg_expr, items);
            }
            items.push(ScopedItem::FnCall {
                name: name.clone(),
            });
        }

        // Pipe — recurse into both sides.
        Expr::Pipe(lhs, rhs) => {
            collect_scoped_items_from_expr(lhs, items);
            collect_scoped_items_from_expr(rhs, items);
        }

        // Binary op — recurse.
        Expr::BinOp(lhs, _, rhs) => {
            collect_scoped_items_from_expr(lhs, items);
            collect_scoped_items_from_expr(rhs, items);
        }

        // Unary op — recurse.
        Expr::UnaryOp(_, inner) => {
            collect_scoped_items_from_expr(inner, items);
        }

        // Record — recurse into field expressions.
        Expr::Record(_, fields) => {
            for (_, field_expr) in fields {
                collect_scoped_items_from_expr(field_expr, items);
            }
        }

        // String interpolation — recurse into parts.
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_scoped_items_from_expr(inner, items);
                }
            }
        }

        // Lambda — recurse into body.
        Expr::Lambda(_, body) => {
            collect_scoped_items_from_expr(body, items);
        }

        // List — recurse into elements.
        Expr::List(elems) => {
            for elem in elems {
                collect_scoped_items_from_expr(elem, items);
            }
        }

        // Map — recurse into entries.
        Expr::Map(entries) => {
            for (key, val) in entries {
                collect_scoped_items_from_expr(key, items);
                collect_scoped_items_from_expr(val, items);
            }
        }

        // Field access — recurse into base.
        Expr::FieldAccess(base, _) => {
            collect_scoped_items_from_expr(base, items);
        }

        // Guarded / After — recurse.
        Expr::Guarded(inner, guard) => {
            collect_scoped_items_from_expr(inner, items);
            collect_scoped_items_from_expr(guard, items);
        }
        Expr::After(inner, _) => {
            collect_scoped_items_from_expr(inner, items);
        }

        // Return — recurse into field values.
        Expr::Return(fields) => {
            for (_, field_expr) in fields {
                collect_scoped_items_from_expr(field_expr, items);
            }
        }

        // Leaf expressions — no recursion needed.
        Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

/// Build a ScopedBody from a single expression (used for branch/loop bodies).
fn scope_from_expr(expr: &Expr) -> ScopedBody {
    let mut items = Vec::new();
    collect_scoped_items_from_expr(expr, &mut items);
    ScopedBody { items }
}

/// Extract a human-readable label from a match pattern.
fn pattern_label(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Ident(name) => name.clone(),
        Pattern::Variant(name, _) => name.clone(),
        Pattern::Wildcard => "_".to_string(),
        Pattern::Literal(lit) => format!("{lit:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_syntax::ast::{Expr, Literal, Stmt};

    #[test]
    fn empty_body_has_no_service_calls() {
        let body = ScopedBody::from_stmts(&[]);
        assert!(!body.has_service_calls());
        assert_eq!(body.total_service_call_count(), 0);
        assert!(body.direct_service_calls().is_empty());
    }

    #[test]
    fn top_level_service_call_is_direct() {
        let stmts = vec![Stmt::Expr(Expr::ServiceCall(
            vec!["gcp".into(), "Storage".into(), "GetBucket".into()],
            vec![],
        ))];
        let body = ScopedBody::from_stmts(&stmts);
        assert_eq!(body.direct_service_calls().len(), 1);
        assert_eq!(body.total_service_call_count(), 1);
        assert_eq!(
            body.direct_service_calls()[0].path,
            vec!["gcp", "Storage", "GetBucket"]
        );
    }

    #[test]
    fn service_call_inside_match_is_nested() {
        let match_expr = Expr::Match(
            Box::new(Expr::Ident("runtime".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Ident("github_oidc".into()),
                    guard: None,
                    body: Expr::ServiceCall(
                        vec!["gcp".into(), "Auth".into(), "ExchangeToken".into()],
                        vec![],
                    ),
                },
                MatchArm {
                    pattern: Pattern::Ident("metadata_oidc".into()),
                    guard: None,
                    body: Expr::ServiceCall(
                        vec!["gcp".into(), "Metadata".into(), "GetToken".into()],
                        vec![],
                    ),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expr::ServiceCall(
                        vec!["gcp".into(), "Auth".into(), "LocalAuth".into()],
                        vec![],
                    ),
                },
            ],
        );
        let stmts = vec![Stmt::Expr(match_expr)];
        let body = ScopedBody::from_stmts(&stmts);

        // No direct service calls at top level
        assert!(body.direct_service_calls().is_empty());

        // Three nested service calls total
        assert_eq!(body.total_service_call_count(), 3);

        // Should be a MatchBranch item with 3 arms
        assert_eq!(body.items.len(), 1);
        match &body.items[0] {
            ScopedItem::MatchBranch { arms } => {
                assert_eq!(arms.len(), 3);
                assert_eq!(arms[0].label, "github_oidc");
                assert_eq!(arms[0].body.direct_service_calls().len(), 1);
                assert_eq!(arms[1].label, "metadata_oidc");
                assert_eq!(arms[1].body.direct_service_calls().len(), 1);
                assert_eq!(arms[2].label, "_");
                assert_eq!(arms[2].body.direct_service_calls().len(), 1);
            }
            other => panic!("expected MatchBranch, got {other:?}"),
        }
    }

    #[test]
    fn service_call_inside_for_loop_is_nested() {
        let for_expr = Expr::For(
            "item".into(),
            Box::new(Expr::Ident("items".into())),
            vec![],
            Box::new(Expr::ServiceCall(
                vec!["github".into(), "Gist".into(), "Create".into()],
                vec![],
            )),
        );
        let stmts = vec![Stmt::Expr(for_expr)];
        let body = ScopedBody::from_stmts(&stmts);

        assert!(body.direct_service_calls().is_empty());
        assert_eq!(body.total_service_call_count(), 1);

        match &body.items[0] {
            ScopedItem::ForLoop {
                element_var, body, ..
            } => {
                assert_eq!(element_var, "item");
                assert_eq!(body.direct_service_calls().len(), 1);
            }
            other => panic!("expected ForLoop, got {other:?}"),
        }
    }

    #[test]
    fn service_call_inside_if_is_nested() {
        let if_expr = Expr::If(
            Box::new(Expr::Ident("enabled".into())),
            Box::new(Expr::ServiceCall(
                vec!["github".into(), "Issues".into(), "Create".into()],
                vec![],
            )),
            Some(Box::new(Expr::ServiceCall(
                vec!["github".into(), "Issues".into(), "Update".into()],
                vec![],
            ))),
        );
        let stmts = vec![Stmt::Expr(if_expr)];
        let body = ScopedBody::from_stmts(&stmts);

        assert!(body.direct_service_calls().is_empty());
        assert_eq!(body.total_service_call_count(), 2);

        match &body.items[0] {
            ScopedItem::IfBranch {
                then_body,
                else_body,
            } => {
                assert_eq!(then_body.direct_service_calls().len(), 1);
                assert_eq!(
                    else_body.as_ref().unwrap().direct_service_calls().len(),
                    1
                );
            }
            other => panic!("expected IfBranch, got {other:?}"),
        }
    }

    #[test]
    fn mixed_top_level_and_nested_calls() {
        let stmts = vec![
            // Top-level service call
            Stmt::Expr(Expr::ServiceCall(
                vec!["github".into(), "Auth".into(), "GetToken".into()],
                vec![],
            )),
            // Match with nested calls
            Stmt::Expr(Expr::Match(
                Box::new(Expr::Ident("mode".into())),
                vec![
                    MatchArm {
                        pattern: Pattern::Ident("Create".into()),
                        guard: None,
                        body: Expr::ServiceCall(
                            vec!["github".into(), "Gist".into(), "Create".into()],
                            vec![],
                        ),
                    },
                    MatchArm {
                        pattern: Pattern::Ident("Update".into()),
                        guard: None,
                        body: Expr::ServiceCall(
                            vec!["github".into(), "Gist".into(), "Update".into()],
                            vec![],
                        ),
                    },
                ],
            )),
        ];
        let body = ScopedBody::from_stmts(&stmts);

        // One direct, two nested
        assert_eq!(body.direct_service_calls().len(), 1);
        assert_eq!(body.total_service_call_count(), 3);
    }

    #[test]
    fn match_without_service_calls_is_other() {
        let match_expr = Expr::Match(
            Box::new(Expr::Ident("x".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Ident("A".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(1)),
                },
                MatchArm {
                    pattern: Pattern::Ident("B".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(2)),
                },
            ],
        );
        let stmts = vec![Stmt::Expr(match_expr)];
        let body = ScopedBody::from_stmts(&stmts);

        assert!(body.direct_service_calls().is_empty());
        assert_eq!(body.total_service_call_count(), 0);
        // Should be ScopedItem::Other, not MatchBranch
        assert!(matches!(body.items[0], ScopedItem::Other));
    }

    #[test]
    fn let_binding_with_service_call_in_rhs() {
        let stmts = vec![Stmt::Let(
            "result".into(),
            Expr::ServiceCall(
                vec!["gcp".into(), "Storage".into(), "Read".into()],
                vec![],
            ),
        )];
        let body = ScopedBody::from_stmts(&stmts);

        // The service call should be direct (it's in the let RHS, not inside control flow)
        assert_eq!(body.direct_service_calls().len(), 1);
    }
}
