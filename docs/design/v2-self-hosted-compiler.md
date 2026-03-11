# v2: Self-hosted compiler design

## Current Status (2026-03-11)

**48/48 v2 compiler tests passing. 0 clippy warnings. Full workspace green.**

Completed work:
- **Tokenizer**: PipeArrow token added (Unit 1)
- **Parser**: NullCoalesce infix, PipeArrow desugaring, `as` cast postfix,
  KwPattern/KwInterface dispatch, `where` refinement clauses,
  response/mock_response block parsing (Unit 2)
- **Typechecker**: Mutual recursion cycle detection via `resolving` param (Unit 3)
- **Emitter**: NullCoalesce (unwrap_or_else), for-loop (into_iter/map/collect),
  pipe methods (count→len, join, split, last, first, enumerate, chars),
  Cargo.toml emission, emit_first_arg helper (Unit 4)
- **Pipeline**: Resolver diagnostic threading verified correct (Unit 5)
- **Evaluator**: List concat bug fixed in eval_binop (Unit 6)
- **Bootstrap**: Driver crate at src/v2/bootstrap/ with placeholder modules (Unit 7)
- **Tests**: 14 new phase4 integration tests (Unit 8)
- **Core types**: NullCoalesce added to BinOpKind

Deferred: `provides` clause, `from "key"` extraction, Option normalization,
Phase 1c native bootstrap, Phase 2 self-compilation, Phase 3 fixed point.

## Premise

The v0/v1 compiler exists to eliminate glue bugs in downstream systems,
but the compiler itself is full of the same glue bugs: string-matching,
fabrication fallbacks, parallel representations, deferred computation.
The sustainability ledger (SUSTAINABILITY.md) traces 38 symptoms to one
deep root: **incomplete compile-time resolution.**

The fix is not to patch each symptom. The fix is to write the compiler
in its own language, so the compiler's invariants are enforced by the
compiler itself.

The compiler is a pure transform: `.dag files → generated files`. It
reads source, analyzes it, and emits target-language source code. There
is no interpreter in the compiler. Interpretation is a downstream
concern — a consumer of the compiler's output, not part of the compiler
itself.

```
.dag source → [compiler] → emitted source files → [target runtime]
```

The compiler doesn't know or care how the emitted code gets run. If
the target is Rust, `cargo` runs it. If the target is Python, `python`
runs it. If the target is a JSON IR, a separate interpreter runs it.
The compiler's job ends at file emission.

## What went wrong with v0/v1

The core mistake: **TypeId is a string.** A port says
`type_id: TypeId("ResourceHandle")` instead of embedding the type
structure. This forces a TypeRegistry (symbol table) to exist at
runtime, forces threading the registry to every consumer, and when
threading fails, forces fabrication fallbacks.

But TypeId is a symptom of a deeper mistake: **treating types as a
separate concern from the program.** Types are defined in .dag files,
compiled into `Dag<TypeOp>`, stored in a registry, and then referenced
by string. But types ARE data. A product type is a record with field
descriptions. A coproduct is an enum. A refined type is a base type
with constraints. There's no reason to store them in a separate lookup
table.

The second mistake: **coupling the compiler to an interpreter.** v1's
pipeline goes `.dag → AST → typed AST → DAG IR → DynOp → execute`.
The last two stages (resolve to DynOp, execute) are an interpreter
embedded in the compiler. This created the parallel implementation
problem: fn bodies had both an AST evaluator AND a DAG executor path,
and they diverged.

The fix: the compiler emits files. Period. If you want to run the
program, run the emitted files with the appropriate runtime. The
compiler and the runtime are separate programs with a file boundary
between them.

## Core principle: types are values, not references

In v2, types are structural values that flow through the pipeline
like any other data. A `TypeExpr` is a sum type describing shape:

```dag
type TypeExpr
  = Named { name: String }
  | Product { name: String?, fields: List<Field> }
  | Coproduct { name: String?, variants: List<Variant> }
  | Container { kind: ContainerKind, element: TypeExpr }
  | Refined { base: TypeExpr, predicates: List<Predicate> }
  | Optional { inner: TypeExpr }
  | MapType { key: TypeExpr, value: TypeExpr }
```

When a function parameter has type `List<Span>`, the compiler doesn't
store the string `"List<Span>"` and look it up later. It stores:

```
Container { kind: List, element: Product { name: "Span", fields: [...] } }
```

The structure IS the type. No registry. No deferred lookup. No stale
copies.

## Architecture

```
.dag source files
    │
    ▼
┌─────────┐
│ Tokenize │  String → List<Token>
└────┬──────┘
     ▼
┌─────────┐
│  Parse   │  List<Token> → Module (AST)
└────┬──────┘
     ▼
┌─────────┐
│ Resolve  │  List<Module> → ModuleGraph (imports resolved)
└────┬──────┘
     ▼
┌──────────┐
│Typecheck  │  ModuleGraph → TypedGraph (types resolved to TypeExpr)
└────┬──────┘
     ▼
┌─────────┐
│  Emit    │  TypedGraph → List<TextFile>
└────┬──────┘
     ▼
  output files (.rs, .py, .json, ...)
```

Five stages, each a pure function. The compiler goes from typed AST
directly to output files.

### Bootstrap subset vs full language

The v2 prototype targets the **gist.dag bootstrap subset**: the 8
Item variants actually used by gist.dag and its transitive
dependencies (`module`, `import`, `type`, `fn`, `func`, `service`,
`resource`, `data`). The current v1 language has 20 Item variants.
The remaining 12 are added incrementally after bootstrap proves the
architecture works.

This is an explicit scope boundary: the prototype does NOT claim to
be a full-language replacement. It is a bootstrap compiler for a
well-defined subset, designed to grow into a full compiler.

The emitter is pluggable — it takes a `TypedGraph` and a `Backend`
and produces files. Different backends produce different languages.
The public contract is a single function:

```dag
type Backend = Rust | Python

func compile(root: FilePath, backend: Backend) -> CompileResult {
  let sources = discover_and_read(root: root)
  let tokenized = map(sources, s => tokenize(source: s.content))
  let parsed = map(tokenized, t => parse(tokens: t))
  let graph = resolve_modules(modules: parsed)
  let typed = typecheck(graph: graph)
  let files = match backend {
    Rust => emit_rust(typed: typed)
    Python => emit_python(typed: typed)
  }
  CompileResult { files: files, diagnostics: typed.diagnostics }
}

type CompileResult {
  files: List<TextFile>
  diagnostics: List<Diagnostic>
}
```

## Testing: the compiler owns its downstream

v1's critical gap: emitted code is text-checked ("does the string
contain this substring?") but never compiled or run. The Go/C/MIPS
emitters could produce broken output and no test would catch it.

v2 fixes this by making testing a compiler responsibility. When the
compiler emits Rust, it also emits Rust tests for that code. The
tests compile and run in the target language, not in the compiler's
language. The emitted test verifies the emitted program works.

```
.dag source → compiler → {program.rs, program_test.rs}
                              │              │
                              ▼              ▼
                          cargo build    cargo test
                              │              │
                              ▼              ▼
                          binary        PASS/FAIL
```

The test and the code are emitted together, in the same language,
as one unit. If the compiler produces broken Rust, `cargo test` on
the emitted tests catches it immediately.

This replaces two v1 concepts:
- **testgen** (generated tests against the DAG IR) → tests against
  the emitted code instead
- **emit string-checking** (assert output contains substrings) →
  assert emitted code compiles and its tests pass

## Emit targets

### Rust (primary)

The v1 compiler already emits Rust. v2 continues this — same output
format, different (simpler) compiler. The acceptance test is: v2's
Rust output for gist.dag compiles with `cargo build` AND its emitted
tests pass with `cargo test`.

### Python (bootstrap accelerator)

Python emission is useful during bootstrap because there's no
compilation step — the emitted code runs immediately. This makes
the development loop faster: edit .dag → run compiler → run emitted
.py → see result.

### JSON (optional, for external tooling)

A structured JSON serialization of the typed AST. Not
executable by itself — requires a separate runtime. Useful for
editor tooling, visualization, and language server protocol.

## Bootstrap path

1. **v1 Rust compiler** compiles v2's .dag source files and executes
   them (v1 is the host for v2 during development)
2. **v2 compiler** (running on v1) reads .dag files and emits target
   source code
3. **v2 compiles gist.dag** → emitted files match v1's output
4. **v2 compiles itself** → the .dag compiler source is compiled by
   v2, producing the same output. Fixed point = self-hosting.

The v1 Rust code remains as the bootstrap host. It doesn't change
after v2 reaches self-hosting. Language evolution happens entirely
in .dag files.

## Sustainability invariants enforced by design

| v1 problem | v2 design |
|---|---|
| TypeId is a string → needs registry | Types are TypeExpr values → no registry |
| Cardinality cached on port | Derived from TypeExpr structure |
| Parallel fn body evaluator + DAG executor | Compiler emits files, no interpreter |
| `mock_element_expr` enumeration | Emitter walks TypeExpr structure |
| `register_core_types()` duplication | Types defined in .dag only |
| String-based classification | Pattern match on typed AST |
| No boundary contracts | Each stage is a typed function, CompileResult is files + diagnostics |
| Transport config as `Map<String, String>` | Typed coproduct per transport kind |
| Emitted code never tested downstream | Compiler emits tests alongside code |

## Dependency chain

```
kernel primitives (String, Int, Bool, List, Map)
  ← dsl/std/types.dag (FilePath, NonEmptyStr, SourceSpan, ...)
    ← src/v2/std/core.dag (Token, AST, Expr, TypeExpr)
      ← src/v2/compiler/*.dag (tokenize, parse, resolve, typecheck, emit)
```

The v2 compiler imports from `std.types` — the same shared type
library that all DSL programs use. The compiler is just another DSL
program.

## Open questions

1. **Primary emit target for bootstrap.** Rust matches v1 output for
   testing. Python is faster to iterate on. Could start with Python
   for development speed and add Rust for parity testing.

2. **Error recovery in parser.** The v1 parser continues after syntax
   errors to report multiple diagnostics. Can .dag express this, or
   does the parser return on first error?

3. **Self-hosting timeline.** Compiling gist.dag proves the compiler
   works. Compiling itself proves it's complete. The gap between
   these depends on how many language features the compiler's own
   .dag source uses beyond what gist.dag exercises.

## Decisions (resolved from review feedback)

### Error recovery: first-error-halt for bootstrap

The parser halts on first error for bootstrap. Multi-error recovery
requires a synchronization-token strategy designed into the grammar
rules from the start, which adds significant complexity. The
bootstrap compiler is a batch compiler — first-error-halt is
sufficient. Error recovery nodes (`ErrorItem`, `ErrorExpr`) will be
added later for LSP tooling.

### Python backend: deferred

Python emission is deferred until Rust self-hosting is proven. Adding
a second backend before the first works recreates the parallel
implementation problem (S36, Branch 2). One backend, fully tested.

### fn vs func

`fn` is a pure function (expression body, no side effects).
`func` is a workflow function (may use resources, services, transport).
The distinction maps to v1's `fn` (evaluated by expression evaluator)
vs `func` (compiled to DAG with transport nodes). Both appear in the
AST because the parser needs to distinguish them; the emitter uses
the distinction to decide whether to emit a pure function or a
service-wired workflow.

### Diagnostics are first-class in every stage

Every stage returns `StageResult<T> { value: T, diagnostics: List<Diagnostic> }`,
not raw values. This prevents diagnostic loss across stage boundaries.
The pipeline threads diagnostics:

```dag
type StageResult<T> { value: T, diagnostics: List<Diagnostic> }

func compile_sources(sources: List<SourceFile>, backend: Backend) -> CompileResult {
  let tokenized = map(sources, s => tokenize(source: s.content))
  let parsed = collect_results(map(tokenized, t => parse(tokens: t.value)))
  let resolved = resolve_modules(modules: parsed.value)
  let typed = typecheck(graph: resolved.value)
  let files = match backend { Rust => emit_rust(typed: typed.value) }
  CompileResult {
    files: files,
    diagnostics: parsed.diagnostics
      + resolved.diagnostics
      + typed.diagnostics
  }
}
```

### Acceptance test levels clarified

Level 3 (diff v1 vs v2) compares only the primary output files,
excluding emitted test files (which are v2-only). The comparison
is normalized: sorted keys, stripped formatting whitespace.

### Bootstrap circularity risk

v1's known fail-open behaviors (S3 fn-body passthrough, S23/S35
unknown-type-to-Json) could silently pass through expressions that
should fail during v2's bootstrap compilation. Mitigation: v2's
invariant tests (the spec-level column in the test matrix) explicitly
check for these — e.g., "no unresolved Named after typecheck" catches
cases where v1's type resolution silently fabricated.

### Nominal identity in the typed graph

Full structural substitution (replacing every `Named` with its
structural expansion) loses nominal identity, alias boundaries,
and generic instantiation information. The typed graph should
retain a **resolved nominal anchor**: the original type name plus
any type arguments, alongside the structural expansion.

After typecheck, a type reference like `List<Span>` becomes:
```
Container {
  kind: List,
  element: Product {
    name: Some("Span"),  // nominal anchor preserved
    fields: [...]         // structural expansion present
  }
}
```

The `name: Some("Span")` is NOT a lookup key — the structural
fields are authoritative. The name is metadata for diagnostics,
emission (generating `struct Span` instead of an anonymous type),
and equality (two structurally identical types with different names
ARE different types in the nominal sense).

This is different from v1's `TypeId("Span")` which was ONLY a name
with no structural information. In v2, the name is supplementary
to the structure, not a replacement for it.

### MapType key constraint

`MapType { key: TypeExpr, value: TypeExpr }` allows arbitrary key
types in the AST (pre-typecheck). The typechecker enforces String-key
constraint (S19). This is correct — the AST represents what the
programmer wrote; the typechecker rejects what's invalid.

## Review comments and resolutions (2026-03-11)

### 1. Bootstrap target larger than the v2 model

**Issue:** gist.dag depends on features not yet in core.dag: default
parameters (`public: Bool = false`), `uses` clauses, `as` casts,
and richer service blocks (`config`, `response`, `mock_response`).

**Resolution:** The AST model needs expansion. The following are
required for the gist.dag corpus:

- `Param.default_value: Expr?` — already present in Field, add to Param
- `uses` clause in func/service bodies — add `uses: List<ResourceUse>`
  to FuncDef/ServiceDef
- `as` cast expression — add `Cast { expr: Expr, target: TypeExpr }`
  to Expr
- Service `config` block — add `ServiceConfig` type to ServiceDef
- `response` / `mock_response` in operations — add to OperationDef
- Default parameter values in service operations (`Bool = false`) —
  Field already has `default_value: Expr?`

These will be added to core.dag as part of C3 (parser) work, since
the parser needs to understand them to parse the corpus.

### 2. Pure/effectful split in compile contract

**Issue:** `compile(root, backend)` mixes pure compilation with
filesystem discovery.

**Resolution:** The pipeline should split into:
- **Driver** (effectful): `discover_files(root) → List<SourceFile>`
  using `Filesystem.read` with `TextFilePath`
- **Compiler** (pure): `compile_sources(sources, backend) → CompileResult`
  taking already-read source text

The public entry point is the driver. The compiler core is pure and
testable without I/O. `pipeline.dag` should reflect this split.

### 3. Builtin type representation after typecheck

**Issue:** The invariant "no Named TypeExpr survives" is too strong.
Primitives (String, Int, Bool) have no structural form to resolve to.

**Resolution:** Add a `Primitive` variant to TypeExpr for kernel types:
```
type TypeExpr
  = Named { ... }           // unresolved reference (pre-typecheck)
  | Primitive { name: String }  // kernel type (String, Int, Bool, etc.)
  | Product { ... }         // resolved record
  | ...
```

After typecheck, the invariant is: **no `Named` references remain.
Every name has been resolved to either `Primitive` (kernel types),
a structural form (Product, Coproduct, etc.), or a cycle-breaking
`Named` reference to a type that IS defined in the module graph.**

### 4. Emitted test hermeticity

**Issue:** gist.dag is networked/authenticated. What do emitted tests
test?

**Resolution:** Emitted tests are hermetic by default. They use
`mock_response` data from the DSL source as test fixtures — no
network, no credentials. This matches v1's DryRun tier. Live
network tests are out of scope for bootstrap.

The emitter consumes `mock_response` blocks in the DSL source and
emits them as test fixtures in the target language:
```rust
// Emitted test (Rust)
#[test]
fn test_gist_create_dry_run() {
    let mock = json!({"id": "gist-abc123", ...}); // from mock_response
    let result = gist_create(mock_transport(201, mock));
    assert!(result.url.starts_with("https://"));
}
```

### 5. AST error recovery

**Issue:** The AST has no Error variants for partial parse results.
LSP tooling needs partial ASTs.

**Resolution:** Error recovery is deferred past bootstrap. The v2
parser fails on first error for bootstrap (sufficient for a batch
compiler). Error recovery nodes (`ErrorItem`, `ErrorExpr`) will be
added when LSP support is prioritized — they're additive to the AST
model, not a redesign.

### 6. Self-hosting gap and intermediate milestones

**Issue:** The gap between "compiles gist.dag" and "compiles itself"
is large. The compiler uses deep pattern matching, recursive data
structures, and stdlib dependencies.

**Resolution:** The project plan (v2-project-plan.md) already defines
per-stage milestones (C2 tokenizer → C3 parser → C4 resolver → C5
typechecker → C6 emitter). Self-hosting adds intermediate milestones
within Phase 5:

1. v2 compiles gist.dag (Phase 4 deliverable)
2. v2 compiles tokenize.dag (simpler — no service blocks, no resources)
3. v2 compiles core.dag (type definitions only — no fn bodies)
4. v2 compiles pipeline.dag (full compiler source)
5. v2 output matches v1 output for all of the above (fixed point)

Each milestone exercises progressively more language features.
The gap between 1 and 2 is small (tokenize.dag is simpler than
gist.dag). The gap between 2 and 4 is where the real complexity
lives (recursive TypeExpr, pattern matching in the parser).
