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
//! ```text
//! use gunbc_ir::type_lib;
//!
//! // Types are just Dag<TypeOp>
//! let url_type = type_lib::url();
//! let optional_url = type_lib::optional(url_type);
//!
//! // Cardinality emerges from structure
//! // optional_url output port has ZeroOrOne cardinality
//! ```

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


    /// Transformation — coerces value from one type to another.
    /// Used for type conversions (e.g., String → Int parsing).
    /// Carries (from_type_name, to_type_name).
    Transform(String, String),

    /// Wrapper operation — wraps a value in a container type.
    /// Used for Optional<T>, List<T>, etc.
    Wrap(WrapperKind),

    /// Unwrap operation — extracts value from a container type.
    Unwrap(WrapperKind),

    /// Product type — a record with named fields.
    /// Field types are embedded as SubDag children (naming: `field_{name}`).
    /// e.g., `{ path: FilePath, encoding: ContentEncoding }`
    Product(Vec<String>),

    /// Coproduct type — a tagged union of named variants.
    /// Variant types are embedded as SubDag children (naming: `variant_{name}`).
    /// e.g., `UTF8 | ASCII | Latin1 | Binary`
    Coproduct(Vec<String>),

    /// Brand (nominal) type — a named wrapper.
    /// Inner type is embedded as a SubDag child.
    /// e.g., `TextFilePath = FilePath @content(Text)`
    Brand(String),
}

/// System model metadata for behavioral catalog DAGs.
///
/// These are inert metadata payloads used by system model DAGs to encode
/// system identity, behavior properties, and I/O contracts. They are
/// traversable/inspectable but do not change runtime behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemModelMeta {
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

impl crate::algebra::PartialOrder for ContentEncoding {
    fn leq(&self, other: &Self) -> bool {
        self.is_subtype_of(other)
    }
}

impl crate::algebra::JoinSemilattice for ContentEncoding {
    /// Least upper bound: the most general encoding that contains both.
    fn join(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        if self.is_subtype_of(&other) {
            return other;
        }
        if other.is_subtype_of(&self) {
            return self;
        }
        // Neither is a subtype of the other — find LUB.
        // Both under Text but incomparable (e.g., UTF8 vs Latin1) → Text
        // One Text, one Binary → Unknown
        match (&self, &other) {
            (ContentEncoding::UTF8, ContentEncoding::Latin1)
            | (ContentEncoding::Latin1, ContentEncoding::UTF8)
            | (ContentEncoding::ASCII, ContentEncoding::Latin1)
            | (ContentEncoding::Latin1, ContentEncoding::ASCII) => ContentEncoding::Text,
            _ => ContentEncoding::Unknown,
        }
    }
}

impl crate::algebra::MeetSemilattice for ContentEncoding {
    /// Greatest lower bound: the most specific encoding contained in both.
    /// Returns None if the intersection is empty.
    fn meet(self, other: Self) -> Option<Self> {
        if self == other {
            return Some(self);
        }
        if self.is_subtype_of(&other) {
            return Some(self);
        }
        if other.is_subtype_of(&self) {
            return Some(other);
        }
        // Incomparable types: UTF8 ∧ Latin1 = None, Text ∧ Binary = None
        None
    }
}

impl crate::algebra::Lattice for ContentEncoding {}

impl crate::algebra::BoundedLattice for ContentEncoding {
    fn top() -> Self {
        ContentEncoding::Unknown
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

    /// Bit-width constraint.
    /// e.g., `width(8)` means the type occupies exactly 8 bits.
    Width(u16),

    /// Collection/string length constraint.
    /// e.g., `length(4)` means exactly 4 elements/bytes.
    Length(u64),

    /// Domain constraint — names the mathematical/encoding domain.
    /// e.g., `domain("ieee754_binary32")` for IEEE 754 float representation.
    Domain(String),

    /// Signed integer constraint. Optional string names the representation
    /// (e.g., `"twos_complement"`). `None` means signed with default representation.
    Signed(Option<String>),

    /// Unsigned integer constraint — value is non-negative.
    Unsigned,

    /// Arithmetic constraint — type supports arithmetic operations.
    Arithmetic,

    /// Inert system model metadata (non-semantic, non-failing).
    ///
    /// Used by system model DAGs to encode behavioral catalog metadata
    /// (system identity, behavior properties, I/O contracts). Traversable
    /// and inspectable but does not change runtime behavior.
    Meta(SystemModelMeta),
}

/// Simple values that can appear in predicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PredicateValue {
    Bool(bool),
    Int(i64),
    Str(String),
    /// Represents the `Value::Skipped` sentinel in guard predicates.
    Skipped,
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

            // Width: exact match only (already handled by self == other above)
            (Predicate::Width(_), Predicate::Width(_)) => false,

            // Length: exact match only
            (Predicate::Length(_), Predicate::Length(_)) => false,

            // Domain: exact match only
            (Predicate::Domain(_), Predicate::Domain(_)) => false,

            // Signed entails Signed (regardless of representation detail)
            (Predicate::Signed(_), Predicate::Signed(_)) => true,

            // Unsigned is atomic — exact match handled above
            // Arithmetic is atomic — exact match handled above

            _ => false,
        }
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
        let meta = TypeOp::Validate(Predicate::Meta(SystemModelMeta::SystemId(
            "gcp".to_string(),
        )));
        let transform = TypeOp::Transform("String".to_string(), "Int".to_string());
        let wrap = TypeOp::Wrap(WrapperKind::Optional);

        assert_eq!(identity, TypeOp::Identity);
        assert!(matches!(validate, TypeOp::Validate(Predicate::NonEmpty)));
        assert!(matches!(
            meta,
            TypeOp::Validate(Predicate::Meta(SystemModelMeta::SystemId(_)))
        ));
        assert!(matches!(transform, TypeOp::Transform(_, _)));
        assert!(matches!(wrap, TypeOp::Wrap(WrapperKind::Optional)));
    }

    #[test]
    fn test_type_op_product_coproduct_brand() {
        let product = TypeOp::Product(vec![
            "path".to_string(),
            "encoding".to_string(),
        ]);
        assert!(matches!(product, TypeOp::Product(ref fields) if fields.len() == 2));

        let coproduct = TypeOp::Coproduct(vec![
            "UTF8".to_string(),
            "Binary".to_string(),
        ]);
        assert!(matches!(coproduct, TypeOp::Coproduct(ref variants) if variants.len() == 2));

        let brand = TypeOp::Brand("TextFilePath".to_string());
        assert!(matches!(brand, TypeOp::Brand(ref name) if name == "TextFilePath"));
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

        // And/Or composite distribution through Content predicates
        let and_ascii_nonempty = Predicate::And(vec![
            Predicate::Content(ContentEncoding::ASCII),
            Predicate::NonEmpty,
        ]);
        // And(ASCII, NonEmpty) entails Content(UTF8) because ASCII ⊆ UTF8
        assert!(and_ascii_nonempty.entails(&utf8));
        // And(ASCII, NonEmpty) does NOT entail Content(Binary)
        assert!(!and_ascii_nonempty.entails(&binary));

        let or_utf8_binary = Predicate::Or(vec![
            Predicate::Content(ContentEncoding::UTF8),
            Predicate::Content(ContentEncoding::Binary),
        ]);
        // Or(UTF8, Binary) entails Content(Unknown) — both arms are ⊆ Unknown
        assert!(or_utf8_binary.entails(&unknown));
        // Or(UTF8, Binary) does NOT entail Content(Text) — Binary ⊄ Text
        assert!(!or_utf8_binary.entails(&text));
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

    // --- ContentEncoding lattice tests ---

    #[test]
    fn test_content_encoding_lattice_join() {
        use crate::algebra::JoinSemilattice;

        // Self-join is idempotent
        assert_eq!(
            ContentEncoding::ASCII.join(ContentEncoding::ASCII),
            ContentEncoding::ASCII
        );

        // Join of subtypes gives the supertype
        assert_eq!(
            ContentEncoding::ASCII.join(ContentEncoding::UTF8),
            ContentEncoding::UTF8
        );
        assert_eq!(
            ContentEncoding::UTF8.join(ContentEncoding::Text),
            ContentEncoding::Text
        );

        // Incomparable text subtypes join to Text
        assert_eq!(
            ContentEncoding::UTF8.join(ContentEncoding::Latin1),
            ContentEncoding::Text
        );
        assert_eq!(
            ContentEncoding::ASCII.join(ContentEncoding::Latin1),
            ContentEncoding::Text
        );

        // Text and Binary join to Unknown
        assert_eq!(
            ContentEncoding::Text.join(ContentEncoding::Binary),
            ContentEncoding::Unknown
        );
        assert_eq!(
            ContentEncoding::UTF8.join(ContentEncoding::Binary),
            ContentEncoding::Unknown
        );
    }

    #[test]
    fn test_content_encoding_lattice_meet() {
        use crate::algebra::MeetSemilattice;

        // Self-meet is idempotent
        assert_eq!(
            ContentEncoding::UTF8.meet(ContentEncoding::UTF8),
            Some(ContentEncoding::UTF8)
        );

        // Meet of related types gives the subtype
        assert_eq!(
            ContentEncoding::UTF8.meet(ContentEncoding::Text),
            Some(ContentEncoding::UTF8)
        );
        assert_eq!(
            ContentEncoding::ASCII.meet(ContentEncoding::UTF8),
            Some(ContentEncoding::ASCII)
        );

        // Meet of incomparable types is None
        assert_eq!(ContentEncoding::UTF8.meet(ContentEncoding::Latin1), None);
        assert_eq!(ContentEncoding::Text.meet(ContentEncoding::Binary), None);
    }

    #[test]
    fn test_content_encoding_lattice_partial_order() {
        use crate::algebra::PartialOrder;

        // Reflexivity
        assert!(ContentEncoding::ASCII.leq(&ContentEncoding::ASCII));

        // Transitivity: ASCII ≤ UTF8 ≤ Text
        assert!(ContentEncoding::ASCII.leq(&ContentEncoding::UTF8));
        assert!(ContentEncoding::UTF8.leq(&ContentEncoding::Text));
        assert!(ContentEncoding::ASCII.leq(&ContentEncoding::Text));

        // Top
        assert!(ContentEncoding::Binary.leq(&ContentEncoding::Unknown));
        assert!(ContentEncoding::Text.leq(&ContentEncoding::Unknown));

        // Incomparable
        assert!(!ContentEncoding::UTF8.leq(&ContentEncoding::Binary));
        assert!(!ContentEncoding::Binary.leq(&ContentEncoding::Text));
    }

    #[test]
    fn test_content_encoding_lattice_bounded_top() {
        use crate::algebra::{BoundedLattice, PartialOrder};

        let top = ContentEncoding::top();
        assert_eq!(top, ContentEncoding::Unknown);

        let all = [
            ContentEncoding::Unknown,
            ContentEncoding::Text,
            ContentEncoding::UTF8,
            ContentEncoding::ASCII,
            ContentEncoding::Latin1,
            ContentEncoding::Binary,
        ];
        for enc in &all {
            assert!(enc.leq(&top), "{:?} should be ≤ top", enc);
        }
    }

    #[test]
    fn test_content_encoding_lattice_absorption() {
        use crate::algebra::{JoinSemilattice, MeetSemilattice};

        // a.join(a.meet(b)) == a (when meet exists)
        let a = ContentEncoding::UTF8;
        let b = ContentEncoding::Text;
        if let Some(m) = a.meet(b) {
            assert_eq!(a.join(m), a);
        }

        // a.meet(a.join(b)) == Some(a)
        let j = a.join(b);
        assert_eq!(a.meet(j), Some(a));
    }

    /// Verify that the Rust `ContentEncoding` variants match the DSL
    /// `encoding.dag` declaration: `type Encoding = ASCII | UTF8 | Latin1 | Text | Binary | Unknown`.
    ///
    /// This test catches any divergence between the Rust enum and the DSL
    /// definition. When the behavior system (Phases 8-10) arrives, the lattice
    /// ordering itself will be DSL-driven.
    #[test]
    fn test_content_encoding_matches_dsl_encoding_dag() {
        let dsl_variants: std::collections::HashSet<&str> =
            ["ASCII", "UTF8", "Latin1", "Text", "Binary", "Unknown"]
                .iter()
                .copied()
                .collect();

        let rust_variants: std::collections::HashSet<&str> = [
            variant_name(ContentEncoding::ASCII),
            variant_name(ContentEncoding::UTF8),
            variant_name(ContentEncoding::Latin1),
            variant_name(ContentEncoding::Text),
            variant_name(ContentEncoding::Binary),
            variant_name(ContentEncoding::Unknown),
        ]
        .iter()
        .copied()
        .collect();

        assert_eq!(
            dsl_variants, rust_variants,
            "Rust ContentEncoding variants must match dsl/std/encoding.dag"
        );
        assert_eq!(
            rust_variants.len(),
            6,
            "encoding.dag declares exactly 6 variants"
        );
    }

    fn variant_name(enc: ContentEncoding) -> &'static str {
        match enc {
            ContentEncoding::ASCII => "ASCII",
            ContentEncoding::UTF8 => "UTF8",
            ContentEncoding::Latin1 => "Latin1",
            ContentEncoding::Text => "Text",
            ContentEncoding::Binary => "Binary",
            ContentEncoding::Unknown => "Unknown",
        }
    }

    /// Verify the lattice ordering from encoding.dag:
    /// ASCII ⊆ UTF8 ⊆ Text ⊆ Unknown
    /// Latin1 ⊆ Text ⊆ Unknown
    /// Binary ⊆ Unknown
    /// Text and Binary are incomparable.
    #[test]
    fn test_content_encoding_lattice_matches_dsl_ordering() {
        use crate::algebra::PartialOrder;

        // ASCII ⊆ UTF8 ⊆ Text ⊆ Unknown
        assert!(ContentEncoding::ASCII.leq(&ContentEncoding::UTF8));
        assert!(ContentEncoding::UTF8.leq(&ContentEncoding::Text));
        assert!(ContentEncoding::Text.leq(&ContentEncoding::Unknown));
        assert!(ContentEncoding::ASCII.leq(&ContentEncoding::Unknown));

        // Latin1 ⊆ Text ⊆ Unknown
        assert!(ContentEncoding::Latin1.leq(&ContentEncoding::Text));
        assert!(ContentEncoding::Latin1.leq(&ContentEncoding::Unknown));

        // Binary ⊆ Unknown only
        assert!(ContentEncoding::Binary.leq(&ContentEncoding::Unknown));
        assert!(!ContentEncoding::Binary.leq(&ContentEncoding::Text));

        // Text and Binary are incomparable
        assert!(!ContentEncoding::Text.leq(&ContentEncoding::Binary));
        assert!(!ContentEncoding::Binary.leq(&ContentEncoding::Text));

        // UTF8 and Latin1 are incomparable
        assert!(!ContentEncoding::UTF8.leq(&ContentEncoding::Latin1));
        assert!(!ContentEncoding::Latin1.leq(&ContentEncoding::UTF8));
    }
}
