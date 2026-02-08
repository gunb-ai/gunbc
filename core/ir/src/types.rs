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

impl Cardinality {
    /// Default cap for test-case generation (prevents huge vectors in tests).
    pub const TEST_CASE_CAP: u32 = 64;

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

    /// Default test cases used by generators (with a safe cap).
    pub fn test_cases_for_tests(&self) -> Vec<u32> {
        self.test_cases_capped(Self::TEST_CASE_CAP)
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

impl TypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
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
