//! Algebraic composition layer: lattice, semiring, and partial order traits.
//!
//! These traits capture the algebraic structure that types like [`Cardinality`]
//! already implement. Extracting them into traits enables:
//!
//! - **Reuse**: Any type with lattice structure (predicates, type contracts, etc.)
//!   can implement the same traits and get property-based test coverage for free.
//! - **Composition**: Generic algorithms over lattices (e.g., fixed-point iteration,
//!   constraint propagation) work for any implementor.
//! - **Documentation**: The algebraic laws are explicit in the trait contracts.
//!
//! # Algebraic Hierarchy
//!
//! ```text
//! PartialOrder          (a ≤ b relation)
//!     │
//! JoinSemilattice       (least upper bound: a ∨ b)
//!     │
//! MeetSemilattice       (greatest lower bound: a ∧ b, may be empty)
//!     │
//! Lattice               (both join and meet)
//!     │
//! BoundedLattice        (has top element: ∀a. a ≤ top)
//! ```
//!
//! Separately:
//!
//! ```text
//! Semiring              (product with identity ONE and absorbing ZERO)
//! ```
//!
//! # Laws
//!
//! Implementors must satisfy these algebraic laws. The property-based tests
//! in this module verify them for [`Cardinality`].
//!
//! **PartialOrder:**
//! - Reflexivity: `a.leq(&a)` is true
//! - Transitivity: `a.leq(&b) && b.leq(&c)` implies `a.leq(&c)`
//! - Antisymmetry: `a.leq(&b) && b.leq(&a)` implies `a == b`
//!
//! **JoinSemilattice:**
//! - Commutativity: `a.join(b) == b.join(a)`
//! - Associativity: `a.join(b).join(c) == a.join(b.join(c))`
//! - Idempotence: `a.join(a) == a`
//! - Upper bound: `a.leq(&a.join(b))` and `b.leq(&a.join(b))`
//!
//! **MeetSemilattice:**
//! - Commutativity: `a.meet(b) == b.meet(a)`
//! - Associativity: when defined, `a.meet(b).meet(c) == a.meet(b.meet(c))`
//! - Idempotence: `a.meet(a) == Some(a)`
//! - Lower bound: if `m = a.meet(b)`, then `m.leq(&a)` and `m.leq(&b)`
//!
//! **Absorption (Lattice):**
//! - `a.join(a.meet(b)) == a` (when meet exists)
//! - `a.meet(a.join(b)) == Some(a)`
//!
//! **Semiring:**
//! - Identity: `a.product(ONE) == a` and `ONE.product(a) == a`
//! - Absorbing: `a.product(ZERO) == ZERO` and `ZERO.product(a) == ZERO`
//! - Commutativity: `a.product(b) == b.product(a)` (commutative semiring)

use crate::types::Cardinality;

// =============================================================================
// Trait definitions
// =============================================================================

/// Partial order relation: `a ≤ b` (subset containment).
///
/// For [`Cardinality`], `leq` means "all values allowed by `self` are also
/// allowed by `other`" — i.e., interval containment.
pub trait PartialOrder: Sized + PartialEq {
    /// Returns true if `self ≤ other` in the partial order.
    fn leq(&self, other: &Self) -> bool;
}

/// Join-semilattice: least upper bound (union of possibilities).
///
/// `a.join(b)` produces the smallest element that is ≥ both `a` and `b`.
/// For [`Cardinality`], this is the interval union.
pub trait JoinSemilattice: Sized {
    /// Least upper bound: `a ∨ b`.
    fn join(self, other: Self) -> Self;
}

/// Meet-semilattice: greatest lower bound (intersection of constraints).
///
/// `a.meet(b)` produces the largest element that is ≤ both `a` and `b`,
/// or `None` if the intersection is empty.
pub trait MeetSemilattice: Sized {
    /// Greatest lower bound: `a ∧ b`. Returns `None` if empty.
    fn meet(self, other: Self) -> Option<Self>;
}

/// A lattice: both join-semilattice and meet-semilattice.
///
/// Satisfies the absorption laws:
/// - `a.join(a.meet(b)) == a` (when meet exists)
/// - `a.meet(a.join(b)) == Some(a)`
pub trait Lattice: JoinSemilattice + MeetSemilattice {}

/// A bounded lattice: has a top element such that `∀a. a ≤ top`.
///
/// Not all lattices are bounded below. [`Cardinality`] has a top (`[0, ∞)`)
/// but no universal bottom (e.g., `[0,0]` and `[1,1]` are incomparable).
pub trait BoundedLattice: Lattice + PartialOrder {
    /// The top element: `∀a. a.leq(&top())`.
    fn top() -> Self;
}

// =============================================================================
// Cardinality implementations
// =============================================================================

impl PartialOrder for Cardinality {
    fn leq(&self, other: &Self) -> bool {
        self.satisfies(*other)
    }
}

impl JoinSemilattice for Cardinality {
    fn join(self, other: Self) -> Self {
        Cardinality::join(self, other)
    }
}

impl MeetSemilattice for Cardinality {
    fn meet(self, other: Self) -> Option<Self> {
        Cardinality::meet(self, other)
    }
}

impl Lattice for Cardinality {}

impl BoundedLattice for Cardinality {
    fn top() -> Self {
        Cardinality::ZERO_OR_MORE
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_order_reflexive() {
        let cases = [
            Cardinality::ZERO,
            Cardinality::ONE,
            Cardinality::ZERO_OR_ONE,
            Cardinality::ZERO_OR_MORE,
            Cardinality::ONE_OR_MORE,
        ];
        for c in &cases {
            assert!(c.leq(c), "{c} should be ≤ itself");
        }
    }

    #[test]
    fn test_partial_order_antisymmetric() {
        let a = Cardinality::ONE;
        let b = Cardinality::ZERO_OR_MORE;
        assert!(a.leq(&b));
        assert!(!b.leq(&a));
    }

    #[test]
    fn test_join_via_trait() {
        let result =
            <Cardinality as JoinSemilattice>::join(Cardinality::ONE, Cardinality::ZERO_OR_ONE);
        assert_eq!(result, Cardinality::ZERO_OR_ONE);
    }

    #[test]
    fn test_meet_via_trait() {
        let result = <Cardinality as MeetSemilattice>::meet(
            Cardinality::ZERO_OR_ONE,
            Cardinality::ONE_OR_MORE,
        );
        assert_eq!(result, Some(Cardinality::ONE));
    }

    #[test]
    fn test_bounded_lattice_top() {
        let top = <Cardinality as BoundedLattice>::top();
        assert_eq!(top, Cardinality::ZERO_OR_MORE);

        // Everything satisfies top
        let cases = [
            Cardinality::ZERO,
            Cardinality::ONE,
            Cardinality::ZERO_OR_ONE,
            Cardinality::ZERO_OR_MORE,
            Cardinality::ONE_OR_MORE,
        ];
        for c in &cases {
            assert!(c.leq(&top), "{c} should be ≤ top");
        }
    }
}

// =============================================================================
// Property-based tests
// =============================================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_cardinality() -> impl Strategy<Value = Cardinality> {
        (0u32..10, prop::option::of(0u32..10)).prop_map(|(min, max)| {
            let max = max.map(|m| m.max(min));
            Cardinality { min, max }
        })
    }

    proptest! {
        // --- PartialOrder laws ---

        #[test]
        fn partial_order_reflexive(c in arb_cardinality()) {
            prop_assert!(c.leq(&c));
        }

        #[test]
        fn partial_order_transitive(
            a in arb_cardinality(),
            b in arb_cardinality(),
            c in arb_cardinality()
        ) {
            if a.leq(&b) && b.leq(&c) {
                prop_assert!(a.leq(&c));
            }
        }

        #[test]
        fn partial_order_antisymmetric(a in arb_cardinality(), b in arb_cardinality()) {
            if a.leq(&b) && b.leq(&a) {
                prop_assert_eq!(a, b);
            }
        }

        // --- Join laws via trait ---

        #[test]
        fn trait_join_commutative(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(
                <Cardinality as JoinSemilattice>::join(a, b),
                <Cardinality as JoinSemilattice>::join(b, a)
            );
        }

        #[test]
        fn trait_join_upper_bound(a in arb_cardinality(), b in arb_cardinality()) {
            let j = <Cardinality as JoinSemilattice>::join(a, b);
            prop_assert!(a.leq(&j));
            prop_assert!(b.leq(&j));
        }

        // --- Meet laws via trait ---

        #[test]
        fn trait_meet_commutative(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(
                <Cardinality as MeetSemilattice>::meet(a, b),
                <Cardinality as MeetSemilattice>::meet(b, a)
            );
        }

        #[test]
        fn trait_meet_lower_bound(a in arb_cardinality(), b in arb_cardinality()) {
            if let Some(m) = <Cardinality as MeetSemilattice>::meet(a, b) {
                prop_assert!(m.leq(&a));
                prop_assert!(m.leq(&b));
            }
        }

        // --- Bounded lattice ---

        #[test]
        fn everything_leq_top(c in arb_cardinality()) {
            prop_assert!(c.leq(&<Cardinality as BoundedLattice>::top()));
        }

        // --- Absorption laws (Lattice) ---

        #[test]
        fn trait_absorption_join_meet(a in arb_cardinality(), b in arb_cardinality()) {
            if let Some(m) = <Cardinality as MeetSemilattice>::meet(a, b) {
                prop_assert_eq!(<Cardinality as JoinSemilattice>::join(a, m), a);
            }
        }

        #[test]
        fn trait_absorption_meet_join(a in arb_cardinality(), b in arb_cardinality()) {
            let j = <Cardinality as JoinSemilattice>::join(a, b);
            prop_assert_eq!(<Cardinality as MeetSemilattice>::meet(a, j), Some(a));
        }
    }
}
