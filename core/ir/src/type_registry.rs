//! Type registry for named type DAGs.
//!
//! The registry stores named type DAGs that can be referenced by `TypeId` in ports.
//! This enables type composition, sharing, and lookup during validation.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::{TypeRegistry, type_lib};
//!
//! let mut registry = TypeRegistry::new();
//!
//! // Register common types
//! registry.register("String", type_lib::string());
//! registry.register("Url", type_lib::url());
//! registry.register("OptionalUrl", type_lib::optional_url());
//!
//! // Look up types by name
//! let url_type = registry.get("Url").unwrap();
//! ```

use crate::dag::Dag;
use crate::type_lib;
use crate::type_op::TypeOp;
use crate::types::{Cardinality, TypeId};
use std::collections::HashMap;

/// Registry for named type DAGs.
///
/// Types are stored as `Dag<TypeOp>` and can be looked up by `TypeId`.
#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
    /// Map from type name to type DAG.
    types: HashMap<TypeId, Dag<TypeOp>>,
}

impl TypeRegistry {
    /// Create a new empty type registry.
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Create a type registry with common primitive types pre-registered.
    pub fn with_primitives() -> Self {
        let mut registry = Self::new();
        registry.register_primitives();
        registry
    }

    /// Register primitive types (String, Bool, Int, Unit, Json).
    pub fn register_primitives(&mut self) {
        self.register("String", type_lib::string());
        self.register("Bool", type_lib::bool());
        self.register("Int", type_lib::int());
        self.register("Unit", type_lib::unit());
        self.register("Json", type_lib::json());
    }

    /// Register common refined/core types used across the repo.
    ///
    /// These are structural refinements over primitives (e.g., Url is a refined String).
    pub fn register_core_types(&mut self) {
        self.register("NonEmptyString", type_lib::non_empty_string());
        self.register("Url", type_lib::url());
        self.register("FilePath", type_lib::file_path());
        self.register("Path", type_lib::file_path());
        self.register("Email", type_lib::email());
        self.register("PositiveInt", type_lib::positive_int());
        self.register("NonNegativeInt", type_lib::non_negative_int());

        // Legacy/container aliases (cardinality now lives in Port.cardinality).
        self.register("OptionalUrl", type_lib::optional_url());
        self.register("UrlList", type_lib::url_list());
        self.register("FilePathList", type_lib::file_path_list());
        self.register("NonEmptyFilePathList", type_lib::non_empty_file_path_list());
    }

    /// Create a type registry with primitives + common refined/core types.
    pub fn with_core_types() -> Self {
        let mut registry = Self::with_primitives();
        registry.register_core_types();
        registry
    }

    /// Register a type DAG with a name.
    pub fn register(&mut self, name: impl Into<TypeId>, type_dag: Dag<TypeOp>) {
        self.types.insert(name.into(), type_dag);
    }

    /// Get a type DAG by name.
    pub fn get(&self, name: &TypeId) -> Option<&Dag<TypeOp>> {
        self.types.get(name)
    }

    /// Get a type DAG by string name.
    pub fn get_by_name(&self, name: &str) -> Option<&Dag<TypeOp>> {
        self.types.get(&TypeId::from(name))
    }

    /// Check if a type is registered.
    pub fn contains(&self, name: &TypeId) -> bool {
        self.types.contains_key(name)
    }

    /// Get all registered type names.
    pub fn type_names(&self) -> impl Iterator<Item = &TypeId> {
        self.types.keys()
    }

    /// Get the number of registered types.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Infer cardinality from a registered type.
    ///
    /// Returns `None` if the type is not registered.
    pub fn infer_cardinality(&self, type_id: &TypeId) -> Option<Cardinality> {
        self.get(type_id).map(type_lib::infer_cardinality)
    }

    /// Get the base type name from a registered type.
    ///
    /// Returns `None` if the type is not registered.
    pub fn base_type_name(&self, type_id: &TypeId) -> Option<String> {
        self.get(type_id).and_then(type_lib::base_type_name)
    }

    /// Check if type A is compatible with type B.
    ///
    /// Compatibility is determined by:
    /// 1. Same type name (exact match)
    /// 2. Target is "Any" (accepts anything)
    /// 3. Structural refinement: A's contract entails B's contract
    pub fn is_compatible(&self, from: &TypeId, to: &TypeId) -> bool {
        // Same type is always compatible.
        if from == to {
            return true;
        }

        // Target Any accepts anything.
        if to.0 == "Any" {
            return true;
        }

        // Source Any does not entail specific targets.
        if from.0 == "Any" {
            return false;
        }

        // Look up both types; if not registered, fall back to name equality (handled above).
        let (Some(from_dag), Some(to_dag)) = (self.get(from), self.get(to)) else {
            return false;
        };

        // Check cardinality compatibility.
        let from_card = type_lib::infer_cardinality(from_dag);
        let to_card = type_lib::infer_cardinality(to_dag);
        if !from_card.satisfies(to_card) {
            return false;
        }

        // Check base type compatibility.
        let from_base = type_lib::base_type_name(from_dag);
        let to_base = type_lib::base_type_name(to_dag);
        match (from_base, to_base) {
            (Some(f), Some(t)) if f == t => {}
            _ => return false,
        }

        // Check predicate entailment (source must cover all target predicates).
        let from_preds = type_lib::predicates(from_dag);
        let to_preds = type_lib::predicates(to_dag);
        if to_preds.is_empty() {
            return true;
        }
        if from_preds.is_empty() && !to_preds.is_empty() {
            return false;
        }

        to_preds.iter().all(|tp| from_preds.iter().any(|fp| fp == tp))
    }
}

/// Error when type lookup fails.
#[derive(Debug, Clone)]
pub struct TypeNotFoundError {
    pub type_id: TypeId,
}

impl std::fmt::Display for TypeNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type not found: {}", self.type_id)
    }
}

impl std::error::Error for TypeNotFoundError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = TypeRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = TypeRegistry::new();
        registry.register("String", type_lib::string());

        assert!(registry.contains(&TypeId::from("String")));
        assert!(registry.get(&TypeId::from("String")).is_some());
        assert!(registry.get_by_name("String").is_some());
        assert!(registry.get_by_name("Unknown").is_none());
    }

    #[test]
    fn test_with_primitives() {
        let registry = TypeRegistry::with_primitives();

        assert!(registry.contains(&TypeId::from("String")));
        assert!(registry.contains(&TypeId::from("Bool")));
        assert!(registry.contains(&TypeId::from("Int")));
        assert!(registry.contains(&TypeId::from("Unit")));
        assert!(registry.contains(&TypeId::from("Json")));
        assert_eq!(registry.len(), 5);
    }

    #[test]
    fn test_cardinality_inference() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("MaybeValue", type_lib::optional(type_lib::string()));
        registry.register("ValueCollection", type_lib::list(type_lib::string()));

        assert_eq!(
            registry.infer_cardinality(&TypeId::from("String")),
            Some(Cardinality::ONE)
        );
        assert_eq!(
            registry.infer_cardinality(&TypeId::from("MaybeValue")),
            Some(Cardinality::ZERO_OR_ONE)
        );
        assert_eq!(
            registry.infer_cardinality(&TypeId::from("ValueCollection")),
            Some(Cardinality::ZERO_OR_MORE)
        );
    }

    #[test]
    fn test_type_compatibility() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());
        registry.register("NonEmptyString", type_lib::non_empty_string());

        // Same type is compatible
        assert!(registry.is_compatible(&TypeId::from("String"), &TypeId::from("String")));

        // Refined types are compatible with their base (Url -> String).
        assert!(registry.is_compatible(&TypeId::from("Url"), &TypeId::from("String")));

        // Base types are NOT compatible with refined types (String -> Url).
        assert!(!registry.is_compatible(&TypeId::from("String"), &TypeId::from("Url")));

        // NonEmptyString is a refinement of String.
        assert!(registry.is_compatible(
            &TypeId::from("NonEmptyString"),
            &TypeId::from("String")
        ));
        assert!(!registry.is_compatible(
            &TypeId::from("String"),
            &TypeId::from("NonEmptyString")
        ));
    }

    #[test]
    fn test_base_type_name() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());

        assert_eq!(
            registry.base_type_name(&TypeId::from("String")),
            Some("String".to_string())
        );
        assert_eq!(
            registry.base_type_name(&TypeId::from("Url")),
            Some("String".to_string())
        );
        assert_eq!(registry.base_type_name(&TypeId::from("Unknown")), None);
    }
}
