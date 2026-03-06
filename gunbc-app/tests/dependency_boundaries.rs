//! Cargo dependency graph boundary enforcement.
//!
//! Uses `cargo_metadata` to resolve the workspace dependency graph and assert
//! that crate boundaries are respected. This catches accidental dependency
//! additions that would bloat tool build pipelines.

use cargo_metadata::{DependencyKind, MetadataCommand};
use std::collections::HashSet;

/// Resolve transitive workspace dependencies for a given package.
/// Only follows normal (non-dev, non-build) dependency edges.
fn workspace_deps(metadata: &cargo_metadata::Metadata, pkg_name: &str) -> HashSet<String> {
    let workspace_members: HashSet<_> = metadata
        .workspace_members
        .iter()
        .map(|id| id.to_string())
        .collect();

    let resolve = metadata.resolve.as_ref().expect("no resolve graph");

    // Find the package ID for the given name
    let pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == pkg_name)
        .unwrap_or_else(|| panic!("package '{pkg_name}' not found in workspace"));

    let pkg_id = pkg.id.to_string();

    // Build adjacency map from resolve, filtering to normal deps only.
    // Dev and build deps are excluded to avoid false positives (e.g.,
    // gunbc-delegate-macros has gunbc-exec as a dev-dependency for testing).
    let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for node in &resolve.nodes {
        let id_str = node.id.repr.as_str();
        let dep_ids: Vec<&str> = node
            .deps
            .iter()
            .filter(|d| {
                d.dep_kinds
                    .iter()
                    .any(|dk| dk.kind == DependencyKind::Normal)
            })
            .map(|d| d.pkg.repr.as_str())
            .collect();
        adj.insert(id_str, dep_ids);
    }

    // BFS from pkg_id
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(pkg_id.as_str());

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current.to_string()) {
            continue;
        }
        if let Some(deps) = adj.get(current) {
            for dep in deps {
                if !visited.contains(*dep) {
                    queue.push_back(dep);
                }
            }
        }
    }

    // Remove self, filter to workspace members, extract names
    visited.remove(&pkg_id);
    let mut result = HashSet::new();
    for id in &visited {
        if workspace_members.contains(id) {
            // Extract package name from the resolved ID
            if let Some(p) = metadata.packages.iter().find(|p| p.id.repr == *id) {
                result.insert(p.name.clone());
            }
        }
    }
    result
}

#[test]
fn tool_crates_do_not_depend_on_unrelated_tools() {
    let metadata = MetadataCommand::new().exec().unwrap();

    // gunbc-deps should not depend on review or llm
    let deps_deps = workspace_deps(&metadata, "gunbc-deps");
    for name in ["gunbc-lib-review", "gunbc-lib-llm-ops"] {
        assert!(
            !deps_deps.contains(name),
            "gunbc-deps must not depend on {name}"
        );
    }
}

#[test]
fn leaf_crates_have_no_workspace_deps() {
    let metadata = MetadataCommand::new().exec().unwrap();

    let infra_deps = workspace_deps(&metadata, "gunbc-infra");
    assert!(
        infra_deps.is_empty(),
        "gunbc-infra is a leaf crate: expected 0 workspace deps, got {:?}",
        infra_deps
    );
}

#[test]
fn no_upward_layer_violations() {
    let metadata = MetadataCommand::new().exec().unwrap();

    // ir must not depend on exec (wrong direction)
    let ir_deps = workspace_deps(&metadata, "gunbc-ir");
    assert!(
        !ir_deps.contains("gunbc-exec"),
        "gunbc-ir must not depend on gunbc-exec (layer violation)"
    );

    // exec must not depend on codegen
    let exec_deps = workspace_deps(&metadata, "gunbc-exec");
    assert!(
        !exec_deps.contains("gunbc-codegen"),
        "gunbc-exec must not depend on gunbc-codegen (layer violation)"
    );
}
