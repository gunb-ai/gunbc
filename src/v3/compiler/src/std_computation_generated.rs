// AUTO-GENERATED from `src/v3/std/computation.dag`.
// Regenerate instead of hand-editing.

use super::{
    positive_descent_count, DescentEvidence, PositiveDescentAmount, ProportionalDivisor,
};

/// `SizeBound` carrier generated from `src/v3/std/computation.dag`.
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

/// `CallPattern` carrier generated from `src/v3/std/computation.dag`.
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

/// `ShrinkFactor` carrier generated from `src/v3/std/computation.dag`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShrinkFactor {
    UnitShrink,
    ConstantShrink { steps: PositiveDescentAmount },
    ProportionalShrink { divisor: ProportionalDivisor },
}

/// `IterationPrimitive` carrier generated from `src/v3/std/computation.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationPrimitive {
    Fold,
    Descend,
    Repeat,
}

/// `LoweringTarget` record generated from `src/v3/std/computation.dag`.
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

/// Signed `Int` top iterate count (`i64::MAX`) for `SizeBound::Forever` / `repeat(max_int)`.
pub fn forever_iteration_bound() -> i64 {
    i64::MAX
}

/// `none` when the bound is not constant.
pub fn constant_bound_value(bound: &SizeBound) -> Option<i64> {
    match bound {
        SizeBound::ExplicitCountZero => Some(0),
        SizeBound::ExplicitCountPositive { steps } => Some(positive_descent_count(steps)),
        SizeBound::Forever => Some(forever_iteration_bound()),
        _ => None,
    }
}

/// `IterationDimension` carrier generated from `src/v3/std/computation.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationDimension {
    TreeDescent,
    CollectionFold,
    ArithmeticRepeat,
}

/// `AlgebraProfile` carrier generated from `src/v3/std/algebra.dag`.
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

    kernel_algebra_profile(type_name).and_then(algebra_profile_to_dimension)
}

/// Kernel type name -> iteration algebra profile (`Int`, `List`, ...).
pub fn kernel_algebra_profile(type_name: &str) -> Option<AlgebraProfile> {
    super::BOOTSTRAPPED_DAG.kernel_algebra_profile(type_name)
}
