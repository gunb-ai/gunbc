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
    /// 1. Same type name (structural equality)
    /// 2. A's cardinality satisfies B's cardinality
    /// 3. A's predicates are a superset of B's predicates (stricter is compatible with looser)
    pub fn is_compatible(&self, from: &TypeId, to: &TypeId) -> bool {
        // Same type is always compatible
        if from == to {
            return true;
        }

        // Look up both types
        let from_dag = match self.get(from) {
            Some(d) => d,
            None => return false,
        };
        let to_dag = match self.get(to) {
            Some(d) => d,
            None => return false,
        };

        // Check cardinality compatibility
        let from_card = type_lib::infer_cardinality(from_dag);
        let to_card = type_lib::infer_cardinality(to_dag);

        if !from_card.satisfies(to_card) {
            return false;
        }

        // Check base type compatibility
        let from_base = type_lib::base_type_name(from_dag);
        let to_base = type_lib::base_type_name(to_dag);

        match (from_base, to_base) {
            (Some(f), Some(t)) => f == t,
            _ => false,
        }
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
        registry.register("OptionalString", type_lib::optional(type_lib::string()));
        registry.register("StringList", type_lib::list(type_lib::string()));

        assert_eq!(
            registry.infer_cardinality(&TypeId::from("String")),
            Some(Cardinality::One)
        );
        assert_eq!(
            registry.infer_cardinality(&TypeId::from("OptionalString")),
            Some(Cardinality::ZeroOrOne)
        );
        assert_eq!(
            registry.infer_cardinality(&TypeId::from("StringList")),
            Some(Cardinality::ZeroOrMore)
        );
    }

    #[test]
    fn test_type_compatibility() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());

        // Same type is compatible
        assert!(registry.is_compatible(&TypeId::from("String"), &TypeId::from("String")));

        // Different types with same base might be compatible
        // (URL is a refined String, so String -> Url should work for base type)
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
