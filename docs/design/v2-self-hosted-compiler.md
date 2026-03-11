# v2: Self-hosted compiler design

## Premise

The v0/v1 compiler exists to eliminate glue bugs in downstream systems,
but the compiler itself is full of the same glue bugs: string-matching,
fabrication fallbacks, parallel representations, deferred computation.
The sustainability ledger (SUSTAINABILITY.md) traces 38 symptoms to one
deep root: **incomplete compile-time resolution.**

The fix is not to patch each symptom. The fix is to write the compiler
in its own language, so the compiler's invariants are enforced by the
compiler itself.

The compiler is a pure transform: `.dag files → generated files`. Every
stage is a function from data to data. The only I/O is reading source
files and writing output files — which the DAG already models as
transport nodes.

## What went wrong with v0/v1

The core mistake: **TypeId is a string.** A port says
`type_id: TypeId("ResourceHandle")` instead of embedding the type
structure. This forces a TypeRegistry (symbol table) to exist at
runtime, forces threading the registry to every consumer, and when
threading fails, forces fabrication fallbacks.

But TypeId is a symptom of a deeper mistake: **treating types as a
separate concern from the DAG.** Types are defined in .dag files,
compiled into `Dag<TypeOp>`, stored in a registry, and then referenced
by string. But types ARE DAGs. A product type is a DAG with field nodes.
A coproduct is a DAG with variant nodes. A refined type is a DAG with
predicate nodes. There's no reason to store them in a separate lookup
table — they're the same structure as everything else.

The same mistake repeats at other levels:
- `PortMultiplicity` was a second field encoding what `Cardinality`
  already said (deleted in FC-7)
- `base_type()` re-walked the type DAG to extract information that
  was already structurally present
- `mock_element_expr` enumerated types by name because it couldn't
  query the type structure

Each of these is the same pattern: **information exists in the
structure, but code accesses a lossy projection of that structure
(a string, a flag, a match arm) instead of the structure itself.**

## Core principle: everything is a DAG node

In v2, there is one data structure: the DAG. Types, functions, service
bindings, data declarations, and the compilation pipeline itself are
all expressed as DAG nodes with typed ports and edges.

There is no TypeId. There is no TypeRegistry. There is no separate AST
vs IR vs executable representation. There is one structure that gets
progressively enriched as it flows through the pipeline.

### Types are nodes

A type is not a string. A type is a node (or subgraph) in the DAG:

```dag
// A product type is a node with field sub-nodes
type Span {
  text: String
  style: SpanStyle
}

// This compiles to a DAG node:
//   Node("Span", body: Product)
//     SubDag("field_text", body: Identity("String"))
//     SubDag("field_style", body: Ref("SpanStyle"))
//
// Where Ref("SpanStyle") is resolved at compile time to the actual
// SpanStyle subgraph — not deferred to a runtime lookup.
```

A port's type is a reference to a node in the same DAG, not a string
key into an external table:

```dag
// v1: port stores a string, needs registry to resolve
Port { name: "text", type_id: TypeId("String"), cardinality: ONE }

// v2: port references a type node directly
Port { name: "text", type_node: NodeRef("String") }
// Cardinality is derived from the type node's structure (is it a
// List wrapper? → list cardinality. Otherwise → scalar.)
```

### Functions are subgraphs

A function body is a subgraph within the DAG. It has input ports
(parameters), internal nodes (let bindings, operations), and output
ports (return values):

```dag
fn span_width(span: Span, tier: Tier) -> Int {
  let text_w = string_display_width(s: span.text)
  match span.style.symbol {
    null => text_w
    sym  => text_w + string_display_width(s: resolve_symbol(id: sym, tier: tier))
  }
}

// Compiles to a SubDag node with:
//   input ports: [span: Ref(Span), tier: Ref(Tier)]
//   internal nodes: field_access, function_call, match, add
//   output ports: [return: Ref(Int)]
//   edges connecting them
```

There is no separate "fn body evaluator" vs "DAG executor." The fn
body IS a DAG. The executor runs it the same way it runs any other
subgraph. No parallel implementation, no divergence, no catch-all
passthrough.

### Services are transport-tagged nodes

A service call is a node with transport metadata. The transport
binding (REST, shell, file) is an annotation on the node, not a
separate dispatch table:

```dag
service github.Gists {
  transport: Rest {
    base_url: "https://api.github.com"
    auth: header("Authorization", "token {credential.token}")
  }

  operation create {
    input { description: String, files: Map<String, String> }
    output { id: GistId, html_url: Url }
  }
}

// Compiles to nodes with transport metadata embedded:
//   Node("prepare_github_gists_create",
//     transport: Rest { method: POST, path: "/gists" },
//     inputs: [...], outputs: [request: TransportRequest])
//   Node("execute_github_gists_create",
//     transport: Execute,
//     inputs: [request: TransportRequest], outputs: [response: TransportResponse])
//   Node("parse_github_gists_create",
//     transport: Rest { response_shape: {id, html_url} },
//     inputs: [response: TransportResponse], outputs: [id: GistId, html_url: Url])
```

No `GenericPrepareOp` / `GenericParseOp` with 5-variant match. The
transport metadata is on the node. The executor reads it directly.

## The compilation pipeline as a DAG

The compiler itself is a DAG:

```dag
module compiler

import std.resources { Filesystem }

// Stage 1: Read source files
func read_sources(root: FilePath) -> List<SourceFile> {
  let paths = discover_dag_files(root: root)
  map(paths, p => {
    let content = Filesystem.read(path: p)
    SourceFile { path: p, content: content.content }
  })
}

// Stage 2: Parse
func parse_all(sources: List<SourceFile>) -> List<Module> {
  map(sources, s => parse(source: s.content))
}

// Stage 3: Resolve imports and build module graph
func resolve_modules(modules: List<Module>) -> ModuleGraph {
  let by_name = index_by(modules, m => m.name)
  resolve_imports(modules: modules, index: by_name)
}

// Stage 4: Typecheck
func typecheck(graph: ModuleGraph) -> TypedGraph {
  let types = collect_types(graph: graph)
  let validated = validate_types(types: types, graph: graph)
  TypedGraph { modules: graph.modules, types: validated }
}

// Stage 5: Lower to DAG
func lower(typed: TypedGraph) -> DAG {
  let nodes = flat_map(typed.modules, m =>
    flat_map(m.items, item => lower_item(item: item, types: typed.types))
  )
  let edges = derive_edges(nodes: nodes)
  DAG { nodes: nodes, edges: edges }
}

// Stage 6: Emit output files
func emit(dag: DAG, backend: Backend) -> List<TextFile> {
  map(dag.nodes, n => emit_node(node: n, backend: backend))
}

// The full pipeline
func compile(root: FilePath, backend: Backend) -> List<TextFile> {
  read_sources(root: root)
    |> parse_all()
    |> resolve_modules()
    |> typecheck()
    |> lower()
    |> emit(backend: backend)
}
```

Each stage is a pure function. The only I/O is `Filesystem.read` in
stage 1. The executor runs this DAG the same way it runs any user DAG.

## What the Rust kernel provides

The Rust kernel is the minimal runtime that executes DAGs. It provides:

1. **DAG executor**: topological sort, value routing, node dispatch
2. **Value type**: `Str`, `Int`, `Bool`, `Map`, `List`, `Unit`, etc.
3. **Transport adapters**: Shell, File, REST (actual I/O)
4. **Intrinsic operations**: string manipulation, arithmetic, comparison,
   collection operations (map, filter, fold)
5. **DAG loader**: reads the bootstrap compiler's serialized DAG

Everything else — parsing, typechecking, lowering, emitting, test
generation, mock generation — is expressed in .dag and executed by
this kernel.

## Bootstrap path

1. **v0 (current Rust compiler)** compiles the v1 .dag compiler source
   into a serialized DAG
2. **v1 .dag compiler** (executed by the Rust kernel) can compile .dag
   source files into serialized DAGs
3. **v1 compiles itself**: the .dag compiler source is compiled by v1,
   producing v2. v1 output == v2 output (fixed point)

The Rust kernel never changes once v1 is self-hosting. Language
evolution happens entirely in .dag files.

## Models to get right on day 1

These are the types that flow between stages. Getting them wrong means
v2 inherits v1's sustainability problems.

### Source representation

```dag
type Span { start: Int, end: Int }
type Token { kind: TokenKind, text: String, span: Span }

type TokenKind
  = Ident
  | Keyword { which: KeywordKind }
  | Literal { which: LiteralKind }
  | Punct { which: PunctKind }
  | Newline
  | Indent
  | Dedent
  | Eof

type KeywordKind
  = Type | Func | Fn | Let | Match | If | Else | Return
  | Import | Module | Service | Resource | Data | Extern
```

### AST

```dag
type Module {
  name: String
  imports: List<Import>
  items: List<Item>
}

type Import { module: String, names: List<String> }

type Item
  = TypeDef { name: String, body: TypeBody }
  | FuncDef { name: String, params: List<Param>, return_type: TypeExpr, body: Expr }
  | ServiceDef { name: String, transport: TransportBinding, operations: List<OperationDef> }
  | ResourceDef { name: String, capabilities: List<CapabilityDef> }
  | DataDef { name: String, type_expr: TypeExpr, value: Expr }

type TypeBody
  = Record { fields: List<Field> }
  | Sum { variants: List<Variant> }
  | Alias { base: TypeExpr, predicates: List<Predicate> }

type TypeExpr
  = Named { name: String }
  | Generic { name: String, args: List<TypeExpr> }
  | Optional { inner: TypeExpr }
  | Inline { fields: List<Field> }

type Field { name: String, type_expr: TypeExpr, optional: Bool }
type Variant { name: String, fields: List<Field> }
type Param { name: String, type_expr: TypeExpr }
```

### Expressions

```dag
type Expr
  = Literal { value: LiteralValue }
  | Var { name: String }
  | FieldAccess { base: Expr, field: String }
  | Call { func: String, args: List<NamedArg> }
  | Match { scrutinee: Expr, arms: List<MatchArm> }
  | If { condition: Expr, then_branch: Expr, else_branch: Expr? }
  | Let { name: String, value: Expr, body: Expr }
  | Record { fields: List<FieldInit> }
  | ListLit { elements: List<Expr> }
  | BinOp { op: BinOpKind, left: Expr, right: Expr }
  | Lambda { params: List<String>, body: Expr }

type NamedArg { name: String?, value: Expr }
type MatchArm { pattern: Pattern, body: Expr }
type FieldInit { name: String, value: Expr }

type Pattern
  = Bind { name: String }
  | Literal { value: LiteralValue }
  | Variant { name: String, bindings: List<String> }
  | Wildcard

type LiteralValue
  = Str { value: String }
  | Int { value: Int }
  | Bool { value: Bool }
  | Null
```

### The DAG IR

This is the key model — the intermediate representation between
stages. Types are node references, not strings.

```dag
type Port {
  name: String
  type_node: NodeRef       // reference to a type node in this DAG
}

type Node {
  id: String
  inputs: List<Port>
  outputs: List<Port>
  body: NodeBody
  metadata: Map<String, String>   // transport class, operation key, etc.
}

type NodeBody
  = Pure { expr: Expr }           // evaluable expression
  | Transport { binding: TransportBinding }  // I/O boundary
  | SubDag { dag: DAG }           // nested subgraph (fn body, loop, branch)
  | TypeDef { body: TypeBody }    // type definition (types are nodes)

type Edge {
  from_node: String
  from_port: String
  to_node: String
  to_port: String
}

type DAG {
  nodes: List<Node>
  edges: List<Edge>
}

// NodeRef is just a node ID within the same DAG.
// No external registry. No string-to-structure lookup.
// The type IS the node.
type NodeRef = String
```

### Compile output

```dag
type CompileOutput {
  dag: DAG                        // the fully resolved graph
  output_files: List<TextFile>    // generated source files
  diagnostics: List<Diagnostic>   // warnings/errors
}

type TextFile {
  path: FilePath
  content: String
}

type Diagnostic {
  severity: Severity
  message: String
  span: Span?
  module: String?
}

type Severity = Error | Warning | Info
```

## Sustainability invariants enforced by design

| v1 problem | v2 design |
|---|---|
| TypeId is a string → needs registry | Types are DAG nodes → NodeRef is structural |
| Cardinality cached on port | Derived from type node structure |
| PortMultiplicity vs Cardinality | One concept: type node determines merge |
| Parallel fn body evaluator | Fn bodies ARE subgraph DAGs, one executor |
| `mock_element_expr` enumeration | Mock generation walks type node structure |
| `register_core_types()` duplication | Types defined in .dag only |
| String-based classification | Pattern match on typed AST nodes |
| No boundary contracts | Each stage is a typed function: input/output types enforce contracts |

## What to build first

The bootstrap order, prioritized by sustainability payoff:

### Phase 1: Parser in .dag

The parser is pure (string → AST) and exercises the core language
features: pattern matching, recursion, string manipulation, sum types.
Writing the parser in .dag proves the language is expressive enough
for real programs.

**Input:** `String` (source text)
**Output:** `Module` (AST)
**Tests:** parse existing .dag files, compare AST to v0 parser output

### Phase 2: Type resolver in .dag

Type resolution walks the AST and builds the type graph. This is where
TypeId gets eliminated — the resolver produces DAG nodes for types
instead of registering strings.

**Input:** `List<Module>` (parsed AST)
**Output:** `TypedGraph` (AST + resolved type nodes)
**Tests:** resolve types in existing .dag files, verify structure

### Phase 3: Lowerer in .dag

The lowerer transforms typed AST into DAG IR. This is the largest
stage but also the most mechanical — each AST item maps to a small
subgraph.

**Input:** `TypedGraph`
**Output:** `DAG`
**Tests:** lower existing .dag files, compare to v0 lowerer output

### Phase 4: Emitter in .dag

The emitter transforms DAG IR into output files. Language models
become .dag data declarations. Emit templates become .dag functions.

**Input:** `DAG` + `Backend`
**Output:** `List<TextFile>`
**Tests:** emit from existing DAGs, compare to v0 emitter output

### Phase 5: Self-compile

Run phases 1–4 on the compiler's own source. Verify the output
matches v0's output. Fixed point = self-hosting.

## Open questions

1. **Error recovery in parser.** The v0 parser has error recovery
   (continues after syntax errors to report multiple diagnostics).
   Can .dag express this pattern, or does the executor need to
   support partial failure?

2. **Performance.** The v0 Rust compiler is fast because Rust is fast.
   A .dag compiler executed by a Rust DAG kernel adds interpretation
   overhead. Is this acceptable for a compiler that runs at build time?

3. **Debugging.** When the compiler has a bug, debugging a DAG
   execution trace is harder than debugging Rust with a debugger.
   What tooling does the kernel need to provide?

4. **Incremental compilation.** The v0 compiler recompiles everything.
   The DAG structure naturally supports incremental computation
   (re-execute only changed subgraphs). Should v2 support this from
   day 1?
