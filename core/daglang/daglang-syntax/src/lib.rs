//! daglang-syntax: Lexer and parser for the .dag DSL.
//!
//! Produces an unresolved AST from `.dag` source files. The AST preserves
//! all syntactic information needed by later phases (resolve, typecheck,
//! lower) without performing any semantic analysis.
//!
//! # Supported declarations
//!
//! - `module` -- module path declaration
//! - `import` -- import with optional selective bindings
//! - `type`   -- records, enums/sums, refinements, generics
//! - `fn`     -- pure functions (no I/O, no side effects)
//! - `func`   -- effectful functions (can call services, use resources)
//! - `pattern` -- reusable DAG templates (compile-time expansion)
//! - `service` -- external service declarations with operations
//! - `resource` -- resource declarations with acquire/release/capability
//! - `interface` -- abstract capability contracts with @contract annotations
//! - `pipeline` -- multi-stage pipeline declarations
//!
//! # Golden targets
//!
//! The `.dag` files in `dsl/` are the spec examples that this parser must
//! handle. Start with `dsl/tools/makegen.dag` (simplest) and work outward.

pub mod ast_utils;
pub mod diagnostic;
pub mod lexer;
pub mod parser;

/// Source location tracking for error messages.
pub mod span {
    /// Byte offset range in the source file.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Span {
        pub start: usize,
        pub end: usize,
    }

    /// A value annotated with its source location.
    #[derive(Debug, Clone)]
    pub struct Spanned<T> {
        pub node: T,
        pub span: Span,
    }
}

/// Abstract Syntax Tree types for the .dag DSL.
pub mod ast {
    use crate::span::Spanned;

    // ── Top-level ───────────────────────────────────────────────────

    /// A complete .dag source file.
    #[derive(Debug)]
    pub struct SourceFile {
        pub module_path: Option<Spanned<ModulePath>>,
        pub imports: Vec<Spanned<Import>>,
        pub items: Vec<Spanned<Item>>,
    }

    #[derive(Debug, Clone)]
    pub struct ModulePath {
        pub segments: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct Import {
        pub path: ModulePath,
        pub bindings: Option<Vec<String>>,
        pub alias: Option<String>,
    }

    /// Top-level declaration.
    #[derive(Debug)]
    pub enum Item {
        TypeDef(TypeDef),
        FnDef(FnDef),
        FuncDef(FuncDef),
        PatternDef(PatternDef),
        ServiceDef(ServiceDef),
        ResourceDef(ResourceDef),
        InterfaceDef(InterfaceDef),
        PipelineDef(PipelineDef),
    }

    // ── Types ───────────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct TypeDef {
        pub name: String,
        pub params: Vec<String>,
        pub body: TypeBody,
    }

    #[derive(Debug, Clone)]
    pub enum TypeBody {
        /// Record: `type Foo { a: Int, b: String }`
        Record(Vec<Field>),
        /// Sum: `type Foo = A | B { x: Int } | C`
        Sum(Vec<Variant>),
        /// Alias: `type Foo = Bar`
        Alias(TypeExpr),
    }

    #[derive(Debug, Clone)]
    pub struct Field {
        pub name: String,
        pub ty: TypeExpr,
        pub default: Option<Expr>,
        pub annotations: Vec<Annotation>,
    }

    #[derive(Debug, Clone)]
    pub struct Variant {
        pub name: String,
        pub fields: Vec<Field>,
    }

    #[derive(Debug, Clone)]
    pub enum TypeExpr {
        Named(String),
        Generic(String, Vec<TypeExpr>),
        Optional(Box<TypeExpr>),
        Annotated(Box<TypeExpr>, Vec<Annotation>),
    }

    // ── Functions ───────────────────────────────────────────────────

    /// Pure function: `fn name(params) -> ReturnType { body }`
    #[derive(Debug)]
    pub struct FnDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub params: Vec<Param>,
        pub return_type: TypeExpr,
        pub body: FnBody,
    }

    /// Effectful function: `func name(params) -> { outputs } uses ... { body }`
    #[derive(Debug)]
    pub struct FuncDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub params: Vec<Param>,
        pub outputs: Vec<Field>,
        pub uses: Vec<UsesClause>,
        pub provides: Vec<ProvidesClause>,
        pub annotations: Vec<Annotation>,
        pub body: FuncBody,
    }

    /// Reusable DAG template: `pattern name(params) -> { outputs } uses ... { body }`
    #[derive(Debug)]
    pub struct PatternDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub params: Vec<Param>,
        pub outputs: Vec<Field>,
        pub uses: Vec<UsesClause>,
        pub body: FuncBody,
    }

    #[derive(Debug, Clone)]
    pub struct Param {
        pub name: String,
        pub ty: TypeExpr,
        pub default: Option<Expr>,
    }

    #[derive(Debug, Clone)]
    pub struct UsesClause {
        pub binding: String,
        pub resource_type: TypeExpr,
        pub config: Option<Vec<(String, Expr)>>,
    }

    #[derive(Debug, Clone)]
    pub struct ProvidesClause {
        pub binding: String,
        pub resource_type: TypeExpr,
    }

    // ── Services ────────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct ServiceDef {
        pub name: String,
        pub implements: Option<String>,
        pub annotations: Vec<Annotation>,
        pub operations: Vec<OperationDef>,
    }

    #[derive(Debug)]
    pub struct OperationDef {
        pub name: String,
        pub inputs: Vec<Field>,
        pub outputs: Vec<Field>,
        pub annotations: Vec<Annotation>,
    }

    // ── Resources ───────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct ResourceDef {
        pub name: String,
        pub implements: Option<String>,
        pub properties: Vec<(String, Expr)>,
        pub config: Vec<Field>,
        pub acquire: Option<FuncBody>,
        pub release: Option<FuncBody>,
        pub capabilities: Vec<CapabilityDef>,
    }

    #[derive(Debug)]
    pub struct CapabilityDef {
        pub name: String,
        pub inputs: Vec<Field>,
        pub outputs: Vec<Field>,
        pub annotations: Vec<Annotation>,
    }

    // ── Interfaces ──────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct InterfaceDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub capabilities: Vec<CapabilityDef>,
        pub contracts: Vec<Annotation>,
    }

    // ── Pipelines ───────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct PipelineDef {
        pub name: String,
        pub stages: Vec<StageDef>,
    }

    #[derive(Debug)]
    pub struct StageDef {
        pub name: String,
        pub body: FuncBody,
        pub after: Vec<String>,
    }

    // ── Expressions (fn bodies) ─────────────────────────────────────

    #[derive(Debug, Clone)]
    pub enum Expr {
        /// Literal: integer, float, string, bool
        Literal(Literal),
        /// Variable reference: `x`
        Ident(String),
        /// Field access: `x.y.z`
        FieldAccess(Box<Expr>, String),
        /// Function call: `f(a, b)`
        Call(String, Vec<(Option<String>, Expr)>),
        /// Service call: `gcp.Storage.GetBucket(name: x)`
        ServiceCall(Vec<String>, Vec<(Option<String>, Expr)>),
        /// Binary op: `a + b`, `a && b`
        BinOp(Box<Expr>, BinOp, Box<Expr>),
        /// Unary op: `!x`
        UnaryOp(UnaryOp, Box<Expr>),
        /// String interpolation: `"hello {name}"`
        StringInterp(Vec<StringPart>),
        /// Record construction: `Foo { a: 1, b: 2 }`
        Record(Option<String>, Vec<(String, Expr)>),
        /// Match expression
        Match(Box<Expr>, Vec<MatchArm>),
        /// If/else
        If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
        /// For loop (map sugar): `for x in list { body }`
        For(String, Box<Expr>, Box<Expr>),
        /// Pipe: `expr |> fn`
        Pipe(Box<Expr>, Box<Expr>),
        /// Lambda (inline only, in |> chains): `x => x.name`
        Lambda(Vec<String>, Box<Expr>),
        /// List literal: `[a, b, c]`
        List(Vec<Expr>),
        /// Map literal: `{ "key": value }`
        Map(Vec<(Expr, Expr)>),
        /// Guarded expression: `expr [when condition]`
        Guarded(Box<Expr>, Box<Expr>),
        /// After dependency: `expr [after dep1, after dep2]`
        After(Box<Expr>, Vec<String>),
        /// Return: `return { field: value }`
        Return(Vec<(String, Expr)>),
    }

    #[derive(Debug, Clone)]
    pub enum Literal {
        Int(i64),
        Float(f64),
        String(String),
        Bool(bool),
        None,
    }

    #[derive(Debug, Clone)]
    pub enum BinOp {
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
    }

    #[derive(Debug, Clone)]
    pub enum UnaryOp {
        Not,
        Neg,
    }

    #[derive(Debug, Clone)]
    pub enum StringPart {
        Literal(String),
        Expr(Expr),
    }

    #[derive(Debug, Clone)]
    pub struct MatchArm {
        pub pattern: Pattern,
        pub guard: Option<Expr>,
        pub body: Expr,
    }

    #[derive(Debug, Clone)]
    pub enum Pattern {
        Ident(String),
        Variant(String, Vec<(String, Pattern)>),
        Wildcard,
        Literal(Literal),
    }

    // ── Annotations ─────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct Annotation {
        pub name: String,
        pub args: Vec<Expr>,
    }

    // ── Bodies ───────────────────────────────────────────────────────

    /// Pure function body (expressions only, no I/O).
    #[derive(Debug, Clone)]
    pub struct FnBody {
        pub stmts: Vec<Stmt>,
        pub lossy: bool,
    }

    /// Effectful function body (can include service calls, resource ops).
    #[derive(Debug, Clone)]
    pub struct FuncBody {
        pub stmts: Vec<Stmt>,
        pub lossy: bool,
    }

    /// Statement in a function body.
    #[derive(Debug, Clone)]
    pub enum Stmt {
        Let(String, Expr),
        Assign(String, Expr),
        Expr(Expr),
        Return(Vec<(String, Expr)>),
    }
}
