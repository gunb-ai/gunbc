//! Type operations for type DAGs.
//!
//! Types are DAGs (`Dag<TypeOp>`) that describe how values are validated and transformed.
//! This unifies types with workflows — same infrastructure, same composition rules.
//!
//! # Philosophy: Types as Causal Chains
//!
//! A type like `Url` is a causal chain:
//! ```text
//! String (raw) → [NonEmpty check] → [URL pattern check] → Url (validated)
//! ```
//!
//! This is not an analogy — type validation IS a causal chain.
//! Using `Dag<TypeOp>` makes this explicit and reuses all DAG infrastructure.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::type_lib;
//!
//! // Types are just Dag<TypeOp>
//! let url_type = type_lib::url();
//! let optional_url = type_lib::optional(url_type);
//!
//! // Cardinality emerges from structure
//! // optional_url output port has ZeroOrOne cardinality
//! ```

use crate::types::TypeId;
use serde::{Deserialize, Serialize};

/// Operations in a type DAG.
///
/// TypeOp is the operation type for type DAGs, analogous to how
/// `GistOp`, `DepsOp`, etc. are operation types for workflow DAGs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeOp {
    /// Identity operation — pass value through unchanged.
    /// Used as the "base" node in type DAGs.
    Identity,

    /// Validation predicate — checks a condition on the value.
    /// If validation fails, the type DAG produces an error.
    Validate(Predicate),

    /// Inert metadata payload (non-semantic, non-failing).
    ///
    /// `Meta` is traversable/inspectable but must not change runtime behavior.
    Meta(MetadataPayload),

    /// Transformation — coerces value from one base type to another.
    /// Used for type conversions (e.g., String → Int parsing).
    Transform(Coercion),

    /// Wrapper operation — wraps a value in a container type.
    /// Used for Optional<T>, List<T>, etc.
    Wrap(WrapperKind),

    /// Unwrap operation — extracts value from a container type.
    Unwrap(WrapperKind),

    /// Product type — a record with named typed fields.
    /// e.g., `{ path: FilePath, encoding: ContentEncoding }`
    Product(Vec<(String, TypeId)>),

    /// Coproduct type — a tagged union of named typed variants.
    /// e.g., `UTF8 | ASCII | Latin1 | Binary`
    Coproduct(Vec<(String, TypeId)>),

    /// Brand (nominal) type — a named wrapper around an inner type with refinement.
    /// e.g., `TextFilePath = FilePath @content(Text)`
    Brand(String, TypeId),
}

/// Typed inert metadata payload carried by [`TypeOp::Meta`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetadataPayload {
    SystemId(String),
    SystemKind(String),
    BehaviorId(String),
    Invocation(String),
    Property(String),
    InputContract {
        name: String,
        type_id: String,
        required: bool,
    },
    OutputContract {
        name: String,
        type_id: String,
    },
}

/// Content encoding lattice for file content classification.
///
/// Models the encoding hierarchy from `types.dag`:
/// ```text
/// Unknown (⊤)
///   ├── Text
///   │   ├── UTF8
///   │   │   └── ASCII
///   │   └── Latin1
///   └── Binary (⊥ of binary branch)
/// ```
///
/// `ASCII ⊆ UTF8 ⊆ Text` — a function expecting Text content accepts UTF8.
/// `Binary` and `Text` are incomparable — a function expecting Text rejects Binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentEncoding {
    /// Unknown encoding (top of lattice — accepts anything).
    Unknown,
    /// Text content (any text encoding).
    Text,
    /// UTF-8 encoded text (subset of Text).
    UTF8,
    /// ASCII encoded text (subset of UTF8).
    ASCII,
    /// Latin-1 encoded text (subset of Text, incomparable with UTF8).
    Latin1,
    /// Binary content (not text — incomparable with Text subtypes).
    Binary,
}

impl ContentEncoding {
    /// Check if `self` is a subtype of `other` in the encoding lattice.
    ///
    /// `self.is_subtype_of(other)` means: any content with encoding `self`
    /// is also valid content with encoding `other`.
    pub fn is_subtype_of(&self, other: &ContentEncoding) -> bool {
        if self == other {
            return true;
        }
        match (self, other) {
            // Everything is a subtype of Unknown (top).
            (_, ContentEncoding::Unknown) => true,
            // ASCII ⊆ UTF8 ⊆ Text
            (ContentEncoding::ASCII, ContentEncoding::UTF8 | ContentEncoding::Text) => true,
            (ContentEncoding::UTF8, ContentEncoding::Text) => true,
            // Latin1 ⊆ Text
            (ContentEncoding::Latin1, ContentEncoding::Text) => true,
            _ => false,
        }
    }
}

/// Predicates that can be validated against values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
    /// Value must be non-empty (non-empty string, non-empty list, etc.)
    NonEmpty,

    /// String must match a regex pattern.
    Matches(String),

    /// Numeric value must be in range [min, max].
    InRange { min: i64, max: i64 },

    /// All elements of a collection must satisfy a predicate.
    All(Box<Predicate>),

    /// Any element of a collection must satisfy a predicate.
    Any(Box<Predicate>),

    /// Value must equal a specific value.
    Equals(PredicateValue),

    /// Logical AND of multiple predicates.
    And(Vec<Predicate>),

    /// Logical OR of multiple predicates.
    Or(Vec<Predicate>),

    /// Logical NOT of a predicate.
    Not(Box<Predicate>),

    /// Custom named predicate (resolved at runtime).
    Custom(String),

    /// Content encoding constraint from `@content` annotations.
    /// e.g., `@content(UTF8)` → `Predicate::Content(ContentEncoding::UTF8)`
    Content(ContentEncoding),
}

/// Simple values that can appear in predicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PredicateValue {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl Predicate {
    /// Check whether this predicate entails another (i.e., is at least as strict).
    ///
    /// This is intentionally conservative: only provable entailments return true.
    pub fn entails(&self, other: &Predicate) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            (Predicate::And(_), Predicate::And(targets)) => targets.iter().all(|t| self.entails(t)),
            (Predicate::And(preds), target) => preds.iter().any(|p| p.entails(target)),
            (source, Predicate::And(targets)) => targets.iter().all(|t| source.entails(t)),

            (Predicate::Or(preds), target) => preds.iter().all(|p| p.entails(target)),
            (source, Predicate::Or(targets)) => targets.iter().any(|t| source.entails(t)),

            (Predicate::All(a), Predicate::All(b)) => a.entails(b),
            (Predicate::Any(a), Predicate::Any(b)) => a.entails(b),
            (Predicate::Not(a), Predicate::Not(b)) => a.entails(b),

            (
                Predicate::InRange { min, max },
                Predicate::InRange {
                    min: target_min,
                    max: target_max,
                },
            ) => min >= target_min && max <= target_max,
            (Predicate::Equals(PredicateValue::Int(v)), Predicate::InRange { min, max }) => {
                v >= min && v <= max
            }
            (Predicate::Equals(a), Predicate::Equals(b)) => a == b,

            // Content encoding subtyping: Content(ASCII).entails(Content(UTF8)) = true
            (Predicate::Content(a), Predicate::Content(b)) => a.is_subtype_of(b),

            _ => false,
        }
    }
}

/// Coercion between base types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coercion {
    /// Source base type.
    pub from: BaseType,
    /// Target base type.
    pub to: BaseType,
}

impl Coercion {
    /// Create a new coercion.
    pub fn new(from: BaseType, to: BaseType) -> Self {
        Self { from, to }
    }
}

/// Base types — the fundamental shapes of data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseType {
    /// Unit type (no data).
    Unit,
    /// Boolean.
    Bool,
    /// Integer.
    Int,
    /// Floating point.
    Float,
    /// String.
    String,
    /// Raw bytes.
    Bytes,
    /// JSON value (dynamic).
    Json,
    /// Secret (redacted string).
    Secret,
    /// List of elements.
    List(Box<BaseType>),
    /// Optional value (may be absent).
    Option(Box<BaseType>),
    /// Map from keys to values.
    Map(Box<BaseType>, Box<BaseType>),
    /// Named/opaque type (user-defined or external).
    Named(String),
}

impl BaseType {
    /// Create a list type.
    pub fn list(element: BaseType) -> Self {
        BaseType::List(Box::new(element))
    }

    /// Create an optional type.
    pub fn option(inner: BaseType) -> Self {
        BaseType::Option(Box::new(inner))
    }

    /// Create a map type.
    pub fn map(key: BaseType, value: BaseType) -> Self {
        BaseType::Map(Box::new(key), Box::new(value))
    }

    /// Create a named type.
    pub fn named(name: impl Into<String>) -> Self {
        BaseType::Named(name.into())
    }
}

/// Wrapper kinds for container types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WrapperKind {
    /// Optional wrapper (T → Option<T>).
    Optional,
    /// List wrapper (T → List<T>).
    List,
    /// Non-empty list wrapper (T → NonEmptyList<T>).
    NonEmptyList,
    /// Set wrapper (T → Set<T>) — unordered, unique elements.
    Set,
    /// Non-empty set wrapper (T → NonEmptySet<T>).
    NonEmptySet,
    /// Map wrapper (V → Map<String, V>) — string-keyed map with typed values.
    Map,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_op_variants() {
        let identity = TypeOp::Identity;
        let validate = TypeOp::Validate(Predicate::NonEmpty);
        let meta = TypeOp::Meta(MetadataPayload::SystemId("gcp".to_string()));
        let transform = TypeOp::Transform(Coercion::new(BaseType::String, BaseType::Int));
        let wrap = TypeOp::Wrap(WrapperKind::Optional);

        assert_eq!(identity, TypeOp::Identity);
        assert!(matches!(validate, TypeOp::Validate(Predicate::NonEmpty)));
        assert!(matches!(meta, TypeOp::Meta(MetadataPayload::SystemId(_))));
        assert!(matches!(transform, TypeOp::Transform(_)));
        assert!(matches!(wrap, TypeOp::Wrap(WrapperKind::Optional)));
    }

    #[test]
    fn test_type_op_product_coproduct_brand() {
        let product = TypeOp::Product(vec![
            ("path".to_string(), TypeId::from("FilePath")),
            ("encoding".to_string(), TypeId::from("ContentEncoding")),
        ]);
        assert!(matches!(product, TypeOp::Product(ref fields) if fields.len() == 2));

        let coproduct = TypeOp::Coproduct(vec![
            ("UTF8".to_string(), TypeId::from("String")),
            ("Binary".to_string(), TypeId::from("Bytes")),
        ]);
        assert!(matches!(coproduct, TypeOp::Coproduct(ref variants) if variants.len() == 2));

        let brand = TypeOp::Brand("TextFilePath".to_string(), TypeId::from("FilePath"));
        assert!(matches!(brand, TypeOp::Brand(ref name, _) if name == "TextFilePath"));
    }

    #[test]
    fn test_predicate_composition() {
        let non_empty = Predicate::NonEmpty;
        let matches_url = Predicate::Matches(r"https?://.*".to_string());
        let combined = Predicate::And(vec![non_empty.clone(), matches_url.clone()]);

        assert!(matches!(combined, Predicate::And(_)));
    }

    #[test]
    fn test_predicate_entails() {
        let non_empty = Predicate::NonEmpty;
        let matches_url = Predicate::Matches(r"https?://.*".to_string());
        let combined = Predicate::And(vec![non_empty.clone(), matches_url.clone()]);

        assert!(combined.entails(&non_empty));
        assert!(!non_empty.entails(&combined));

        let narrow = Predicate::InRange { min: 1, max: 5 };
        let wide = Predicate::InRange { min: 0, max: 10 };
        assert!(narrow.entails(&wide));
        assert!(!wide.entails(&narrow));
    }

    #[test]
    fn test_predicate_content_entails() {
        let ascii = Predicate::Content(ContentEncoding::ASCII);
        let utf8 = Predicate::Content(ContentEncoding::UTF8);
        let text = Predicate::Content(ContentEncoding::Text);
        let binary = Predicate::Content(ContentEncoding::Binary);
        let unknown = Predicate::Content(ContentEncoding::Unknown);

        // ASCII ⊆ UTF8 ⊆ Text ⊆ Unknown
        assert!(ascii.entails(&utf8));
        assert!(ascii.entails(&text));
        assert!(ascii.entails(&unknown));
        assert!(utf8.entails(&text));
        assert!(utf8.entails(&unknown));
        assert!(text.entails(&unknown));

        // But not the reverse
        assert!(!utf8.entails(&ascii));
        assert!(!text.entails(&utf8));

        // Binary is not a subtype of Text
        assert!(!binary.entails(&text));
        assert!(!text.entails(&binary));

        // Binary ⊆ Unknown
        assert!(binary.entails(&unknown));
    }

    #[test]
    fn test_content_encoding_subtype() {
        assert!(ContentEncoding::ASCII.is_subtype_of(&ContentEncoding::UTF8));
        assert!(ContentEncoding::ASCII.is_subtype_of(&ContentEncoding::Text));
        assert!(ContentEncoding::UTF8.is_subtype_of(&ContentEncoding::Text));
        assert!(ContentEncoding::Latin1.is_subtype_of(&ContentEncoding::Text));
        assert!(!ContentEncoding::Binary.is_subtype_of(&ContentEncoding::Text));
        assert!(!ContentEncoding::UTF8.is_subtype_of(&ContentEncoding::Binary));
        // Everything ⊆ Unknown
        assert!(ContentEncoding::Binary.is_subtype_of(&ContentEncoding::Unknown));
        assert!(ContentEncoding::Text.is_subtype_of(&ContentEncoding::Unknown));
    }

    #[test]
    fn test_base_type_construction() {
        let string_type = BaseType::String;
        let list_of_strings = BaseType::list(BaseType::String);
        let optional_int = BaseType::option(BaseType::Int);
        let map_type = BaseType::map(BaseType::String, BaseType::Json);

        assert_eq!(string_type, BaseType::String);
        assert!(matches!(list_of_strings, BaseType::List(_)));
        assert!(matches!(optional_int, BaseType::Option(_)));
        assert!(matches!(map_type, BaseType::Map(_, _)));

        // New base types
        assert_eq!(BaseType::Float, BaseType::Float);
        assert_eq!(BaseType::Bytes, BaseType::Bytes);
        assert_eq!(BaseType::Secret, BaseType::Secret);
    }

    #[test]
    fn test_coercion() {
        let string_to_int = Coercion::new(BaseType::String, BaseType::Int);
        assert_eq!(string_to_int.from, BaseType::String);
        assert_eq!(string_to_int.to, BaseType::Int);
    }
}
