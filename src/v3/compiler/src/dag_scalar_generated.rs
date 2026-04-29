// AUTO-GENERATED from `src/v3/std/substrate.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralBits {
    Int(i64),
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

// PR-PreF domain tags — substrate `type Cardinal = Int` / `Ordinal = Int`;
// Rust mirrors use nonnegative width carriers matching `ExactInterval` uses.
pub type Cardinal = u32;

pub type Ordinal = u32;


// For ordered `D`, well-formed `ExactInterval` (`lo <= hi`): use
// `Interval::try_exact_interval` in `dag.rs` (substrate cannot express the constraint).


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval<D: Copy + PartialEq + Eq> {
    ExactInterval {
        lo: D,
        hi: D,
    },
    Unbounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalityBound {
    Exact(i64),
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
