//! Core identifier types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Set-theoretic cardinality for port values.
///
/// Every port has a cardinality that describes how many values can flow through it.
/// This enables semantic test generation and runtime validation.
///
/// # Mathematical Mapping
///
/// - `Zero` = ∅ (empty set)
/// - `One` = {x} (singleton, exactly one element)
/// - `ZeroOrOne` = {x}? (optional, zero or one element)
/// - `ZeroOrMore` = {x}* (Kleene star, any number of elements)
/// - `OneOrMore` = {x}+ (Kleene plus, at least one element)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Cardinality {
    /// Exactly zero elements (empty set, void).
    /// Used for signals that carry no data, just timing.
    Zero,

    /// Exactly one element (scalar, required).
    /// This is the default for most ports.
    #[default]
    One,

    /// Zero or one element (optional/nullable).
    /// The value may or may not be present.
    ZeroOrOne,

    /// Zero or more elements (list, may be empty).
    /// Represents a potentially empty collection.
    ZeroOrMore,

    /// One or more elements (non-empty list).
    /// Represents a collection with at least one element.
    OneOrMore,
}

impl Cardinality {
    /// Returns true if this cardinality allows zero elements.
    pub fn allows_empty(&self) -> bool {
        matches!(self, Cardinality::Zero | Cardinality::ZeroOrOne | Cardinality::ZeroOrMore)
    }

    /// Returns true if this cardinality allows exactly one element.
    pub fn allows_one(&self) -> bool {
        !matches!(self, Cardinality::Zero)
    }

    /// Returns true if this cardinality allows multiple elements.
    pub fn allows_many(&self) -> bool {
        matches!(self, Cardinality::ZeroOrMore | Cardinality::OneOrMore)
    }

    /// Returns true if this cardinality requires at least one element.
    pub fn requires_one(&self) -> bool {
        matches!(self, Cardinality::One | Cardinality::OneOrMore)
    }

    /// Returns the test cases that should be generated for this cardinality.
    pub fn test_cases(&self) -> Vec<CardinalityCase> {
        match self {
            Cardinality::Zero => vec![CardinalityCase::Empty],
            Cardinality::One => vec![CardinalityCase::One],
            Cardinality::ZeroOrOne => vec![CardinalityCase::Empty, CardinalityCase::One],
            Cardinality::ZeroOrMore => vec![CardinalityCase::Empty, CardinalityCase::One, CardinalityCase::Many],
            Cardinality::OneOrMore => vec![CardinalityCase::One, CardinalityCase::Many],
        }
    }

    /// Check if this output cardinality satisfies an input cardinality requirement.
    ///
    /// Returns true if ALL possible outputs from `self` are acceptable by `input_requirement`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gunbc_ir::Cardinality;
    ///
    /// // One always satisfies One
    /// assert!(Cardinality::One.satisfies(Cardinality::One));
    ///
    /// // OneOrMore satisfies ZeroOrMore (non-empty fits in any-length)
    /// assert!(Cardinality::OneOrMore.satisfies(Cardinality::ZeroOrMore));
    ///
    /// // ZeroOrMore does NOT satisfy OneOrMore (might produce empty)
    /// assert!(!Cardinality::ZeroOrMore.satisfies(Cardinality::OneOrMore));
    ///
    /// // ZeroOrOne does NOT satisfy One (might produce zero)
    /// assert!(!Cardinality::ZeroOrOne.satisfies(Cardinality::One));
    /// ```
    pub fn satisfies(&self, input_requirement: Cardinality) -> bool {
        use Cardinality::*;
        
        match (self, input_requirement) {
            // Zero output
            (Zero, Zero) => true,
            (Zero, ZeroOrOne) => true,
            (Zero, ZeroOrMore) => true,
            (Zero, _) => false,  // Zero can't satisfy One or OneOrMore
            
            // One output - scalar always present
            (One, Zero) => false,  // Can't send one to void
            (One, One) => true,
            (One, ZeroOrOne) => true,
            (One, ZeroOrMore) => true,
            (One, OneOrMore) => true,
            
            // ZeroOrOne output - might be absent
            (ZeroOrOne, Zero) => false,
            (ZeroOrOne, One) => false,     // Might produce zero
            (ZeroOrOne, ZeroOrOne) => true,
            (ZeroOrOne, ZeroOrMore) => true,
            (ZeroOrOne, OneOrMore) => false, // Might produce zero
            
            // ZeroOrMore output - might be empty
            (ZeroOrMore, Zero) => false,
            (ZeroOrMore, One) => false,      // Might produce zero or many
            (ZeroOrMore, ZeroOrOne) => false, // Might produce many
            (ZeroOrMore, ZeroOrMore) => true,
            (ZeroOrMore, OneOrMore) => false, // Might produce zero
            
            // OneOrMore output - at least one, maybe more
            (OneOrMore, Zero) => false,
            (OneOrMore, One) => false,       // Might produce many
            (OneOrMore, ZeroOrOne) => false, // Might produce many
            (OneOrMore, ZeroOrMore) => true,
            (OneOrMore, OneOrMore) => true,
        }
    }

    /// Check if this output can satisfy the input, with detailed error.
    pub fn check_satisfies(&self, input_requirement: Cardinality) -> Result<(), CardinalityMismatch> {
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

    fn mismatch_reason(&self, input: Cardinality) -> String {
        use Cardinality::*;
        match (self, input) {
            (Zero, One) | (Zero, OneOrMore) => 
                "output produces nothing but input requires at least one".into(),
            (ZeroOrOne, One) | (ZeroOrMore, One) => 
                "output might be empty but input requires exactly one".into(),
            (ZeroOrOne, OneOrMore) | (ZeroOrMore, OneOrMore) => 
                "output might be empty but input requires non-empty".into(),
            (OneOrMore, One) => 
                "output might have multiple but input requires exactly one".into(),
            (OneOrMore, ZeroOrOne) | (ZeroOrMore, ZeroOrOne) => 
                "output might have multiple but input accepts at most one".into(),
            _ => format!("cardinality {} cannot satisfy {}", self, input),
        }
    }
}

/// Error when output cardinality doesn't satisfy input requirement.
#[derive(Debug, Clone)]
pub struct CardinalityMismatch {
    pub output: Cardinality,
    pub input: Cardinality,
    pub reason: String,
}

impl fmt::Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cardinality::Zero => write!(f, "0"),
            Cardinality::One => write!(f, "1"),
            Cardinality::ZeroOrOne => write!(f, "0..1"),
            Cardinality::ZeroOrMore => write!(f, "0..*"),
            Cardinality::OneOrMore => write!(f, "1..*"),
        }
    }
}

/// A specific cardinality case for test generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalityCase {
    /// Test with zero elements (empty list, None, etc.)
    Empty,
    /// Test with exactly one element
    One,
    /// Test with multiple elements (typically 2-3)
    Many,
}

/// Unique identifier for a node within a DAG.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
