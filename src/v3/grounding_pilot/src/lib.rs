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
//   1. Two's-complement-wrap is a closed-enum field on IntegerPrimitive
//      rather than a where-clause refinement on the algebra carrier (DB-11).
//   2. IntegerAlgebra/NonIntegerAlgebra/TargetCarrier are tag enums
//      standing in for first-class algebra/type references-as-data
//      (T-Ground-Dissolve).
//   3. Unit modeled with Terminal sentinels; DB-11 makes this
//      Cardinality<T, Exactly(1)>.
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
//
// State-space discipline: algebra tags are partitioned into
// integer-bearing (IntegerAlgebra) and non-integer-bearing
// (NonIntegerAlgebra). RustPrimitive is sum-typed so that overflow lives
// only on IntegerPrimitive — making `bool: Some(TwoComplementWrap)` and
// `i64: None` structurally unrepresentable rather than ruled out by
// convention.
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerAlgebra {
    OrderedRing,
    Semiring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonIntegerAlgebra {
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
pub enum RustPrimitive {
    IntegerPrimitive {
        target_name: &'static str,
        algebra: IntegerAlgebra,
        carrier: TargetCarrier,
        range_min_inclusive: &'static str,
        range_max_inclusive: &'static str,
        is_copy: bool,
        overflow: IntegerOverflow,
    },
    NonIntegerPrimitive {
        target_name: &'static str,
        algebra: NonIntegerAlgebra,
        carrier: TargetCarrier,
        is_copy: bool,
    },
}

/// Free-function accessor for the shared `target_name` field across both
/// `RustPrimitive` variants. CODING.md prefers data + free functions over
/// trait/impl method dispatch.
pub fn target_name(p: &RustPrimitive) -> &'static str {
    match p {
        RustPrimitive::IntegerPrimitive { target_name, .. } => target_name,
        RustPrimitive::NonIntegerPrimitive { target_name, .. } => target_name,
    }
}

pub fn is_copy(p: &RustPrimitive) -> bool {
    match p {
        RustPrimitive::IntegerPrimitive { is_copy, .. } => *is_copy,
        RustPrimitive::NonIntegerPrimitive { is_copy, .. } => *is_copy,
    }
}

/// Routing key — `(algebra, carrier)` pair flattened across the
/// integer/non-integer partition. The key is what `find_inhabitant`
/// matches on; the partition stays load-bearing in the data declaration
/// so overflow can attach only on the integer side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingKey {
    Integer {
        algebra: IntegerAlgebra,
        carrier: TargetCarrier,
    },
    NonInteger {
        algebra: NonIntegerAlgebra,
        carrier: TargetCarrier,
    },
}

pub fn routing_key(p: &RustPrimitive) -> RoutingKey {
    match p {
        RustPrimitive::IntegerPrimitive {
            algebra, carrier, ..
        } => RoutingKey::Integer {
            algebra: *algebra,
            carrier: *carrier,
        },
        RustPrimitive::NonIntegerPrimitive {
            algebra, carrier, ..
        } => RoutingKey::NonInteger {
            algebra: *algebra,
            carrier: *carrier,
        },
    }
}

pub const RUST_PILOT_PRIMITIVES: &[RustPrimitive] = &[
    // Signed integers — OrderedRing over machine-word carriers.
    RustPrimitive::IntegerPrimitive {
        target_name: "i8",
        algebra: IntegerAlgebra::OrderedRing,
        carrier: TargetCarrier::Byte,
        range_min_inclusive: "-128",
        range_max_inclusive: "127",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    RustPrimitive::IntegerPrimitive {
        target_name: "i16",
        algebra: IntegerAlgebra::OrderedRing,
        carrier: TargetCarrier::Word16,
        range_min_inclusive: "-32768",
        range_max_inclusive: "32767",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    RustPrimitive::IntegerPrimitive {
        target_name: "i32",
        algebra: IntegerAlgebra::OrderedRing,
        carrier: TargetCarrier::Word32,
        range_min_inclusive: "-2147483648",
        range_max_inclusive: "2147483647",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    RustPrimitive::IntegerPrimitive {
        target_name: "i64",
        algebra: IntegerAlgebra::OrderedRing,
        carrier: TargetCarrier::Word64,
        range_min_inclusive: "-9223372036854775808",
        range_max_inclusive: "9223372036854775807",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    // Unsigned integers — Semiring over machine-word carriers.
    RustPrimitive::IntegerPrimitive {
        target_name: "u8",
        algebra: IntegerAlgebra::Semiring,
        carrier: TargetCarrier::Byte,
        range_min_inclusive: "0",
        range_max_inclusive: "255",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    RustPrimitive::IntegerPrimitive {
        target_name: "u16",
        algebra: IntegerAlgebra::Semiring,
        carrier: TargetCarrier::Word16,
        range_min_inclusive: "0",
        range_max_inclusive: "65535",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    RustPrimitive::IntegerPrimitive {
        target_name: "u32",
        algebra: IntegerAlgebra::Semiring,
        carrier: TargetCarrier::Word32,
        range_min_inclusive: "0",
        range_max_inclusive: "4294967295",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    RustPrimitive::IntegerPrimitive {
        target_name: "u64",
        algebra: IntegerAlgebra::Semiring,
        carrier: TargetCarrier::Word64,
        range_min_inclusive: "0",
        range_max_inclusive: "18446744073709551615",
        is_copy: true,
        overflow: IntegerOverflow::TwoComplementWrap,
    },
    // Bool — BooleanAlgebra over Bit.
    RustPrimitive::NonIntegerPrimitive {
        target_name: "bool",
        algebra: NonIntegerAlgebra::BooleanAlgebra,
        carrier: TargetCarrier::Bit,
        is_copy: true,
    },
    // Unit — terminal object.
    RustPrimitive::NonIntegerPrimitive {
        target_name: "()",
        algebra: NonIntegerAlgebra::Terminal,
        carrier: TargetCarrier::Terminal,
        is_copy: true,
    },
];

// =============================================================================
// Structural .dag-side facts.
//
// Mirrors dsl/std/integer.dag (Int8..Int64, UInt8..UInt64) and the
// std-side declarations of Bool and Unit. Each .dag-side type unfolds
// to a RoutingKey; production resolution will read the real type-alias
// chain via the v3 substrate's resolve_item_types.
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

/// Unfold a pilot .dag-side type to its routing-key facts.
///
/// Authority: dsl/std/integer.dag (Int8..Int64, UInt8..UInt64), plus the
/// canonical std modeling of Bool as BooleanAlgebra<Bit> and Unit as the
/// terminal object.
pub fn dag_type_facts(t: DagType) -> RoutingKey {
    match t {
        DagType::Int8 => RoutingKey::Integer {
            algebra: IntegerAlgebra::OrderedRing,
            carrier: TargetCarrier::Byte,
        },
        DagType::Int16 => RoutingKey::Integer {
            algebra: IntegerAlgebra::OrderedRing,
            carrier: TargetCarrier::Word16,
        },
        DagType::Int32 => RoutingKey::Integer {
            algebra: IntegerAlgebra::OrderedRing,
            carrier: TargetCarrier::Word32,
        },
        DagType::Int64 => RoutingKey::Integer {
            algebra: IntegerAlgebra::OrderedRing,
            carrier: TargetCarrier::Word64,
        },
        DagType::UInt8 => RoutingKey::Integer {
            algebra: IntegerAlgebra::Semiring,
            carrier: TargetCarrier::Byte,
        },
        DagType::UInt16 => RoutingKey::Integer {
            algebra: IntegerAlgebra::Semiring,
            carrier: TargetCarrier::Word16,
        },
        DagType::UInt32 => RoutingKey::Integer {
            algebra: IntegerAlgebra::Semiring,
            carrier: TargetCarrier::Word32,
        },
        DagType::UInt64 => RoutingKey::Integer {
            algebra: IntegerAlgebra::Semiring,
            carrier: TargetCarrier::Word64,
        },
        DagType::Bool => RoutingKey::NonInteger {
            algebra: NonIntegerAlgebra::BooleanAlgebra,
            carrier: TargetCarrier::Bit,
        },
        DagType::Unit => RoutingKey::NonInteger {
            algebra: NonIntegerAlgebra::Terminal,
            carrier: TargetCarrier::Terminal,
        },
    }
}

// =============================================================================
// The toy inhabitance-search engine.
//
// Selection is by RoutingKey agreement. Pilot scope per brief:
// single-satisfier match is acceptable; minimum-satisfier discipline and
// fail-closed tie-breaking with structured diagnostics are
// T-Ground-Engine, not Pilot.
//
// The pilot set is constructed so each RoutingKey has exactly one
// satisfying primitive. If a future extension introduces ambiguity, the
// engine surfaces GroundingError::Ambiguous so callers can't silently
// pick — fail-closed by construction even at pilot scope.
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundingError {
    /// No declared primitive inhabits the requested routing key.
    NoInhabitant { key: RoutingKey },
    /// More than one declared primitive inhabits the requested routing
    /// key. Pilot fails closed; T-Ground-Engine will produce a
    /// structured diagnostic naming candidates.
    Ambiguous {
        key: RoutingKey,
        candidates: Vec<&'static str>,
    },
}

/// Search RUST_PILOT_PRIMITIVES for the unique primitive whose routing
/// key matches. This is the algebra-homomorphism match the proposal
/// calls "the mapping should fall out from the algebra, not from a
/// hand-maintained table."
pub fn find_inhabitant(key: RoutingKey) -> Result<&'static RustPrimitive, GroundingError> {
    let matches: Vec<&'static RustPrimitive> = RUST_PILOT_PRIMITIVES
        .iter()
        .filter(|p| routing_key(p) == key)
        .collect();
    match matches.as_slice() {
        [] => Err(GroundingError::NoInhabitant { key }),
        [only] => Ok(*only),
        many => Err(GroundingError::Ambiguous {
            key,
            candidates: many.iter().map(|p| target_name(p)).collect(),
        }),
    }
}

/// Top-level: ground a .dag-side pilot type to its Rust target primitive
/// by algebra-homomorphism search. This is the routing the production
/// walker will replace.
pub fn ground(t: DagType) -> Result<&'static RustPrimitive, GroundingError> {
    find_inhabitant(dag_type_facts(t))
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
        assert_eq!(target_name(p), "i64");
        assert!(is_copy(p));
    }

    /// Stratum A.2 — Bool routes to "bool".
    #[test]
    fn stratum_a_bool_routes_to_bool() {
        let p = ground(DagType::Bool).expect("Bool must ground");
        assert_eq!(target_name(p), "bool");
        assert!(is_copy(p));
    }

    /// Stratum A.3 — Unit routes to "()".
    #[test]
    fn stratum_a_unit_routes_to_unit_tuple() {
        let p = ground(DagType::Unit).expect("Unit must ground");
        assert_eq!(target_name(p), "()");
        assert!(is_copy(p));
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
            assert_eq!(target_name(p), expected, "routing for {dag:?}");
            assert!(matches!(
                p,
                RustPrimitive::IntegerPrimitive {
                    algebra: IntegerAlgebra::OrderedRing,
                    ..
                }
            ));
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
            assert_eq!(target_name(p), expected, "routing for {dag:?}");
            assert!(matches!(
                p,
                RustPrimitive::IntegerPrimitive {
                    algebra: IntegerAlgebra::Semiring,
                    ..
                }
            ));
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
            let got = target_name(ground(dag).unwrap());
            assert_eq!(got, want, "routing parity for {dag:?}");
        }
    }

    /// Selection happens by RoutingKey agreement — not by .dag-side name.
    /// Verified by routing through find_inhabitant directly with synthetic
    /// facts and observing the same primitive selection as the matching
    /// DagType produces.
    #[test]
    fn selection_is_by_algebra_homomorphism_not_name() {
        // Synthetic key equivalent to UInt32 (Semiring over Word32).
        let key = RoutingKey::Integer {
            algebra: IntegerAlgebra::Semiring,
            carrier: TargetCarrier::Word32,
        };
        let direct = find_inhabitant(key).expect("Semiring × Word32 must have an inhabitant");
        let via_dag = ground(DagType::UInt32).expect("UInt32 must ground");
        assert_eq!(target_name(direct), target_name(via_dag));
        assert_eq!(target_name(direct), "u32");
    }

    /// Pilot is constructed so each RoutingKey is uniquely inhabited.
    /// Confirms no two primitive declarations collide on the same key.
    #[test]
    fn pilot_primitives_have_unique_routing_keys() {
        let mut seen: Vec<RoutingKey> = Vec::new();
        for p in RUST_PILOT_PRIMITIVES {
            let key = routing_key(p);
            assert!(
                !seen.contains(&key),
                "duplicate routing key for {}: {:?}",
                target_name(p),
                key
            );
            seen.push(key);
        }
    }

    /// Fail-closed shape — a routing key with no declared inhabitant
    /// returns NoInhabitant rather than silently picking. (BooleanAlgebra
    /// over Word64 is out-of-pilot-scope; using it here as a probe.)
    #[test]
    fn missing_inhabitant_fails_closed() {
        let key = RoutingKey::NonInteger {
            algebra: NonIntegerAlgebra::BooleanAlgebra,
            carrier: TargetCarrier::Word64,
        };
        let r = find_inhabitant(key);
        assert!(matches!(r, Err(GroundingError::NoInhabitant { .. })));
    }

    /// State-space discipline — overflow is a field on IntegerPrimitive
    /// only; NonIntegerPrimitive structurally cannot carry an overflow.
    /// This test exists to lock the partition into the contract: if a
    /// future change collapses the variants back into a single record
    /// with `Option<IntegerOverflow>`, this test breaks. The match is
    /// exhaustive by sum-type construction.
    #[test]
    fn overflow_lives_only_on_integer_variant() {
        for p in RUST_PILOT_PRIMITIVES {
            match p {
                RustPrimitive::IntegerPrimitive { overflow, .. } => {
                    // Pilot population: every integer primitive uses
                    // two's-complement wrap (Rust release-mode arithmetic).
                    assert_eq!(*overflow, IntegerOverflow::TwoComplementWrap);
                }
                RustPrimitive::NonIntegerPrimitive { .. } => {
                    // No overflow field exists on this variant — the
                    // exhaustive match is the structural assertion.
                }
            }
        }
    }
}
