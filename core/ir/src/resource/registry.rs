//! Resource registry for dependency resolution.
//!
//! The registry holds all known resource definitions and provides:
//! - Lookup by ResourceId
//! - Topological sort for dependency ordering
//! - Cycle detection
//!
//! This is separate from `ToolRegistry` (platform satisfiability) —
//! this registry handles freshness-based resource acquisition.

use super::def::{InputPattern, ResourceDef};
use super::super::ResourceId;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Error during resource resolution.
#[derive(Debug, Error)]
pub enum ResolutionError {
    /// Resource not found in registry.
    #[error("Resource '{0}' not found in registry")]
    NotFound(ResourceId),

    /// Cycle detected in dependency graph.
    #[error("Dependency cycle detected: {}", format_cycle(.0))]
    Cycle(Vec<ResourceId>),
}

fn format_cycle(cycle: &[ResourceId]) -> String {
    cycle
        .iter()
        .map(|id| id.0.as_str())
        .collect::<Vec<_>>()
        .join(" → ")
}

/// Registry of resource definitions.
///
/// Holds all known resources and provides dependency resolution.
#[derive(Debug, Default)]
pub struct ResourceRegistry {
    resources: HashMap<ResourceId, ResourceDef>,
}

impl ResourceRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Register a resource definition.
    pub fn register(&mut self, def: ResourceDef) {
        self.resources.insert(def.id.clone(), def);
    }

    /// Register multiple resource definitions.
    pub fn register_all(&mut self, defs: impl IntoIterator<Item = ResourceDef>) {
        for def in defs {
            self.register(def);
        }
    }

    /// Get a resource definition by ID.
    pub fn get(&self, id: &ResourceId) -> Option<&ResourceDef> {
        self.resources.get(id)
    }

    /// Check if a resource is registered.
    pub fn contains(&self, id: &ResourceId) -> bool {
        self.resources.contains_key(id)
    }

    /// Get the number of registered resources.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Iterate over all registered resources.
    pub fn iter(&self) -> impl Iterator<Item = (&ResourceId, &ResourceDef)> {
        self.resources.iter()
    }

    /// Resolve all dependencies for a resource (topological order).
    ///
    /// Returns resources in dependency order (dependencies first).
    /// The target resource is included as the last element.
    ///
    /// # Errors
    ///
    /// Returns `ResolutionError::NotFound` if any resource in the chain is missing.
    /// Returns `ResolutionError::Cycle` if a dependency cycle is detected.
    pub fn resolve(&self, target: &ResourceId) -> Result<Vec<ResourceId>, ResolutionError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new(); // For cycle detection (path tracking)

        self.visit(target, &mut visited, &mut stack, &mut result)?;
        Ok(result)
    }

    /// Resolve dependencies for multiple targets.
    ///
    /// Returns all resources needed for all targets, in dependency order.
    /// Duplicates are eliminated (each resource appears once).
    pub fn resolve_all(
        &self,
        targets: impl IntoIterator<Item = ResourceId>,
    ) -> Result<Vec<ResourceId>, ResolutionError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for target in targets {
            self.visit(&target, &mut visited, &mut stack, &mut result)?;
        }

        Ok(result)
    }

    fn visit(
        &self,
        id: &ResourceId,
        visited: &mut HashSet<ResourceId>,
        stack: &mut Vec<ResourceId>,
        order: &mut Vec<ResourceId>,
    ) -> Result<(), ResolutionError> {
        // Already processed
        if visited.contains(id) {
            return Ok(());
        }

        // Cycle detection: if id is in current path
        if let Some(pos) = stack.iter().position(|x| x == id) {
            // Extract cycle path
            let mut cycle: Vec<_> = stack[pos..].to_vec();
            cycle.push(id.clone()); // Complete the cycle
            return Err(ResolutionError::Cycle(cycle));
        }

        // Get the definition
        let def = self
            .resources
            .get(id)
            .ok_or_else(|| ResolutionError::NotFound(id.clone()))?;

        // Add to current path
        stack.push(id.clone());

        // Visit all resource dependencies
        for input in &def.inputs {
            if let InputPattern::Resource(dep_id) = input {
                self.visit(dep_id, visited, stack, order)?;
            }
        }

        // Remove from path
        stack.pop();

        // Mark as visited and add to result
        visited.insert(id.clone());
        order.push(id.clone());

        Ok(())
    }

    /// Get the direct dependencies of a resource.
    ///
    /// Returns only `InputPattern::Resource` dependencies, not file/env inputs.
    pub fn direct_deps(&self, id: &ResourceId) -> Result<Vec<&ResourceId>, ResolutionError> {
        let def = self
            .resources
            .get(id)
            .ok_or_else(|| ResolutionError::NotFound(id.clone()))?;

        Ok(def.resource_dependencies().collect())
    }

    /// Check if the dependency graph is acyclic.
    ///
    /// Returns `Ok(())` if no cycles, or `Err` with the first cycle found.
    pub fn check_acyclic(&self) -> Result<(), ResolutionError> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut order = Vec::new();

        for id in self.resources.keys() {
            if !visited.contains(id) {
                self.visit(id, &mut visited, &mut stack, &mut order)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::def::InputPattern;

    fn make_def(id: &str, deps: &[&str]) -> ResourceDef {
        let mut def = ResourceDef::new(ResourceId::new(id));
        for dep in deps {
            def = def.with_input(InputPattern::resource(ResourceId::new(*dep)));
        }
        def
    }

    #[test]
    fn test_registry_basic() {
        let mut registry = ResourceRegistry::new();
        let def = ResourceDef::new(ResourceId::new("test:a"));
        registry.register(def);

        assert_eq!(registry.len(), 1);
        assert!(registry.contains(&ResourceId::new("test:a")));
        assert!(!registry.contains(&ResourceId::new("test:b")));
    }

    #[test]
    fn test_resolve_no_deps() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &[]));

        let result = registry.resolve(&ResourceId::new("a")).unwrap();
        assert_eq!(result, vec![ResourceId::new("a")]);
    }

    #[test]
    fn test_resolve_linear_chain() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &[]));
        registry.register(make_def("b", &["a"]));
        registry.register(make_def("c", &["b"]));

        let result = registry.resolve(&ResourceId::new("c")).unwrap();

        // Should be: a, b, c (dependencies first)
        assert_eq!(
            result,
            vec![
                ResourceId::new("a"),
                ResourceId::new("b"),
                ResourceId::new("c")
            ]
        );
    }

    #[test]
    fn test_resolve_diamond() {
        //     a
        //    / \
        //   b   c
        //    \ /
        //     d
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &[]));
        registry.register(make_def("b", &["a"]));
        registry.register(make_def("c", &["a"]));
        registry.register(make_def("d", &["b", "c"]));

        let result = registry.resolve(&ResourceId::new("d")).unwrap();

        // a must come before b and c
        // b and c must come before d
        let a_pos = result.iter().position(|x| x.0 == "a").unwrap();
        let b_pos = result.iter().position(|x| x.0 == "b").unwrap();
        let c_pos = result.iter().position(|x| x.0 == "c").unwrap();
        let d_pos = result.iter().position(|x| x.0 == "d").unwrap();

        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
    }

    #[test]
    fn test_resolve_not_found() {
        let registry = ResourceRegistry::new();

        let result = registry.resolve(&ResourceId::new("missing"));
        assert!(matches!(result, Err(ResolutionError::NotFound(_))));
    }

    #[test]
    fn test_resolve_missing_dep() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &["missing"]));

        let result = registry.resolve(&ResourceId::new("a"));
        assert!(matches!(result, Err(ResolutionError::NotFound(_))));
    }

    #[test]
    fn test_resolve_cycle_self() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &["a"])); // Self-reference

        let result = registry.resolve(&ResourceId::new("a"));
        assert!(matches!(result, Err(ResolutionError::Cycle(_))));
    }

    #[test]
    fn test_resolve_cycle_two() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &["b"]));
        registry.register(make_def("b", &["a"]));

        let result = registry.resolve(&ResourceId::new("a"));
        assert!(matches!(result, Err(ResolutionError::Cycle(_))));
    }

    #[test]
    fn test_resolve_cycle_three() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &["b"]));
        registry.register(make_def("b", &["c"]));
        registry.register(make_def("c", &["a"]));

        let result = registry.resolve(&ResourceId::new("a"));
        assert!(matches!(result, Err(ResolutionError::Cycle(_))));
    }

    #[test]
    fn test_resolve_all() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &[]));
        registry.register(make_def("b", &["a"]));
        registry.register(make_def("c", &["a"]));

        let result = registry
            .resolve_all([ResourceId::new("b"), ResourceId::new("c")])
            .unwrap();

        // a should appear once (not twice)
        assert_eq!(result.iter().filter(|x| x.0 == "a").count(), 1);
        // All three should be present
        assert!(result.iter().any(|x| x.0 == "a"));
        assert!(result.iter().any(|x| x.0 == "b"));
        assert!(result.iter().any(|x| x.0 == "c"));
    }

    #[test]
    fn test_check_acyclic_ok() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &[]));
        registry.register(make_def("b", &["a"]));

        assert!(registry.check_acyclic().is_ok());
    }

    #[test]
    fn test_check_acyclic_cycle() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &["b"]));
        registry.register(make_def("b", &["a"]));

        assert!(registry.check_acyclic().is_err());
    }

    #[test]
    fn test_direct_deps() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &[]));
        registry.register(make_def("b", &["a"]));
        registry.register(
            ResourceDef::new(ResourceId::new("c"))
                .with_input(InputPattern::resource(ResourceId::new("a")))
                .with_input(InputPattern::glob("src/**/*.rs"))
                .with_input(InputPattern::resource(ResourceId::new("b"))),
        );

        let deps = registry.direct_deps(&ResourceId::new("c")).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.0 == "a"));
        assert!(deps.iter().any(|d| d.0 == "b"));
    }

    #[test]
    fn test_cycle_error_message() {
        let mut registry = ResourceRegistry::new();
        registry.register(make_def("a", &["b"]));
        registry.register(make_def("b", &["c"]));
        registry.register(make_def("c", &["a"]));

        let err = registry.resolve(&ResourceId::new("a")).unwrap_err();
        let msg = err.to_string();

        // Should contain the cycle path
        assert!(msg.contains("cycle"));
        assert!(msg.contains("→"));
    }
}
