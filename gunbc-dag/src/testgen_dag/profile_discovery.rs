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

use daglang_syntax::ast::{Expr, Item, Literal};
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
                let bound_interfaces: HashSet<String> = def
                    .binds
                    .iter()
                    .map(|b| b.interface_type.clone())
                    .collect();

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

/// Return profiles that bind at least one interface the module imports.
pub fn profiles_for_module<'a>(
    profiles: &'a [DiscoveredProfile],
    interface_imports: &HashSet<String>,
) -> Vec<&'a DiscoveredProfile> {
    profiles
        .iter()
        .filter(|p| !p.bound_interfaces.is_disjoint(interface_imports))
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
        Expr::Call(name, args) if name == "env" => {
            if let [(None, Expr::Literal(Literal::String(var_name)))] = args.as_slice() {
                env_vars.push(var_name.clone());
            }
        }
        Expr::Call(_, args) | Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                collect_env_calls(arg, env_vars);
            }
        }
        Expr::FieldAccess(base, _) | Expr::UnaryOp(_, base) => {
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
        Expr::Literal(_) | Expr::Ident(_) | Expr::After(_, _) => {}
        _ => {} // StringInterp, Match, For, Lambda, Map — rare in config entries
    }
}
