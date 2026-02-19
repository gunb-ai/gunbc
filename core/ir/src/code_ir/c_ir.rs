//! Tier 3 — CStyleIR: C-level intermediate representation.
//!
//! Lowered from AbstractIR (Tier 0). Introduces explicit memory management,
//! pointers, goto/label control flow, and C type system constructs.
//! This tier is the input to [`super::register_ir`] (Tier 4) lowering.

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

/// A C-level statement.
#[derive(Debug, Clone)]
pub enum CStmt {
    /// Variable declaration with optional initializer: `type name [= expr];`
    Decl {
        name: String,
        ty: CType,
        init: Option<CExpr>,
    },
    /// Assignment: `lhs = rhs;`
    Assign { lhs: CExpr, rhs: CExpr },
    /// Expression statement (side-effectful call, etc.).
    Expr(CExpr),
    /// `if (cond) { then } [else { else_body }]`
    If {
        cond: CExpr,
        then_body: Vec<CStmt>,
        else_body: Option<Vec<CStmt>>,
    },
    /// `for (init; cond; step) { body }`
    For {
        init: Box<CStmt>,
        cond: CExpr,
        step: Box<CStmt>,
        body: Vec<CStmt>,
    },
    /// `while (cond) { body }`
    While { cond: CExpr, body: Vec<CStmt> },
    /// `return [expr];`
    Return(Option<CExpr>),
    /// `goto label;`
    Goto(String),
    /// `label:`
    Label(String),
    /// `{ stmt... }` block scope
    BlockScope(Vec<CStmt>),
    /// `free(expr);`
    Free(CExpr),
    /// C comment: `/* text */` or `// text`
    Comment(String),
    /// Blank line.
    Blank,
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

/// A C-level expression.
#[derive(Debug, Clone)]
pub enum CExpr {
    /// Variable reference.
    Var(String),
    /// Integer literal.
    IntLit(i64),
    /// String literal (null-terminated `const char*`).
    StrLit(String),
    /// Character literal.
    CharLit(char),
    /// Boolean (rendered as `0`/`1` in C).
    BoolLit(bool),
    /// `NULL`
    Null,
    /// Function call: `func(args...)`
    Call { func: String, args: Vec<CExpr> },
    /// Binary operation: `left op right`
    BinOp {
        left: Box<CExpr>,
        op: String,
        right: Box<CExpr>,
    },
    /// Unary operation: `op expr` (e.g., `!`, `-`, `~`)
    UnaryOp { op: String, expr: Box<CExpr> },
    /// Field access: `expr.field`
    Field(Box<CExpr>, String),
    /// Arrow field access: `expr->field`
    Arrow(Box<CExpr>, String),
    /// Array index: `expr[index]`
    Index { expr: Box<CExpr>, index: Box<CExpr> },
    /// Address-of: `&expr`
    AddressOf(Box<CExpr>),
    /// Dereference: `*expr`
    Deref(Box<CExpr>),
    /// Type cast: `(type)expr`
    Cast { ty: CType, expr: Box<CExpr> },
    /// `sizeof(type)`
    SizeOf(CType),
    /// `malloc(size)` — returns `void*`
    Malloc(Box<CExpr>),
    /// Ternary: `cond ? then : else`
    Ternary {
        cond: Box<CExpr>,
        then_expr: Box<CExpr>,
        else_expr: Box<CExpr>,
    },
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A C type expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CType {
    /// `void`
    Void,
    /// `int`, `long`, `size_t`, etc.
    Int(CIntKind),
    /// `char`
    Char,
    /// `float` or `double`
    Float(CFloatKind),
    /// Pointer to type: `type*`
    Ptr(Box<CType>),
    /// `const type`
    Const(Box<CType>),
    /// Array type: `type[size]`
    Array {
        element: Box<CType>,
        size: Option<usize>,
    },
    /// Named struct/typedef: `struct name` or `name`
    Named(String),
    /// Function pointer: `return_type (*)(param_types...)`
    FnPtr {
        return_type: Box<CType>,
        param_types: Vec<CType>,
    },
}

/// Integer width variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CIntKind {
    /// `int`
    Int,
    /// `long`
    Long,
    /// `size_t`
    SizeT,
    /// `int8_t`, `int16_t`, `int32_t`, `int64_t`
    Fixed(u8),
    /// `uint8_t`, `uint16_t`, `uint32_t`, `uint64_t`
    UFixed(u8),
}

/// Float width variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CFloatKind {
    Float,
    Double,
}

// ---------------------------------------------------------------------------
// Top-level items
// ---------------------------------------------------------------------------

/// A C top-level declaration.
#[derive(Debug, Clone)]
pub enum CItem {
    /// `#include <header>` or `#include "header"`
    Include { path: String, system: bool },
    /// `typedef old_type new_name;`
    Typedef { name: String, ty: CType },
    /// `struct name { fields... };`
    StructDef {
        name: String,
        fields: Vec<(String, CType)>,
    },
    /// Tagged union (for `Value` type): `struct { tag enum + union }`
    TaggedUnion {
        name: String,
        tag_name: String,
        variants: Vec<(String, Vec<(String, CType)>)>,
    },
    /// Function definition.
    FnDef(CFnDef),
    /// Function forward declaration: `return_type name(params...);`
    FnDecl(CFnDecl),
    /// `#define NAME value`
    Define { name: String, value: String },
    /// Comment block.
    Comment(String),
}

/// A C function definition.
#[derive(Debug, Clone)]
pub struct CFnDef {
    pub name: String,
    pub return_type: CType,
    pub params: Vec<(String, CType)>,
    pub body: Vec<CStmt>,
    pub is_static: bool,
}

/// A C function forward declaration.
#[derive(Debug, Clone)]
pub struct CFnDecl {
    pub name: String,
    pub return_type: CType,
    pub params: Vec<(String, CType)>,
}

/// A complete C source file.
#[derive(Debug, Clone)]
pub struct CSourceFile {
    pub includes: Vec<CItem>,
    pub items: Vec<CItem>,
}
