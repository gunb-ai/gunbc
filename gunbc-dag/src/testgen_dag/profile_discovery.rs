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

use daglang_syntax::ast::Item;
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

                profiles.push(DiscoveredProfile {
                    name: def.name.clone(),
                    module_path: module_path.clone(),
                    bound_interfaces,
                    test_class,
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
