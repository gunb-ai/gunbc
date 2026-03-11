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

The emitter is pluggable — it takes a `TypedGraph` and a `Backend`
and produces files. Different backends produce different languages:

```dag
type Backend = Rust | Python | JsonIR

func compile(root: FilePath, backend: Backend) -> List<TextFile> {
  let sources = read_sources(root: root)
  let tokens = map(sources, s => tokenize(source: s.content))
  let modules = map(tokens, t => parse(tokens: t))
  let graph = resolve_modules(modules: modules)
  let typed = typecheck(graph: graph)
  emit(typed: typed, backend: backend)
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

### JSON IR (optional, for external tooling)

A structured JSON representation of the typed program. Not
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
| Parallel fn body evaluator + DAG executor | Compiler emits files, doesn't interpret |
| `mock_element_expr` enumeration | Test generation walks TypeExpr structure |
| `register_core_types()` duplication | Types defined in .dag only |
| String-based classification | Pattern match on typed AST |
| No boundary contracts | Each stage is a typed function with TypeExpr contracts |
| Resolver re-resolves at runtime | All resolution at compile time, in the compiler |

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
