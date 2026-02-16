//! daglang-typecheck: Type checking and interface resolution.
//!
//! Validates that all types are well-formed, all references resolve to
//! compatible types, refinement constraints are satisfiable, and
//! `implements` clauses are fulfilled.
//!
//! # Pipeline position
//!
//! ```text
//! ResolvedAST → [daglang-typecheck] → TypedAST
//! ```
//!
//! # Key responsibilities
//!
//! - Record and sum type validation
//! - Refinement type constraint checking (`@range`, `@pattern`, etc.)
//! - Generic type instantiation (`List<T>`, `Map<K,V>`, `Queue<T>`)
//! - Interface conformance (`resource X implements Y` — all capabilities present)
//! - `CloudConfig` sum type → provider resolution at compile time
//! - `@contract` annotation validation (behavioral specs are well-typed)
//! - Subtyping via the bounded lattice (§4.1.4 of dsl-design.md)

/// Errors during type checking.
#[derive(Debug)]
pub enum TypeError {
    /// A type name was used but not defined.
    UndefinedType(String),
    /// A field was accessed on a type that doesn't have it.
    NoSuchField { ty: String, field: String },
    /// Type mismatch in assignment or call.
    TypeMismatch { expected: String, got: String },
    /// A resource doesn't implement all capabilities of its interface.
    MissingCapability {
        resource: String,
        interface: String,
        capability: String,
    },
    /// A refinement constraint is unsatisfiable.
    UnsatisfiableRefinement { ty: String, constraint: String },
    /// Generic type parameter count mismatch.
    ArityMismatch { name: String, expected: usize, got: usize },
}
