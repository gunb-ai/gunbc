//! Structured intermediate representation for generated source files.
//!
//! These types capture the *intent* of generated code without encoding
//! any target-language syntax. A `TestFile` or `SourceFile` is rendered
//! to source text by a `CodeRenderer` implementation.
//!
//! # Tier classification
//!
//! Each construct belongs to an abstraction tier:
//!
//! - **Tier 0 (AbstractIR)** — Universal: variables, calls, conditionals, loops.
//!   Every target language can express these directly.
//! - **Tier 1 (SystemsIR)** — Rust/C++ extensions: ownership (`Deref`, `Ref`,
//!   `RefMut`), pattern matching (`Match`), implicit return (`TailExpr`),
//!   macros (`MacroCall`), impl blocks, derive attributes.
//! - **Tier 2 (ManagedIR)** — Go/Python: see [`go_ir`](crate::go_ir).
//! - **Tier 3 (CStyleIR)** — C: malloc/free, pointers, goto/label, typedef.
//!   See [`c_ir`].
//! - **Tier 4 (RegisterIR)** — MIPS/x86: registers, instructions, syscalls.
//!   See [`register_ir`].
//!
//! Lowering flows downward: AbstractIR → SystemsIR/ManagedIR → CStyleIR → RegisterIR.
//! The lowering trait is defined in [`lower`].
//!
//! # History
//!
//! The test-specific types (`TestFile`, `Stmt`, `Expr`, `Assert`) were
//! originally in `gunbc-codegen::testgen::test_ir`. They were moved here
//! in Phase 2 to be shared across testgen, dag_gen, and cli_gen.

pub mod c_ir;
pub mod lower;
pub mod register_ir;

use crate::ValueExpr;
use serde::{Deserialize, Serialize};

// ===========================================================================
// Test file structure (moved from testgen::test_ir)
// ===========================================================================

/// A complete generated test file.
#[derive(Debug, Clone)]
pub struct TestFile {
    /// Comment lines at the top (generated-by notice, hash, stats).
    pub header: Vec<String>,
    /// Module-level imports.
    pub imports: Vec<Import>,
    /// Helper functions (e.g., `fn mock_spec() -> MockSpec`).
    pub helpers: Vec<HelperFn>,
    /// Test sections, each with a header and list of tests.
    pub sections: Vec<TestSection>,
}

/// A module-level import.
#[derive(Debug, Clone)]
pub struct Import {
    /// Module path segments (e.g., ["gunbc_exec"] or ["gunbc_ir"]).
    pub path: Vec<String>,
    /// Items imported from the module (e.g., ["Value", "Cardinality"]).
    pub items: Vec<String>,
}

/// A helper function defined at module scope.
#[derive(Debug, Clone)]
pub struct HelperFn {
    pub name: String,
    pub return_type: String,
    /// The function body as statements.
    pub body: Vec<Stmt>,
}

/// A group of related tests with a section header.
#[derive(Debug, Clone)]
pub struct TestSection {
    /// Section title (e.g., "Bucket A: Execution semantics").
    pub title: String,
    /// Comment lines immediately under the section header.
    pub notes: Vec<String>,
    pub tests: Vec<TestFn>,
}

// ===========================================================================
// Test functions
// ===========================================================================

/// A single test function.
#[derive(Debug, Clone)]
pub struct TestFn {
    /// Function name (e.g., "test_a1_dry_run_execution").
    pub name: String,
    /// Doc comment lines (rendered as `///` in Rust, docstrings elsewhere).
    pub doc: Vec<String>,
    /// Extra attributes emitted between `#[test]` and `fn` (e.g., `#[ignore]`).
    pub attributes: Vec<String>,
    /// Ordered statements in the test body.
    pub body: Vec<Stmt>,
}

// ===========================================================================
// Statements
// ===========================================================================

/// A statement in generated code.
///
/// Variants are annotated with their abstraction tier.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// **Tier 0.** Variable binding: `let [mut] name = expr;`
    Let {
        name: String,
        mutable: bool,
        expr: Expr,
    },
    /// **Tier 2 (ManagedIR).** Managed-language binding with explicit intent:
    /// multi-target declaration/assignment (e.g., Go `a, err := call()`).
    Bind {
        targets: Vec<BindTarget>,
        intent: BindIntent,
        expr: Expr,
    },
    /// **Tier 0.** Assignment to an existing variable/path: `dest = value;`
    Assign { dest: Expr, value: Expr },
    /// **Tier 0.** Expression as statement (for side effects like `map.insert(...)`).
    Expr(Expr),
    /// **Tier 0.** An assertion.
    Assert(Assert),
    /// **Tier 0.** A comment line.
    Comment(String),
    /// **Tier 0.** Blank line for readability.
    Blank,
    /// **Tier 0.** Explicit return statement.
    Return(Expr),
    /// **Tier 1 (SystemsIR).** Implicit return: final expression without semicolon (Rust idiom).
    TailExpr(Expr),
    /// **Tier 0.** For loop: `for binding in iter { body }`.
    For {
        binding: String,
        iter: Expr,
        body: Vec<Stmt>,
    },
    /// **Tier 0.** Nested item (e.g., inner function).
    Item(Item),
    /// **Tier 0.** Lexical block statement `{ stmts }` for isolating variable scope without returning a value.
    BlockScope(Vec<Stmt>),
}

/// Binding target for managed-language multi-target bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindTarget {
    /// A named variable target.
    Name(String),
    /// Discard target (e.g., Go `_`).
    Discard,
}

/// Binding intent for managed-language bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindIntent {
    /// Declare new variables (e.g., Go `:=`).
    Declare,
    /// Assign to existing variables (e.g., Go `=`).
    Assign,
}

// ===========================================================================
// Expressions
// ===========================================================================

/// An expression in generated code.
///
/// Variants are annotated with their abstraction tier.
#[derive(Debug, Clone)]
pub enum Expr {
    /// **Tier 0.** A value literal.
    Value(ValueExpr),
    /// **Tier 0.** A variable reference.
    Var(String),
    /// **Tier 0.** A string literal (for keys, messages, identifiers — not Value::Str).
    Str(String),
    /// **Tier 0.** Function call: `func(args...)`.
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        /// Optional obligation metadata propagated from lowering.
        ///
        /// This lets target lowerers and renderers reason about call semantics
        /// without relying on fragile name-prefix heuristics.
        obligation: Option<CallObligation>,
    },
    /// **Tier 0.** Method call: `receiver.method(args...)`.
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// **Tier 0.** Field access: `expr.field`.
    Field(Box<Expr>, String),
    /// **Tier 1 (SystemsIR).** Dereference: `*expr`.
    Deref(Box<Expr>),
    /// **Tier 1 (SystemsIR).** Reference: `&expr`.
    Ref(Box<Expr>),
    /// **Tier 1 (SystemsIR).** Mutable reference: `&mut expr`.
    RefMut(Box<Expr>),
    /// **Tier 1 (SystemsIR).** Path expression: `path::to::Item` (for enum variants, associated fns).
    Path(Vec<String>),
    /// **Tier 0.** Struct construction: `Name { field: value, ... }`.
    Struct {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    /// **Tier 0.** Closure/lambda: `|args| body` / `lambda args: body`.
    Closure { args: Vec<String>, body: Box<Expr> },
    /// **Tier 0.** Binary operation: `left op right` (e.g., `n >= 2`, `a == b`).
    BinOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    /// **Tier 0.** Unary operation: `op expr` (e.g., `!flag`).
    UnaryOp { op: String, expr: Box<Expr> },
    /// **Tier 0.** Bare integer literal (not `Value::Int`). For use in expressions
    /// operating on unwrapped values (e.g., closure bodies after `as_int()`).
    IntLit(i64),
    /// **Tier 0.** Bare boolean literal (true/false).
    BoolLit(bool),
    /// **Tier 1 (SystemsIR).** Match expression: `match expr { arms }`.
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    /// **Tier 0.** If expression: `if cond { then } else { else }`.
    If {
        cond: Box<Expr>,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    /// **Tier 0.** Block expression: `{ stmts }`.
    Block(Vec<Stmt>),
    /// **Tier 0.** Format string: `format!("template", args...)` / `fmt.Sprintf(...)` / `f"..."`.
    FormatStr { template: String, args: Vec<Expr> },
    /// **Tier 1 (SystemsIR).** General macro invocation: `name!(args...)`.
    MacroCall { name: String, args: Vec<Expr> },
    /// **Tier 0.** Tuple expression: `(a, b, c)`.
    Tuple(Vec<Expr>),
    /// **Tier 0.** Array expression: `[a, b, c]`.
    Array(Vec<Expr>),
    /// **Tier 0.** Escape hatch for complex expressions that don't fit the IR.
    RawCode(String),
}

/// Obligation category attached to a call expression.
///
/// Mirrors the lowered obligation categories used by daglang without creating
/// a crate dependency from `gunbc-ir` to `daglang-lower`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallObligation {
    ServiceTransportExecute,
    ServiceTransportPrepare,
    ServiceTransportParse,
    ServiceParamSource,
    ResourceProvide,
    ResourceAcquire,
    ResourceRelease,
    InterfaceContractVerification,
}

impl CallObligation {
    /// Canonical parity-kind string for this obligation.
    pub fn canonical_kind(self) -> &'static str {
        match self {
            Self::ServiceTransportExecute => "transport",
            Self::ServiceTransportPrepare
            | Self::ServiceTransportParse
            | Self::ServiceParamSource
            | Self::ResourceProvide
            | Self::ResourceAcquire
            | Self::ResourceRelease
            | Self::InterfaceContractVerification => "pattern-expanded",
        }
    }

    /// Whether this call participates in a transport runtime triplet.
    pub fn is_transport_runtime(self) -> bool {
        matches!(
            self,
            Self::ServiceTransportExecute
                | Self::ServiceTransportPrepare
                | Self::ServiceTransportParse
        )
    }

    /// Whether this call is a transport or resource runtime call that
    /// requires special import/error-handling treatment in generated code.
    pub fn is_runtime_call(self) -> bool {
        matches!(
            self,
            Self::ServiceTransportExecute
                | Self::ServiceTransportPrepare
                | Self::ServiceTransportParse
                | Self::ResourceAcquire
                | Self::ResourceRelease
                | Self::ResourceProvide
        )
    }
}

// ===========================================================================
// Assertions
// ===========================================================================

/// A test assertion.
#[derive(Debug, Clone)]
pub enum Assert {
    /// Equality: `assert_eq!(left, right, message)`.
    Eq {
        left: Expr,
        right: Expr,
        message: String,
    },
    /// Truthiness: `assert!(expr, message)`.
    True { expr: Expr, message: String },
    /// Non-emptiness: `assert!(!expr.is_empty(), message)`.
    NonEmpty { expr: Expr, message: String },
    /// String containment: `assert!(expr.contains(substring), message)`.
    Contains {
        expr: Expr,
        substring: String,
        message: String,
    },
}

// ===========================================================================
// Match arm
// ===========================================================================

/// A single arm in a match expression.
#[derive(Debug, Clone)]
pub struct MatchArm {
    /// Pattern text (e.g., "Some(x)", "Variant::A").
    pub pattern: String,
    /// Body statements.
    pub body: Vec<Stmt>,
}

// ===========================================================================
// General code constructs (for dag_gen, cli_gen)
// ===========================================================================

/// Top-level items in a source file.
///
/// Variants are annotated with their abstraction tier.
#[derive(Debug, Clone)]
pub enum Item {
    /// **Tier 0.** Module import.
    Use(Import),
    /// **Tier 0.** Function definition.
    Fn(FnDef),
    /// **Tier 0.** Enum definition.
    Enum(EnumDef),
    /// **Tier 1 (SystemsIR).** Impl block (Rust trait implementations).
    Impl(ImplBlock),
    /// **Tier 0.** Struct definition.
    Struct(StructDef),
    /// **Tier 0.** Escape hatch for complex constructs.
    Raw(String),
}

/// **Tier 0** (core) / **Tier 1** (`attributes` field). A function definition.
#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub is_pub: bool,
    /// Parameters: (name, type).
    pub params: Vec<(String, String)>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
    /// Doc comment lines.
    pub doc: Vec<String>,
    /// **Tier 1 (SystemsIR).** Attributes (e.g., `#[test]`, `#[derive(Debug)]`).
    pub attributes: Vec<String>,
}

/// **Tier 0** (core) / **Tier 1** (`derives` field). An enum definition.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub is_pub: bool,
    /// **Tier 1 (SystemsIR).** Derive macros (e.g., `Debug`, `Clone`).
    pub derives: Vec<String>,
    /// Variant text (e.g., "Primitive(PrimitiveOp)").
    pub variants: Vec<String>,
    /// Doc comment lines.
    pub doc: Vec<String>,
}

/// An impl block.
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub type_name: String,
    pub trait_name: Option<String>,
    pub items: Vec<FnDef>,
}

/// **Tier 0** (core) / **Tier 1** (`derives` field). A struct definition.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub is_pub: bool,
    /// **Tier 1 (SystemsIR).** Derive macros (e.g., `Debug`, `Clone`).
    pub derives: Vec<String>,
    /// Fields: (name, type, is_pub).
    pub fields: Vec<(String, String, bool)>,
    /// Doc comment lines.
    pub doc: Vec<String>,
}

/// A complete generated source file (more general than TestFile).
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Module-level doc comments.
    pub doc: Vec<String>,
    /// Top-level items.
    pub items: Vec<Item>,
}

// ===========================================================================
// Tier checks
// ===========================================================================

/// Returns `true` when a statement is expressible in Tier 0 (AbstractIR).
pub fn is_abstract(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { expr, .. } | Stmt::Expr(expr) | Stmt::Return(expr) => is_abstract_expr(expr),
        Stmt::Bind { .. } => false,
        Stmt::Assign { dest, value } => is_abstract_expr(dest) && is_abstract_expr(value),
        Stmt::Assert(assertion) => is_abstract_assert(assertion),
        Stmt::Comment(_) | Stmt::Blank => true,
        Stmt::TailExpr(_) => false,
        Stmt::For { iter, body, .. } => is_abstract_expr(iter) && body.iter().all(is_abstract),
        Stmt::BlockScope(stmts) => stmts.iter().all(is_abstract),
        Stmt::Item(item) => is_abstract_item(item),
    }
}

/// Returns `true` when an expression is expressible in Tier 0 (AbstractIR).
pub fn is_abstract_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Value(_) | Expr::Var(_) | Expr::Str(_) | Expr::IntLit(_) | Expr::BoolLit(_) => true,
        Expr::Call { func, args, .. } => {
            is_abstract_expr(func) && args.iter().all(is_abstract_expr)
        }
        Expr::MethodCall { receiver, args, .. } => {
            is_abstract_expr(receiver) && args.iter().all(is_abstract_expr)
        }
        Expr::Field(receiver, _) => is_abstract_expr(receiver),
        Expr::Struct { fields, .. } => fields.iter().all(|(_, value)| is_abstract_expr(value)),
        Expr::Closure { body, .. } => is_abstract_expr(body),
        Expr::BinOp { left, right, .. } => is_abstract_expr(left) && is_abstract_expr(right),
        Expr::UnaryOp { expr, .. } => is_abstract_expr(expr),
        Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            is_abstract_expr(cond)
                && then_body.iter().all(is_abstract)
                && else_body
                    .as_ref()
                    .is_none_or(|body| body.iter().all(is_abstract))
        }
        Expr::Block(stmts) => stmts.iter().all(is_abstract),
        Expr::FormatStr { args, .. } => args.iter().all(is_abstract_expr),
        Expr::Tuple(values) | Expr::Array(values) => values.iter().all(is_abstract_expr),
        Expr::Deref(_)
        | Expr::Ref(_)
        | Expr::RefMut(_)
        | Expr::Path(_)
        | Expr::Match { .. }
        | Expr::MacroCall { .. }
        | Expr::RawCode(_) => false,
    }
}

fn is_abstract_assert(assertion: &Assert) -> bool {
    match assertion {
        Assert::Eq { left, right, .. } => is_abstract_expr(left) && is_abstract_expr(right),
        Assert::True { expr, .. } | Assert::NonEmpty { expr, .. } => is_abstract_expr(expr),
        Assert::Contains { expr, .. } => is_abstract_expr(expr),
    }
}

fn is_abstract_item(item: &Item) -> bool {
    match item {
        Item::Use(_) => true,
        Item::Fn(function) => {
            function.attributes.is_empty() && function.body.iter().all(is_abstract)
        }
        Item::Enum(definition) => definition.derives.is_empty(),
        Item::Struct(definition) => definition.derives.is_empty(),
        Item::Impl(_) | Item::Raw(_) => false,
    }
}

// ===========================================================================
// Convenience constructors
// ===========================================================================

impl Expr {
    /// Shorthand for `Expr::Var(name)`.
    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(name.into())
    }

    /// Shorthand for `Expr::Str(s)`.
    pub fn str_lit(s: impl Into<String>) -> Self {
        Expr::Str(s.into())
    }

    /// Shorthand for `Expr::Path(segments)`.
    pub fn path(segments: &[&str]) -> Self {
        Expr::Path(segments.iter().map(|s| s.to_string()).collect())
    }

    /// Shorthand for a free function call: `func(args...)`.
    pub fn call(func: impl Into<String>, args: Vec<Expr>) -> Self {
        Expr::Call {
            func: Box::new(Expr::Var(func.into())),
            args,
            obligation: None,
        }
    }

    /// Shorthand for a call with obligation metadata.
    pub fn call_with_obligation(
        func: impl Into<String>,
        args: Vec<Expr>,
        obligation: CallObligation,
    ) -> Self {
        Expr::Call {
            func: Box::new(Expr::Var(func.into())),
            args,
            obligation: Some(obligation),
        }
    }

    /// Shorthand for a method call on self: `self.method(args...)`.
    pub fn method(self, method: impl Into<String>, args: Vec<Expr>) -> Self {
        Expr::MethodCall {
            receiver: Box::new(self),
            method: method.into(),
            args,
        }
    }

    /// Shorthand for field access: `self.field`.
    pub fn field(self, field: impl Into<String>) -> Self {
        Expr::Field(Box::new(self), field.into())
    }

    /// Shorthand for `.expect("message")`.
    pub fn expect(self, msg: impl Into<String>) -> Self {
        self.method("expect", vec![Expr::Str(msg.into())])
    }

    /// Shorthand for `*self`.
    pub fn deref(self) -> Self {
        Expr::Deref(Box::new(self))
    }

    /// Shorthand for `&self`.
    pub fn ref_of(self) -> Self {
        Expr::Ref(Box::new(self))
    }

    /// Shorthand for `&mut self`.
    pub fn ref_mut(self) -> Self {
        Expr::RefMut(Box::new(self))
    }

    /// Shorthand for `self op right` (e.g., `n.ge(Expr::int(2))`).
    pub fn bin_op(self, op: impl Into<String>, right: Expr) -> Self {
        Expr::BinOp {
            left: Box::new(self),
            op: op.into(),
            right: Box::new(right),
        }
    }

    /// Shorthand for unary operations (e.g., `!expr`).
    pub fn unary(self, op: impl Into<String>) -> Self {
        Expr::UnaryOp {
            op: op.into(),
            expr: Box::new(self),
        }
    }

    /// Shorthand for logical not: `!expr`.
    pub fn logical_not(self) -> Self {
        self.unary("!")
    }

    /// Shorthand for a bare integer literal (renders as `n`, not `Value::Int(n)`).
    pub fn int(n: i64) -> Self {
        Expr::IntLit(n)
    }

    /// Shorthand for a bare boolean literal.
    pub fn bool_lit(b: bool) -> Self {
        Expr::BoolLit(b)
    }

    /// Shorthand for raw code.
    pub fn raw(code: impl Into<String>) -> Self {
        Expr::RawCode(code.into())
    }
}

impl Stmt {
    /// `let name = expr;`
    pub fn let_bind(name: impl Into<String>, expr: Expr) -> Self {
        Stmt::Let {
            name: name.into(),
            mutable: false,
            expr,
        }
    }

    /// `let mut name = expr;`
    pub fn let_mut(name: impl Into<String>, expr: Expr) -> Self {
        Stmt::Let {
            name: name.into(),
            mutable: true,
            expr,
        }
    }

    /// Managed binding declaration (e.g., Go `a, err := expr`).
    pub fn bind_declare(targets: Vec<BindTarget>, expr: Expr) -> Self {
        Stmt::Bind {
            targets,
            intent: BindIntent::Declare,
            expr,
        }
    }

    /// Managed binding assignment (e.g., Go `a, err = expr`).
    pub fn bind_assign(targets: Vec<BindTarget>, expr: Expr) -> Self {
        Stmt::Bind {
            targets,
            intent: BindIntent::Assign,
            expr,
        }
    }

    /// A comment line.
    pub fn comment(text: impl Into<String>) -> Self {
        Stmt::Comment(text.into())
    }

    /// `return expr;`
    pub fn ret(expr: Expr) -> Self {
        Stmt::Return(expr)
    }

    /// Implicit return: `expr` (no semicolon, no return keyword).
    pub fn tail(expr: Expr) -> Self {
        Stmt::TailExpr(expr)
    }

    /// Assignment: `dest = value;`
    pub fn assign(dest: Expr, value: Expr) -> Self {
        Stmt::Assign { dest, value }
    }

    /// Lexical block scope: `{ stmts }`
    pub fn block_scope(stmts: Vec<Stmt>) -> Self {
        Stmt::BlockScope(stmts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abstract_stmt_let_is_true() {
        let stmt = Stmt::let_bind("value", Expr::int(1));
        assert!(is_abstract(&stmt));
    }

    #[test]
    fn systems_stmt_tail_expr_is_false() {
        let stmt = Stmt::tail(Expr::int(1));
        assert!(!is_abstract(&stmt));
    }

    #[test]
    fn managed_stmt_bind_is_false_for_abstract_tier() {
        let stmt = Stmt::bind_declare(
            vec![
                BindTarget::Name("value".to_string()),
                BindTarget::Name("err".to_string()),
            ],
            Expr::call("fetch", vec![]),
        );
        assert!(!is_abstract(&stmt));
    }

    #[test]
    fn abstract_expr_call_is_true() {
        let expr = Expr::call("render", vec![Expr::str_lit("input")]);
        assert!(is_abstract_expr(&expr));
    }

    #[test]
    fn systems_expr_deref_is_false() {
        let expr = Expr::Deref(Box::new(Expr::var("ptr")));
        assert!(!is_abstract_expr(&expr));
    }

    #[test]
    fn systems_expr_macro_call_is_false() {
        let expr = Expr::MacroCall {
            name: "format".to_string(),
            args: vec![Expr::str_lit("hello")],
        };
        assert!(!is_abstract_expr(&expr));
    }
}
