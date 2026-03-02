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
//! - `interface` -- abstract capability contracts with `contract` declarations
//! - `pipeline` -- multi-stage pipeline declarations
//! - `profile` -- deployment profile interface bindings
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
        ExternFuncDecl(ExternFuncDecl),
        ExternAssetDecl(ExternAssetDecl),
    }

    /// Module-level parameter: `param name: Type` or `param name: Type = default`
    #[derive(Debug)]
    pub struct ParamDecl {
        pub name: String,
        pub ty: TypeExpr,
        pub default: Option<Expr>,
    }

    /// Module-level static data declaration: `data name: Type = value`.
    #[derive(Debug)]
    pub struct DataDef {
        pub name: String,
        pub ty: TypeExpr,
        pub value: Expr,
    }

    /// `extern func name(input1: Type1, input2: Type2) -> { output1: Type1, output2: Type2 }`
    #[derive(Debug)]
    pub struct ExternFuncDecl {
        pub name: String,
        pub inputs: Vec<Field>,
        pub outputs: Vec<Field>,
    }

    /// `extern asset name: Type`
    #[derive(Debug)]
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
        Optional(Box<TypeExpr>),
        /// Refined type: `Base where constraint1, constraint2`
        Refined(Box<TypeExpr>, Vec<Refinement>),
        /// Anonymous record return type: `-> { field: Type, ... }`
        Record(Vec<Field>),
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
        pub body: FuncBody,
        pub declared_outputs: Vec<String>,
    }

    /// Reusable DAG template: `pattern name(params) -> { outputs } uses ... provides ... { body }`
    #[derive(Debug)]
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

    #[derive(Debug)]
    pub struct ServiceDef {
        pub name: String,
        pub implements: Option<String>,
        pub operations: Vec<OperationDef>,
        pub config: ServiceConfig,
    }

    #[derive(Debug)]
    pub struct OperationDef {
        pub name: String,
        pub inputs: Vec<Field>,
        pub outputs: Vec<Field>,
        pub idempotent: bool,
        pub readonly: bool,
        pub permissions: Vec<String>,
        pub transport: Option<TransportBinding>,
        /// Response contract: maps HTTP status codes/patterns to response types.
        pub response: Vec<ResponseEntry>,
        /// Exit contract: maps shell exit codes to output types.
        pub exit: Vec<ExitEntry>,
    }

    /// Response contract entry: STATUS => TYPE.
    /// STATUS is a pattern (exact code like 200, or wildcard like 2xx, 4xx, 5xx).
    #[derive(Debug, Clone)]
    pub struct ResponseEntry {
        pub status: StatusPattern,
        pub response_type: TypeExpr,
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
        pub idempotent: bool,
        pub readonly: bool,
    }

    // ── Interfaces ──────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct InterfaceDef {
        pub name: String,
        pub type_params: Vec<String>,
        pub capabilities: Vec<CapabilityDef>,
        pub contracts: Vec<ContractDef>,
    }

    // ── Pipelines ───────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct PipelineDef {
        pub name: String,
        pub uses: Vec<UsesClause>,
        pub stages: Vec<StageDef>,
    }

    #[derive(Debug)]
    pub struct StageDef {
        pub name: String,
        pub body: FuncBody,
        pub after: Vec<String>,
        pub when: Option<Expr>,
    }

    // ── Profiles ────────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct ProfileDef {
        pub name: String,
        pub binds: Vec<ProfileBind>,
    }

    #[derive(Debug)]
    pub struct ProfileBind {
        pub interface_type: String,
        pub implementation_type: String,
        pub config_entries: Vec<(String, Expr)>,
    }

    // ── SDLC & Infra Blocks ─────────────────────────────────────────

    #[derive(Debug)]
    pub struct ProjectDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug)]
    pub struct FeatureDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug)]
    pub struct TaskDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug)]
    pub struct DesignDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug)]
    pub struct ComponentDef {
        pub name: String,
        pub properties: Vec<(String, Expr)>,
    }

    #[derive(Debug)]
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
        pub auto_mock: bool,
        pub mock_helpers: Option<String>,
    }

    /// A mock declaration: `mock <node_path>.<port> -> <value>`.
    ///
    /// The `node_segments` are joined with `/` to form the DAG node ID.
    /// The last dotted segment is the port name.
    #[derive(Debug, Clone)]
    pub struct MockDecl {
        /// Node path segments (joined with `/` to form node ID).
        pub node_segments: Vec<String>,
        /// Port name (the segment after the last `.`).
        pub port: String,
        /// The mock value expression.
        pub value: Expr,
    }

    /// An input declaration: `input <node_path>.<port> = <value>`.
    #[derive(Debug, Clone)]
    pub struct InputDecl {
        pub node_segments: Vec<String>,
        pub port: String,
        pub value: Expr,
    }

    /// A local let-binding inside a test block: `let <name> = <expr>`.
    #[derive(Debug, Clone)]
    pub struct LetDecl {
        pub name: String,
        pub value: Expr,
    }

    /// An expect assertion.
    #[derive(Debug, Clone)]
    pub enum ExpectStmt {
        /// `expect <expr> == <expr>`
        Eq(Expr, Expr),
        /// `expect <expr> != <expr>`
        Ne(Expr, Expr),
        /// `expect <expr> < <expr>`
        Lt(Expr, Expr),
        /// `expect <expr> > <expr>`
        Gt(Expr, Expr),
        /// `expect <expr> <= <expr>`
        Le(Expr, Expr),
        /// `expect <expr> >= <expr>`
        Ge(Expr, Expr),
        /// `expect <expr> contains <string_expr>`
        Contains(Expr, Expr),
        /// `expect <expr> is <type_name>` (e.g., String, Bool, Int, NonEmpty)
        Is(Expr, String),
        /// `expect <expr>` -- truthiness check
        Truthy(Expr),
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
        /// 4. body expression
        For(String, Box<Expr>, Vec<String>, Box<Expr>),
        /// Pipe: `expr |> fn`
        Pipe(Box<Expr>, Box<Expr>),
        /// Built-in pipe method call: `expr |> method(args)`
        PipeCall(Box<Expr>, PipeMethod, Vec<(Option<String>, Expr)>),
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

    /// Built-in pipe methods resolved by parser/typechecker/lowerer as
    /// first-class syntax, not free-form callable names.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PipeMethod {
        // Collection -> Collection
        Map,
        Filter,
        FilterMap,
        FlatMap,
        SortBy,
        Append,
        // Collection -> Scalar
        Fold,
        Join,
        Count,
        Sum,
        First,
        Last,
        MaxBy,
        Any,
        All,
        Contains,
        // String methods
        StartsWith,
        EndsWith,
        Repeat,
        ReplaceSection,
        Chars,
        // Conversion methods
        ToBytes,
        ToJson,
        Hash,
    }

    impl PipeMethod {
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::Map => "map",
                Self::Filter => "filter",
                Self::FilterMap => "filter_map",
                Self::FlatMap => "flat_map",
                Self::SortBy => "sort_by",
                Self::Append => "append",
                Self::Fold => "fold",
                Self::Join => "join",
                Self::Count => "count",
                Self::Sum => "sum",
                Self::First => "first",
                Self::Last => "last",
                Self::MaxBy => "max_by",
                Self::Any => "any",
                Self::All => "all",
                Self::Contains => "contains",
                Self::StartsWith => "starts_with",
                Self::EndsWith => "ends_with",
                Self::Repeat => "repeat",
                Self::ReplaceSection => "replace_section",
                Self::Chars => "chars",
                Self::ToBytes => "to_bytes",
                Self::ToJson => "to_json",
                Self::Hash => "hash",
            }
        }
    }

    impl std::fmt::Display for PipeMethod {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }

    impl std::str::FromStr for PipeMethod {
        type Err = ();

        fn from_str(name: &str) -> Result<Self, Self::Err> {
            match name {
                "map" => Ok(Self::Map),
                "filter" => Ok(Self::Filter),
                "filter_map" => Ok(Self::FilterMap),
                "flat_map" => Ok(Self::FlatMap),
                "sort_by" => Ok(Self::SortBy),
                "append" => Ok(Self::Append),
                "fold" => Ok(Self::Fold),
                "join" => Ok(Self::Join),
                "count" => Ok(Self::Count),
                "sum" => Ok(Self::Sum),
                "first" => Ok(Self::First),
                "last" => Ok(Self::Last),
                "max_by" => Ok(Self::MaxBy),
                "any" => Ok(Self::Any),
                "all" => Ok(Self::All),
                "contains" => Ok(Self::Contains),
                "starts_with" => Ok(Self::StartsWith),
                "ends_with" => Ok(Self::EndsWith),
                "repeat" => Ok(Self::Repeat),
                "replace_section" => Ok(Self::ReplaceSection),
                "chars" => Ok(Self::Chars),
                "to_bytes" => Ok(Self::ToBytes),
                "to_json" => Ok(Self::ToJson),
                "hash" => Ok(Self::Hash),
                _ => Err(()),
            }
        }
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
        pub lossy: bool,
    }

    /// Effectful function body (can include service calls, resource ops).
    #[derive(Debug, Clone)]
    pub struct FuncBody {
        pub stmts: Vec<Stmt>,
        pub lossy: bool,
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
