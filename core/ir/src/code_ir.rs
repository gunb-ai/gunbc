//! Structured intermediate representation for generated source files.
//!
//! These types capture the *intent* of generated code without encoding
//! any target-language syntax. A `TestFile` or `SourceFile` is rendered
//! to source text by a `CodeRenderer` implementation.
//!
//! # History
//!
//! The test-specific types (`TestFile`, `Stmt`, `Expr`, `Assert`) were
//! originally in `gunbc-codegen::testgen::test_ir`. They were moved here
//! in Phase 2 to be shared across testgen, dag_gen, and cli_gen.

use crate::ValueExpr;

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
    /// Ordered statements in the test body.
    pub body: Vec<Stmt>,
}

// ===========================================================================
// Statements
// ===========================================================================

/// A statement in generated code.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Variable binding: `let [mut] name = expr;`
    Let {
        name: String,
        mutable: bool,
        expr: Expr,
    },
    /// Expression as statement (for side effects like `map.insert(...)`).
    Expr(Expr),
    /// An assertion.
    Assert(Assert),
    /// A comment line.
    Comment(String),
    /// Blank line for readability.
    Blank,
    /// Explicit return statement.
    Return(Expr),
    /// Implicit return: final expression without semicolon (Rust idiom).
    TailExpr(Expr),
    /// For loop: `for binding in iter { body }`.
    For {
        binding: String,
        iter: Expr,
        body: Vec<Stmt>,
    },
    /// Nested item (e.g., inner function).
    Item(Item),
}

// ===========================================================================
// Expressions
// ===========================================================================

/// An expression in generated code.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A value literal.
    Value(ValueExpr),
    /// A variable reference.
    Var(String),
    /// A string literal (for keys, messages, identifiers — not Value::Str).
    Str(String),
    /// Function call: `func(args...)`.
    Call { func: Box<Expr>, args: Vec<Expr> },
    /// Method call: `receiver.method(args...)`.
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// Field access: `expr.field`.
    Field(Box<Expr>, String),
    /// Dereference: `*expr`.
    Deref(Box<Expr>),
    /// Reference: `&expr`.
    Ref(Box<Expr>),
    /// Mutable reference: `&mut expr`.
    RefMut(Box<Expr>),
    /// Path expression: `path::to::Item` (for enum variants, associated fns).
    Path(Vec<String>),
    /// Struct construction: `Name { field: value, ... }`.
    Struct {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    /// Closure/lambda: `|args| body` / `lambda args: body`.
    Closure { args: Vec<String>, body: Box<Expr> },
    /// Binary operation: `left op right` (e.g., `n >= 2`, `a == b`).
    BinOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    /// Unary operation: `op expr` (e.g., `!flag`).
    UnaryOp { op: String, expr: Box<Expr> },
    /// Bare integer literal (not `Value::Int`). For use in expressions
    /// operating on unwrapped values (e.g., closure bodies after `as_int()`).
    IntLit(i64),
    /// Bare boolean literal (true/false).
    BoolLit(bool),
    /// Match expression: `match expr { arms }`.
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    /// If expression: `if cond { then } else { else }`.
    If {
        cond: Box<Expr>,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    /// Block expression: `{ stmts }`.
    Block(Vec<Stmt>),
    /// format!() macro: `format!("template", args...)`.
    FormatStr { template: String, args: Vec<Expr> },
    /// General macro invocation: `name!(args...)`.
    MacroCall { name: String, args: Vec<Expr> },
    /// Tuple expression: `(a, b, c)`.
    Tuple(Vec<Expr>),
    /// Array expression: `[a, b, c]`.
    Array(Vec<Expr>),
    /// Escape hatch for complex expressions that don't fit the IR.
    RawCode(String),
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
#[derive(Debug, Clone)]
pub enum Item {
    Use(Import),
    Fn(FnDef),
    Enum(EnumDef),
    Impl(ImplBlock),
    Struct(StructDef),
    /// Escape hatch for complex constructs.
    Raw(String),
}

/// A function definition.
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
    /// Attributes (e.g., "#[test]", "#[derive(Debug)]").
    pub attributes: Vec<String>,
}

/// An enum definition.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub is_pub: bool,
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

/// A struct definition.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub is_pub: bool,
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
}
