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
//! Platform primitives are detected by walking the type DAG for structural
//! predicates (Width, Signed, Domain, etc.) from Validate nodes.

use crate::dag::Dag;
use crate::node::NodeBody;
use crate::type_op::{Predicate, TypeOp, WrapperKind};

/// Structural classification derived from a type DAG.
///
/// Each variant represents an algebraic shape that backends can
/// pattern-match on to derive their native representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeShape {
    /// Platform primitive with structural properties derived from DAG predicates.
    ///
    /// Carries width, signedness, domain, and other properties extracted from
    /// `Validate(Width/Signed/Domain/...)` nodes in the type DAG. Backends
    /// derive their native integer/float type from these properties.
    Platform(StructuralProperties),

    /// Coproduct (tagged union) with named variants.
    ///
    /// Each variant has a name and a recursive `TypeShape` for its payload.
    /// A coproduct where all variants have `TypeShape::Opaque("Unit")` is
    /// an all-unit enum (e.g., Bool, HttpMethod).
    ///
    /// The optional `String` is the declared type name (e.g., `"ContentEncoding"`).
    /// `None` for anonymous coproducts.
    Coproduct(Option<String>, Vec<(String, TypeShape)>),

    /// Product (record) with named fields.
    ///
    /// Each field has a name and a recursive `TypeShape` for its type.
    ///
    /// The optional `String` is the declared type name (e.g., `"CliResult"`).
    /// `None` for anonymous records.
    Product(Option<String>, Vec<(String, TypeShape)>),

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

/// Structural platform properties derived from type DAG predicates.
///
/// Extracted from `Validate(Width/Signed/Domain/...)` nodes in the type DAG.
/// Backends use these to derive their native integer/float/byte types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructuralProperties {
    /// Bit width of the type (from `Width` predicate).
    pub width: Option<u16>,
    /// Signedness (from `Signed`/`Unsigned` predicates).
    pub signed: Option<bool>,
    /// Domain hint (from `Domain` predicate, e.g. "ieee754_binary32").
    pub domain: Option<String>,
    /// Whether arithmetic operations are valid (from `Arithmetic` predicate).
    pub arithmetic: bool,
    /// Collection length (from `Length` predicate).
    pub length: Option<u64>,
}

/// Extract the structural shape from a type DAG.
///
/// Walks the `Dag<TypeOp>`, classifying by root node's `TypeOp` variant:
///
/// - Structural predicates (Width/Signed/Domain) => `TypeShape::Platform(StructuralProperties)`
/// - `TypeOp::Coproduct(variants)` => `TypeShape::Coproduct(...)`
/// - `TypeOp::Product(fields)` => `TypeShape::Product(...)`
/// - `TypeOp::Brand(name)` => `TypeShape::Brand(name, inner_shape)`
/// - `TypeOp::Wrap(kind)` with inner SubDag => `TypeShape::Container(...)`
/// - `TypeOp::Identity` for simple identity types => `TypeShape::Opaque(type_name)`
///
/// The extractor does NOT resolve type references through the registry.
/// Each variant's inner type is classified as `Opaque(type_id)` unless
/// the type DAG itself carries structural information (e.g., a SubDag).
pub fn type_shape(dag: &Dag<TypeOp>) -> TypeShape {
    // Priority 1: Look for Coproduct node. Products and coproducts take
    // priority over platform detection because their field SubDags may
    // contain structural predicates that shouldn't classify the compound
    // type itself as a platform primitive.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Coproduct(variants)) = &node.body {
            // Extract the type name from the node's output port type_id.
            let type_name = node
                .outputs
                .first()
                .map(|p| p.type_id.0.clone());
            let shaped_variants: Vec<(String, TypeShape)> = variants
                .iter()
                .map(|name| {
                    let child_id = format!("variant_{name}");
                    let inner = named_subdag(dag, &child_id)
                        .map(type_shape)
                        .unwrap_or_else(|| TypeShape::Opaque(name.clone()));
                    (name.clone(), inner)
                })
                .collect();
            return TypeShape::Coproduct(type_name, shaped_variants);
        }
    }

    // Priority 2: Look for Product node.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Product(fields)) = &node.body {
            // Extract the type name from the node's output port type_id.
            let type_name = node
                .outputs
                .first()
                .map(|p| p.type_id.0.clone());
            let shaped_fields: Vec<(String, TypeShape)> = fields
                .iter()
                .map(|name| {
                    let child_id = format!("field_{name}");
                    let inner = named_subdag(dag, &child_id)
                        .map(type_shape)
                        .unwrap_or_else(|| TypeShape::Opaque(name.clone()));
                    (name.clone(), inner)
                })
                .collect();
            return TypeShape::Product(type_name, shaped_fields);
        }
    }

    // Priority 3: Look for Brand node.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Brand(name)) = &node.body {
            let inner_shape = inner_subdag(dag)
                .map(type_shape)
                .unwrap_or_else(|| TypeShape::Opaque(name.clone()));
            return TypeShape::Brand(name.clone(), Box::new(inner_shape));
        }
    }

    // Priority 4: Look for Wrap (Container) node. Checked before Platform
    // because container SubDags may contain structural predicates (e.g.,
    // Optional<Int> has Width/Signed from the Int element) that shouldn't
    // classify the container itself as a platform primitive.
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
                WrapperKind::Map => {
                    let key_shape = named_subdag(dag, "key_type")
                        .map(type_shape)
                        .unwrap_or_else(|| TypeShape::Opaque("String".to_string()));
                    let value_shape = named_subdag(dag, "value_type")
                        .map(type_shape)
                        .unwrap_or(inner_shape);
                    TypeShape::Container(ContainerShape::Map(
                        Box::new(key_shape),
                        Box::new(value_shape),
                    ))
                }
            };
        }
    }

    // Priority 5: Look for structural predicates (Width, Signed, Domain, etc.)
    // that indicate a platform primitive type. Checked after Product/Coproduct,
    // Brand, and Container because their SubDags may contain predicates that
    // shouldn't classify the compound type as a platform primitive.
    let props = derive_structural_properties(dag);
    if props.width.is_some() || props.signed.is_some() || props.domain.is_some() {
        return TypeShape::Platform(props);
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
    eprintln!("warning: unknown type shape for dag with {} node(s), using Opaque", dag.nodes.len());
    TypeShape::Opaque("Unknown".to_string())
}

/// Walk a type DAG and extract structural properties from Validate predicates.
///
/// Recurses into SubDag children, inheriting properties from inner type DAGs
/// when the current level doesn't specify them. This ensures that refined
/// aliases (e.g., `Float32 = Word32 + Domain(ieee754)`) correctly inherit
/// base properties like width and signedness.
pub fn derive_structural_properties(dag: &Dag<TypeOp>) -> StructuralProperties {
    let mut props = StructuralProperties::default();

    // Step 1: Collect explicit predicates on this DAG's own Validate nodes.
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Validate(pred)) = &node.body {
            collect_predicate(&mut props, pred);
        }
    }

    // Step 2: Compositional width derivation for Product types.
    // Must run BEFORE SubDag recursion — otherwise recursion inherits the
    // element width (e.g., Bit's width=1) rather than the composed width
    // (e.g., Byte's width=8×1=8).
    if props.width.is_none() {
        if let Some(width) = derive_compositional_width(dag) {
            props.width = Some(width);
        }
    }

    // Step 3: Recurse into SubDags — inherit missing properties from children.
    for node in &dag.nodes {
        if let NodeBody::SubDag(subdag, _) = &node.body {
            let inner = derive_structural_properties(subdag);
            if props.width.is_none() {
                props.width = inner.width;
            }
            if props.signed.is_none() {
                props.signed = inner.signed;
            }
            if !props.arithmetic {
                props.arithmetic = inner.arithmetic;
            }
            if props.domain.is_none() {
                props.domain = inner.domain;
            }
            if props.length.is_none() {
                props.length = inner.length;
            }
        }
    }

    props
}

/// Derive width compositionally from Product→List→element structure.
///
/// Walks Product nodes with exactly one field. If that field is (or contains)
/// a List container with a `Length(N)` predicate, and the list element has
/// known width `W`, the composed width is `N * W`.
///
/// Handles the `refined_with_base` pattern where the List is nested inside
/// a `base_type` SubDag with `Validate(Length(N))` at the outer level.
fn derive_compositional_width(dag: &Dag<TypeOp>) -> Option<u16> {
    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Product(fields)) = &node.body {
            if fields.len() != 1 {
                continue;
            }
            let field_name = &fields[0];
            let child_id = format!("field_{field_name}");
            let field_dag = named_subdag(dag, &child_id)?;

            if let Some(width) = extract_list_composed_width(field_dag) {
                return Some(width);
            }
        }
    }
    None
}

/// Extract composed width from a DAG that represents a constrained list.
///
/// Handles two layouts:
/// 1. Direct: `Wrap(List)` + `Validate(Length(N))` + element SubDag at same level
/// 2. Refined: `SubDag("base_type", list_dag)` + `Validate(Length(N))` at outer level
///    (produced by `refined_with_base(list_dag, [Length(N)])`)
fn extract_list_composed_width(dag: &Dag<TypeOp>) -> Option<u16> {
    let mut is_list = false;
    let mut list_length: Option<u64> = None;
    let mut element_dag: Option<&Dag<TypeOp>> = None;

    for fnode in &dag.nodes {
        match &fnode.body {
            NodeBody::Opaque(TypeOp::Wrap(WrapperKind::List)) => {
                is_list = true;
            }
            NodeBody::Opaque(TypeOp::Validate(Predicate::Length(l))) => {
                list_length = Some(*l);
            }
            _ => {}
        }
    }

    if is_list {
        element_dag = inner_subdag(dag);
    } else {
        // Check for refined_with_base pattern: base_type SubDag contains the list
        if let Some(base) = named_subdag(dag, "base_type") {
            for bnode in &base.nodes {
                if let NodeBody::Opaque(TypeOp::Wrap(WrapperKind::List)) = &bnode.body {
                    is_list = true;
                }
            }
            if is_list {
                element_dag = inner_subdag(base);
                if list_length.is_none() {
                    for fnode in &dag.nodes {
                        if let NodeBody::Opaque(TypeOp::Validate(Predicate::Length(l))) =
                            &fnode.body
                        {
                            list_length = Some(*l);
                        }
                    }
                }
            }
        }
    }

    if !is_list {
        return None;
    }
    let length = list_length?;
    let elem_props = derive_structural_properties(element_dag?);
    let elem_width = elem_props.width?;

    Some((length as u16) * elem_width)
}

/// Collect a single predicate into structural properties.
fn collect_predicate(props: &mut StructuralProperties, pred: &Predicate) {
    match pred {
        Predicate::Width(w) => props.width = Some(*w),
        Predicate::Signed(_) => props.signed = Some(true),
        Predicate::Unsigned => props.signed = Some(false),
        Predicate::Domain(d) => props.domain = Some(d.clone()),
        Predicate::Arithmetic => props.arithmetic = true,
        Predicate::Length(l) => props.length = Some(*l),
        Predicate::And(preds) => {
            for p in preds {
                collect_predicate(props, p);
            }
        }
        _ => {}
    }
}

/// Find the first SubDag node in a type DAG and return a reference to its inner DAG.
fn inner_subdag(dag: &Dag<TypeOp>) -> Option<&Dag<TypeOp>> {
    dag.nodes.iter().find_map(|node| {
        if let NodeBody::SubDag(subdag, _) = &node.body {
            Some(subdag)
        } else {
            None
        }
    })
}

/// Find a SubDag child by node ID name.
fn named_subdag<'a>(dag: &'a Dag<TypeOp>, name: &str) -> Option<&'a Dag<TypeOp>> {
    dag.nodes.iter().find_map(|node| {
        if node.id.0 == name {
            if let NodeBody::SubDag(subdag, _) = &node.body {
                return Some(subdag);
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::Port;
    use crate::node::Node;
    use crate::type_lib;

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
            vec![("UTF8", "String"), ("ASCII", "String"), ("Binary", "Bytes")],
        );
        let shape = type_shape(&encoding_dag);
        match &shape {
            TypeShape::Coproduct(type_name, variants) => {
                assert_eq!(type_name.as_deref(), Some("ContentEncoding"));
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
            TypeShape::Coproduct(type_name, variants) => {
                assert_eq!(type_name.as_deref(), Some("Bool"));
                assert_eq!(variants.len(), 2);
                assert_eq!(
                    variants[0],
                    ("True".to_string(), TypeShape::Opaque("Unit".to_string()))
                );
                assert_eq!(
                    variants[1],
                    ("False".to_string(), TypeShape::Opaque("Unit".to_string()))
                );
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
            TypeShape::Product(type_name, fields) => {
                assert_eq!(type_name.as_deref(), Some("CliResult"));
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
        let map_dag = type_lib::map(type_lib::string(), type_lib::int());
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
    // Structural predicate-based platform type DAGs
    // =========================================================================

    /// Build a type DAG with structural predicates for platform type detection.
    fn build_predicate_platform_dag(type_name: &str, predicates: Vec<Predicate>) -> Dag<TypeOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "identity",
            vec![Port::scalar("in", type_name)],
            vec![Port::scalar("out", type_name)],
            TypeOp::Identity,
        ));
        for (i, pred) in predicates.into_iter().enumerate() {
            let id = format!("validate_{i}");
            dag.add_node(Node::opaque(
                id.as_str(),
                vec![Port::scalar("in", type_name)],
                vec![Port::scalar("out", type_name)],
                TypeOp::Validate(pred),
            ));
        }
        dag
    }

    #[test]
    fn shape_of_platform_int64_from_predicates() {
        let dag = build_predicate_platform_dag(
            "Int64",
            vec![Predicate::Width(64), Predicate::Signed(None)],
        );
        let shape = type_shape(&dag);
        match &shape {
            TypeShape::Platform(props) => {
                assert_eq!(props.width, Some(64));
                assert_eq!(props.signed, Some(true));
                assert_eq!(props.domain, None);
            }
            other => panic!("expected Platform, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_platform_float64_from_predicates() {
        let dag = build_predicate_platform_dag(
            "Float64",
            vec![
                Predicate::Width(64),
                Predicate::Signed(None),
                Predicate::Domain("ieee754_binary64".to_string()),
            ],
        );
        let shape = type_shape(&dag);
        match &shape {
            TypeShape::Platform(props) => {
                assert_eq!(props.width, Some(64));
                assert_eq!(props.signed, Some(true));
                assert_eq!(props.domain, Some("ieee754_binary64".to_string()));
            }
            other => panic!("expected Platform, got {:?}", other),
        }
    }

    #[test]
    fn shape_of_platform_uint8_from_predicates() {
        let dag = build_predicate_platform_dag(
            "UInt8",
            vec![Predicate::Width(8), Predicate::Unsigned],
        );
        let shape = type_shape(&dag);
        match &shape {
            TypeShape::Platform(props) => {
                assert_eq!(props.width, Some(8));
                assert_eq!(props.signed, Some(false));
            }
            other => panic!("expected Platform, got {:?}", other),
        }
    }

    #[test]
    fn structural_predicates_take_priority_over_identity() {
        // A DAG with both an Identity node and Validate(Width) node
        // should classify as Platform, not Opaque.
        let dag = build_predicate_platform_dag(
            "Int64",
            vec![Predicate::Width(64), Predicate::Signed(None)],
        );
        let shape = type_shape(&dag);
        assert!(
            matches!(shape, TypeShape::Platform(_)),
            "Structural predicates should take priority over Identity"
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
            TypeShape::Container(ContainerShape::Optional(inner)) => match inner.as_ref() {
                TypeShape::Container(ContainerShape::List(elem)) => {
                    assert_eq!(**elem, TypeShape::Opaque("String".to_string()));
                }
                other => panic!("expected inner List, got {:?}", other),
            },
            other => panic!("expected Optional, got {:?}", other),
        }
    }

    // =========================================================================
    // Resolved product/coproduct — structural recursion through fields
    // =========================================================================

    #[test]
    fn resolved_product_fields_carry_structural_shape() {
        // A product with resolved field DAGs should expose field shapes,
        // not Opaque wrappers.
        let int64_dag = {
            let mut dag = Dag::new();
            dag.add_node(Node::opaque(
                "identity",
                vec![Port::scalar("in", "Int64")],
                vec![Port::scalar("out", "Int64")],
                TypeOp::Identity,
            ));
            dag.add_node(Node::opaque(
                "validate_0",
                vec![Port::scalar("in", "Int64")],
                vec![Port::scalar("out", "Int64")],
                TypeOp::Validate(Predicate::Width(64)),
            ));
            dag.add_node(Node::opaque(
                "validate_1",
                vec![Port::scalar("in", "Int64")],
                vec![Port::scalar("out", "Int64")],
                TypeOp::Validate(Predicate::Signed(None)),
            ));
            dag
        };
        let product_dag = type_lib::product_resolved(
            "CliResult",
            vec![
                ("stdout", type_lib::string()),
                ("exit_code", int64_dag),
            ],
        );
        let shape = type_shape(&product_dag);
        match &shape {
            TypeShape::Product(type_name, fields) => {
                assert_eq!(type_name.as_deref(), Some("CliResult"));
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "stdout");
                assert_eq!(fields[0].1, TypeShape::Opaque("String".to_string()));
                assert_eq!(fields[1].0, "exit_code");
                // With resolved DAGs, the field carries structural info.
                match &fields[1].1 {
                    TypeShape::Platform(props) => {
                        assert_eq!(props.width, Some(64));
                        assert_eq!(props.signed, Some(true));
                    }
                    other => panic!("expected Platform for exit_code, got {:?}", other),
                }
            }
            other => panic!("expected Product, got {:?}", other),
        }
    }

    #[test]
    fn derive_structural_properties_recurses_into_subdags() {
        // Build a refined type with base SubDag: Int + Width(32) + Signed
        let base_dag = {
            let mut dag = Dag::new();
            dag.add_node(Node::opaque(
                "identity",
                vec![Port::scalar("in", "Int")],
                vec![Port::scalar("out", "Int")],
                TypeOp::Identity,
            ));
            dag.add_node(Node::opaque(
                "validate_0",
                vec![Port::scalar("in", "Int")],
                vec![Port::scalar("out", "Int")],
                TypeOp::Validate(Predicate::Width(32)),
            ));
            dag.add_node(Node::opaque(
                "validate_1",
                vec![Port::scalar("in", "Int")],
                vec![Port::scalar("out", "Int")],
                TypeOp::Validate(Predicate::Signed(None)),
            ));
            dag.add_node(Node::opaque(
                "validate_2",
                vec![Port::scalar("in", "Int")],
                vec![Port::scalar("out", "Int")],
                TypeOp::Validate(Predicate::Arithmetic),
            ));
            dag
        };
        // Outer DAG wraps base in a SubDag with additional Domain predicate
        let outer = type_lib::refined_with_base(
            "Float32",
            base_dag,
            vec![Predicate::Domain("ieee754_binary32".to_string())],
        );
        let props = derive_structural_properties(&outer);
        // Width and Signed should be inherited from the SubDag
        assert_eq!(props.width, Some(32));
        assert_eq!(props.signed, Some(true));
        assert!(props.arithmetic);
        assert_eq!(props.domain.as_deref(), Some("ieee754_binary32"));
    }

    // =========================================================================
    // Phase A2: Container refinement predicates propagate
    // =========================================================================

    #[test]
    fn container_refinement_length_propagates() {
        let bit_dag = type_lib::refined("Bit", vec![Predicate::Width(1)]);
        let list_bit = type_lib::list(bit_dag);
        let list_bit_len8 = type_lib::refined_with_base("Byte.bits", list_bit, vec![Predicate::Length(8)]);

        let props = derive_structural_properties(&list_bit_len8);
        assert_eq!(props.length, Some(8), "Length(8) predicate should propagate through container refinement");
        assert_eq!(props.width, Some(1), "Inner element width should be inherited from SubDag");
    }

    // =========================================================================
    // Phase B: Compositional width derivation
    // =========================================================================

    #[test]
    fn compositional_width_byte_from_8_bits() {
        let bit_dag = type_lib::refined("Bit", vec![Predicate::Width(1)]);
        let list_bit_len8 = type_lib::refined_with_base(
            "Byte.bits",
            type_lib::list(bit_dag),
            vec![Predicate::Length(8)],
        );
        let byte_dag = type_lib::product_resolved("Byte", vec![("bits", list_bit_len8)]);

        let props = derive_structural_properties(&byte_dag);
        assert_eq!(props.width, Some(8), "Byte = Product(bits: List<Bit> where length(8)) should derive width 8×1=8");
    }

    #[test]
    fn compositional_width_word32_from_4_bytes() {
        let bit_dag = type_lib::refined("Bit", vec![Predicate::Width(1)]);
        let byte_bits = type_lib::refined_with_base(
            "Byte.bits",
            type_lib::list(bit_dag),
            vec![Predicate::Length(8)],
        );
        let byte_dag = type_lib::product_resolved("Byte", vec![("bits", byte_bits)]);
        let list_byte_len4 = type_lib::refined_with_base(
            "Word32.bytes",
            type_lib::list(byte_dag),
            vec![Predicate::Length(4)],
        );
        let word32_dag = type_lib::product_resolved("Word32", vec![("bytes", list_byte_len4)]);

        let props = derive_structural_properties(&word32_dag);
        assert_eq!(props.width, Some(32), "Word32 = Product(bytes: List<Byte> where length(4)) should derive width 4×8=32");
    }

    #[test]
    fn width_inheritance_through_alias() {
        let bit_dag = type_lib::refined("Bit", vec![Predicate::Width(1)]);
        let byte_bits = type_lib::refined_with_base(
            "Byte.bits",
            type_lib::list(bit_dag),
            vec![Predicate::Length(8)],
        );
        let byte_dag = type_lib::product_resolved("Byte", vec![("bits", byte_bits)]);
        let uint8_dag = type_lib::refined_with_base(
            "UInt8",
            byte_dag,
            vec![Predicate::Unsigned, Predicate::Arithmetic],
        );

        let props = derive_structural_properties(&uint8_dag);
        assert_eq!(props.width, Some(8), "UInt8 = Byte where unsigned, arithmetic should inherit width 8 from Byte");
        assert_eq!(props.signed, Some(false));
        assert!(props.arithmetic);
    }
}
