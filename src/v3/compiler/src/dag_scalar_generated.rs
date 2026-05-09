// AUTO-GENERATED from `src/v3/std/substrate.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralBits {
    Int(String),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub enum AtomPayload {
    Literal(LiteralBits),
    UnresolvedIdentifier(String),
    ResolvedByStructure(DeclarationId),
    ResolvedByName(DeclarationId),
    TypeParam(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval<D> {
    BoundedInterval {
        lower: D,
        width: IntervalWidth,
    },
    Unbounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositiveIntervalWidth {
    OneUnit,
    AdditionalUnit {
        previous: Box<PositiveIntervalWidth>,
    },
    UnitCount {
        units: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalWidth {
    ZeroWidth,
    PositiveWidth(PositiveIntervalWidth),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalityBound {
    Exact(u32),
    AtMostOne,
    Unbounded,
}

#[derive(Debug, Clone)]
pub struct TemplateArgument {
    pub parameter: DeclarationId,
    pub value: DeclarationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortState {
    Uninferred,
    Resolved(TypeShape),
    Unresolved,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorKind {
    Arithmetic(ArithmeticOp),
    Comparison(ComparisonOp),
    Logical(LogicalOp),
}
