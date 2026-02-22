//! Core identifier types.

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Set-theoretic cardinality for port values, modeled as a closed interval
/// `[min, max]` on ℕ ∪ {∞}.
///
/// Every port has a cardinality that describes how many values can flow through it.
/// This enables semantic test generation, runtime validation, and lattice algebra.
///
/// # Mathematical Basis
///
/// A cardinality `[min, max]` represents the set of valid multiplicities.
/// `[1, 1]` means "exactly one value". `[0, ∞)` means "any number of values".
/// This is the same concept as regex quantifiers: `{min,max}`.
///
/// # Named Constants
///
/// The five standard cardinalities are available as constants:
///
/// - [`Cardinality::ZERO`] = `[0, 0]` — ∅ (empty set, signal-only)
/// - [`Cardinality::ONE`] = `[1, 1]` — {x} (singleton, exactly one)
/// - [`Cardinality::ZERO_OR_ONE`] = `[0, 1]` — {x}? (optional)
/// - [`Cardinality::ZERO_OR_MORE`] = `[0, ∞)` — {x}* (Kleene star)
/// - [`Cardinality::ONE_OR_MORE`] = `[1, ∞)` — {x}+ (Kleene plus)
///
/// But arbitrary cardinalities like `Cardinality::new(2, Some(5))` are also
/// valid without any code changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cardinality {
    /// Minimum number of values (inclusive).
    pub min: u32,
    /// Maximum number of values (inclusive). `None` = unbounded.
    pub max: Option<u32>,
}

/// Sampling policy for cardinality-driven test case generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalitySamplingStrategy {
    /// Use only boundary-valid cases (`min`, optional `min+1`, optional `max`).
    BoundaryOnly,
    /// Use boundary-valid cases and clamp values above the given upper bound.
    BoundaryWithUpperBound(u32),
}

impl Cardinality {
    /// ∅ — signal-only, no data.
    pub const ZERO: Self = Self {
        min: 0,
        max: Some(0),
    };

    /// {x} — exactly one (scalar). Default for most ports.
    pub const ONE: Self = Self {
        min: 1,
        max: Some(1),
    };

    /// {x}? — optional (zero or one).
    pub const ZERO_OR_ONE: Self = Self {
        min: 0,
        max: Some(1),
    };

    /// {x}* — Kleene star (zero or more, list).
    pub const ZERO_OR_MORE: Self = Self { min: 0, max: None };

    /// {x}+ — Kleene plus (one or more, non-empty list).
    pub const ONE_OR_MORE: Self = Self { min: 1, max: None };

    /// Create a new cardinality with explicit bounds.
    ///
    /// # Panics
    ///
    /// Panics if `max` is `Some(m)` and `m < min` (empty interval).
    pub fn new(min: u32, max: Option<u32>) -> Self {
        if let Some(m) = max {
            assert!(m >= min, "invalid cardinality: max ({m}) < min ({min})");
        }
        Self { min, max }
    }

    // =========================================================================
    // Derived queries (predicates on bounds, no match statements)
    // =========================================================================

    /// Returns true if this cardinality allows zero elements.
    pub fn allows_empty(&self) -> bool {
        self.min == 0
    }

    /// Returns true if this cardinality allows exactly one element.
    pub fn allows_one(&self) -> bool {
        self.max_at_least(1) && self.min <= 1
    }

    /// Returns true if this cardinality allows multiple elements.
    pub fn allows_many(&self) -> bool {
        self.max_at_least(2)
    }

    /// Returns true if this cardinality requires at least one element.
    pub fn requires_one(&self) -> bool {
        self.min >= 1
    }

    /// Returns true if this cardinality is bounded (has a finite max).
    pub fn is_bounded(&self) -> bool {
        self.max.is_some()
    }

    /// Returns true if this is exactly scalar: `[1, 1]`.
    pub fn is_scalar(&self) -> bool {
        self.min == 1 && self.max == Some(1)
    }

    /// Returns true if this cardinality can hold multiple elements.
    pub fn is_list(&self) -> bool {
        self.allows_many()
    }

    fn max_at_least(&self, n: u32) -> bool {
        self.max.is_none_or(|m| m >= n)
    }

    // =========================================================================
    // Test case generation
    // =========================================================================

    /// Returns boundary values that should be tested for this cardinality.
    ///
    /// Computes boundary values from the interval using standard boundary-value
    /// analysis: min, min+1 (if distinct from max), max, plus below-min and
    /// above-max when applicable. For standard cardinalities, this produces
    /// the same coverage as the old `Empty/One/Many` enum but also handles
    /// arbitrary intervals like `[2, 5]` → `{1, 2, 3, 5, 6}`.
    ///
    /// Values within the interval represent valid inputs (should succeed).
    /// Values outside represent invalid inputs (should fail/skip).
    /// Use `allows_count(n)` to distinguish the two.
    pub fn boundary_values(&self) -> Vec<u32> {
        let mut cases = Vec::new();
        // Below min (invalid boundary — should fail)
        if self.min > 0 {
            cases.push(self.min - 1);
        }
        // Min (lowest valid value)
        cases.push(self.min);
        // Min + 1 (just above min, if in range)
        if let Some(min_plus) = self.min.checked_add(1) {
            if self.allows_count(min_plus) && Some(min_plus) != self.max {
                cases.push(min_plus);
            }
        }
        if let Some(max) = self.max {
            // Max (highest valid value)
            if max > self.min {
                cases.push(max);
            }
            // Above max (invalid boundary — should fail)
            if let Some(above) = max.checked_add(1) {
                cases.push(above);
            }
        } else {
            // Unbounded: no synthetic "large" value here.
            // Test generators can choose a fermi-sized "many" case explicitly.
        }
        cases.sort();
        cases.dedup();
        cases
    }

    /// Returns the test cases that should be generated for this cardinality.
    ///
    /// Returns in-range boundary values only (values that the cardinality
    /// accepts). For out-of-range boundary testing, use `boundary_values()`
    /// and filter with `allows_count()`.
    pub fn test_cases(&self) -> Vec<u32> {
        self.boundary_values()
            .into_iter()
            .filter(|&n| self.allows_count(n))
            .collect()
    }

    /// Returns test cases with a cap for large counts.
    ///
    /// This keeps boundary coverage while avoiding enormous test vectors when
    /// a bounded max is very large. If `cap` is below the minimum, no capping
    /// occurs (to preserve validity).
    pub fn test_cases_capped(&self, cap: u32) -> Vec<u32> {
        let mut cases = self.test_cases();
        if cap < self.min {
            return cases;
        }
        for n in &mut cases {
            if *n > cap {
                *n = cap;
            }
        }
        cases.sort();
        cases.dedup();
        cases
    }

    /// Default test cases used by generators (boundary-only sampling).
    pub fn test_cases_for_tests(&self) -> Vec<u32> {
        self.test_cases_with_strategy(CardinalitySamplingStrategy::BoundaryOnly)
    }

    /// Returns test cases using an explicit sampling strategy.
    pub fn test_cases_with_strategy(&self, strategy: CardinalitySamplingStrategy) -> Vec<u32> {
        match strategy {
            CardinalitySamplingStrategy::BoundaryOnly => self.test_cases(),
            CardinalitySamplingStrategy::BoundaryWithUpperBound(max) => self.test_cases_capped(max),
        }
    }

    /// Check if the given count is within this cardinality's interval.
    pub fn allows_count(&self, count: u32) -> bool {
        count >= self.min && self.max.is_none_or(|max| count <= max)
    }

    // =========================================================================
    // Satisfaction (interval containment, not a truth table)
    // =========================================================================

    /// Check if this output cardinality satisfies an input cardinality requirement.
    ///
    /// This is subset containment: `self ⊆ target`.
    /// `[1,1]` satisfies `[0,∞)` because {1} ⊆ {0,1,2,...}.
    /// `[0,∞)` does NOT satisfy `[1,1]` because 0 ∉ {1}.
    ///
    /// Returns true if ALL possible outputs from `self` are acceptable by
    /// `input_requirement`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gunbc_ir::Cardinality;
    ///
    /// // One always satisfies One
    /// assert!(Cardinality::ONE.satisfies(Cardinality::ONE));
    ///
    /// // OneOrMore satisfies ZeroOrMore (non-empty fits in any-length)
    /// assert!(Cardinality::ONE_OR_MORE.satisfies(Cardinality::ZERO_OR_MORE));
    ///
    /// // ZeroOrMore does NOT satisfy OneOrMore (might produce empty)
    /// assert!(!Cardinality::ZERO_OR_MORE.satisfies(Cardinality::ONE_OR_MORE));
    ///
    /// // ZeroOrOne does NOT satisfy One (might produce zero)
    /// assert!(!Cardinality::ZERO_OR_ONE.satisfies(Cardinality::ONE));
    /// ```
    pub fn satisfies(&self, input_requirement: Cardinality) -> bool {
        self.min >= input_requirement.min && self.max_leq(input_requirement.max)
    }

    /// Check if this output can satisfy the input, with detailed error.
    pub fn check_satisfies(
        &self,
        input_requirement: Cardinality,
    ) -> Result<(), CardinalityMismatch> {
        if self.satisfies(input_requirement) {
            Ok(())
        } else {
            Err(CardinalityMismatch {
                output: *self,
                input: input_requirement,
                reason: self.mismatch_reason(input_requirement),
            })
        }
    }

    fn max_leq(&self, target_max: Option<u32>) -> bool {
        match (self.max, target_max) {
            (_, None) => true,            // target is unbounded, anything fits
            (None, Some(_)) => false,     // we're unbounded, target is bounded
            (Some(a), Some(b)) => a <= b, // both bounded, compare
        }
    }

    fn mismatch_reason(&self, input: Cardinality) -> String {
        if self.min < input.min && !self.max_leq(input.max) {
            format!(
                "output {} is incompatible with input {} (both min and max mismatch)",
                self, input
            )
        } else if self.min < input.min {
            format!(
                "output might be empty (min={}) but input requires at least {} element(s)",
                self.min, input.min
            )
        } else if !self.max_leq(input.max) {
            format!(
                "output might have many elements (max={}) but input accepts at most {}",
                self.max.map_or("∞".to_string(), |m| m.to_string()),
                input.max.map_or("∞".to_string(), |m| m.to_string())
            )
        } else {
            format!("cardinality {} cannot satisfy {}", self, input)
        }
    }

    // =========================================================================
    // Lattice algebra
    // =========================================================================

    /// Interval sum (Minkowski sum): fan-in composition of independent sources.
    ///
    /// If one edge can contribute `[a,b]` elements and another `[c,d]`, the
    /// combined fan-in can contribute `[a+c, b+d]`.
    pub fn sum(self, other: Cardinality) -> Cardinality {
        Cardinality {
            min: self.min.saturating_add(other.min),
            max: match (self.max, other.max) {
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => a.checked_add(b),
            },
        }
    }

    /// Join (least upper bound): union of possibilities.
    ///
    /// "What cardinality can hold values from either self or other?"
    /// `[1,1] ∨ [0,1] = [0,1]`  — either scalar or optional → optional
    /// `[1,1] ∨ [1,∞) = [1,∞)` — either one or many → one-or-more
    /// `[0,1] ∨ [1,∞) = [0,∞)` — either optional or non-empty → any
    pub fn join(self, other: Cardinality) -> Cardinality {
        Cardinality {
            min: self.min.min(other.min),
            max: match (self.max, other.max) {
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => Some(a.max(b)),
            },
        }
    }

    /// Meet (greatest lower bound): intersection of constraints.
    ///
    /// "What cardinality satisfies both self and other?"
    /// `[0,1] ∧ [1,∞) = [1,1]`  — must be optional AND non-empty → exactly one
    /// `[0,∞) ∧ [1,1] = [1,1]`  — any AND scalar → scalar
    ///
    /// Returns `None` if the intersection is empty (min > max).
    pub fn meet(self, other: Cardinality) -> Option<Cardinality> {
        let min = self.min.max(other.min);
        let max = match (self.max, other.max) {
            (None, x) | (x, None) => x,
            (Some(a), Some(b)) => Some(a.min(b)),
        };
        // Check if interval is valid (min <= max)
        match max {
            Some(m) if min > m => None, // empty intersection
            _ => Some(Cardinality { min, max }),
        }
    }

    /// Product: cardinality of nested iteration.
    ///
    /// If outer produces `[a,b]` items each containing `[c,d]` items,
    /// the flattened result has `[a*c, b*d]` items.
    ///
    /// `[1,∞) × [1,1] = [1,∞)`   — many scalars → many
    /// `[0,∞) × [1,∞) = [0,∞)`   — optional many × non-empty → any
    /// `[1,1] × [1,1] = [1,1]`    — one scalar → one scalar
    pub fn product(self, other: Cardinality) -> Cardinality {
        Cardinality {
            min: self.min.saturating_mul(other.min),
            max: match (self.max, other.max) {
                (Some(0), _) | (_, Some(0)) => Some(0),
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => a.checked_mul(b), // None on overflow → unbounded
            },
        }
    }
}

impl Default for Cardinality {
    /// Default cardinality is ONE (scalar, required).
    fn default() -> Self {
        Self::ONE
    }
}

// =============================================================================
// Serde: Named strings for the 5 standard cardinalities, {min, max} for custom
// =============================================================================

impl Serialize for Cardinality {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match *self {
            Self::ZERO => s.serialize_str("Zero"),
            Self::ONE => s.serialize_str("One"),
            Self::ZERO_OR_ONE => s.serialize_str("ZeroOrOne"),
            Self::ZERO_OR_MORE => s.serialize_str("ZeroOrMore"),
            Self::ONE_OR_MORE => s.serialize_str("OneOrMore"),
            _ => {
                // Non-standard cardinality: serialize as {min, max}
                let mut map = s.serialize_map(Some(2))?;
                map.serialize_entry("min", &self.min)?;
                map.serialize_entry("max", &self.max)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Cardinality {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CardinalityVisitor;

        impl<'de> Visitor<'de> for CardinalityVisitor {
            type Value = Cardinality;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(
                    "a cardinality string (\"Zero\", \"One\", ...) or {\"min\": N, \"max\": N}",
                )
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Cardinality, E> {
                match value {
                    "Zero" => Ok(Cardinality::ZERO),
                    "One" => Ok(Cardinality::ONE),
                    "ZeroOrOne" => Ok(Cardinality::ZERO_OR_ONE),
                    "ZeroOrMore" => Ok(Cardinality::ZERO_OR_MORE),
                    "OneOrMore" => Ok(Cardinality::ONE_OR_MORE),
                    _ => Err(de::Error::unknown_variant(
                        value,
                        &["Zero", "One", "ZeroOrOne", "ZeroOrMore", "OneOrMore"],
                    )),
                }
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Cardinality, M::Error> {
                let mut min: Option<u32> = None;
                let mut max: Option<Option<u32>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "min" => {
                            min = Some(map.next_value()?);
                        }
                        "max" => {
                            max = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let min = min.ok_or_else(|| de::Error::missing_field("min"))?;
                let max = max.ok_or_else(|| de::Error::missing_field("max"))?;

                if let Some(m) = max {
                    if m < min {
                        return Err(de::Error::custom(format!(
                            "invalid cardinality: max ({m}) < min ({min})"
                        )));
                    }
                }

                Ok(Cardinality { min, max })
            }
        }

        deserializer.deserialize_any(CardinalityVisitor)
    }
}

// =============================================================================
// Display
// =============================================================================

impl fmt::Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (0, Some(0)) => write!(f, "0"),
            (1, Some(1)) => write!(f, "1"),
            (n, Some(m)) if n == m => write!(f, "{}", n),
            (n, Some(m)) => write!(f, "{}..{}", n, m),
            (0, None) => write!(f, "0..*"),
            (n, None) => write!(f, "{}..*", n),
        }
    }
}

// =============================================================================
// Error type
// =============================================================================

/// Error when output cardinality doesn't satisfy input requirement.
#[derive(Debug, Clone)]
pub struct CardinalityMismatch {
    pub output: Cardinality,
    pub input: Cardinality,
    pub reason: String,
}

// =============================================================================
// Cardinality case helpers
// =============================================================================

/// Human-readable label for a boundary value count.
///
/// Used by codegen and test generation to produce readable test names.
pub fn boundary_label(count: u32) -> &'static str {
    match count {
        0 => "empty",
        1 => "one",
        _ => "many",
    }
}

// =============================================================================
// Identifier types (unchanged)
// =============================================================================

/// Unique identifier for a node within a DAG.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Name of a port on a node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PortName(pub String);

impl PortName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl From<&str> for PortName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for PortName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for PortName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type identifier for type checking edges.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeId(pub String);

/// Policy for placeholder seed generation in generated tests.
///
/// This classification lives with the IR type model so downstream generators
/// can share one canonical policy instead of maintaining local string lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedPlaceholderPolicy {
    /// Generated placeholder seeds are considered safe for this type.
    Generated,
    /// This type must use an explicit authored seed.
    ExplicitSeedRequired,
}

/// Semantic carrier classification for seed-policy enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticCarrierClass {
    /// Structural type where synthesized placeholders are valid.
    StructuralGeneratable,
    /// Semantically meaningful type requiring authored seeds in strict contexts.
    SemanticCarrier,
}

/// Refined semantic carrier kind for strict compatibility checks.
///
/// `UnknownSemantic` is fail-closed: strict compatibility rejects it unless
/// the call site opts into legacy structural-only behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticCarrierKind {
    Structural,
    TransportRequest,
    TransportResponse,
    Credential,
    Secret,
    FilesystemHandle,
    NetworkHandle,
    ToolHandle,
    Platform,
    Timestamp,
    UnknownSemantic,
}

impl TypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Classify placeholder seed policy for this type.
    ///
    /// This is fail-closed: unknown/new types default to
    /// `ExplicitSeedRequired`.
    pub fn seed_placeholder_policy(&self) -> SeedPlaceholderPolicy {
        seed_placeholder_policy_for_type_id(&self.0)
    }

    /// Classify this type into structural vs semantic-carrier seed class.
    pub fn semantic_carrier_class(&self) -> SemanticCarrierClass {
        semantic_carrier_class_for_type_id(&self.0)
    }

    /// Classify this type into a refined semantic-carrier kind.
    pub fn semantic_carrier_kind(&self) -> SemanticCarrierKind {
        semantic_carrier_kind_for_type_id(&self.0)
    }
}

/// Parse a parametric map type-id of the form `Map<K,V>`.
///
/// Returns `(K, V)` when the type-id is syntactically valid.
/// Supports nested generic values by splitting on the top-level comma.
pub fn parse_map_type_id(type_id: &str) -> Option<(String, String)> {
    let inner = type_id.strip_prefix("Map<")?.strip_suffix('>')?;
    let mut depth = 0usize;
    let mut split_idx: Option<usize> = None;
    for (idx, ch) in inner.char_indices() {
        match ch {
            '<' => depth = depth.saturating_add(1),
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                split_idx = Some(idx);
                break;
            }
            _ => {}
        }
    }

    let comma = split_idx?;
    let key = inner[..comma].trim();
    let value = inner[comma + 1..].trim();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key.to_string(), value.to_string()))
}

fn parse_unary_generic_type_id<'a>(type_id: &'a str, wrapper: &str) -> Option<&'a str> {
    let rest = type_id.strip_prefix(wrapper)?;
    let inner = rest.strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

fn optional_inner_type_id(type_id: &str) -> Option<&str> {
    if let Some(inner) = parse_unary_generic_type_id(type_id, "Optional") {
        return Some(inner);
    }
    let inner = type_id.strip_prefix("Optional")?;
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

fn parse_container_alias_inner<'a>(type_id: &'a str, suffix: &str) -> Option<&'a str> {
    let inner = type_id.strip_suffix(suffix)?;
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

/// Classify placeholder seed policy for a raw type ID.
pub fn seed_placeholder_policy_for_type_id(type_id: &str) -> SeedPlaceholderPolicy {
    match semantic_carrier_class_for_type_id(type_id) {
        SemanticCarrierClass::StructuralGeneratable => SeedPlaceholderPolicy::Generated,
        SemanticCarrierClass::SemanticCarrier => SeedPlaceholderPolicy::ExplicitSeedRequired,
    }
}

/// Classify semantic-carrier class for a raw type ID.
pub fn semantic_carrier_class_for_type_id(type_id: &str) -> SemanticCarrierClass {
    match semantic_carrier_kind_for_type_id(type_id) {
        SemanticCarrierKind::Structural => SemanticCarrierClass::StructuralGeneratable,
        _ => SemanticCarrierClass::SemanticCarrier,
    }
}

/// Refined semantic carrier kind for a raw type ID.
pub fn semantic_carrier_kind_for_type_id(type_id: &str) -> SemanticCarrierKind {
    if let Some((key_type, value_type)) = parse_map_type_id(type_id) {
        if key_type == "String"
            && semantic_carrier_kind_for_type_id(&value_type) == SemanticCarrierKind::Structural
        {
            return SemanticCarrierKind::Structural;
        }
        return SemanticCarrierKind::UnknownSemantic;
    }

    if let Some(inner) = optional_inner_type_id(type_id) {
        return semantic_carrier_kind_for_type_id(inner);
    }

    if let Some(inner) = parse_unary_generic_type_id(type_id, "List")
        .or_else(|| parse_unary_generic_type_id(type_id, "Set"))
    {
        return semantic_carrier_kind_for_type_id(inner);
    }

    if let Some(inner) = parse_container_alias_inner(type_id, "List")
        .or_else(|| parse_container_alias_inner(type_id, "Set"))
    {
        return semantic_carrier_kind_for_type_id(inner);
    }

    match type_id {
        // Primitives.
        "String" | "Bool" | "Int" | "Float" | "Bytes" | "Unit" | "Json" | "Void" | "Any"
        | "Error"
        // Refined primitives.
        | "NonEmptyString" | "SecretName" | "Url" | "FilePath" | "Path" | "Email"
        | "PositiveInt" | "NonNegativeInt"
        // Refined GCP identity/resource aliases.
        | "OidcAudience" | "WifAudience"
        | "GcpProjectId" | "GcpSecretId" | "GcpSecretVersion"
        | "GcpServiceAccountEmail" | "GcpSubjectToken" | "OidcSubjectToken"
        // Common wrappers/container aliases.
        | "OptionalString" | "OptionalInt" | "OptionalBool" | "OptionalJson"
        | "OptionalUrl"
        | "StringList" | "IntList" | "BoolList" | "JsonList"
        | "UrlList" | "FilePathList"
        | "NonEmptyStringList" | "NonEmptyFilePathList"
        => SemanticCarrierKind::Structural,
        // Transport envelopes.
        "TransportRequest" | "FileRequest" | "ShellRequest" | "RestRequest" | "HttpRequest"
        | "TcpRequest" => SemanticCarrierKind::TransportRequest,
        "TransportResponse" | "FileResponse" | "ShellResponse" | "RestResponse" | "HttpResponse"
        | "TcpResponse" => SemanticCarrierKind::TransportResponse,
        // Capability + secret carriers.
        "Credential" => SemanticCarrierKind::Credential,
        "Secret" | "SecretString" => SemanticCarrierKind::Secret,
        "FilesystemHandle" => SemanticCarrierKind::FilesystemHandle,
        "NetworkHandle" => SemanticCarrierKind::NetworkHandle,
        "ToolHandle" => SemanticCarrierKind::ToolHandle,
        "Platform" | "RuntimePlatform" => SemanticCarrierKind::Platform,
        "Timestamp" => SemanticCarrierKind::Timestamp,
        _ => SemanticCarrierKind::UnknownSemantic,
    }
}

/// Strict semantic carrier compatibility.
///
/// This is intentionally stricter than structural compatibility:
/// - structural ↔ structural is allowed
/// - known semantic carrier kinds must match exactly
/// - unknown semantic kinds fail closed
pub fn semantic_carrier_compatible(from: &TypeId, to: &TypeId) -> bool {
    use SemanticCarrierKind as Kind;

    let from_kind = semantic_carrier_kind_for_type_id(&from.0);
    let to_kind = semantic_carrier_kind_for_type_id(&to.0);

    match (from_kind, to_kind) {
        (Kind::Structural, Kind::Structural) => true,
        // Allow matching unknown semantics when both ports share the same type ID.
        (Kind::UnknownSemantic, Kind::UnknownSemantic) if from.0 == to.0 => true,
        (Kind::UnknownSemantic, _) | (_, Kind::UnknownSemantic) => false,
        (lhs, rhs) => lhs == rhs,
    }
}

/// How a `TypeId` serializes into a `Value` variant at runtime.
///
/// Used by testgen to validate that mock values are compatible with port types
/// without hardcoded lists of type names in codegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueBacking {
    String,
    Bool,
    Int,
    Float,
    Json,
    Map,
    List,
    Set,
    Unit,
    Bytes,
}

impl ValueBacking {
    /// Whether a [`ValueKind`] is compatible with this backing.
    pub fn accepts_value_kind(&self, kind: crate::value::ValueKind) -> bool {
        use crate::value::ValueKind;
        match self {
            ValueBacking::String => kind == ValueKind::String,
            ValueBacking::Bool => kind == ValueKind::Bool,
            ValueBacking::Int => kind == ValueKind::Int,
            ValueBacking::Float => kind == ValueKind::Int, // Float can accept Int
            ValueBacking::Json => true,                    // Json accepts anything
            ValueBacking::Map => kind == ValueKind::Map,
            ValueBacking::List => kind == ValueKind::List,
            ValueBacking::Set => kind == ValueKind::Set,
            ValueBacking::Unit => kind == ValueKind::Unit,
            ValueBacking::Bytes => kind == ValueKind::List, // byte arrays are lists
        }
    }
}

/// Determine how a `TypeId` string serializes into a `Value` variant.
///
/// Uses strict `PortType` parsing for known structural/domain types, then
/// falls back to legacy naming heuristics for unknown opaque type strings.
pub fn value_backing_for_type_id(type_id: &str) -> ValueBacking {
    use crate::port_type::PortType;
    use crate::type_registry::TypeRegistry;

    // Check for parametric Map<K,V> first
    if parse_map_type_id(type_id).is_some() {
        return ValueBacking::Map;
    }

    if parse_unary_generic_type_id(type_id, "Set").is_some() {
        return ValueBacking::Set;
    }

    if parse_unary_generic_type_id(type_id, "List").is_some() {
        return ValueBacking::List;
    }

    if let Some(inner) = optional_inner_type_id(type_id) {
        return value_backing_for_type_id(inner);
    }

    // Capability carriers with custom Value envelope representation.
    if type_id == "Credential" {
        return ValueBacking::Map;
    }

    let registry = TypeRegistry::with_core_types();
    let port_type =
        PortType::try_parse(type_id).or_else(|| PortType::from_registry(type_id, &registry).ok());
    match port_type {
        Some(PortType::String) => ValueBacking::String,
        Some(PortType::Bool) => ValueBacking::Bool,
        Some(PortType::Int) => ValueBacking::Int,
        Some(PortType::Float) => ValueBacking::Float,
        Some(PortType::Json) => ValueBacking::Json,
        Some(PortType::Bytes) => ValueBacking::Bytes,
        Some(PortType::Secret) => ValueBacking::String,
        Some(PortType::List(_)) => ValueBacking::List,
        None => match type_id {
            // Legacy list aliases
            s if s.ends_with("List") => ValueBacking::List,
            // Legacy set aliases
            s if s.ends_with("Set") => ValueBacking::Set,
            // Optional wrappers inherit inner type's backing
            s if s.starts_with("Optional") => value_backing_for_type_id(&s["Optional".len()..]),
            // Default: Json accepts anything
            _ => ValueBacking::Json,
        },
    }
}

/// Canonical human-readable type label for a runtime value's kind.
pub fn value_kind_name(value: &crate::value::Value) -> &'static str {
    value.kind().type_name()
}

/// Whether a runtime value is compatible with a `TypeId` string.
///
/// This mirrors the compatibility rules used by testgen and typed mock
/// requirements, centralized to avoid divergence.
pub fn value_compatible_with_type_id(type_id: &str, value: &crate::value::Value) -> bool {
    use crate::{
        transport::Credential,
        value::{Value, ValueKind},
    };

    let kind = value.kind();
    let kind_name = kind.type_name();

    // Exact match
    if type_id == kind_name {
        return true;
    }

    // Any matches anything
    if type_id == "Any" {
        return true;
    }

    // Optional<T> and OptionalT accept T or Unit
    if let Some(inner) = optional_inner_type_id(type_id) {
        if kind == ValueKind::Unit {
            return true;
        }
        return value_compatible_with_type_id(inner, value);
    }

    // Credential is encoded as a capability-marked map. Validate using the
    // canonical runtime conversion rather than raw Value backing.
    if type_id == "Credential" {
        return Credential::try_from(value).is_ok();
    }

    // Skipped is compatible with any type
    if kind == ValueKind::Skipped {
        return true;
    }

    // Json is intentionally flexible
    if type_id == "Json" || kind == ValueKind::Json {
        return true;
    }

    // Parametric map types: Map<String, T>
    if let Some((key_type, value_type)) = parse_map_type_id(type_id) {
        if key_type != "String" {
            return false;
        }
        if let Value::Map(entries) = value {
            return entries
                .values()
                .all(|entry| value_compatible_with_type_id(&value_type, entry));
        }
        return false;
    }

    // Platform has dual backing (String or Map)
    if type_id == "Platform" && (kind == ValueKind::String || kind == ValueKind::Map) {
        return true;
    }

    // Default to structural backing compatibility
    value_backing_for_type_id(type_id).accepts_value_kind(kind)
}

impl From<&str> for TypeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Query tests ---

    #[test]
    fn test_allows_empty() {
        assert!(Cardinality::ZERO.allows_empty());
        assert!(!Cardinality::ONE.allows_empty());
        assert!(Cardinality::ZERO_OR_ONE.allows_empty());
        assert!(Cardinality::ZERO_OR_MORE.allows_empty());
        assert!(!Cardinality::ONE_OR_MORE.allows_empty());
    }

    #[test]
    fn test_allows_one() {
        assert!(!Cardinality::ZERO.allows_one());
        assert!(Cardinality::ONE.allows_one());
        assert!(Cardinality::ZERO_OR_ONE.allows_one());
        assert!(Cardinality::ZERO_OR_MORE.allows_one());
        assert!(Cardinality::ONE_OR_MORE.allows_one());
    }

    #[test]
    fn test_allows_many() {
        assert!(!Cardinality::ZERO.allows_many());
        assert!(!Cardinality::ONE.allows_many());
        assert!(!Cardinality::ZERO_OR_ONE.allows_many());
        assert!(Cardinality::ZERO_OR_MORE.allows_many());
        assert!(Cardinality::ONE_OR_MORE.allows_many());
    }

    #[test]
    fn test_requires_one() {
        assert!(!Cardinality::ZERO.requires_one());
        assert!(Cardinality::ONE.requires_one());
        assert!(!Cardinality::ZERO_OR_ONE.requires_one());
        assert!(!Cardinality::ZERO_OR_MORE.requires_one());
        assert!(Cardinality::ONE_OR_MORE.requires_one());
    }

    #[test]
    fn test_is_scalar() {
        assert!(!Cardinality::ZERO.is_scalar());
        assert!(Cardinality::ONE.is_scalar());
        assert!(!Cardinality::ZERO_OR_ONE.is_scalar());
        assert!(!Cardinality::ZERO_OR_MORE.is_scalar());
        assert!(!Cardinality::ONE_OR_MORE.is_scalar());
    }

    #[test]
    fn test_is_list() {
        assert!(!Cardinality::ZERO.is_list());
        assert!(!Cardinality::ONE.is_list());
        assert!(!Cardinality::ZERO_OR_ONE.is_list());
        assert!(Cardinality::ZERO_OR_MORE.is_list());
        assert!(Cardinality::ONE_OR_MORE.is_list());
    }

    // --- Custom cardinality tests ---

    #[test]
    fn test_custom_cardinality() {
        let exactly_two = Cardinality::new(2, Some(2));
        assert!(!exactly_two.allows_empty());
        assert!(!exactly_two.allows_one());
        assert!(exactly_two.allows_many());
        assert!(exactly_two.requires_one());
        assert!(!exactly_two.is_scalar());
        assert!(exactly_two.is_list());

        let two_to_five = Cardinality::new(2, Some(5));
        assert!(!two_to_five.allows_empty());
        assert!(!two_to_five.allows_one());
        assert!(two_to_five.allows_many());
    }

    #[test]
    #[should_panic(expected = "invalid cardinality")]
    fn test_invalid_cardinality_panics() {
        Cardinality::new(5, Some(2));
    }

    // --- Test case generation ---

    #[test]
    fn test_test_cases() {
        // ZERO [0,0]: only 0 is in-range
        assert_eq!(Cardinality::ZERO.test_cases(), vec![0]);
        // ONE [1,1]: only 1 is in-range
        assert_eq!(Cardinality::ONE.test_cases(), vec![1]);
        // ZERO_OR_ONE [0,1]: 0 and 1 are in-range
        assert_eq!(Cardinality::ZERO_OR_ONE.test_cases(), vec![0, 1]);
        // ZERO_OR_MORE [0,∞): 0 and 1 are in-range
        assert_eq!(Cardinality::ZERO_OR_MORE.test_cases(), vec![0, 1]);
        // ONE_OR_MORE [1,∞): 1 and 2 are in-range
        assert_eq!(Cardinality::ONE_OR_MORE.test_cases(), vec![1, 2]);
    }

    #[test]
    fn test_test_cases_capped() {
        // Large bounded max should be capped.
        let bounded = Cardinality::new(0, Some(1000));
        assert_eq!(bounded.test_cases_capped(10), vec![0, 1, 10]);

        // Cap within range for non-zero min.
        let bounded = Cardinality::new(50, Some(1000));
        assert_eq!(bounded.test_cases_capped(60), vec![50, 51, 60]);

        // If cap < min, do not cap (preserve validity).
        let bounded = Cardinality::new(70, Some(1000));
        assert_eq!(bounded.test_cases_capped(60), vec![70, 71, 1000]);

        // Unbounded: no synthetic "large" values are added here.
        let unbounded = Cardinality::new(0, None);
        assert_eq!(unbounded.test_cases_capped(5), vec![0, 1]);
    }

    #[test]
    fn test_test_cases_with_strategy() {
        let bounded = Cardinality::new(0, Some(1000));
        assert_eq!(
            bounded.test_cases_with_strategy(CardinalitySamplingStrategy::BoundaryOnly),
            vec![0, 1, 1000]
        );
        assert_eq!(
            bounded
                .test_cases_with_strategy(CardinalitySamplingStrategy::BoundaryWithUpperBound(12)),
            vec![0, 1, 12]
        );
    }

    #[test]
    fn test_test_cases_for_tests_uses_boundary_only_sampling() {
        let bounded = Cardinality::new(0, Some(1000));
        assert_eq!(bounded.test_cases_for_tests(), vec![0, 1, 1000]);
    }

    #[test]
    fn test_seed_placeholder_policy_known_types() {
        assert_eq!(
            seed_placeholder_policy_for_type_id("String"),
            SeedPlaceholderPolicy::Generated
        );
        assert_eq!(
            seed_placeholder_policy_for_type_id("OptionalString"),
            SeedPlaceholderPolicy::Generated
        );
        assert_eq!(
            seed_placeholder_policy_for_type_id("TransportResponse"),
            SeedPlaceholderPolicy::ExplicitSeedRequired
        );
    }

    #[test]
    fn test_semantic_carrier_class_known_types() {
        assert_eq!(
            semantic_carrier_class_for_type_id("String"),
            SemanticCarrierClass::StructuralGeneratable
        );
        assert_eq!(
            semantic_carrier_class_for_type_id("OptionalString"),
            SemanticCarrierClass::StructuralGeneratable
        );
        assert_eq!(
            semantic_carrier_class_for_type_id("TransportResponse"),
            SemanticCarrierClass::SemanticCarrier
        );
        assert_eq!(
            TypeId::from("ToolHandle").semantic_carrier_class(),
            SemanticCarrierClass::SemanticCarrier
        );
        assert_eq!(
            TypeId::from("Map<String,String>").semantic_carrier_class(),
            SemanticCarrierClass::StructuralGeneratable
        );
        assert_eq!(
            TypeId::from("Map<String,Credential>").semantic_carrier_class(),
            SemanticCarrierClass::SemanticCarrier
        );
    }

    #[test]
    fn test_semantic_carrier_class_parametric_wrappers() {
        assert_eq!(
            semantic_carrier_class_for_type_id("Optional<String>"),
            SemanticCarrierClass::StructuralGeneratable
        );
        assert_eq!(
            semantic_carrier_class_for_type_id("List<Map<String,Int>>"),
            SemanticCarrierClass::StructuralGeneratable
        );
        assert_eq!(
            semantic_carrier_class_for_type_id("Set<Credential>"),
            SemanticCarrierClass::SemanticCarrier
        );
        assert_eq!(
            semantic_carrier_class_for_type_id("CredentialList"),
            SemanticCarrierClass::SemanticCarrier
        );
    }

    #[test]
    fn test_semantic_carrier_kind_known_types() {
        assert_eq!(
            semantic_carrier_kind_for_type_id("String"),
            SemanticCarrierKind::Structural
        );
        assert_eq!(
            semantic_carrier_kind_for_type_id("SecretName"),
            SemanticCarrierKind::Structural
        );
        assert_eq!(
            semantic_carrier_kind_for_type_id("GcpProjectId"),
            SemanticCarrierKind::Structural
        );
        assert_eq!(
            semantic_carrier_kind_for_type_id("GcpServiceAccountEmail"),
            SemanticCarrierKind::Structural
        );
        assert_eq!(
            semantic_carrier_kind_for_type_id("TransportRequest"),
            SemanticCarrierKind::TransportRequest
        );
        assert_eq!(
            semantic_carrier_kind_for_type_id("RestResponse"),
            SemanticCarrierKind::TransportResponse
        );
        assert_eq!(
            semantic_carrier_kind_for_type_id("Credential"),
            SemanticCarrierKind::Credential
        );
        assert_eq!(
            semantic_carrier_kind_for_type_id("Map<String,Credential>"),
            SemanticCarrierKind::UnknownSemantic
        );
    }

    #[test]
    fn test_semantic_carrier_compatibility_strict() {
        assert!(semantic_carrier_compatible(
            &TypeId::from("String"),
            &TypeId::from("Any")
        ));
        assert!(semantic_carrier_compatible(
            &TypeId::from("TransportResponse"),
            &TypeId::from("RestResponse")
        ));
        assert!(!semantic_carrier_compatible(
            &TypeId::from("Credential"),
            &TypeId::from("Any")
        ));
        assert!(!semantic_carrier_compatible(
            &TypeId::from("Credential"),
            &TypeId::from("Secret")
        ));
        assert!(!semantic_carrier_compatible(
            &TypeId::from("CustomAuthToken"),
            &TypeId::from("Any")
        ));
    }

    #[test]
    fn test_parse_map_type_id() {
        assert_eq!(
            parse_map_type_id("Map<String,String>"),
            Some(("String".to_string(), "String".to_string()))
        );
        assert_eq!(
            parse_map_type_id("Map<String, Map<String,Int>>"),
            Some(("String".to_string(), "Map<String,Int>".to_string()))
        );
        assert_eq!(parse_map_type_id("Map<String>"), None);
    }

    #[test]
    fn test_value_backing_for_parametric_wrappers() {
        assert_eq!(
            value_backing_for_type_id("List<String>"),
            ValueBacking::List
        );
        assert_eq!(value_backing_for_type_id("Set<String>"), ValueBacking::Set);
        assert_eq!(
            value_backing_for_type_id("Optional<String>"),
            ValueBacking::String
        );
        assert_eq!(
            value_backing_for_type_id("GcpProjectId"),
            ValueBacking::String
        );
        assert_eq!(
            value_backing_for_type_id("GcpSubjectToken"),
            ValueBacking::String
        );
        assert_eq!(value_backing_for_type_id("Credential"), ValueBacking::Map);
    }

    #[test]
    fn test_value_compatible_with_type_id() {
        use crate::value::Value;
        use std::collections::BTreeMap;

        assert!(value_compatible_with_type_id(
            "Optional<String>",
            &Value::Unit
        ));
        assert!(value_compatible_with_type_id(
            "Optional<String>",
            &Value::Str("x".to_string())
        ));
        assert!(!value_compatible_with_type_id(
            "Optional<String>",
            &Value::Int(1)
        ));

        let mut map = BTreeMap::new();
        map.insert("a".to_string(), Value::Int(1));
        map.insert("b".to_string(), Value::Int(2));
        assert!(value_compatible_with_type_id(
            "Map<String,Int>",
            &Value::Map(map)
        ));

        assert!(value_compatible_with_type_id(
            "Platform",
            &Value::Str("linux".into())
        ));
        let mut credential_map = BTreeMap::new();
        credential_map.insert(
            "token".to_string(),
            Value::Secret(crate::SecretString::new("tok")),
        );
        credential_map.insert("scheme".to_string(), Value::Str("bearer".into()));
        credential_map.insert("source_type".to_string(), Value::Str("static".into()));
        credential_map.insert(
            "cap".to_string(),
            Value::Secret(crate::SecretString::new("capability")),
        );
        assert!(value_compatible_with_type_id(
            "Credential",
            &Value::Map(credential_map)
        ));
        assert!(value_compatible_with_type_id("Any", &Value::Skipped));

        assert_eq!(value_kind_name(&Value::Int(7)), "Int");
    }

    #[test]
    fn test_seed_placeholder_policy_fail_closed() {
        assert_eq!(
            seed_placeholder_policy_for_type_id("CustomAuthToken"),
            SeedPlaceholderPolicy::ExplicitSeedRequired
        );
        assert_eq!(
            TypeId::from("SomeNewCarrierType").seed_placeholder_policy(),
            SeedPlaceholderPolicy::ExplicitSeedRequired
        );
    }

    #[test]
    fn test_boundary_values() {
        // ZERO [0,0]: {0, 1(above-max)}
        assert_eq!(Cardinality::ZERO.boundary_values(), vec![0, 1]);
        // ONE [1,1]: {0(below-min), 1, 2(above-max)}
        assert_eq!(Cardinality::ONE.boundary_values(), vec![0, 1, 2]);
        // ZERO_OR_ONE [0,1]: {0, 1, 2(above-max)}
        assert_eq!(Cardinality::ZERO_OR_ONE.boundary_values(), vec![0, 1, 2]);
        // ZERO_OR_MORE [0,∞): {0, 1}
        assert_eq!(Cardinality::ZERO_OR_MORE.boundary_values(), vec![0, 1]);
        // ONE_OR_MORE [1,∞): {0(below-min), 1, 2}
        assert_eq!(Cardinality::ONE_OR_MORE.boundary_values(), vec![0, 1, 2]);
        // Custom [2,5]: {1(below-min), 2, 3(min+1), 5, 6(above-max)}
        assert_eq!(
            Cardinality::new(2, Some(5)).boundary_values(),
            vec![1, 2, 3, 5, 6]
        );
    }

    #[test]
    fn test_allows_count() {
        assert!(Cardinality::ONE.allows_count(1));
        assert!(!Cardinality::ONE.allows_count(0));
        assert!(!Cardinality::ONE.allows_count(2));

        assert!(Cardinality::ZERO_OR_MORE.allows_count(0));
        assert!(Cardinality::ZERO_OR_MORE.allows_count(100));

        let custom = Cardinality::new(2, Some(5));
        assert!(!custom.allows_count(1));
        assert!(custom.allows_count(2));
        assert!(custom.allows_count(3));
        assert!(custom.allows_count(5));
        assert!(!custom.allows_count(6));
    }

    // --- Satisfaction tests (verify the 25-case truth table) ---

    #[test]
    fn test_satisfies_complete_matrix() {
        const ZERO: Cardinality = Cardinality::ZERO;
        const ONE: Cardinality = Cardinality::ONE;
        const ZERO_OR_ONE: Cardinality = Cardinality::ZERO_OR_ONE;
        const ZERO_OR_MORE: Cardinality = Cardinality::ZERO_OR_MORE;
        const ONE_OR_MORE: Cardinality = Cardinality::ONE_OR_MORE;

        // Zero output
        assert!(ZERO.satisfies(ZERO));
        assert!(!ZERO.satisfies(ONE));
        assert!(ZERO.satisfies(ZERO_OR_ONE));
        assert!(ZERO.satisfies(ZERO_OR_MORE));
        assert!(!ZERO.satisfies(ONE_OR_MORE));

        // One output
        assert!(!ONE.satisfies(ZERO));
        assert!(ONE.satisfies(ONE));
        assert!(ONE.satisfies(ZERO_OR_ONE));
        assert!(ONE.satisfies(ZERO_OR_MORE));
        assert!(ONE.satisfies(ONE_OR_MORE));

        // ZeroOrOne output
        assert!(!ZERO_OR_ONE.satisfies(ZERO));
        assert!(!ZERO_OR_ONE.satisfies(ONE));
        assert!(ZERO_OR_ONE.satisfies(ZERO_OR_ONE));
        assert!(ZERO_OR_ONE.satisfies(ZERO_OR_MORE));
        assert!(!ZERO_OR_ONE.satisfies(ONE_OR_MORE));

        // ZeroOrMore output
        assert!(!ZERO_OR_MORE.satisfies(ZERO));
        assert!(!ZERO_OR_MORE.satisfies(ONE));
        assert!(!ZERO_OR_MORE.satisfies(ZERO_OR_ONE));
        assert!(ZERO_OR_MORE.satisfies(ZERO_OR_MORE));
        assert!(!ZERO_OR_MORE.satisfies(ONE_OR_MORE));

        // OneOrMore output
        assert!(!ONE_OR_MORE.satisfies(ZERO));
        assert!(!ONE_OR_MORE.satisfies(ONE));
        assert!(!ONE_OR_MORE.satisfies(ZERO_OR_ONE));
        assert!(ONE_OR_MORE.satisfies(ZERO_OR_MORE));
        assert!(ONE_OR_MORE.satisfies(ONE_OR_MORE));
    }

    #[test]
    fn test_check_satisfies_ok() {
        assert!(Cardinality::ONE.check_satisfies(Cardinality::ONE).is_ok());
    }

    #[test]
    fn test_check_satisfies_err() {
        let err = Cardinality::ZERO_OR_ONE
            .check_satisfies(Cardinality::ONE)
            .unwrap_err();
        assert_eq!(err.output, Cardinality::ZERO_OR_ONE);
        assert_eq!(err.input, Cardinality::ONE);
        assert!(!err.reason.is_empty());
    }

    // --- Lattice algebra tests ---

    #[test]
    fn test_join() {
        const ZERO: Cardinality = Cardinality::ZERO;
        const ONE: Cardinality = Cardinality::ONE;
        const ZERO_OR_ONE: Cardinality = Cardinality::ZERO_OR_ONE;
        const ZERO_OR_MORE: Cardinality = Cardinality::ZERO_OR_MORE;
        const ONE_OR_MORE: Cardinality = Cardinality::ONE_OR_MORE;
        assert_eq!(ONE.join(ZERO_OR_ONE), ZERO_OR_ONE);
        assert_eq!(ONE.join(ONE_OR_MORE), ONE_OR_MORE);
        assert_eq!(ZERO_OR_ONE.join(ONE_OR_MORE), ZERO_OR_MORE);
        assert_eq!(ONE.join(ONE), ONE);
        assert_eq!(ZERO.join(ZERO_OR_MORE), ZERO_OR_MORE);
    }

    #[test]
    fn test_meet() {
        const ZERO: Cardinality = Cardinality::ZERO;
        const ONE: Cardinality = Cardinality::ONE;
        const ZERO_OR_ONE: Cardinality = Cardinality::ZERO_OR_ONE;
        const ZERO_OR_MORE: Cardinality = Cardinality::ZERO_OR_MORE;
        const ONE_OR_MORE: Cardinality = Cardinality::ONE_OR_MORE;
        assert_eq!(ZERO_OR_ONE.meet(ONE_OR_MORE), Some(ONE));
        assert_eq!(ZERO_OR_MORE.meet(ONE), Some(ONE));
        assert_eq!(ZERO_OR_MORE.meet(ZERO_OR_ONE), Some(ZERO_OR_ONE));
        assert_eq!(ONE.meet(ONE), Some(ONE));
        // Empty intersection
        assert_eq!(ZERO.meet(ONE_OR_MORE), None);
    }

    #[test]
    fn test_product() {
        const ZERO: Cardinality = Cardinality::ZERO;
        const ONE: Cardinality = Cardinality::ONE;
        const ZERO_OR_MORE: Cardinality = Cardinality::ZERO_OR_MORE;
        const ONE_OR_MORE: Cardinality = Cardinality::ONE_OR_MORE;
        assert_eq!(ONE.product(ONE), ONE);
        assert_eq!(ONE_OR_MORE.product(ONE), ONE_OR_MORE);
        assert_eq!(ZERO_OR_MORE.product(ONE_OR_MORE), ZERO_OR_MORE);
        assert_eq!(ONE.product(ZERO), ZERO);
    }

    #[test]
    fn test_sum() {
        assert_eq!(
            Cardinality::ONE.sum(Cardinality::ZERO_OR_ONE),
            Cardinality::new(1, Some(2))
        );
        assert_eq!(
            Cardinality::ZERO_OR_MORE.sum(Cardinality::ONE),
            Cardinality::ONE_OR_MORE
        );
        assert_eq!(
            Cardinality::new(u32::MAX, Some(u32::MAX)).sum(Cardinality::ONE),
            Cardinality::new(u32::MAX, None)
        );
    }

    // --- Display tests ---

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Cardinality::ZERO), "0");
        assert_eq!(format!("{}", Cardinality::ONE), "1");
        assert_eq!(format!("{}", Cardinality::ZERO_OR_ONE), "0..1");
        assert_eq!(format!("{}", Cardinality::ZERO_OR_MORE), "0..*");
        assert_eq!(format!("{}", Cardinality::ONE_OR_MORE), "1..*");
        assert_eq!(format!("{}", Cardinality::new(2, Some(5))), "2..5");
        assert_eq!(format!("{}", Cardinality::new(3, Some(3))), "3");
    }

    // --- Default test ---

    #[test]
    fn test_default_is_one() {
        assert_eq!(Cardinality::default(), Cardinality::ONE);
    }

    // --- Serde tests ---

    #[test]
    fn test_serde_named_roundtrip() {
        for card in &[
            Cardinality::ZERO,
            Cardinality::ONE,
            Cardinality::ZERO_OR_ONE,
            Cardinality::ZERO_OR_MORE,
            Cardinality::ONE_OR_MORE,
        ] {
            let json = serde_json::to_string(card).unwrap();
            let back: Cardinality = serde_json::from_str(&json).unwrap();
            assert_eq!(*card, back, "roundtrip failed for {card}");
        }
    }

    #[test]
    fn test_serde_named_strings() {
        assert_eq!(
            serde_json::to_string(&Cardinality::ZERO).unwrap(),
            "\"Zero\""
        );
        assert_eq!(serde_json::to_string(&Cardinality::ONE).unwrap(), "\"One\"");
        assert_eq!(
            serde_json::to_string(&Cardinality::ZERO_OR_ONE).unwrap(),
            "\"ZeroOrOne\""
        );
        assert_eq!(
            serde_json::to_string(&Cardinality::ZERO_OR_MORE).unwrap(),
            "\"ZeroOrMore\""
        );
        assert_eq!(
            serde_json::to_string(&Cardinality::ONE_OR_MORE).unwrap(),
            "\"OneOrMore\""
        );
    }

    #[test]
    fn test_serde_custom_cardinality() {
        let custom = Cardinality::new(2, Some(5));
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("\"min\":2"));
        assert!(json.contains("\"max\":5"));

        let back: Cardinality = serde_json::from_str(&json).unwrap();
        assert_eq!(custom, back);
    }

    #[test]
    fn test_serde_backward_compat_from_string() {
        // Old serialized format was just a string variant name
        let card: Cardinality = serde_json::from_str("\"One\"").unwrap();
        assert_eq!(card, Cardinality::ONE);
    }

    #[test]
    fn test_serde_unbounded_max() {
        let unbounded = Cardinality::new(3, None);
        let json = serde_json::to_string(&unbounded).unwrap();
        let back: Cardinality = serde_json::from_str(&json).unwrap();
        assert_eq!(unbounded, back);
        assert_eq!(back.max, None);
    }
}

// =============================================================================
// Property-based tests for algebraic laws
// =============================================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_cardinality() -> impl Strategy<Value = Cardinality> {
        (0u32..10, prop::option::of(0u32..10)).prop_map(|(min, max)| {
            let max = max.map(|m| m.max(min)); // ensure valid interval
            Cardinality { min, max }
        })
    }

    proptest! {
        // --- Satisfies laws ---

        /// Reflexivity: everything satisfies itself
        #[test]
        fn satisfies_reflexive(c in arb_cardinality()) {
            prop_assert!(c.satisfies(c));
        }

        /// Transitivity: if a satisfies b and b satisfies c, then a satisfies c
        #[test]
        fn satisfies_transitive(
            a in arb_cardinality(),
            b in arb_cardinality(),
            c in arb_cardinality()
        ) {
            if a.satisfies(b) && b.satisfies(c) {
                prop_assert!(a.satisfies(c));
            }
        }

        // --- Join laws ---

        /// Join is commutative
        #[test]
        fn join_commutative(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(a.join(b), b.join(a));
        }

        /// Join is associative
        #[test]
        fn join_associative(
            a in arb_cardinality(),
            b in arb_cardinality(),
            c in arb_cardinality()
        ) {
            prop_assert_eq!(a.join(b).join(c), a.join(b.join(c)));
        }

        /// Join is idempotent
        #[test]
        fn join_idempotent(a in arb_cardinality()) {
            prop_assert_eq!(a.join(a), a);
        }

        /// Join is upper bound: a.join(b) accepts both a and b
        #[test]
        fn join_is_upper_bound(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert!(a.satisfies(a.join(b)));
            prop_assert!(b.satisfies(a.join(b)));
        }

        // --- Meet laws ---

        /// Meet is commutative
        #[test]
        fn meet_commutative(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(a.meet(b), b.meet(a));
        }

        /// Meet is associative
        #[test]
        fn meet_associative(
            a in arb_cardinality(),
            b in arb_cardinality(),
            c in arb_cardinality()
        ) {
            // Only test when all meets produce valid results
            if let (Some(ab), Some(bc)) = (a.meet(b), b.meet(c)) {
                prop_assert_eq!(ab.meet(c), a.meet(bc));
            }
        }

        /// Meet is idempotent
        #[test]
        fn meet_idempotent(a in arb_cardinality()) {
            prop_assert_eq!(a.meet(a), Some(a));
        }

        /// Meet is lower bound: meet(a,b) satisfies both a and b
        #[test]
        fn meet_is_lower_bound(a in arb_cardinality(), b in arb_cardinality()) {
            if let Some(m) = a.meet(b) {
                prop_assert!(m.satisfies(a));
                prop_assert!(m.satisfies(b));
            }
        }

        // --- Product laws ---

        /// Product with ONE is identity
        #[test]
        fn product_identity(c in arb_cardinality()) {
            prop_assert_eq!(c.product(Cardinality::ONE), c);
            prop_assert_eq!(Cardinality::ONE.product(c), c);
        }

        /// Product with ZERO is absorbing
        #[test]
        fn product_zero_absorbing(c in arb_cardinality()) {
            prop_assert_eq!(c.product(Cardinality::ZERO), Cardinality::ZERO);
            prop_assert_eq!(Cardinality::ZERO.product(c), Cardinality::ZERO);
        }

        /// Product is commutative
        #[test]
        fn product_commutative(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(a.product(b), b.product(a));
        }

        // --- Absorption laws ---

        /// Absorption: a.join(a.meet(b)) == a (when meet exists)
        #[test]
        fn absorption_join_meet(a in arb_cardinality(), b in arb_cardinality()) {
            if let Some(m) = a.meet(b) {
                prop_assert_eq!(a.join(m), a);
            }
        }

        /// Absorption: a.meet(a.join(b)) == a
        #[test]
        fn absorption_meet_join(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(a.meet(a.join(b)), Some(a));
        }

        // --- Serde roundtrip ---

        /// Serde roundtrip preserves value
        #[test]
        fn serde_roundtrip(c in arb_cardinality()) {
            let json = serde_json::to_string(&c).unwrap();
            let back: Cardinality = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(c, back);
        }

        // --- Display roundtrip (formatting is consistent) ---

        /// Display produces non-empty output
        #[test]
        fn display_nonempty(c in arb_cardinality()) {
            let displayed = format!("{}", c);
            prop_assert!(!displayed.is_empty());
        }
    }
}
