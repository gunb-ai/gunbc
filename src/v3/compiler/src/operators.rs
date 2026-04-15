// Structural operator representation.
//
// **Dissolution receipt — Q3, Q4 operator dispatch.** Before M1(2.7)
// this module owned the `OPERATOR_FIELD_MAP` name-based bridge that
// infer.rs read twice: once to decide "is this target an operator?"
// (by inspecting `AtomPayload::UnresolvedIdentifier(String)` payload)
// and once to decide "arithmetic or comparison?" (via a hardcoded
// `is_comparison_operator` string match). That shape put the operator
// dispatch fact in two places with the string as the discriminator.
//
// The new shape is a structural coproduct. `OperatorKind` is a
// closed set of 10 binary operators split into two sub-enums:
// `ArithmeticOp` (returns operand type) and `ComparisonOp` (returns
// Bool). The parser knows at parse time which of the ten operators
// a source symbol represents, so the string→enum translation
// happens exactly once, at `OperatorKind::from_symbol`. Downstream
// code dispatches on the `OperatorKind` variant, not on a string.
//
// Consumers:
//   - `parse.rs` builds `SurfaceExpr::Operator { op, args, span }` when
//     it sees `+ - * / == != < <= > >=` in an expression position.
//   - `lower.rs` lowers `SurfaceExpr::Operator` to a `TransformNode`
//     whose `target: TransformTarget::Operator(OperatorKind)`.
//   - `infer.rs` dispatches on `TransformTarget::Operator`: arithmetic
//     operators produce `(T, T) -> T`, comparisons produce
//     `(T, T) -> Bool`. The output-type rule is encoded in the
//     variant, not in an adjacent string match.
//   - `lower.rs::descent_provable` checks for
//     `SurfaceExpr::Operator { op: ArithmeticOp::Sub, ... }` when
//     verifying structural descent on recursive self-calls. The old
//     `target == "-"` string match is gone.
//
// 4-pattern check on `OperatorKind`:
// - Pattern 1 (fact placement): the arithmetic-vs-comparison split is
//   a fact about the operator's signature shape, and it now lives on
//   the variant itself rather than scattered across infer.rs helpers.
// - Pattern 2 (variant-is-data): each sub-enum carries identical
//   Kind payload (no fields); they're label-only. The split is
//   algebraic, not data-carrying.
// - Pattern 3 (algebraic form): the coproduct partitions operators
//   by output-type rule, which is the substrate-level distinction
//   inference actually cares about.
// - Pattern 4 (dimensional): fails.
//
// Verdict: terminal at M1(2.7) modulo M2's larger operator story
// (where `1 + 2` desugars to `Int.add(1, 2)` and OperatorKind
// dissolves into regular algebra-field calls). When that lands, the
// dissolution trigger removes this enum entirely.

/// Arithmetic binary operators. All four have signature `(T, T) -> T`
/// where T is the operand type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Comparison binary operators. All six have signature `(T, T) -> Bool`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Top-level operator kind. Structural dispatch target for the
/// `TransformTarget::Operator` variant. The split between
/// `Arithmetic` and `Comparison` encodes the output-type rule:
/// arithmetic returns the operand type, comparison returns Bool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorKind {
    Arithmetic(ArithmeticOp),
    Comparison(ComparisonOp),
}

impl OperatorKind {
    /// Translate a source symbol to a structural `OperatorKind`. Used
    /// at parse time to commit to the enum variant as early as
    /// possible, so downstream code never re-parses the symbol
    /// string. Returns `None` for non-operator identifiers.
    pub fn from_symbol(symbol: &str) -> Option<OperatorKind> {
        match symbol {
            "+" => Some(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            "-" => Some(OperatorKind::Arithmetic(ArithmeticOp::Sub)),
            "*" => Some(OperatorKind::Arithmetic(ArithmeticOp::Mul)),
            "/" => Some(OperatorKind::Arithmetic(ArithmeticOp::Div)),
            "==" => Some(OperatorKind::Comparison(ComparisonOp::Eq)),
            "!=" => Some(OperatorKind::Comparison(ComparisonOp::Ne)),
            "<" => Some(OperatorKind::Comparison(ComparisonOp::Lt)),
            "<=" => Some(OperatorKind::Comparison(ComparisonOp::Le)),
            ">" => Some(OperatorKind::Comparison(ComparisonOp::Gt)),
            ">=" => Some(OperatorKind::Comparison(ComparisonOp::Ge)),
            _ => None,
        }
    }

    /// Human-readable source symbol for this operator. Used by
    /// diagnostics when the compiler needs to display the operator
    /// to the user. Total inverse of `from_symbol`.
    pub fn symbol(self) -> &'static str {
        match self {
            OperatorKind::Arithmetic(ArithmeticOp::Add) => "+",
            OperatorKind::Arithmetic(ArithmeticOp::Sub) => "-",
            OperatorKind::Arithmetic(ArithmeticOp::Mul) => "*",
            OperatorKind::Arithmetic(ArithmeticOp::Div) => "/",
            OperatorKind::Comparison(ComparisonOp::Eq) => "==",
            OperatorKind::Comparison(ComparisonOp::Ne) => "!=",
            OperatorKind::Comparison(ComparisonOp::Lt) => "<",
            OperatorKind::Comparison(ComparisonOp::Le) => "<=",
            OperatorKind::Comparison(ComparisonOp::Gt) => ">",
            OperatorKind::Comparison(ComparisonOp::Ge) => ">=",
        }
    }
}
