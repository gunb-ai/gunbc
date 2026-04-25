// v3-grounding-pilot crate root -- T-Ground-Pilot toy inhabitance-search engine.
//
// PROBE SCOPE (T-Ground-Pilot worker brief):
//   Validate that algebra-homomorphism inhabitance search reproduces
//   today's name-keyed table-lookup routing for the Rust target on a
//   bounded primitive set: {i8, i16, i32, i64, u8, u16, u32, u64, bool, ()}.
//
// FRAMING QUESTION:
//   Does inhabitance-search routing — consuming structural target-primitive
//   declarations and selecting by algebra-homomorphism — produce the same
//   target-primitive selection as today's name-keyed table lookup, on a
//   small Rust pilot set?
//
// PROBE — NOT PRODUCTION:
//   This crate is a deliberate side-channel. It does not feed the emit
//   pipeline; it does not consume the v3 substrate's parsed-Dag form; it
//   does not edit dsl/extdeps/languages/rust/types.dag or dsl/std/coercion.dag.
//   It mirrors the structural facts authored in dsl/extdeps/languages/rust/
//   primitives.dag and dsl/std/integer.dag as Rust constants and walks them
//   to demonstrate routing equivalence on the pilot set.
//
//   Production routing through the .dag substrate ("the real walker")
//   lands in T-Ground-Engine after the proposal greenlights the approach.
//   This probe's job is the greenlight signal.
//
// CRATE-LEVEL ISOLATION:
//   Lives as a sibling crate of v3-compiler (not a module within it) so
//   the probe's lifecycle is fully isolated: zero compiler-internal deps,
//   no coupling to the production pipeline, and the SG-0 hand-Rust
//   ratchet on src/v3/compiler is untouched.
//
// DISSOLUTION:
//   When T-Ground-Engine produces the production walker, this entire
//   crate disappears alongside removal from workspace members. No
//   upstream consumers, no tests outside this file.
//
// SUBSTRATE-GAP FLAGS (carried forward from primitives.dag):
//   1. Two's-complement-wrap is a closed-enum field rather than a
//      where-clause refinement on the algebra carrier (DB-11).
//   2. TargetAlgebra/TargetCarrier are tag enums standing in for
//      first-class algebra/type references-as-data (T-Ground-Dissolve).
//   3. Unit modeled with TerminalAlgebra/TerminalCarrier sentinels;
//      DB-11 makes this Cardinality<T, Exactly(1)>.
//
// ESCALATION (per brief, do not absorb in lane):
//   - Any pilot-set type can't be structurally declared without inventing
//     a substrate capability beyond what's flagged above.
//   - Any routing-parity failure on the pilot set.
//   - Any case where inhabitance-search composition fails for a primitive
//     class (e.g. can't distinguish signed/unsigned via single homomorphism).
//   None of these triggered while authoring this probe; see PR description
//   for the parity-stratum finding flagged for manager judgment.

#![cfg_attr(not(test), allow(dead_code))]

// =============================================================================
// Structural target-side facts.
//
// Mirrors dsl/extdeps/languages/rust/primitives.dag. Authority for these
// values is the .dag file; the duplication here is a probe-scoped
// convenience until the production walker reads .dag declarations
// directly.
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetAlgebra {
    OrderedRing,
    Semiring,
    BooleanAlgebra,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCarrier {
    Bit,
    Byte,
    Word16,
    Word32,
    Word64,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerOverflow {
    TwoComplementWrap,
    Saturating,
    Trap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustPrimitive {
    pub target_name: &'static str,
    pub algebra: TargetAlgebra,
    pub carrier: TargetCarrier,
    pub is_copy: bool,
    pub overflow: Option<IntegerOverflow>,
}

const WRAP: Option<IntegerOverflow> = Some(IntegerOverflow::TwoComplementWrap);

pub const RUST_PILOT_PRIMITIVES: &[RustPrimitive] = &[
    // Signed integers — OrderedRing over machine-word carriers.
    RustPrimitive {
        target_name: "i8",
        algebra: TargetAlgebra::OrderedRing,
        carrier: TargetCarrier::Byte,
        is_copy: true,
        overflow: WRAP,
    },
    RustPrimitive {
        target_name: "i16",
        algebra: TargetAlgebra::OrderedRing,
        carrier: TargetCarrier::Word16,
        is_copy: true,
        overflow: WRAP,
    },
    RustPrimitive {
        target_name: "i32",
        algebra: TargetAlgebra::OrderedRing,
        carrier: TargetCarrier::Word32,
        is_copy: true,
        overflow: WRAP,
    },
    RustPrimitive {
        target_name: "i64",
        algebra: TargetAlgebra::OrderedRing,
        carrier: TargetCarrier::Word64,
        is_copy: true,
        overflow: WRAP,
    },
    // Unsigned integers — Semiring over machine-word carriers.
    RustPrimitive {
        target_name: "u8",
        algebra: TargetAlgebra::Semiring,
        carrier: TargetCarrier::Byte,
        is_copy: true,
        overflow: WRAP,
    },
    RustPrimitive {
        target_name: "u16",
        algebra: TargetAlgebra::Semiring,
        carrier: TargetCarrier::Word16,
        is_copy: true,
        overflow: WRAP,
    },
    RustPrimitive {
        target_name: "u32",
        algebra: TargetAlgebra::Semiring,
        carrier: TargetCarrier::Word32,
        is_copy: true,
        overflow: WRAP,
    },
    RustPrimitive {
        target_name: "u64",
        algebra: TargetAlgebra::Semiring,
        carrier: TargetCarrier::Word64,
        is_copy: true,
        overflow: WRAP,
    },
    // Bool — BooleanAlgebra over Bit.
    RustPrimitive {
        target_name: "bool",
        algebra: TargetAlgebra::BooleanAlgebra,
        carrier: TargetCarrier::Bit,
        is_copy: true,
        overflow: None,
    },
    // Unit — terminal object.
    RustPrimitive {
        target_name: "()",
        algebra: TargetAlgebra::Terminal,
        carrier: TargetCarrier::Terminal,
        is_copy: true,
        overflow: None,
    },
];

// =============================================================================
// Structural .dag-side facts.
//
// Mirrors dsl/std/integer.dag (Int8..Int64, UInt8..UInt64) and the
// std-side declarations of Bool and Unit. Each .dag-side type unfolds
// to an (algebra, carrier) pair; production resolution will read the
// real type-alias chain via the v3 substrate's resolve_item_types.
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DagType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Bool,
    Unit,
}

pub const DAG_PILOT_TYPES: &[DagType] = &[
    DagType::Int8,
    DagType::Int16,
    DagType::Int32,
    DagType::Int64,
    DagType::UInt8,
    DagType::UInt16,
    DagType::UInt32,
    DagType::UInt64,
    DagType::Bool,
    DagType::Unit,
];

/// Unfold a pilot .dag-side type to its structural (algebra, carrier) facts.
///
/// Authority: dsl/std/integer.dag (Int8..Int64, UInt8..UInt64), plus the
/// canonical std modeling of Bool as BooleanAlgebra<Bit> and Unit as the
/// terminal object.
pub fn dag_type_facts(t: DagType) -> (TargetAlgebra, TargetCarrier) {
    match t {
        DagType::Int8 => (TargetAlgebra::OrderedRing, TargetCarrier::Byte),
        DagType::Int16 => (TargetAlgebra::OrderedRing, TargetCarrier::Word16),
        DagType::Int32 => (TargetAlgebra::OrderedRing, TargetCarrier::Word32),
        DagType::Int64 => (TargetAlgebra::OrderedRing, TargetCarrier::Word64),
        DagType::UInt8 => (TargetAlgebra::Semiring, TargetCarrier::Byte),
        DagType::UInt16 => (TargetAlgebra::Semiring, TargetCarrier::Word16),
        DagType::UInt32 => (TargetAlgebra::Semiring, TargetCarrier::Word32),
        DagType::UInt64 => (TargetAlgebra::Semiring, TargetCarrier::Word64),
        DagType::Bool => (TargetAlgebra::BooleanAlgebra, TargetCarrier::Bit),
        DagType::Unit => (TargetAlgebra::Terminal, TargetCarrier::Terminal),
    }
}

// =============================================================================
// The toy inhabitance-search engine.
//
// Selection is by structural agreement on (algebra, carrier). Pilot scope
// per brief: single-satisfier match is acceptable; minimum-satisfier
// discipline and fail-closed tie-breaking with structured diagnostics are
// T-Ground-Engine, not Pilot.
//
// The pilot set is constructed so each (algebra, carrier) pair has exactly
// one satisfying primitive. If a future extension introduces ambiguity,
// the engine surfaces GroundingError::Ambiguous so callers can't silently
// pick — fail-closed by construction even at pilot scope.
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundingError {
    /// No declared primitive inhabits the requested (algebra, carrier).
    NoInhabitant {
        algebra: TargetAlgebra,
        carrier: TargetCarrier,
    },
    /// More than one declared primitive inhabits the requested
    /// (algebra, carrier). Pilot fails closed; T-Ground-Engine will
    /// produce a structured diagnostic naming candidates.
    Ambiguous {
        algebra: TargetAlgebra,
        carrier: TargetCarrier,
        candidates: Vec<&'static str>,
    },
}

/// Search RUST_PILOT_PRIMITIVES for the unique primitive inhabiting
/// (algebra, carrier). This is the algebra-homomorphism match the
/// proposal calls "the mapping should fall out from the algebra, not
/// from a hand-maintained table."
pub fn find_inhabitant(
    algebra: TargetAlgebra,
    carrier: TargetCarrier,
) -> Result<&'static RustPrimitive, GroundingError> {
    let matches: Vec<&'static RustPrimitive> = RUST_PILOT_PRIMITIVES
        .iter()
        .filter(|p| p.algebra == algebra && p.carrier == carrier)
        .collect();
    match matches.as_slice() {
        [] => Err(GroundingError::NoInhabitant { algebra, carrier }),
        [only] => Ok(*only),
        many => Err(GroundingError::Ambiguous {
            algebra,
            carrier,
            candidates: many.iter().map(|p| p.target_name).collect(),
        }),
    }
}

/// Top-level: ground a .dag-side pilot type to its Rust target primitive
/// by algebra-homomorphism search. This is the routing the production
/// walker will replace.
pub fn ground(t: DagType) -> Result<&'static RustPrimitive, GroundingError> {
    let (algebra, carrier) = dag_type_facts(t);
    find_inhabitant(algebra, carrier)
}

// =============================================================================
// Phase 3 — routing-stability tests.
//
// Two strata (see PR description):
//   A. Name-keyed parity. dsl/extdeps/languages/rust/types.dag's
//      rust_type_checkpoints declares 3 of the 10 pilot types: Int (= Int64)
//      → "i64", Bool → "bool", Unit → "()". The engine must produce the
//      same target_name on these.
//   B. Algebra-homomorphism extension. The remaining 7 pilot types
//      (Int8/16/32, UInt8..UInt64) have no name-keyed checkpoint in
//      types.dag — the only fallback is the OrderedRing → "i64"
//      algebra-inhabitant entry, which is width-blind and would
//      mis-route Int8 to i64. The engine must produce the
//      width-correct primitive (Int8 → "i8", UInt32 → "u32", etc.).
//      This is precisely the gap the proposal predicts and what the
//      pilot validates: structural (algebra, carrier) matching reaches
//      types the name-keyed table cannot.
//
// Per TESTING.md (hermetic, behavior-driven, unit-first) and
// feedback_test_timeout_2s.md (>2s = broken). These are pure-function
// table walks — sub-millisecond.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Stratum A.1 — Int (= Int64) routes to "i64" per the
    /// rust_type_checkpoints entry { dag_name: "Int", target_type: "i64" }.
    #[test]
    fn stratum_a_int_routes_to_i64() {
        let p = ground(DagType::Int64).expect("Int64 must ground");
        assert_eq!(p.target_name, "i64");
        assert!(p.is_copy);
    }

    /// Stratum A.2 — Bool routes to "bool".
    #[test]
    fn stratum_a_bool_routes_to_bool() {
        let p = ground(DagType::Bool).expect("Bool must ground");
        assert_eq!(p.target_name, "bool");
        assert!(p.is_copy);
    }

    /// Stratum A.3 — Unit routes to "()".
    #[test]
    fn stratum_a_unit_routes_to_unit_tuple() {
        let p = ground(DagType::Unit).expect("Unit must ground");
        assert_eq!(p.target_name, "()");
        assert!(p.is_copy);
    }

    /// Stratum B — width-distinct signed integers route to width-correct
    /// Rust primitives. These have no name-keyed checkpoint in types.dag;
    /// today's algebra-inhabitant fallback (OrderedRing → "i64") is
    /// width-blind.
    #[test]
    fn stratum_b_signed_widths_route_correctly() {
        for (dag, expected) in [
            (DagType::Int8, "i8"),
            (DagType::Int16, "i16"),
            (DagType::Int32, "i32"),
            (DagType::Int64, "i64"),
        ] {
            let p = ground(dag).unwrap_or_else(|e| panic!("{dag:?} must ground: {e:?}"));
            assert_eq!(p.target_name, expected, "routing for {dag:?}");
            assert_eq!(p.algebra, TargetAlgebra::OrderedRing);
        }
    }

    /// Stratum B — width-distinct unsigned integers route to width-correct
    /// Rust primitives. types.dag has *no* Semiring inhabitant declared,
    /// so today's machinery is fail-closed for these; the engine
    /// produces the canonical mapping.
    #[test]
    fn stratum_b_unsigned_widths_route_correctly() {
        for (dag, expected) in [
            (DagType::UInt8, "u8"),
            (DagType::UInt16, "u16"),
            (DagType::UInt32, "u32"),
            (DagType::UInt64, "u64"),
        ] {
            let p = ground(dag).unwrap_or_else(|e| panic!("{dag:?} must ground: {e:?}"));
            assert_eq!(p.target_name, expected, "routing for {dag:?}");
            assert_eq!(p.algebra, TargetAlgebra::Semiring);
        }
    }

    /// Coverage — every type in DAG_PILOT_TYPES grounds to exactly one
    /// primitive. Asserts the 10-element pilot set is fully covered.
    #[test]
    fn pilot_set_fully_covered() {
        for &dag in DAG_PILOT_TYPES {
            let r = ground(dag);
            assert!(r.is_ok(), "pilot type {dag:?} must ground; got {r:?}");
        }
    }

    /// Coverage — engine output names exactly match the canonical
    /// expectation across all 10 pilot types in one place.
    #[test]
    fn full_pilot_routing_table() {
        let expected: &[(DagType, &str)] = &[
            (DagType::Int8, "i8"),
            (DagType::Int16, "i16"),
            (DagType::Int32, "i32"),
            (DagType::Int64, "i64"),
            (DagType::UInt8, "u8"),
            (DagType::UInt16, "u16"),
            (DagType::UInt32, "u32"),
            (DagType::UInt64, "u64"),
            (DagType::Bool, "bool"),
            (DagType::Unit, "()"),
        ];
        for &(dag, want) in expected {
            let got = ground(dag).unwrap().target_name;
            assert_eq!(got, want, "routing parity for {dag:?}");
        }
    }

    /// Selection happens by structural (algebra, carrier) agreement —
    /// not by .dag-side name. Verified by routing through find_inhabitant
    /// directly with synthetic facts and observing the same primitive
    /// selection as the matching DagType produces.
    #[test]
    fn selection_is_by_algebra_homomorphism_not_name() {
        // Synthetic fact set equivalent to UInt32 (Semiring over Word32).
        let direct = find_inhabitant(TargetAlgebra::Semiring, TargetCarrier::Word32)
            .expect("Semiring × Word32 must have an inhabitant");
        let via_dag = ground(DagType::UInt32).expect("UInt32 must ground");
        assert_eq!(direct.target_name, via_dag.target_name);
        assert_eq!(direct.target_name, "u32");
    }

    /// Pilot is constructed so each (algebra, carrier) is uniquely
    /// inhabited. Confirms no two primitive declarations collide on the
    /// same structural key.
    #[test]
    fn pilot_primitives_have_unique_algebra_carrier_keys() {
        let mut seen: Vec<(TargetAlgebra, TargetCarrier)> = Vec::new();
        for p in RUST_PILOT_PRIMITIVES {
            let key = (p.algebra, p.carrier);
            assert!(
                !seen.contains(&key),
                "duplicate (algebra, carrier) key for {}: {:?}",
                p.target_name,
                key
            );
            seen.push(key);
        }
    }

    /// Fail-closed shape — a structural key with no declared inhabitant
    /// returns NoInhabitant rather than silently picking. (Word128 is
    /// out-of-pilot-scope; using it here as a probe for fail-closed
    /// behavior.)
    #[test]
    fn missing_inhabitant_fails_closed() {
        // BooleanAlgebra over Word64 is not a declared primitive in the
        // pilot set; fail-closed contract requires NoInhabitant.
        let r = find_inhabitant(TargetAlgebra::BooleanAlgebra, TargetCarrier::Word64);
        assert!(matches!(r, Err(GroundingError::NoInhabitant { .. })));
    }
}
