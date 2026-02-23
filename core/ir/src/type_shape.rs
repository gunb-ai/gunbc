//! Structural classification derived from type DAGs.
//!
//! `TypeShape` is an algebraic decomposition of a `Dag<TypeOp>` into its
//! structural form. Backends inspect the shape — not the type name — to
//! derive their native representations.
//!
//! # Motivation
//!
//! Today each emit backend has an independent `map_to_*_type()` that
//! pattern-matches on type name strings. This leads to semantic drift
//! (each backend independently decides what "Bool" means) and silent
//! fallthrough when new types are added without updating all backends.
//!
//! `TypeShape` is the shared intermediate: the type DAG is read once into
//! a `TypeShape`, and every backend pattern-matches on the shape instead
//! of the name.
//!
//! # Phase 0 Scope
//!
//! This module adds the data structures and the `type_shape()` extractor.
//! It does **not** modify any emit backends or type_lib functions yet.
//! Those changes come in later phases (see `docs/design/modeling/structural-primitives-codegen.md`).

use crate::dag::Dag;
use crate::node::NodeBody;
use crate::type_op::{MetadataPayload, PlatformRepr, TypeOp, WrapperKind};

/// Structural classification derived from a type DAG.
///
/// Each variant represents an algebraic shape that backends can
/// pattern-match on to derive their native representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeShape {
    /// Platform primitive with machine representation contract.
    ///
    /// The `PlatformRepr` carries the bit width, signedness, and
    /// float/discrete flags. Backends derive their native integer/float
    /// type from these properties.
    Platform(PlatformRepr),

    /// Coproduct (tagged union) with named variants.
    ///
    /// Each variant has a name and a recursive `TypeShape` for its payload.
    /// A coproduct where all variants have `TypeShape::Opaque("Unit")` is
    /// an all-unit enum (e.g., Bool, HttpMethod).
    Coproduct(Vec<(String, TypeShape)>),

    /// Product (record) with named fields.
    ///
    /// Each field has a name and a recursive `TypeShape` for its type.
    Product(Vec<(String, TypeShape)>),

    /// Branded wrapper around an inner type.
    ///
    /// Nominal distinctness: `TextFilePath` is not `FilePath` even though
    /// structurally identical, unless the brand allows coercion.
    Brand(String, Box<TypeShape>),

    /// Container: Optional, List, Set, Map.
    Container(ContainerShape),

    /// Opaque/unresolved type (legacy fallback).
    ///
    /// This is a **diagnostic**, not a silent fallback. The goal is to
    /// shrink Opaque to zero over time as all types get structural
    /// definitions. Codegen backends should emit a warning (strict mode:
    /// error) when they encounter Opaque.
    Opaque(String),
}

/// Container shape variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerShape {
    /// Optional wrapping: zero or one value.
    Optional(Box<TypeShape>),
    /// List wrapping: zero or more values.
    List(Box<TypeShape>),
    /// Set wrapping: zero or more unique values.
    Set(Box<TypeShape>),
    /// Map wrapping: string-keyed map with typed values.
    ///
    /// The first TypeShape is the key type (always String today),
    /// the second is the value type.
    Map(Box<TypeShape>, Box<TypeShape>),
}

/// Extract the structural shape from a type DAG.
///
/// Walks the `Dag<TypeOp>`, classifying by root node's `TypeOp` variant:
///
/// - `TypeOp::Meta(MetadataPayload::PlatformRepr(repr))` => `TypeShape::Platform(repr)`
/// - `TypeOp::Coproduct(variants)` => `TypeShape::Coproduct(...)`
/// - `TypeOp::Product(fields)` => `TypeShape::Product(...)`
/// - `TypeOp::Brand(name, inner_type_id)` => `TypeShape::Brand(name, inner_shape)`
/// - `TypeOp::Wrap(kind)` with inner SubDag => `TypeShape::Container(...)`
/// - `TypeOp::Identity` for simple identity types => `TypeShape::Opaque(type_name)`
///
/// The extractor does NOT resolve type references through the registry.
/// Each variant's inner type is classified as `Opaque(type_id)` unless
/// the type DAG itself carries structural information (e.g., a SubDag).
pub fn type_shape(dag: &Dag<TypeOp>) -> TypeShape {
    // Priority 1: Look for PlatformRepr metadata node.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Meta(MetadataPayload::PlatformRepr(repr))) = &node.body {
            return TypeShape::Platform(repr.clone());
        }
    }

    // Priority 2: Look for Coproduct node.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Coproduct(variants)) = &node.body {
            let shaped_variants: Vec<(String, TypeShape)> = variants
                .iter()
                .map(|(name, type_id)| (name.clone(), TypeShape::Opaque(type_id.0.clone())))
                .collect();
            return TypeShape::Coproduct(shaped_variants);
        }
    }

    // Priority 3: Look for Product node.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Product(fields)) = &node.body {
            let shaped_fields: Vec<(String, TypeShape)> = fields
                .iter()
                .map(|(name, type_id)| (name.clone(), TypeShape::Opaque(type_id.0.clone())))
                .collect();
            return TypeShape::Product(shaped_fields);
        }
    }

    // Priority 4: Look for Brand node.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Brand(name, _type_id)) = &node.body {
            // Recurse into the SubDag if present to get the inner shape.
            let inner_shape = inner_subdag(dag)
                .map(type_shape)
                .unwrap_or_else(|| TypeShape::Opaque(name.clone()));
            return TypeShape::Brand(name.clone(), Box::new(inner_shape));
        }
    }

    // Priority 5: Look for Wrap (container) node.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Wrap(kind)) = &node.body {
            let inner_shape = inner_subdag(dag)
                .map(type_shape)
                .unwrap_or(TypeShape::Opaque("Any".to_string()));
            return match kind {
                WrapperKind::Optional => {
                    TypeShape::Container(ContainerShape::Optional(Box::new(inner_shape)))
                }
                WrapperKind::List | WrapperKind::NonEmptyList => {
                    TypeShape::Container(ContainerShape::List(Box::new(inner_shape)))
                }
                WrapperKind::Set | WrapperKind::NonEmptySet => {
                    TypeShape::Container(ContainerShape::Set(Box::new(inner_shape)))
                }
                WrapperKind::Map => TypeShape::Container(ContainerShape::Map(
                    Box::new(TypeShape::Opaque("String".to_string())),
                    Box::new(inner_shape),
                )),
            };
        }
    }

    // Priority 6: Identity node => Opaque with the type name from the port.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Identity) = &node.body {
            if let Some(output) = node.outputs.first() {
                return TypeShape::Opaque(output.type_id.0.clone());
            }
        }
    }

    // Fallback: completely unknown.
    TypeShape::Opaque("Unknown".to_string())
}

/// Find the first SubDag node in a type DAG and return a reference to its inner DAG.
fn inner_subdag(dag: &Dag<TypeOp>) -> Option<&Dag<TypeOp>> {
    dag.nodes.iter().find_map(|node| {
        if let NodeBody::SubDag(subdag) = &node.body {
            Some(subdag)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{Edge, Port};
    use crate::node::Node;
    use crate::type_lib;
    use crate::type_op::PlatformRepr;

    // =========================================================================
    // PlatformRepr construction and serialization
    // =========================================================================

    #[test]
    fn platform_repr_construction() {
        let repr = PlatformRepr {
            bits: 64,
            signed: true,
            float: false,
            discrete: true,
        };
        assert_eq!(repr.bits, 64);
        assert!(repr.signed);
        assert!(!repr.float);
        assert!(repr.discrete);
    }

    #[test]
    fn platform_repr_serialization_roundtrip() {
        let repr = PlatformRepr {
            bits: 64,
            signed: true,
            float: true,
            discrete: false,
        };
        let json = serde_json::to_string(&repr).expect("serialize PlatformRepr");
        let parsed: PlatformRepr = serde_json::from_str(&json).expect("deserialize PlatformRepr");
        assert_eq!(repr, parsed);
    }

    #[test]
    fn metadata_payload_platform_repr_serialization() {
        let payload = MetadataPayload::PlatformRepr(PlatformRepr {
            bits: 32,
            signed: false,
            float: false,
            discrete: true,
        });
        let json = serde_json::to_string(&payload).expect("serialize MetadataPayload");
        let parsed: MetadataPayload =
            serde_json::from_str(&json).expect("deserialize MetadataPayload");
        assert_eq!(payload, parsed);
    }

    // =========================================================================
    // TypeShape extraction from type DAGs built by type_lib
    // =========================================================================

    #[test]
    fn shape_of_identity_is_opaque() {
        let string_dag = type_lib::string();
        let shape = type_shape(&string_dag);
        assert_eq!(shape, TypeShape::Opaque("String".to_string()));
    }

    #[test]
    fn shape_of_bool_identity_is_opaque() {
        let bool_dag = type_lib::bool();
        let shape = type_shape(&bool_dag);
        assert_eq!(shape, TypeShape::Opaque("Bool".to_string()));
    }

    #[test]
    fn shape_of_int_identity_is_opaque() {
        let int_dag = type_lib::int();
        let shape = type_shape(&int_dag);
        assert_eq!(shape, TypeShape::Opaque("Int".to_string()));
    }

    #[test]
    fn shape_of_coproduct() {
        let encoding_dag = type_lib::coproduct(
            "ContentEncoding",
            vec![
                ("UTF8", "String"),
                ("ASCII", "String"),
                ("Binary", "Bytes"),
            ],
        );
        let shape = type_shape(&encoding_dag);
        match &shape {
            TypeShape::Coproduct(variants) => {
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].0, "UTF8");
                assert_eq!(variants[0].1, TypeShape::Opaque("String".to_string()));
                assert_eq!(variants[1].0, "ASCII");
                assert_eq!(variants[2].0, "Binary");
                assert_eq!(variants[2].1, TypeShape::Opaque("Bytes".to_string()));
            }
            other => panic!("expected Coproduct, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_unit_coproduct_bool() {
        let bool_dag = type_lib::coproduct("Bool", vec![("True", "Unit"), ("False", "Unit")]);
        let shape = type_shape(&bool_dag);
        match &shape {
            TypeShape::Coproduct(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0], ("True".to_string(), TypeShape::Opaque("Unit".to_string())));
                assert_eq!(variants[1], ("False".to_string(), TypeShape::Opaque("Unit".to_string())));
            }
            other => panic!("expected Coproduct, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_product() {
        let cli_result_dag = type_lib::product(
            "CliResult",
            vec![
                ("stdout", "String"),
                ("stderr", "String"),
                ("exit_code", "Int"),
            ],
        );
        let shape = type_shape(&cli_result_dag);
        match &shape {
            TypeShape::Product(fields) => {
                assert_eq!(fields.len(), 3);
                assert_eq!(fields[0].0, "stdout");
                assert_eq!(fields[0].1, TypeShape::Opaque("String".to_string()));
                assert_eq!(fields[2].0, "exit_code");
                assert_eq!(fields[2].1, TypeShape::Opaque("Int".to_string()));
            }
            other => panic!("expected Product, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_brand() {
        let branded_dag = type_lib::branded("TextFilePath", type_lib::string());
        let shape = type_shape(&branded_dag);
        match &shape {
            TypeShape::Brand(name, inner) => {
                assert_eq!(name, "TextFilePath");
                // Inner is the string identity, extracted from the SubDag.
                assert_eq!(**inner, TypeShape::Opaque("String".to_string()));
            }
            other => panic!("expected Brand, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_optional() {
        let opt_dag = type_lib::optional(type_lib::string());
        let shape = type_shape(&opt_dag);
        match &shape {
            TypeShape::Container(ContainerShape::Optional(inner)) => {
                assert_eq!(**inner, TypeShape::Opaque("String".to_string()));
            }
            other => panic!("expected Container(Optional(...)), got {:?}", other),
        }
    }

    #[test]
    fn shape_of_list() {
        let list_dag = type_lib::list(type_lib::int());
        let shape = type_shape(&list_dag);
        match &shape {
            TypeShape::Container(ContainerShape::List(inner)) => {
                assert_eq!(**inner, TypeShape::Opaque("Int".to_string()));
            }
            other => panic!("expected Container(List(...)), got {:?}", other),
        }
    }

    #[test]
    fn shape_of_set() {
        let set_dag = type_lib::set(type_lib::string());
        let shape = type_shape(&set_dag);
        match &shape {
            TypeShape::Container(ContainerShape::Set(inner)) => {
                assert_eq!(**inner, TypeShape::Opaque("String".to_string()));
            }
            other => panic!("expected Container(Set(...)), got {:?}", other),
        }
    }

    #[test]
    fn shape_of_map() {
        let map_dag = type_lib::map(type_lib::int());
        let shape = type_shape(&map_dag);
        match &shape {
            TypeShape::Container(ContainerShape::Map(key, value)) => {
                assert_eq!(**key, TypeShape::Opaque("String".to_string()));
                assert_eq!(**value, TypeShape::Opaque("Int".to_string()));
            }
            other => panic!("expected Container(Map(...)), got {:?}", other),
        }
    }

    #[test]
    fn shape_of_non_empty_list() {
        let ne_list_dag = type_lib::non_empty_list(type_lib::string());
        let shape = type_shape(&ne_list_dag);
        // NonEmptyList is structurally a List in TypeShape.
        match &shape {
            TypeShape::Container(ContainerShape::List(inner)) => {
                assert_eq!(**inner, TypeShape::Opaque("String".to_string()));
            }
            other => panic!("expected Container(List(...)), got {:?}", other),
        }
    }

    // =========================================================================
    // PlatformRepr-annotated type DAGs
    // =========================================================================

    /// Build a type DAG with a PlatformRepr metadata node (as the design
    /// envisions for Phase 2). This tests that type_shape() correctly
    /// classifies such DAGs as TypeShape::Platform.
    fn build_platform_dag(type_name: &str, repr: PlatformRepr) -> Dag<TypeOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "repr",
            vec![Port::scalar("in", type_name)],
            vec![Port::scalar("out", type_name)],
            TypeOp::Meta(MetadataPayload::PlatformRepr(repr)),
        ));
        dag
    }

    #[test]
    fn shape_of_platform_int64() {
        let dag = build_platform_dag(
            "Int64",
            PlatformRepr {
                bits: 64,
                signed: true,
                float: false,
                discrete: true,
            },
        );
        let shape = type_shape(&dag);
        match &shape {
            TypeShape::Platform(repr) => {
                assert_eq!(repr.bits, 64);
                assert!(repr.signed);
                assert!(!repr.float);
                assert!(repr.discrete);
            }
            other => panic!("expected Platform, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_platform_float64() {
        let dag = build_platform_dag(
            "Float64",
            PlatformRepr {
                bits: 64,
                signed: true,
                float: true,
                discrete: false,
            },
        );
        let shape = type_shape(&dag);
        match &shape {
            TypeShape::Platform(repr) => {
                assert_eq!(repr.bits, 64);
                assert!(repr.signed);
                assert!(repr.float);
                assert!(!repr.discrete);
            }
            other => panic!("expected Platform, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_platform_uint8() {
        let dag = build_platform_dag(
            "UInt8",
            PlatformRepr {
                bits: 8,
                signed: false,
                float: false,
                discrete: true,
            },
        );
        let shape = type_shape(&dag);
        match &shape {
            TypeShape::Platform(repr) => {
                assert_eq!(repr.bits, 8);
                assert!(!repr.signed);
                assert!(!repr.float);
                assert!(repr.discrete);
            }
            other => panic!("expected Platform, got {:?}", other),
        }
    }

    // =========================================================================
    // PlatformRepr takes priority over Identity
    // =========================================================================

    #[test]
    fn platform_repr_takes_priority_over_identity() {
        // A DAG with both an Identity node and a PlatformRepr Meta node
        // should classify as Platform, not Opaque.
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "identity",
            vec![Port::scalar("in", "Int64")],
            vec![Port::scalar("out", "Int64")],
            TypeOp::Identity,
        ));
        dag.add_node(Node::opaque(
            "repr",
            vec![Port::scalar("in", "Int64")],
            vec![Port::scalar("out", "Int64")],
            TypeOp::Meta(MetadataPayload::PlatformRepr(PlatformRepr {
                bits: 64,
                signed: true,
                float: false,
                discrete: true,
            })),
        ));
        dag.add_edge(Edge::new("identity", "out", "repr", "in"));

        let shape = type_shape(&dag);
        assert!(
            matches!(shape, TypeShape::Platform(_)),
            "PlatformRepr metadata should take priority over Identity"
        );
    }

    // =========================================================================
    // Opaque for unknown/empty DAGs
    // =========================================================================

    #[test]
    fn shape_of_empty_dag_is_opaque_unknown() {
        let dag: Dag<TypeOp> = Dag::new();
        let shape = type_shape(&dag);
        assert_eq!(shape, TypeShape::Opaque("Unknown".to_string()));
    }

    // =========================================================================
    // Refined types (with validation predicates) are Opaque
    // =========================================================================

    #[test]
    fn shape_of_refined_type_is_opaque() {
        // Refined types (e.g., Url = String + NonEmpty + Matches) have
        // an Identity node as their root, so they classify as Opaque.
        // This is correct for Phase 0 — backends treat them like their
        // base type name.
        let url_dag = type_lib::url();
        let shape = type_shape(&url_dag);
        assert_eq!(shape, TypeShape::Opaque("String".to_string()));
    }

    // =========================================================================
    // Nested containers
    // =========================================================================

    #[test]
    fn shape_of_optional_list_of_string() {
        let dag = type_lib::optional(type_lib::list(type_lib::string()));
        let shape = type_shape(&dag);
        match &shape {
            TypeShape::Container(ContainerShape::Optional(inner)) => {
                match inner.as_ref() {
                    TypeShape::Container(ContainerShape::List(elem)) => {
                        assert_eq!(**elem, TypeShape::Opaque("String".to_string()));
                    }
                    other => panic!("expected inner List, got {:?}", other),
                }
            }
            other => panic!("expected Optional, got {:?}", other),
        }
    }
}
