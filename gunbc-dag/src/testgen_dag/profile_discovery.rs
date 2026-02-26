//! Profile discovery for test generation.
//!
//! Scans `dsl/profiles/*.dag` for `profile` blocks, extracts the interface
//! bindings, and infers a [`TestClass`] for each discovered profile.
//!
//! Two entry points:
//!
//! - [`discover_profiles`]: Scan a DSL root and return all profiles found.
//! - [`profiles_for_module`]: Filter profiles to those relevant to a module
//!   based on its interface imports.

use daglang_syntax::ast::{Expr, Item, Literal, StringPart};
use gunbc_test::TestClass;
use std::collections::HashSet;
use std::path::Path;

/// A profile discovered from `dsl/profiles/*.dag`.
#[derive(Debug, Clone)]
pub struct DiscoveredProfile {
    /// Profile name (e.g., "unit_test", "local").
    pub name: String,
    /// Dot-separated module path of the file containing this profile.
    pub module_path: String,
    /// Interface names this profile binds (e.g., {"IssueProvider", "ClaimStore"}).
    pub bound_interfaces: HashSet<String>,
    /// Inferred test class: Hermetic for unit_test / test-containing names,
    /// Integration for everything else.
    pub test_class: TestClass,
    /// Environment variables required by this profile (extracted from `env()` calls
    /// in profile bind config entries).
    pub required_env: Vec<String>,
}

/// Discover all profiles under `dsl_root/profiles/*.dag`.
///
/// Parses each `.dag` file, extracts `ProfileDef` items, and builds a
/// [`DiscoveredProfile`] for each one. Results are sorted by name.
#[allow(clippy::disallowed_methods)]
pub fn discover_profiles(dsl_root: &Path) -> Vec<DiscoveredProfile> {
    let profiles_dir = dsl_root.join("profiles");
    let mut profiles = Vec::new();

    let entries = match std::fs::read_dir(&profiles_dir) {
        Ok(entries) => entries,
        Err(_) => return profiles,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("dag") {
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let ast = match daglang_syntax::parser::parse(&source) {
            Ok(ast) => ast,
            Err(_) => continue,
        };

        let module_path = ast
            .module_path
            .as_ref()
            .map(|mp| mp.node.segments.join("."))
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

        for item in &ast.items {
            if let Item::ProfileDef(def) = &item.node {
                let bound_interfaces: HashSet<String> =
                    def.binds.iter().map(|b| b.interface_type.clone()).collect();

                let test_class = infer_test_class(&def.name);
                let required_env = extract_env_vars(def);

                profiles.push(DiscoveredProfile {
                    name: def.name.clone(),
                    module_path: module_path.clone(),
                    bound_interfaces,
                    test_class,
                    required_env,
                });
            }
        }
    }

    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    profiles
}

/// Return profiles that bind ALL interfaces the module imports.
/// A profile that only binds a subset would leave some interfaces unresolved,
/// causing runtime errors for the unbound capabilities.
pub fn profiles_for_module<'a>(
    profiles: &'a [DiscoveredProfile],
    interface_imports: &HashSet<String>,
) -> Vec<&'a DiscoveredProfile> {
    if interface_imports.is_empty() {
        return vec![];
    }
    profiles
        .iter()
        .filter(|p| {
            interface_imports
                .iter()
                .all(|iface| p.bound_interfaces.contains(iface))
        })
        .collect()
}

fn infer_test_class(profile_name: &str) -> TestClass {
    if profile_name == "unit_test" || profile_name.contains("test") {
        TestClass::Hermetic
    } else {
        TestClass::Integration
    }
}

/// Extract all env var names from `env("VAR")` calls in a profile's bind config entries.
fn extract_env_vars(profile: &daglang_syntax::ast::ProfileDef) -> Vec<String> {
    let mut env_vars = Vec::new();
    for bind in &profile.binds {
        for (_key, expr) in &bind.config_entries {
            collect_env_calls(expr, &mut env_vars);
        }
    }
    env_vars.sort();
    env_vars.dedup();
    env_vars
}

/// Walk an expression tree and collect env var names from `env("VAR")` calls.
fn collect_env_calls(expr: &Expr, env_vars: &mut Vec<String>) {
    match expr {
        Expr::Call(name, args) => {
            if name == "env" {
                if let Some((_, Expr::Literal(Literal::String(var_name)))) = args.first() {
                    env_vars.push(var_name.clone());
                }
            }
            for (_, arg) in args {
                collect_env_calls(arg, env_vars);
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                collect_env_calls(arg, env_vars);
            }
        }
        Expr::FieldAccess(base, _) | Expr::UnaryOp(_, base) | Expr::After(base, _) => {
            collect_env_calls(base, env_vars);
        }
        Expr::BinOp(lhs, _, rhs) | Expr::Pipe(lhs, rhs) | Expr::Guarded(lhs, rhs) => {
            collect_env_calls(lhs, env_vars);
            collect_env_calls(rhs, env_vars);
        }
        Expr::Record(_, fields) | Expr::Return(fields) => {
            for (_, value) in fields {
                collect_env_calls(value, env_vars);
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_env_calls(cond, env_vars);
            collect_env_calls(then_expr, env_vars);
            if let Some(otherwise) = else_expr {
                collect_env_calls(otherwise, env_vars);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_env_calls(item, env_vars);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_env_calls(key, env_vars);
                collect_env_calls(value, env_vars);
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let StringPart::Expr(inner) = part {
                    collect_env_calls(inner, env_vars);
                }
            }
        }
        Expr::Match(subject, arms) => {
            collect_env_calls(subject, env_vars);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_env_calls(guard, env_vars);
                }
                collect_env_calls(&arm.body, env_vars);
            }
        }
        Expr::For(_, iterable, _, body) => {
            collect_env_calls(iterable, env_vars);
            collect_env_calls(body, env_vars);
        }
        Expr::Lambda(_, body) => {
            collect_env_calls(body, env_vars);
        }
        Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_syntax::ast::{MatchArm, Pattern};

    fn env_call(var: &str) -> Expr {
        Expr::Call(
            "env".to_string(),
            vec![(None, Expr::Literal(Literal::String(var.to_string())))],
        )
    }

    #[test]
    fn collect_env_calls_reads_first_env_arg_with_default() {
        let expr = Expr::Call(
            "env".to_string(),
            vec![
                (None, Expr::Literal(Literal::String("GITHUB_TOKEN".into()))),
                (None, Expr::Literal(Literal::String("fallback".into()))),
            ],
        );
        let mut env_vars = Vec::new();
        collect_env_calls(&expr, &mut env_vars);
        assert_eq!(env_vars, vec!["GITHUB_TOKEN".to_string()]);
    }

    #[test]
    fn collect_env_calls_traverses_all_expr_variants() {
        let expr = Expr::Record(
            None,
            vec![
                (
                    "interp".into(),
                    Expr::StringInterp(vec![
                        StringPart::Literal("prefix".into()),
                        StringPart::Expr(env_call("FROM_INTERP")),
                    ]),
                ),
                (
                    "map".into(),
                    Expr::Map(vec![(
                        Expr::Literal(Literal::String("k".into())),
                        env_call("FROM_MAP"),
                    )]),
                ),
                (
                    "match".into(),
                    Expr::Match(
                        Box::new(Expr::Ident("subject".into())),
                        vec![MatchArm {
                            pattern: Pattern::Wildcard,
                            guard: Some(env_call("FROM_GUARD")),
                            body: env_call("FROM_MATCH_BODY"),
                        }],
                    ),
                ),
                (
                    "for_loop".into(),
                    Expr::For(
                        "x".into(),
                        Box::new(env_call("FROM_FOR_ITERABLE")),
                        vec![],
                        Box::new(env_call("FROM_FOR_BODY")),
                    ),
                ),
                (
                    "lambda".into(),
                    Expr::Lambda(vec!["x".into()], Box::new(env_call("FROM_LAMBDA"))),
                ),
                (
                    "after".into(),
                    Expr::After(Box::new(env_call("FROM_AFTER")), vec!["dep".into()]),
                ),
            ],
        );

        let mut env_vars = Vec::new();
        collect_env_calls(&expr, &mut env_vars);
        env_vars.sort();
        env_vars.dedup();

        assert_eq!(
            env_vars,
            vec![
                "FROM_AFTER".to_string(),
                "FROM_FOR_BODY".to_string(),
                "FROM_FOR_ITERABLE".to_string(),
                "FROM_GUARD".to_string(),
                "FROM_INTERP".to_string(),
                "FROM_LAMBDA".to_string(),
                "FROM_MAP".to_string(),
                "FROM_MATCH_BODY".to_string(),
            ]
        );
    }
}
