//! Host lowering projection for Lane E-C (`std.computation`).
//!
//! Authority: `dsl/std/computation.dag` ↔ `src/v3/std/computation.dag`. These Rust
//! spellings stay in structural parity with the `.dag` carriers while std block
//! bodies remain `ArrowBody::Unparsed` at bootstrap.
//!
//! **Scope:** structural isolation only — the same host projections previously
//! lived in `dag.rs`; this file splits them out so the E-P induction producer stays
//! in the monolith without interleaving. This is **not** R3 gate
//! `tier3_computation_mirror_dissolved` closure (no mirror deletion, no evaluator
//! substitution for those bodies); full dissolution remains T-Tier3-Dissolution
//! lane work.

use super::{positive_descent_count, DescentEvidence, PositiveDescentAmount, ProportionalDivisor};

/// 🟡 SCAFFOLD — `SizeBound` coproduct (`docs/modeling-discipline.md` §4).
///
/// Authority: `src/v3/std/computation.dag`. Variant taxonomy is durable; `param: String` and
/// other bootstrap bridges dissolve when size parameters become first-class substrate refs.
/// **Named trigger:** evaluated `std.computation` std block bodies (same dissolution wave as
/// the termination lattice mirror). **Ledger:** parity ratchet
/// `m2_substrate_inhabitance_test::computation_size_bound_helpers_match_dag_authority`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizeBound {
    CollectionSize { param: String },
    ParserStreamSize { witness: String },
    WorklistDrainSize { element: String },
    TreeSize { param: String },
    ArithmeticParam { param: String },
    ExplicitCountZero,
    ExplicitCountPositive { steps: PositiveDescentAmount },
    Forever,
}

pub fn tree_size_bound(param: String) -> SizeBound {
    SizeBound::TreeSize { param }
}

/// 🟡 SCAFFOLD — `CallPattern` coproduct (`docs/modeling-discipline.md` §4).
///
/// Authority: `src/v3/std/computation.dag`. Peano shrink payloads are proof-grade (terminal
/// at witness shape); `String` slots on `CallPattern` forward into `SizeBound.param` via
/// `lower_call_pattern` (no fabricated size labels). Dissolves with structural parameter refs
/// (E-P). **Named trigger:** same as [`SizeBound`]. **Ledger:**
/// `m2_substrate_inhabitance_test::computation_lowering_rust_mirror_matches_dag_authority`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallPattern {
    ChildAccessorCall {
        accessor: String,
    },
    CollectionShrinkCall {
        amount: PositiveDescentAmount,
        collection: String,
    },
    ArithmeticSubtractCall {
        steps: PositiveDescentAmount,
        ring_param: String,
    },
    ArithmeticDivideCall {
        divisor: ProportionalDivisor,
        ring_param: String,
    },
    ParserAdvanceCall {
        witness: String,
    },
    WorklistDrainCall {
        element: String,
    },
    FoldBodyCall {
        outer_collection: String,
    },
    SameArgumentCall,
}

/// 🟢 TERMINAL — `ShrinkFactor` coproduct (`docs/modeling-discipline.md` §4).
///
/// Authority: `src/v3/std/computation.dag`. Only unit / Peano constant / Peano proportional
/// shrink — illegal rates stay unrepresentable at the carrier. **Ledger:** exercised through
/// the same `m2_substrate_inhabitance_test` computation rows as [`CallPattern`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShrinkFactor {
    UnitShrink,
    ConstantShrink { steps: PositiveDescentAmount },
    ProportionalShrink { divisor: ProportionalDivisor },
}

/// 🟢 TERMINAL — `IterationPrimitive` coproduct (`docs/modeling-discipline.md` §4).
///
/// Closed `{Fold, Descend, Repeat}` behavioral alphabet (MODELING.md M9). Authority:
/// `src/v3/std/computation.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationPrimitive {
    Fold,
    Descend,
    Repeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringTarget {
    pub primitive: IterationPrimitive,
    pub bound: SizeBound,
    pub evidence: DescentEvidence,
    pub factor: Option<ShrinkFactor>,
}

pub fn lower_call_pattern(pattern: CallPattern) -> LoweringTarget {
    match pattern {
        CallPattern::ChildAccessorCall { accessor } => LoweringTarget {
            primitive: IterationPrimitive::Descend,
            bound: SizeBound::TreeSize { param: accessor },
            evidence: DescentEvidence::Strict,
            factor: None,
        },
        CallPattern::CollectionShrinkCall { amount, collection } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::CollectionSize { param: collection },
            evidence: DescentEvidence::Strict,
            factor: Some(ShrinkFactor::ConstantShrink { steps: amount }),
        },
        CallPattern::ArithmeticSubtractCall { steps, ring_param } => LoweringTarget {
            primitive: IterationPrimitive::Repeat,
            bound: SizeBound::ArithmeticParam { param: ring_param },
            evidence: DescentEvidence::Strict,
            factor: Some(ShrinkFactor::ConstantShrink { steps }),
        },
        CallPattern::ArithmeticDivideCall {
            divisor,
            ring_param,
        } => LoweringTarget {
            primitive: IterationPrimitive::Repeat,
            bound: SizeBound::ArithmeticParam { param: ring_param },
            evidence: DescentEvidence::Strict,
            factor: Some(ShrinkFactor::ProportionalShrink { divisor }),
        },
        CallPattern::ParserAdvanceCall { witness } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::ParserStreamSize { witness },
            evidence: DescentEvidence::Strict,
            factor: None,
        },
        CallPattern::WorklistDrainCall { element } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::WorklistDrainSize { element },
            evidence: DescentEvidence::Strict,
            factor: None,
        },
        CallPattern::FoldBodyCall { outer_collection } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::CollectionSize {
                param: outer_collection,
            },
            evidence: DescentEvidence::NonIncreasing,
            factor: None,
        },
        CallPattern::SameArgumentCall => LoweringTarget {
            primitive: IterationPrimitive::Repeat,
            bound: SizeBound::Forever,
            evidence: DescentEvidence::NonIncreasing,
            factor: None,
        },
    }
}

pub fn size_bound_param(bound: &SizeBound) -> Option<&str> {
    match bound {
        SizeBound::TreeSize { param } => Some(param.as_str()),
        SizeBound::CollectionSize { param } => Some(param.as_str()),
        SizeBound::ParserStreamSize { witness } => Some(witness.as_str()),
        SizeBound::WorklistDrainSize { element } => Some(element.as_str()),
        SizeBound::ArithmeticParam { param } => Some(param.as_str()),
        SizeBound::ExplicitCountZero
        | SizeBound::ExplicitCountPositive { .. }
        | SizeBound::Forever => None,
    }
}

pub fn is_constant_bound(bound: &SizeBound) -> bool {
    matches!(
        bound,
        SizeBound::ExplicitCountZero | SizeBound::ExplicitCountPositive { .. } | SizeBound::Forever
    )
}

/// Signed `Int` top iterate count (`i64::MAX`) for [`SizeBound::Forever`] / `repeat(max_int)`.
pub fn forever_iteration_bound() -> i64 {
    i64::MAX
}

/// `None` when `bound` is not constant (`ExplicitCount*` / `Forever` only).
pub fn constant_bound_value(bound: &SizeBound) -> Option<i64> {
    match bound {
        SizeBound::ExplicitCountZero => Some(0),
        SizeBound::ExplicitCountPositive { steps } => Some(positive_descent_count(steps)),
        SizeBound::Forever => Some(forever_iteration_bound()),
        _ => None,
    }
}

/// 🟢 TERMINAL — `IterationDimension` coproduct (`docs/modeling-discipline.md` §4).
///
/// Three-way projection from kernel algebra profiles onto iteration regimes. Authority:
/// `src/v3/std/computation.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationDimension {
    TreeDescent,
    CollectionFold,
    ArithmeticRepeat,
}

/// 🟢 TERMINAL — `AlgebraProfile` coproduct (`docs/modeling-discipline.md` §4).
///
/// Closed seven-variant mirror of `dsl/std/algebra.dag` `AlgebraProfile`.
/// The profile table itself is read from the lowered `kernel_algebra_profile`
/// `ValueBody::Map`, not from a hand-maintained Rust lookup table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlgebraProfile {
    OrderedRingProfile,
    ApproximateFieldProfile,
    BooleanAlgebraProfile,
    BooleanAlgebraCollectionProfile,
    FreeMonoidScalarProfile,
    FreeMonoidCollectionProfile,
    PartialFunctionProfile,
}

pub fn algebra_profile_to_dimension(profile: AlgebraProfile) -> Option<IterationDimension> {
    match profile {
        AlgebraProfile::FreeMonoidCollectionProfile
        | AlgebraProfile::FreeMonoidScalarProfile
        | AlgebraProfile::BooleanAlgebraCollectionProfile
        | AlgebraProfile::PartialFunctionProfile => Some(IterationDimension::CollectionFold),
        AlgebraProfile::OrderedRingProfile | AlgebraProfile::ApproximateFieldProfile => {
            Some(IterationDimension::ArithmeticRepeat)
        }
        AlgebraProfile::BooleanAlgebraProfile => None,
    }
}

pub fn type_iteration_dimension(type_name: &str) -> Option<IterationDimension> {
    if type_name == "Node" {
        return Some(IterationDimension::TreeDescent);
    }

    super::BOOTSTRAPPED_DAG
        .kernel_algebra_profile(type_name)
        .and_then(algebra_profile_to_dimension)
}
