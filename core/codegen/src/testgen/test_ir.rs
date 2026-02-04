//! Structured intermediate representation for generated test files.
//!
//! These types capture the *intent* of a test (what to set up, execute,
//! and assert) without encoding any target-language syntax. A `TestFile`
//! is rendered to source text by a `TestRenderer` backend.

use gunbc_ir::ValueExpr;

// ===========================================================================
// File structure
// ===========================================================================

/// A complete generated test file.
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
pub struct Import {
    /// Module path segments (e.g., ["gunbc_exec"] or ["gunbc_ir"]).
    pub path: Vec<String>,
    /// Items imported from the module (e.g., ["Value", "Cardinality"]).
    pub items: Vec<String>,
}

/// A helper function defined at module scope.
pub struct HelperFn {
    pub name: String,
    pub return_type: String,
    /// The function body as a single expression (e.g., "crate::graph_mock::mock_spec()").
    pub body_expr: String,
}

/// A group of related tests with a section header.
pub struct TestSection {
    /// Section title (e.g., "Bucket A: Execution semantics").
    pub title: String,
    pub tests: Vec<TestFn>,
}

// ===========================================================================
// Test functions
// ===========================================================================

/// A single test function.
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

/// A statement in test code.
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
}

// ===========================================================================
// Expressions
// ===========================================================================

/// An expression in generated code.
pub enum Expr {
    /// A value literal.
    Value(ValueExpr),
    /// A variable reference.
    Var(String),
    /// A string literal (for keys, messages, identifiers — not Value::Str).
    Str(String),
    /// Function call: `func(args...)`.
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
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
    /// Path expression: `path::to::Item` (for enum variants, associated fns).
    Path(Vec<String>),
    /// Struct construction: `Name { field: value, ... }`.
    Struct {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    /// Closure/lambda: `|args| body` / `lambda args: body`.
    Closure {
        args: Vec<String>,
        body: Box<Expr>,
    },
}

// ===========================================================================
// Assertions
// ===========================================================================

/// A test assertion.
pub enum Assert {
    /// Equality: `assert_eq!(left, right, message)`.
    Eq {
        left: Expr,
        right: Expr,
        message: String,
    },
    /// Truthiness: `assert!(expr, message)`.
    True {
        expr: Expr,
        message: String,
    },
    /// Non-emptiness: `assert!(!expr.is_empty(), message)`.
    NonEmpty {
        expr: Expr,
        message: String,
    },
    /// String containment: `assert!(expr.contains(substring), message)`.
    Contains {
        expr: Expr,
        substring: String,
        message: String,
    },
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
}
