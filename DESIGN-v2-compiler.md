# v2: Self-hosted compiler design

Canonical docs:
- `src/v2/WORKBOARD.md` for current status and work queue
- `src/v2/DESIGN.md` for the canonical v2 design

This file is kept for continuity with older references. New design updates
should go to `src/v2/DESIGN.md` rather than being duplicated here.

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

## Core principle: DAG nodes are facts, composition is conjunction

A DAG node is a **fact** — a tautological assertion about a computation.
"This operation takes a String and produces an Int." "This operation is
idempotent." "This operation has cardinality [0,∞)." Each fact is true
independent of any target language or runtime.

**Composition is conjunction.** When a DAG contains multiple nodes
connected by edges, the DAG asserts all of their facts simultaneously.
The edges assert data flow relationships between the facts.

**SubDag encapsulation is abstraction.** When a node contains an inner
DAG (`NodeBody::SubDag`), it does two things:
1. **Stacks facts** — the outer node adds properties (cardinality,
   idempotency, resource requirements) on top of the inner DAG's facts.
2. **Hides complexity** — consumers see only the outer node's ports,
   not the inner structure. The inner facts are still true; they're
   just not visible from outside.

This is not progressive lowering (where you lose information at each
step). It's **abstraction by composition** — you never lose facts, you
stack them. The inner DAG's truths are preserved inside the encapsulation.

**Rendering reads facts, never changes them.** A backend doesn't
transform the DAG — it reads the accumulated facts and renders them in
its idiom. A Rust renderer reads "this type is Optional" and emits
`Option<T>`. A C renderer reads the same fact and emits `T*` with NULL.
A Verilog renderer reads it and emits a valid bit + data bus. The facts
don't change; the rendering does.

This creates a clean **separation between computation facts and rendering
facts**:
- **Computation facts** live in the DAG: types, cardinality, data flow,
  operation semantics. These are target-agnostic truths.
- **Rendering facts** live in the backend: how to express Optional, how
  to handle ownership, how to name types. These are target-specific
  decisions.

The structural test for correct architecture: **can you swap rendering
facts without changing computation facts?** If yes, the IR is
target-agnostic. If you find a rendering fact mixed into the computation
layer, you've found a design bug.

This is why backends should themselves be compositional .dag models —
rendering strategies are facts about how to map computation concepts to
target idioms, expressible in the same compositional vocabulary.

### Comparison with LLVM IR and MLIR

**LLVM IR** is a single-level, fixed-type SSA representation. It's the
contract between language frontends and hardware backends. LLVM's design
is clean — the IR is target-agnostic, and all target knowledge lives in
the backend passes. But LLVM only handles one abstraction level (close
to machine code), and its type system is fixed (i32, float, ptr, struct).

**MLIR** extends LLVM with multi-level IR via dialects. Each dialect
defines operations and types at a specific abstraction level (TensorFlow
ops, linalg, affine loops, LLVM). Interfaces allow transformations to
work across dialects polymorphically. MLIR solves "how do I lower
through many abstraction levels?" via progressive dialect conversion.

**gunbc's DAG IR** solves a different problem. It doesn't lower through
abstraction levels — it composes facts and then renders them. The IR
is not a staging area for optimization passes. It's a **compositional
knowledge graph** where each node asserts truths about computation,
and backends read those truths to produce target-specific output.

| | LLVM | MLIR | gunbc |
|---|---|---|---|
| Model | Lower to machine code | Lower through abstraction levels | Compose facts, render to any target |
| Extension | Fixed instruction set | Dialect system + interfaces | `Dag<T>` generic over closed op enum + SubDag composition |
| Types | Fixed (i32, ptr, struct) | Extensible per-dialect | Structural `TypeExpr` values (products, sums, containers) |
| Target knowledge | Backend passes | Dialect conversion | Rendering .dag models |
| Information flow | Lossy (lowering drops info) | Lossy (progressive) | Lossless (facts compose, never lost) |
| World I/O | Annotated (readonly, writeonly) | Interface (MemoryEffects) | Structural (graph topology) |
| Cardinality | None | None | First-class [min, max] intervals |

Key properties that gunbc preserves and must not lose:
- **World I/O is structural** — detected by graph inspection, no annotations
- **Cardinality is first-class** — ports carry multiplicity, not metadata
- **Types are compositional values** — products, sums, containers stack like
  protobuf messages and map to any target (Rust structs, C structs, Verilog
  wire bundles, PSPICE subcircuit interfaces)

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

## Gap analysis: v1→v2 compilation path (2026-03-13)

Status assessment of using the v1 codegen pipeline (`daglang compile`)
to compile the v2 `.dag` compiler sources into a native Rust crate.

### What works today

| Component | Status |
|---|---|
| Module discovery | `daglang.toml` multi-root config can discover all 7 v2 modules (tokenize, parse, core, pipeline, resolve, typecheck, emit) |
| Parsing | All v2 .dag files parse to AST successfully |
| Type codegen | `typedef_to_code_ir` handles structs, enums, type aliases |
| Fn codegen | Records, match, if/else, for, lambda, string interpolation, intrinsics (map/filter/fold/any/contains/sum/join/last/enumerate) |
| Rust rendering | `render_rust` produces valid Rust from CodeIR |
| Anonymous records (S72) | Fixed — three-tier inference: exact struct match → best-match by field count → synthesized helper struct |

### Blockers

**1. `build_context` ignores `daglang.toml` (trivial)**

`build_context()` in `compile/context.rs` hardcodes `dsl/` as the only
source root. The v2 modules live under `src/v2/`. Fix: call
`resolve_default_roots()` which already reads `daglang.toml` multi-root
config. One-line change.

**2. Typechecker rejects v2 idioms (12 errors)**

The v1 typechecker (TC003/TC021) rejects patterns used pervasively in
v2 source:

- **TC003 ×7**: bare enum variant used where parent type expected.
  v2 writes `UnterminatedString` where the typechecker expects
  `StringScanResult`, `Some` where it expects `Option<T>`. The fix:
  accept variant-as-type in assignment/return context by looking up the
  variant's parent enum.
- **TC003 ×2**: `Record { ... }` literal where typechecker expects `Map`.
  v2 uses record literals for constructing typed output; the typechecker
  classifies them as Map.
- **TC021 ×3**: named arguments not recognized. v2 uses `ch:` in
  `code_point(ch: c)` etc. Fix: register named parameters in the
  callable signature registry.

**3. Missing fn_codegen constructs**

| Construct | Usage in v2 | Required fix |
|---|---|---|
| `with(state, { field: value })` | Immutable record update, ~100 call sites for state threading | Add `with` as intrinsic → emit struct update expression |
| `char_at()`, `substring()`, `string_length()` | String processing in tokenizer | Add string builtin → Rust method mappings |
| `lookup()` | Map/record field access by name | Map to `.get()` or field access |
| Recursive enum variants | `Expr` contains `Expr` via sum variants | Detect type cycles in codegen, insert `Box<>` |
| Optional fields (`T?` → `Option<T>`) | Throughout core.dag type definitions | Map `?` suffix to `Option<>` wrapper in type codegen |
| Optional match patterns | `Some { value: x }` destructuring | Emit `Some(x)` match arms |

**4. Crate assembly harness**

No mechanism to emit a complete Cargo crate from compiled modules:
- `Cargo.toml` with correct dependencies
- `lib.rs` with `mod` declarations for each compiled module
- Per-module `.rs` files from codegen output
- `main.rs` driver (or integration as library)
- v2-specific stdlib shims (string ops, list ops, Option helpers
  that v2 .dag code calls but have no Rust equivalent today)

### Recommended sequence (next PR)

```
1. build_context reads daglang.toml          ─── trivial, unblocks everything
2. Fix 12 typecheck errors                   ─── variant-as-type, named args
3. Add with() + string builtins to fn_codegen ─── most-used missing constructs
4. Recursive type detection → Box<>          ─── required for Expr/TypeExpr
5. Crate assembly harness                    ─── emit compilable Cargo project
6. Iterative: generate crate → cargo build → fix errors → repeat
```

Steps 1-2 unblock end-to-end pipeline testing. Steps 3-4 are the bulk
of the codegen work. Step 5 creates the integration harness. Step 6 is
the convergence loop where remaining gaps surface as Rust compile errors.

### Relationship to self-hosting milestones

This gap analysis covers **Phase 1** of the self-hosting path (section 6
above): emit per-module Rust + driver → first native binary. Once Phase 1
produces a compiling crate, the milestones proceed:

- Phase 2: v2 compiles tokenize.dag → core.dag → pipeline.dag
  (progressive self-compilation)
- Phase 3: v2 output matches v1 output for all modules (fixed point
  verification = self-hosting achieved)

---

## Evaluator architecture (as of 2026-03-12)

The v2 compiler's ~80 mutually recursive functions are evaluated by an
explicit-stack machine (`eval_stack.rs`), not native Rust recursion.
The old recursive evaluator is deleted.

**Stack machine properties:**
- Heap continuations: deep mutual recursion uses O(1) native stack
- Self-contained: builtins, intrinsics, match evaluation all inline
- ANF contract: sibling fn calls appear only at statement level
- Limits: MAX_STACK_DEPTH=100K, MAX_TRANSITIONS=10M

**Accepted evaluator design decisions (see DESIGN-eval-redesign.md):**
- `wrap_value_as_output` flattens Map trailing expressions into the
  output HashMap. A function ending with `Some { value: x }` produces
  output `{"_variant": "Some", "value": x}` (flattened), not
  `{"return": Map({"_variant": "Some", "value": x})}`. Callers rely on
  `extract_projection` reconstructing the Map. The `"value"` fallback
  is structurally necessary (S67-4).
- `LoweredExpr::Return` removed — return is now statement-only.
  `Block` and `For` remain as expression forms because they genuinely
  return values (accepted permanent, S67-2).
- Debug-mode OOM: evaluating 11 real .dag files (gist pipeline) exceeds
  16GB heap. Not a stack issue — interpreter overhead from cloning
  Values through deep call chains. Release mode or self-hosting would
  resolve this.
