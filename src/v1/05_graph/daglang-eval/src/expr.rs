//! Lowered expression IR — the compiler's representation of fn body computation.
//!
//! Translates `ast::FnBody` → `LoweredFnBody` during lowering. Each pipeline
//! stage produces its own representation: `.dag → parse (AST) → typecheck →
//! lower (LoweredExpr) → eval`. Downstream consumers never see parser types.

use serde::{Deserialize, Serialize};

// ── IR types ────────────────────────────────────────────────────────────────

/// A lowered function body — the unit of computation for `fn` items.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredFnBody {
    pub stmts: Vec<LoweredStmt>,
    /// Parameter types for runtime boundary checks: `(name, type_id)`.
    /// Empty when type info is not available (e.g. synthetic bodies).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_types: Vec<(String, String)>,
    /// Return type for runtime boundary checks.
    /// `None` when type info is not available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
}

impl LoweredFnBody {
    /// Create a fn body with only statements (no type metadata).
    /// Used by synthetic/test bodies where type info is not available.
    pub fn from_stmts(stmts: Vec<LoweredStmt>) -> Self {
        Self {
            stmts,
            param_types: vec![],
            return_type: None,
        }
    }

    /// Create a fn body with full type metadata for boundary checks.
    pub fn with_types(
        stmts: Vec<LoweredStmt>,
        param_types: Vec<(String, String)>,
        return_type: Option<String>,
    ) -> Self {
        Self {
            stmts,
            param_types,
            return_type,
        }
    }
}

/// Typed reference to an expression leaf source used by lowerer wiring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeafRef {
    Param {
        name: String,
        field: Option<String>,
        ty: String,
    },
    Callable {
        endpoint: String,
        port: String,
    },
    Service {
        endpoint: String,
        port: String,
    },
}

/// A lowered statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredStmt {
    /// `let name = expr`
    Let(String, LoweredExpr),
    /// Expression statement (side effects or trailing return)
    Expr(LoweredExpr),
    /// `return { field: expr, ... }`
    Return(Vec<(String, LoweredExpr)>),
}

/// A lowered expression — fully independent of parser AST types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredExpr {
    /// Literal value
    Literal(LoweredLiteral),
    /// Variable reference
    Ident(String),
    /// Field access: `expr.field`
    FieldAccess {
        expr: Box<LoweredExpr>,
        field: String,
    },
    /// String interpolation: `"hello {name}"`
    StringInterp(Vec<LoweredStringPart>),
    /// Binary operation: `a + b`, `a == b`
    BinOp {
        left: Box<LoweredExpr>,
        op: LoweredBinOp,
        right: Box<LoweredExpr>,
    },
    /// Unary operation: `!x`, `-x`
    UnaryOp {
        op: LoweredUnaryOp,
        expr: Box<LoweredExpr>,
    },
    /// Conditional: `if cond { then } else { else_ }`
    IfElse {
        cond: Box<LoweredExpr>,
        then_: Box<LoweredExpr>,
        else_: Option<Box<LoweredExpr>>,
    },
    /// Pattern match
    Match {
        expr: Box<LoweredExpr>,
        arms: Vec<LoweredMatchArm>,
    },
    /// Function call: `f(a: x, b: y)` — named args preserved
    Call {
        name: String,
        args: Vec<(Option<String>, LoweredExpr)>,
    },
    /// Lambda: `x => body` or `(x, y) => body`
    Lambda {
        params: Vec<String>,
        body: Box<LoweredExpr>,
    },
    /// List literal: `[a, b, c]`
    List(Vec<LoweredExpr>),
    /// Block expression with local statements and a trailing value.
    Block(Vec<LoweredStmt>),
    /// Record literal: `Name { a: 1 }` or `{ a: 1 }`
    Record {
        type_name: Option<String>,
        fields: Vec<(String, LoweredExpr)>,
    },
    /// For loop (map sugar): `for x in iterable { body }`
    For {
        binding: String,
        iterable: Box<LoweredExpr>,
        body: Box<LoweredExpr>,
    },
    /// Sum-type variant construction: `Closed` or `Ok { value: x }`
    VariantConstruct {
        tag: String,
        fields: Vec<(String, LoweredExpr)>,
    },
}

/// Literal value (no Float — LoweredOp requires Eq; add via ordered-float if needed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredLiteral {
    Int(i64),
    Bool(bool),
    String(String),
    None,
}

/// String interpolation part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredStringPart {
    Literal(String),
    Expr(LoweredExpr),
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    NullCoalesce,
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredUnaryOp {
    Not,
    Neg,
}

/// Match arm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredMatchArm {
    pub pattern: LoweredPattern,
    pub guard: Option<LoweredExpr>,
    pub body: LoweredExpr,
}

/// Match pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredPattern {
    /// Bind to name (or unit variant)
    Ident(String),
    /// Variant with destructured fields
    Variant(String, Vec<(String, LoweredPattern)>),
    /// Wildcard `_`
    Wildcard,
    /// Literal value
    Literal(LoweredLiteral),
}
