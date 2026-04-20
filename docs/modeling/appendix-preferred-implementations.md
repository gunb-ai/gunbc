## Appendix: Preferred implementations

Concrete design targets for each major issue. These are the models
someone should implement — the "what it should look like" for each fix.

### A1: Generic result types (replaces P1 result proliferation)

Two patterns exist in the codebase. Both should be generic.

```dag
// Pattern 1: Parsing — threads state, may fail
type ParseResult<T>
  = Ok { value: T, state: ParserState }
  | Err { error: Diagnostic, state: ParserState }

// Pattern 2: Analysis — accumulates diagnostics
type Checked<T> {
  value: T
  diagnostics: List<Diagnostic>
}
```

**ParseResult as a sum type** is the key insight. The current design
has independent `value: T?` and `error: Diagnostic?` — four states
(both, neither, one, other) where only two are valid. The sum type
makes illegal states unrepresentable.

Replaces in 02_parse.dag: `ExprResult`, `ItemResult`, `TypeResult`,
`NameResult`, `FieldResult`, `VariantResult`, `ParamResult`,
`ImportResult`, `ModuleResult`, ... (57 types → 1).

Replaces in 04_typecheck.dag: `ResolveResult`, `InferResult`,
`TypedItemResult`, `AccessCheckResult`, ... (13 types → 1).

Helper for the common "try then continue" pattern:

```dag
fn try_parse<T>(r: ParseResult<T>) -> ParseResult<T> {
  // Identity — but makes intent explicit at call sites.
  // The sum type forces callers to match Ok/Err.
  r
}

// Example usage (current):
//   let r = parse_expr(state: state)
//   if has_err(err: r.err) { return XxxResult { ..., err: r.err } }
//   // use r.value
//
// Becomes:
//   match parse_expr(state: state) {
//     Ok { value: expr, state: s } => // use expr and s
//     Err { error: e, state: s } => Err { error: e, state: s }
//   }
```

### A2: Structural token dispatch (replaces P1 kind_tag)

The problem: `kind_tag` extracts a string from `TokenKind` (a sum type),
then callers compare strings. This defeats the type system.

```dag
// Option A: TokenTag enum (parallel to TokenKind but without payloads)
type TokenTag
  = TagKwModule | TagKwImport | TagKwType | TagKwResource
  | TagKwCapability | TagKwOperation | TagKwPattern
  | TagKwInput | TagKwOutput | TagKwData | TagKwMatch
  | TagKwService | TagKwFn | TagKwFunc | TagKwExtern
  | TagKwLet | TagKwReturn | TagKwIf | TagKwElse
  | TagKwFor | TagKwIn | TagKwWhere | TagKwWith
  | TagKwTrue | TagKwFalse | TagKwImport | TagKwModule
  | TagKwInterface | TagKwPipeline | TagKwProfile
  | TagKwIdempotent | TagKwReadonly | TagKwHermetic
  | TagIdent | TagLitStr | TagLitInt | TagLitFloat | TagLitNull
  | TagLBrace | TagRBrace | TagLParen | TagRParen
  | TagLBracket | TagRBracket
  | TagColon | TagComma | TagDot | TagEq | TagFatArrow
  | TagPipeArrow | TagPlus | TagMinus | TagStar | TagSlash
  | TagPercent | TagBang | TagQuestion
  | TagAnd | TagOr | TagLt | TagGt | TagLtEq | TagGtEq
  | TagEqEq | TagBangEq | TagNewline | TagEof | TagUnknown

fn token_tag(kind: TokenKind) -> TokenTag {
  match kind {
    KwModule => TagKwModule
    Ident { name: _ } => TagIdent
    LitStr { value: _ } => TagLitStr
    LBrace => TagLBrace
    // ... exhaustive — compiler catches missing variants
  }
}

fn check(state: ParserState, expected: TokenTag) -> Bool {
  match peek(state: state) {
    Some { value: t } => token_tag(kind: t.kind) == expected
    None => false
  }
}
```

**Option B (preferred if language supports it):** Skip `TokenTag`
entirely and match patterns directly in callers:

```dag
fn check_kind(state: ParserState, expected: TokenKind) -> Bool {
  match peek(state: state) {
    Some { value: t } => matches_variant(t.kind, expected)
    None => false
  }
}

// Usage: check_kind(state: s, expected: LBrace)
// Requires: `matches_variant` intrinsic or pattern-match sugar
```

### A3: Optional returns in typecheck (replaces P0 fabrication)

Every lookup that currently returns `unit_type()` on miss should
return `TypeExpr?` and force the caller to handle absence.

```dag
// CURRENT (fabrication):
fn lookup_in_scope(scope: InferScope, name: String) -> TypeExpr {
  // ... search locals, params, types ...
  unit_type()  // miss → silent Unit
}

// PREFERRED:
fn lookup_in_scope(scope: InferScope, name: String) -> TypeExpr? {
  let local = find(scope.locals, b => b.name == name)
  match local {
    Some { value: binding } => Some { value: binding.resolved }
    None =>
      let param = find(scope.func_params, p => p.name == name)
      match param {
        Some { value: p } => Some { value: p.type_expr }
        None => lookup_type(env: scope.type_env, name: name)
          // returns TypeExpr? — None propagates naturally
      }
  }
}

// Callers become explicit:
fn infer_var(scope: InferScope, name: String, span: SourceSpan) -> Checked<TypedExpr> {
  match lookup_in_scope(scope: scope, name: name) {
    Some { value: te } =>
      Checked {
        value: TypedExpr { expr: Var { name: name }, resolved_type: te },
        diagnostics: []
      }
    None =>
      Checked {
        value: TypedExpr { expr: Var { name: name }, resolved_type: unit_type() },
        diagnostics: [Diagnostic {
          severity: Error,
          message: concat("undefined variable: ", name),
          span: Some { value: span },
          module_name: scope.module_name
        }]
      }
  }
}
```

Same pattern for `lookup_field_type`:

```dag
// CURRENT:
fn lookup_field_type(type_expr: TypeExpr, field_name: String) -> TypeExpr {
  match type_expr {
    Product { fields: fields } =>
      // ... search ... else unit_type()
    _ => unit_type()
  }
}

// PREFERRED:
fn lookup_field_type(type_expr: TypeExpr, field_name: String) -> TypeExpr? {
  match type_expr {
    Product { name: _, fields: fields } =>
      match find(fields, f => f.name == field_name) {
        Some { value: f } => Some { value: f.type_expr }
        None => None  // field not found — caller decides what to do
      }
    Optional { inner: inner } =>
      lookup_field_type(type_expr: inner, field_name: field_name)
    _ => None  // not a product — caller must report error
  }
}
```

### A4: Method and predicate enums (replaces P1 string dispatch)

```dag
// Method names are a closed set — model them as such
type PipeMethod
  = Map | Filter | Fold | First | Last
  | Count | Join | Any | Contains
  | Enumerate | Sum | Chars | Split

// Method type inference becomes structural
fn infer_method_result(receiver: TypeExpr, method: PipeMethod) -> TypeExpr? {
  match method {
    Map => Some { value: receiver }
    Filter => Some { value: receiver }
    Fold => None  // depends on accumulator — caller resolves from args
    First => extract_element_type(container: receiver)
    Last => extract_element_type(container: receiver)
    Count => Some { value: Primitive { name: "Int", span: no_span() } }
    Join => Some { value: Primitive { name: "String", span: no_span() } }
    Any => Some { value: Primitive { name: "Bool", span: no_span() } }
    Contains => Some { value: Primitive { name: "Bool", span: no_span() } }
    Enumerate => Some { value: receiver }  // List<T> → List<(Int, T)>
    Sum => Some { value: Primitive { name: "Int", span: no_span() } }
    Chars => Some { value: Container { kind: "List", element: Primitive { name: "String", span: no_span() }, span: no_span() } }
    Split => Some { value: Container { kind: "List", element: Primitive { name: "String", span: no_span() }, span: no_span() } }
  }
}

// Predicate kinds — also a closed set
type PredicateKind
  = PatternPred | FormatPred | BrandPred
  | ContentPred | DomainPred | RangePred | NonEmptyPred

// Parser produces PredicateKind, not strings:
fn parse_predicate_kind(name: String) -> PredicateKind? {
  match name {
    "pattern" => Some { value: PatternPred }
    "format" => Some { value: FormatPred }
    "brand" => Some { value: BrandPred }
    "content" => Some { value: ContentPred }
    "domain" => Some { value: DomainPred }
    "range" => Some { value: RangePred }
    "non_empty" => Some { value: NonEmptyPred }
    _ => None  // unknown predicate — caller emits diagnostic
  }
}
```

### A5: Import names as sum type (replaces P2 wildcard sentinel)

```dag
// CURRENT: uses ["*"] as sentinel for "import all"
type Import {
  module_path: String
  names: List<String>  // ["*"] means all, [] means empty block
  span: SourceSpan
}

// PREFERRED:
type ImportNames
  = ImportAll                          // import foo.bar
  | ImportSpecific { names: List<String> }  // import foo.bar { X, Y }

type Import {
  module_path: String
  names: ImportNames
  span: SourceSpan
}

// Parser:
//   bare import (no braces) → ImportAll
//   import foo { X, Y }    → ImportSpecific { names: ["X", "Y"] }
//   import foo { }          → ImportSpecific { names: [] }
//
// Resolver:
//   match import.names {
//     ImportAll => export all names from target module
//     ImportSpecific { names } => validate each name exists
//   }
```

### A6: Resolve index lookups (replaces P0 sentinel values)

```dag
// CURRENT: returns -1, 0, [] as sentinels
fn find_index_in_list(names: List<String>, target: String) -> Int {
  // ... None => -1
}

// PREFERRED:
fn find_index_in_list(names: List<String>, target: String) -> Int? {
  let matches = names |> enumerate |> filter(pair => pair.second == target)
  match matches |> first {
    Some { value: pair } => Some { value: pair.first }
    None => None
  }
}

// For in-degree lookup (Kahn's algorithm):
fn get_in_degree(in_degrees: List<Int>, index: Int) -> Int {
  match get_at_index_safe(items: in_degrees, index: index) {
    Some { value: n } => n
    None => panic("get_in_degree: index out of bounds")
    // Panic is correct here — out-of-bounds in Kahn's is a
    // programming error, not a data error. Silent 0 corrupts
    // the topological sort.
  }
}
```

### A7: Pipeline result as sum type (replaces P2 anemic pipeline)

```dag
// CURRENT:
type ParseFileResult {
  module: Module?    // independent optionals —
  error: Diagnostic? // four states, only two valid
}

// PREFERRED:
type ParseFileResult
  = ParseOk { module: Module, state: ParserState }
  | ParseFail { error: Diagnostic, state: ParserState }

// Pipeline becomes:
fn compile_file(source: String) -> ParseFileResult {
  let tokens = tokenize(source: source)
  match parse_module(tokens: tokens) {
    Ok { value: module, state: s } => ParseOk { module: module, state: s }
    Err { error: e, state: s } => ParseFail { error: e, state: s }
  }
}

// Compile pipeline error gating:
fn compile(sources: List<SourceFile>) -> CompileResult {
  let parse_results = map(sources, s => compile_file(source: s.content))

  // Type-safe error check — no list length comparison needed
  let failures = filter(parse_results, r => match r {
    ParseFail { error: _, state: _ } => true
    _ => false
  })

  if count(failures) > 0 {
    CompileResult {
      files: [],
      diagnostics: map(failures, f => f.error)
    }
  } else {
    let modules = map(parse_results, r => r.module)
    // ... continue pipeline with guaranteed-valid modules
  }
}
```

### A8: ParserState with context (replaces P2 anemic state)

```dag
// CURRENT:
type ParserState {
  tokens: List<Token>
  pos: Int
}

// PREFERRED:
type ParserState {
  tokens: List<Token>
  pos: Int
  filename: String       // for error messages
  module_path: String    // for qualified name context
}

// Diagnostic construction gets context automatically:
fn parse_error(state: ParserState, msg: String) -> Diagnostic {
  Diagnostic {
    severity: Error,
    message: msg,
    span: current_span(state: state),
    module_name: Some { value: state.module_path }
  }
}
```

### A9: Keyword table as shared data (replaces P1 duplication)

```dag
// In 01_tokenize.dag (already exists):
data keywords: Map<String, TokenKind> = [
  { key: "type", value: KwType },
  { key: "resource", value: KwResource },
  // ... 30+ entries
]

// In 00_core.dag or shared module — derive reverse mapping:
data keyword_names: Map<TokenKind, String> = reverse_map(keywords)
// or: fn keyword_name(kind: TokenKind) -> String? = lookup(keyword_names, kind)

// 02_parse.dag uses the shared table:
fn keyword_to_name(kind: TokenKind) -> String? {
  lookup(keyword_names, key: kind)
}

// Eliminates: 28 if-else clauses in keyword_to_name
// Eliminates: 23 if-else clauses in keyword_to_arg_label
// Single authority: the keywords data table in tokenize.dag
```

### A10: Export separation in resolve (replaces P2 conflation)

```dag
// CURRENT: variant names mixed with type names in flat list

// PREFERRED:
type ExportedNames {
  types: List<String>          // type Foo, type Bar
  constructors: List<String>   // Foo's variants: A, B, C
  functions: List<String>      // fn baz, func qux
  data: List<String>           // data constants
}

fn collect_exports(module: Module) -> ExportedNames {
  ExportedNames {
    types: map(filter(module.items, i => is_type_def(item: i)), i => i.name),
    constructors: flat_map(
      filter(module.items, i => is_coproduct(item: i)),
      i => get_variant_names(item: i)
    ),
    functions: map(filter(module.items, i => is_fn_def(item: i)), i => i.name),
    data: map(filter(module.items, i => is_data_def(item: i)), i => i.name)
  }
}

// Resolver can now validate: "did you import a type, a constructor,
// or a function?" — different validation rules per kind.
```

### A11: Encoding type reconciliation (P2 — M2 violation)

`types.dag` defines `ContentEncoding`, `encoding.dag` defines `Encoding`.
Both claim authority over the same concept. `string_type.dag` imports
from `encoding.dag`, making it the foundation chain's authority.

```dag
// encoding.dag — KEEP as single authority
// Ref: IANA Character Sets registry, MIME charset parameter
type Encoding = ASCII | UTF8 | Latin1 | Binary

// types.dag — DELETE ContentEncoding definition, import instead:
import std.encoding { Encoding }

// NOTE: encoding.dag currently has Text and Unknown variants that
// don't correspond to real encodings. Remove them:
//   Text   → not an encoding, it's a classification (use is_text_readable)
//   Unknown → violates M5 (silence is fabrication) — if encoding
//             is unknown, that's an error, not a valid state
```

### A12: Emit anonymous record fallback (P0 — fabrication)

```dag
// CURRENT in 05_emit.dag:
fn emit_product_type_expr(name: String?, fields: List<Field>) -> String {
  match name {
    Some { value: n } => n
    None =>
      if count(fields) == 1 { emit_type_expr(type_expr: first(fields).type_expr) }
      else { "serde_json::Value" }  // <-- silent structure erasure
  }
}

// PREFERRED — anonymous records MUST be named before emission.
// The fix belongs in the typechecker, not the emitter:

// In 04_typecheck.dag — name anonymous records during type resolution:
fn name_anonymous_record(fields: List<Field>, context: String) -> TypeExpr {
  // Context is the enclosing function/let binding name.
  // { left: Int, right: Int } in fn parse_binop → BinopRecord
  Product {
    name: Some { value: synthesize_name(fields: fields, context: context) },
    fields: fields,
    span: no_span()
  }
}

// The emitter then NEVER sees unnamed products. If one arrives,
// it's a bug — fail loudly:
fn emit_product_type_expr(name: String?, fields: List<Field>) -> String {
  match name {
    Some { value: n } => n
    None => panic("emit_product_type_expr: unnamed record reached emitter")
  }
}
```

### A13: Symbol resolution without fabrication (P2 — M5)

```dag
// CURRENT in std/symbols.dag:
fn resolve_symbol(id: SymbolId, tier: SymbolTier) -> String {
  let matches = filter(standard_symbols, s => s.id == id)
  match first(matches) {
    Some { value: entry } => // extract tier
    None => ""  // <-- silent empty string on miss
  }
}

// PREFERRED:
fn resolve_symbol(id: SymbolId, tier: SymbolTier) -> String? {
  let matches = filter(standard_symbols, s => s.id == id)
  match first(matches) {
    Some { value: entry } =>
      match tier {
        Emoji => Some { value: entry.emoji }
        Unicode => Some { value: entry.unicode }
        Ascii => Some { value: entry.ascii }
      }
    None => None  // caller decides: fallback to ascii? error?
  }
}
```

### A14: Fidelity without wildcard fallback (P2 — M5)

```dag
// CURRENT in std/fidelity.dag:
fn transport_depth(tc: TransportClass) -> FermiDepth {
  match tc {
    LocalDirect => Xs
    ShellLocal => Sm
    FileBoundary => Md
    RestNetwork => Lg
    InterfaceStub => Xs
    _ => Xl  // <-- silent fallback: new transport class → Xl
  }
}

// PREFERRED — exhaustive, no wildcard:
fn transport_depth(tc: TransportClass) -> FermiDepth {
  match tc {
    LocalDirect => Xs
    ShellLocal => Sm
    FileBoundary => Md
    RestNetwork => Lg
    InterfaceStub => Xs
    Unknown => Xl
  }
  // If a new TransportClass variant is added, the compiler
  // forces you to add a case here. No silent default.
}
```

### A15: Fermi timeout — single authority (P2 — M7)

```dag
// CURRENT: same mapping as both data AND function
data fermi_timeouts: List<FermiTimeout> = [
  { depth: Xs, timeout_ms: 30000, label: "30 seconds" },
  { depth: Sm, timeout_ms: 300000, label: "5 minutes" },
  // ...
]

fn timeout_for_depth(depth: FermiDepth) -> Int {
  match depth {
    Xs => 30000    // <-- DUPLICATE of data above
    Sm => 300000
    // ...
  }
}

// PREFERRED — derive function from data:
data fermi_timeouts: List<FermiTimeout> = [
  { depth: Xs, timeout_ms: 30000 },
  { depth: Sm, timeout_ms: 300000 },
  { depth: Md, timeout_ms: 600000 },
  { depth: Lg, timeout_ms: 1800000 },
  { depth: Xl, timeout_ms: 3600000 }
]

fn timeout_for_depth(depth: FermiDepth) -> Int {
  let matches = filter(fermi_timeouts, t => t.depth == depth)
  match first(matches) {
    Some { value: t } => t.timeout_ms
    None => panic("timeout_for_depth: unknown depth")
  }
}
// Single authority: fermi_timeouts data table.
// label field removed — derive from timeout_ms if needed.
```

### A16: GitHub → Git type references (P2 — objective relationships)

GitHub's branching, commit, and diff models are built on Git's.
This relationship is documented in GitHub's own docs. The DAG
modeling should reflect it.

```dag
// In extdeps/github/github.dag:
import extdeps.git { CommitSha, GitRef, GitCommit, DiffHunk }

// CURRENT:
type Repository {
  owner: String
  name: String
  full_name: String
  default_branch: String  // <-- bare string
  // ...
}

// PREFERRED:
type Repository {
  owner: GitHubUser
  name: String
  full_name: String
  default_branch: GitRef  // ← references Git's branch model
  // ...
}

// Pull request references Git concepts directly:
type PullRequest {
  number: Int
  head: GitRef       // ← Git branch
  base: GitRef       // ← Git branch
  merge_commit: CommitSha?  // ← Git commit SHA
  // ...
}

// The relationship is factual: GitHub's API docs say
// "head" and "base" are Git refs. Not interpretation.
```

### A17: LLM multimodal content (P2 — M1 faithfulness)

Both Anthropic and OpenAI support rich content beyond plain strings.
The shared `LlmMessage` type should reflect this.

```dag
// CURRENT in extdeps/llm/llm.dag:
type LlmMessage {
  role: Role
  content: String  // <-- doesn't model multimodal
}

// PREFERRED — content is a list of typed blocks:
type ContentBlock
  = TextContent { text: String }
  | ImageContent { source: ImageSource }

type ImageSource
  = Base64Image { media_type: String, data: String }
  | UrlImage { url: String }

type LlmMessage {
  role: Role
  content: List<ContentBlock>
}

// This matches what BOTH providers actually accept:
// - Anthropic: content is List<ContentBlock> (text, image, tool_use, tool_result)
// - OpenAI: content is string OR array of {type: "text"/"image_url", ...}
//
// Provider-specific block types (ToolUseBlock, etc.) stay in
// anthropic.dag / openai.dag. The shared type covers the
// intersection that both providers document.
```

### A18: String fields → enums across extdeps (P3 — M4)

```dag
// In 00_core.dag — AuthConfig:
// CURRENT:  scheme: String
// PREFERRED:
type AuthScheme = Bearer | ApiKey | Basic | Custom { header: String }

type AuthConfig {
  scheme: AuthScheme  // closed set, not open string
  // ...
}

// In extdeps/llm/anthropic.dag — ThinkingConfig:
// DONE: ThinkingMode = Enabled | Disabled, mode: ThinkingMode, budget_tokens: Int?

// In extdeps/github/github.dag — scopes:
// CURRENT:  scopes: List<String>
// PREFERRED:
type GitHubScope
  = RepoRead | RepoWrite | RepoAdmin
  | GistRead | GistWrite
  | UserRead | UserEmail
  | OrgRead | OrgAdmin
  | Workflow
  // Ref: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps

type GitHubAuthToken {
  token: Secret
  scopes: List<GitHubScope>
}
```
