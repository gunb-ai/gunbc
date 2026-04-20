//! Structural operator identities carried by `TransformTarget::Operator`.
//!
//! Extracted from `dag.rs` (L4b). These enums are a parse/lower/infer bridge
//! until the M2+ surface grammar rewrites operators to direct algebra-field
//! calls. See the module-level doc on [`ArithmeticOp`] for the full
//! dissolution receipt.

/// Structural operator identities carried by
/// [`super::TransformTarget::Operator`].
///
/// **🟡 Scaffold — operator shim family.** Re-homed receipt from the
/// deleted `operators.rs`: the richer source already exists in
/// `dsl/std/algebra.dag`, where arithmetic/comparison/logical
/// operations are declared as algebra fields. These enums remain the
/// current parse/lower/infer bridge only until the M2+ parser /
/// desugarer rewrites surface operators to direct algebra-field calls
/// (or explicit field-access syntax).
///
/// Q3/Q4 audit pointer: `ROADMAP.md` M1(2.7) Class 2 operator
/// dispatch, plus the mixed-lifecycle `TransformTarget` receipt below.
///
/// 4-pattern check on (`ArithmeticOp`, `ComparisonOp`, `LogicalOp`,
/// `OperatorKind`):
/// - Pattern 1 (fact placement): fails today. Parse, lowering,
///   inference, and emit all still dispatch on an operator-family
///   carrier.
/// - Pattern 2 (variant-is-data): fails. The labels encode distinct
///   source operators and typing rules; they are not interchangeable
///   payloads of one record.
/// - Pattern 3 (algebraic form): succeeds in the long-term design.
///   The richer source is the algebra field declaration (`add`, `eq`,
///   `meet`, `join`, ...), so this family is not terminal.
/// - Pattern 4 (dimensional): fails. The current surface does not yet
///   expose a smaller coordinate system that replaces operator
///   identity.
///
/// Verdict: 🟡 scaffold. Dissolution trigger: M2+ operator desugaring
/// / explicit algebra-field surface syntax eliminates
/// `TransformTarget::Operator` and these enum carriers collapse to
/// declaration refs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalOp {
    And,
    Or,
}

/// Operator family root for [`super::TransformTarget::Operator`]. Inherits the
/// scaffold receipt on [`ArithmeticOp`]; do not reclassify this as
/// terminal without also updating the shared operator-shim trigger in
/// `ROADMAP.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorKind {
    Arithmetic(ArithmeticOp),
    Comparison(ComparisonOp),
    Logical(LogicalOp),
}
