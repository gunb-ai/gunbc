//! **Stage 1 — Parse**: Transforms `&str` source text into an unresolved
//! `SourceFile` AST.
//!
//! # Pipeline position
//!
//! - **Before**: raw `.dag` source text (from filesystem or embedded string)
//! - **After**: [`daglang-resolve`] discovers imports and builds a `ModuleGraph`
//!
//! # Sequential steps
//!
//! 1. Lex source text into a token stream (`lexer`)
//! 2. Parse token stream into `SourceFile` AST (`parser`)
//! 3. Preserve all syntactic information (spans, declarations) without
//!    performing any semantic analysis
//!
//! # Purity
//!
//! Pure — no side effects. Operates entirely on an in-memory `&str` slice.
//!
//! # Failure
//!
//! Returns `ParseError` with byte-offset `Span` for each syntax error.

pub mod ast_utils;
pub mod callable;
pub mod diagnostic;
pub mod lexer;
pub mod parser;

pub use callable::CallableItemExt;

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
    #[derive(Debug, Clone)]
    pub struct SourceFile {
        pub module_path: Option<Spanned<ModulePath>>,
        pub imports: Vec<Spanned<Import>>,
        pub items: Vec<Spanned<Item>>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ModulePath {
        pub segments: Vec<String>,
    }

    impl ModulePath {
        /// Create a `ModulePath` from segments.
        pub fn new(segments: Vec<String>) -> Self {
            Self { segments }
        }

        /// Dot-joined string representation (e.g., `"std.render"`).
        pub fn as_dotted(&self) -> String {
            self.segments.join(".")
        }
    }

    impl std::fmt::Display for ModulePath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut first = true;
            for seg in &self.segments {
                if !first {
                    f.write_str(".")?;
                }
                f.write_str(seg)?;
                first = false;
            }
            Ok(())
        }
    }

    impl From<Vec<String>> for ModulePath {
        fn from(segments: Vec<String>) -> Self {
            Self { segments }
        }
    }

    impl From<ModulePath> for Vec<String> {
        fn from(mp: ModulePath) -> Self {
            mp.segments
        }
    }

    #[derive(Debug, Clone)]
    pub struct Import {
        pub path: ModulePath,
        pub bindings: Option<Vec<String>>,
        pub alias: Option<String>,
    }

    /// Top-level declaration.
    #[derive(Debug, Clone)]
    pub enum Item {
        TypeDef(TypeDef),
        FnDef(FnDef),
        FuncDef(FuncDef),
        PatternDef(PatternDef),
        ServiceDef(Box<ServiceDef>),
        ResourceDef(ResourceDef),
        InterfaceDef(InterfaceDef),
        PipelineDef(PipelineDef),
        ProfileDef(ProfileDef),
        TestDef(TestDef),
        FixtureDef(FixtureDef),
        ProjectDef(ProjectDef),
        FeatureDef(FeatureDef),
        TaskDef(TaskDef),
        DesignDef(DesignDef),
        ComponentDef(ComponentDef),
        EnvironmentDef(EnvironmentDef),
        ParamDecl(ParamDecl),
        DataDef(DataDef),
        ExternAssetDecl(ExternAssetDecl),
    }

    /// Module-level parameter: `param name: Type` or `param name: Type = default`
    #[derive(Debug, Clone)]
    pub struct ParamDecl {
        pub name: String,
        pub ty: TypeExpr,
        pub default: Option<Expr>,
    }

    /// Module-level static data declaration: `data name: Type = value`.
    #[derive(Debug, Clone)]
    pub struct DataDef {
        pub name: String,
        pub ty: TypeExpr,
        pub value: Expr,
    }

    /// `extern asset name: Type`
    #[derive(Debug, Clone)]
    pub struct ExternAssetDecl {
        pub name: String,
        pub ty: TypeExpr,
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

    impl TypeDef {
        /// A bare type declaration (`type String`, `type Int`, etc.) has no body
        /// and produces Alias(Named(name)) where the base name equals the def name.
        /// These are kernel primitives — the v1 compiler already provides them,
        /// so the AST item should be skipped by downstream processing.
        pub fn is_bare_primitive(&self) -> bool {
            matches!(&self.body, TypeBody::Alias(TypeExpr::Named(ref n)) if n == &self.name)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Field {
        pub name: String,
        pub ty: TypeExpr,
        pub default: Option<Expr>,
        pub from_path: Option<String>,
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
        /// Associated output type: `Check.Output`
        AssociatedOutput(String),
        /// Function type: `fn(A, B) -> C`
        Function(Vec<TypeExpr>, Box<TypeExpr>),
        Optional(Box<TypeExpr>),
        /// Refined type: `Base where constraint1, constraint2`
        Refined(Box<TypeExpr>, Vec<Refinement>),
        /// Anonymous record return type: `-> { field: Type, ... }`
        Record(Vec<Field>),
    }

    // ── Functions ───────────────────────────────────────────────────

    /// Pure function: `fn name(params) -> ReturnType { body }`
    #[derive(Debug, Clone)]
    pub struct FnDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub params: Vec<Param>,
        pub return_type: TypeExpr,
        pub body: FnBody,
    }

    /// Effectful function: `func name(params) -> { outputs } uses ... { body }`
    #[derive(Debug, Clone)]
    pub struct FuncDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub params: Vec<Param>,
        pub outputs: Vec<Field>,
        pub uses: Vec<UsesClause>,
        pub provides: Vec<ProvidesClause>,
        pub body: FuncBody,
    }

    /// Reusable DAG template: `pattern name(params) -> { outputs } uses ... provides ... { body }`
    #[derive(Debug, Clone)]
    pub struct PatternDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub params: Vec<Param>,
        pub outputs: Vec<Field>,
        pub uses: Vec<UsesClause>,
        pub provides: Vec<ProvidesClause>,
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

    #[derive(Debug, Clone)]
    pub struct ServiceDef {
        pub name: String,
        pub implements: Option<String>,
        pub operations: Vec<OperationDef>,
        pub config: ServiceConfig,
    }

    #[derive(Debug, Clone)]
    pub struct OperationDef {
        pub name: String,
        pub inputs: Vec<Field>,
        pub outputs: Vec<Field>,
        pub idempotent: bool,
        pub readonly: bool,
        pub transport: Option<TransportBinding>,
        /// Response contract: maps HTTP status codes/patterns to response types.
        pub response: Vec<ResponseEntry>,
        /// Exit contract: maps shell exit codes to output types.
        pub exit: Vec<ExitEntry>,
        /// Mock response definitions for test generation.
        /// Populated from `mock_response { STATUS => { body } }` blocks.
        pub mock_responses: Vec<MockResponseDef>,
        /// Explicit output parsing mode (S44). When set, overrides inference
        /// from output field types. Valid values: `TrimStdout`, `SplitLines`,
        /// `SuccessStdoutStderr`, `ExitCodeBool`.
        pub output_parsing: Option<String>,
    }

    /// Response contract entry: STATUS => TYPE.
    /// STATUS is a pattern (exact code like 200, or wildcard like 2xx, 4xx, 5xx).
    #[derive(Debug, Clone)]
    pub struct ResponseEntry {
        pub status: StatusPattern,
        pub response_type: TypeExpr,
        pub description: Option<String>,
    }

    /// Mock response entry for test generation: STATUS => { json body }.
    ///
    /// Provides realistic mock response bodies for each status code.
    /// Used by the testgen pipeline to generate typed mock data instead of
    /// relying on generic provider-level synthesis.
    #[derive(Debug, Clone)]
    pub struct MockResponseDef {
        /// HTTP status code for this mock response.
        pub status: u16,
        /// Mock response body as a DSL expression (typically a record literal).
        pub body: Expr,
        /// Optional description for documentation.
        pub description: Option<String>,
    }

    /// HTTP status code pattern.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum StatusPattern {
        /// Exact status code: 200, 201, 404, etc.
        Exact(u16),
        /// 2xx wildcard: any 2xx status.
        Success2xx,
        /// 3xx wildcard: redirects.
        Redirect3xx,
        /// 4xx wildcard: client errors.
        ClientError4xx,
        /// 5xx wildcard: server errors.
        ServerError5xx,
    }

    /// Exit code contract entry for shell operations: CODE => TYPE.
    #[derive(Debug, Clone)]
    pub struct ExitEntry {
        pub code: ExitCode,
        pub output_type: TypeExpr,
        pub description: Option<String>,
    }

    /// Shell exit code pattern.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ExitCode {
        /// Exact exit code: 0, 1, 2, etc.
        Exact(i32),
        /// Non-zero wildcard: any non-zero exit code (failure).
        NonZero,
    }

    // ── Resources ───────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct ResourceDef {
        pub name: String,
        pub implements: Option<String>,
        pub properties: Vec<(String, Expr)>,
        pub config: Vec<Field>,
        pub acquire: Option<FuncBody>,
        pub release: Option<FuncBody>,
        pub capabilities: Vec<CapabilityDef>,
    }

    #[derive(Debug, Clone)]
    pub struct CapabilityDef {
        pub name: String,
        pub inputs: Vec<Field>,
        pub outputs: Vec<Field>,
        pub idempotent: bool,
        pub readonly: bool,
    }

    // ── Interfaces ──────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct InterfaceDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub capabilities: Vec<CapabilityDef>,
        pub contracts: Vec<ContractDef>,
        pub type_defs: Vec<TypeDef>,
    }

    // ── Pipelines ───────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct PipelineDef {
        pub name: String,
        pub uses: Vec<UsesClause>,
        pub stages: Vec<StageDef>,
    }

    #[derive(Debug, Clone)]
    pub struct StageDef {
        pub name: String,
        pub body: FuncBody,
        pub after: Vec<String>,
        pub when: Option<Expr>,
    }

    // ── Profiles ────────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct ProfileDef {
        pub name: String,
        pub binds: Vec<ProfileBind>,
    }

    #[derive(Debug, Clone)]
    pub struct ProfileBind {
        pub interface_type: String,
        pub implementation_type: String,
        pub config_entries: Vec<(String, Expr)>,
    }

    // ── Profile & Infra Blocks ─────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct ProjectDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug, Clone)]
    pub struct FeatureDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug, Clone)]
    pub struct TaskDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug, Clone)]
    pub struct DesignDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug, Clone)]
    pub struct ComponentDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug, Clone)]
    pub struct EnvironmentDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    // ── Tests ────────────────────────────────────────────────────────

    /// A named fixture: reusable mock declarations shared across tests.
    ///
    /// ```dag
    /// fixture cloud_env {
    ///     mock cloud_env.config -> { project: "mock-project" }
    ///     mock cloud_env.request_url -> "https://example.com/oidc"
    /// }
    /// ```
    #[derive(Debug, Clone)]
    pub struct FixtureDef {
        pub name: String,
        pub mocks: Vec<MockDecl>,
    }

    /// A test case, optionally inheriting from a fixture.
    ///
    /// ```dag
    /// test gist_snapshot_dryrun : cloud_env_fixture {
    ///     tier: Unit
    ///     hermetic
    ///
    ///     input render_snapshot.topology_json = "{}"
    ///
    ///     mock execute.response -> rest_response(200, { ok: true })
    ///
    ///     expect result.url is String
    ///     expect result.ok == true
    /// }
    /// ```
    #[derive(Debug, Clone)]
    pub struct TestDef {
        pub name: String,
        /// Optional parent fixture name.
        pub fixture: Option<String>,
        /// Local let-bindings available to subsequent expectations.
        pub lets: Vec<LetDecl>,
        /// Mock declarations local to this test.
        pub mocks: Vec<MockDecl>,
        /// Input value injections for dangling DAG entry ports.
        pub inputs: Vec<InputDecl>,
        /// Assertions on the DAG result.
        pub expects: Vec<ExpectStmt>,
        pub tier: Option<String>,
        pub hermetic: bool,
        pub skip: bool,
    }

    /// A node reference inside an inline test block.
    ///
    /// Local references are resolved against the surrounding module. Qualified
    /// references carry their target module explicitly at parse time.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TestNodeRef {
        Local {
            node_segments: Vec<String>,
        },
        Qualified {
            module: ModulePath,
            node_segments: Vec<String>,
        },
    }

    impl TestNodeRef {
        pub fn local(node_segments: Vec<String>) -> Self {
            Self::Local { node_segments }
        }

        pub fn qualified(module: ModulePath, node_segments: Vec<String>) -> Self {
            Self::Qualified {
                module,
                node_segments,
            }
        }

        pub fn node_segments(&self) -> &[String] {
            match self {
                Self::Local { node_segments } | Self::Qualified { node_segments, .. } => {
                    node_segments
                }
            }
        }

        /// Render the node ID exactly as written by the test reference.
        ///
        /// Local references stay unqualified; qualified references retain their
        /// explicit module prefix.
        pub fn as_source_node_id(&self) -> String {
            let node_id = self.node_segments().join("/");
            match self {
                Self::Local { .. } => node_id,
                Self::Qualified { module, .. } => format!("{module}::{node_id}"),
            }
        }
    }

    /// A mock declaration: `mock <node_path>.<port> -> <value>`.
    ///
    /// The node reference preserves whether the target was written relative to
    /// the local module or explicitly qualified as `module.path::node/path`.
    /// The last dotted segment is the port name.
    #[derive(Debug, Clone)]
    pub struct MockDecl {
        /// Node reference targeted by this mock.
        pub node_ref: TestNodeRef,
        /// Port name (the segment after the last `.`).
        pub port: String,
        /// The mock value expression.
        pub value: Expr,
    }

    /// An input declaration: `input <node_path>.<port> = <value>`.
    #[derive(Debug, Clone)]
    pub struct InputDecl {
        pub node_ref: TestNodeRef,
        pub port: String,
        pub value: Expr,
    }

    /// A local let-binding inside a test block: `let <name> = <expr>`.
    #[derive(Debug, Clone)]
    pub struct LetDecl {
        pub name: String,
        pub value: Expr,
    }

    /// The left-hand side of an expect assertion: a structured path identifying
    /// which node output to check.
    ///
    /// `result.port` targets the DAG's terminal output. All other references
    /// follow the same local-vs-qualified scheme as mock/input targets.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ExpectTarget {
        /// `result.port` — the DAG's terminal output.
        Result { port: String },
        /// `node_ref.port` — a specific node's output.
        Node { node_ref: TestNodeRef, port: String },
    }

    impl ExpectTarget {
        pub fn port(&self) -> &str {
            match self {
                Self::Result { port } | Self::Node { port, .. } => port,
            }
        }

        pub fn node_ref(&self) -> Option<&TestNodeRef> {
            match self {
                Self::Result { .. } => None,
                Self::Node { node_ref, .. } => Some(node_ref),
            }
        }
    }

    /// An expect assertion.
    #[derive(Debug, Clone)]
    pub enum ExpectStmt {
        /// `expect <target> == <expr>`
        Eq(ExpectTarget, Expr),
        /// `expect <target> != <expr>`
        Ne(ExpectTarget, Expr),
        /// `expect <target> < <expr>`
        Lt(ExpectTarget, Expr),
        /// `expect <target> > <expr>`
        Gt(ExpectTarget, Expr),
        /// `expect <target> <= <expr>`
        Le(ExpectTarget, Expr),
        /// `expect <target> >= <expr>`
        Ge(ExpectTarget, Expr),
        /// `expect <target> contains <string_expr>`
        Contains(ExpectTarget, Expr),
        /// `expect <target> is <type_name>` (e.g., String, Bool, Int, NonEmpty)
        Is(ExpectTarget, String),
        /// `expect <target>` -- truthiness check
        Truthy(ExpectTarget),
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
        /// For loop (map sugar): `for x in list with {ctx} { body }`
        ///
        /// Tuple fields:
        /// 1. element binding variable name
        /// 2. iterable expression
        /// 3. passthrough bindings explicitly forwarded into body scope
        /// 4. body expression or block
        For(String, Box<Expr>, Vec<String>, ForBody),
        /// Lambda: `x => x.name`
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
        /// Statement block in expression position: `{ let x = 1\n x + 1 }`
        Block(Vec<Stmt>),
    }

    #[derive(Debug, Clone)]
    pub enum ForBody {
        Expr(Box<Expr>),
        Block(Vec<Stmt>),
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
        NullCoalesce,
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

    // ── Typed syntax ────────────────────────────────────────────────

    #[derive(Debug, Clone, Default)]
    pub struct ServiceConfig {
        pub endpoint: Option<String>,
        pub auth: Option<String>,
        /// Name of the input field that carries the authentication credential.
        /// When set, the lowerer wires this argument to `res:credential` on the
        /// execute node instead of including it in the prepare body.
        pub auth_input: Option<String>,
        /// Rate limiting configuration for this service.
        pub rate_limits: Vec<RateLimitDef>,
        /// Retry policy for this service.
        pub retry: Option<RetryDef>,
        /// Error shape mapping for this service.
        pub error_shapes: Vec<ErrorShapeDef>,
        /// Credential configuration for this service.
        pub credential: Option<CredentialDef>,
        /// Provider-specific configuration fields not covered by the core
        /// schema. Parsed as typed declarations for downstream validation
        /// (e.g., `bucket: NonEmptyStr`, `project_id: ProjectId`).
        pub extra: Vec<ProviderConfigField>,
        /// Explicit response provider classification (S45). When set, overrides
        /// inference from service name substrings. Valid values: `GitHub`,
        /// `Gcp`, `Anthropic`, `OpenAi`, `Generic`.
        pub response_provider: Option<String>,
    }

    /// A provider-specific config field: `name: Type` or `name: Type = default`.
    #[derive(Debug, Clone)]
    pub struct ProviderConfigField {
        /// Field name (e.g., `bucket`, `project_id`).
        pub name: String,
        /// Type annotation (e.g., `NonEmptyStr`, `List<Json>`).
        pub ty: TypeExpr,
        /// Optional default value expression (e.g., `"anthropic"`, `5`, `[]`).
        pub default: Option<Expr>,
    }

    // ── Transport blocks (TL-11) ─────────────────────────────────────

    /// Rate limit definition: `rate_limit { requests: 5000, per: hour, scope: core }`
    #[derive(Debug, Clone)]
    pub struct RateLimitDef {
        /// Number of requests allowed in the window.
        pub requests: i64,
        /// Time unit for the rate limit window.
        pub per: RateLimitUnit,
        /// Optional scope name (e.g., "core", "search").
        pub scope: Option<String>,
    }

    /// Rate limit time unit.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RateLimitUnit {
        Second,
        Minute,
        Hour,
        Day,
    }

    /// Retry policy definition: `retry { max_attempts: 3, backoff: exponential }`
    #[derive(Debug, Clone)]
    pub struct RetryDef {
        /// Maximum number of retry attempts.
        pub max_attempts: i64,
        /// Backoff strategy.
        pub backoff: BackoffStrategy,
        /// Optional base delay in milliseconds.
        pub base_delay_ms: Option<i64>,
        /// Optional max delay in milliseconds.
        pub max_delay_ms: Option<i64>,
        /// HTTP status codes that trigger retry.
        pub retry_on: Vec<i64>,
    }

    /// Backoff strategy for retries.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BackoffStrategy {
        Constant,
        Linear,
        Exponential,
    }

    /// Error shape definition for provider-specific error mapping.
    #[derive(Debug, Clone)]
    pub struct ErrorShapeDef {
        /// HTTP status code or range (e.g., 400, "4xx").
        pub status: String,
        /// Error type field path in response body.
        pub error_type_path: Option<String>,
        /// Error message field path in response body.
        pub message_path: Option<String>,
        /// Whether this error is retryable.
        pub retryable: bool,
    }

    /// Credential configuration for service authentication.
    #[derive(Debug, Clone)]
    pub struct CredentialDef {
        /// Credential type (e.g., "bearer", "api_key", "oauth2").
        pub credential_type: String,
        /// Header name for the credential (if applicable).
        pub header: Option<String>,
        /// Environment variable or secret path.
        pub source: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub enum TransportBinding {
        Rest {
            method: String,
            path: String,
            body: Option<Expr>,
            headers: Option<Expr>,
        },
        Shell {
            argv: Vec<Expr>,
        },
        File {
            op: String,
            path: String,
        },
        Local,
    }

    #[derive(Debug, Clone)]
    pub enum Refinement {
        Pattern(String),
        Range {
            min: Option<Expr>,
            max: Option<Expr>,
        },
        Brand(String),
        NonEmpty,
        Content(String),
        Format(String),
        Predicate(String),
        FileTypes(Vec<String>),
        RawBody,
        /// Bit-width constraint: `width(8)`, `width(16)`, etc.
        Width(Expr),
        /// Collection/string length constraint: `length(4)`, `length(8)`, etc.
        Length(Expr),
        /// Signed integer: `signed` or `signed("twos_complement")`.
        Signed(Option<String>),
        /// Unsigned integer: `unsigned`.
        Unsigned,
        /// Arithmetic type: `arithmetic`.
        Arithmetic,
        /// Domain constraint: `domain("ieee754_binary32")`.
        Domain(String),
    }

    #[derive(Debug, Clone)]
    pub struct ContractDef {
        pub text: String,
    }

    // ── Bodies ───────────────────────────────────────────────────────

    /// Pure function body (expressions only, no I/O).
    #[derive(Debug, Clone)]
    pub struct FnBody {
        pub stmts: Vec<Stmt>,
    }

    /// Effectful function body (can include service calls, resource ops).
    #[derive(Debug, Clone)]
    pub struct FuncBody {
        pub stmts: Vec<Stmt>,
    }

    /// A `node` statement with optional guards: `node name [after a, when cond]: expr`
    #[derive(Debug, Clone)]
    pub struct NodeStmt {
        pub name: String,
        pub expr: Expr,
        pub after: Vec<String>,
        pub when_guard: Option<Expr>,
    }

    /// Statement in a function body.
    #[derive(Debug, Clone)]
    pub enum Stmt {
        Let(String, Expr),
        Assign(String, Expr),
        Expr(Expr),
        Return(Vec<(String, Expr)>),
        Node(NodeStmt),
    }
}
