# v2 Compiler Design Principles

These are the governing design decisions for the self-hosted compiler.
For current status, phases, and workboards, see `ROADMAP.md`.
For invariants, see `INVARIANTS.md`.

## Core claim

The compiler is a pure transform: `.dag source → backend artifacts`.
Each stage is a pure function, but the middle is a staged enrichment
pipeline over one carrier rather than a hard AST→IR split:

```
.dag source
→ tokenize
→ parse (syntax-faithful Node carrier)
→ resolve / normalize
→ typecheck / reconcile (ResolvedGraph, TypedModule)
→ emitter fact precompute (EmitGraphInfo)
→ per-language renderers
→ output files
```

No interpreter in the pipeline. Interpretation is a downstream concern.

## The carrier is explicit; GraphIR is currently implicit

There is no standalone executable GraphIR in v2 today. The compiler uses
one structural carrier, `Node`, across parse, resolve, infer, and emit.
Semantic stages enrich that carrier with additional facts:

- Parse returns `ParseResult { module: Node?, error: ErrorNode? }`.
- Inline `operation` / `capability` signatures preserve authored return
  annotations on the `Node`; only block `output { ... }` forms lower
  through output-field shape during parse.
- Resolve / normalize lift module collections into `ModuleGraph`.
- Typecheck / reconcile produce `TypedModule` and `ResolvedGraph`.
- Emit consumes `ResolvedGraph` plus `EmitGraphInfo`, a precomputed fact
  bundle for value context, type summaries, recursive sets, and other
  backend-facing summaries.

That means the honest architecture story is:

```
frontend syntax
→ unified Node carrier
→ graph enrichment
→ emitter facts
→ renderers
```

The long-term direction is still to sharpen the backend contract so
renderers consume lowered graph facts, not parser-era recovery or
source-text heuristics.

## Types are values, not references

Types are structural values that flow through the pipeline. No registry,
no deferred lookup. The structure IS the type. A `List<Span>` is
structurally represented, not stored as a string `"List<Span>"`.

## The compiler is just another .dag program

The v2 compiler imports from `std.types` — the same shared type library
that all DSL programs use. The compiler's own invariants are enforced by
the compiler itself.

## fn vs func

- `fn` = pure function (expression body, no side effects)
- `func` = workflow function (may use resources, services, transport)

## Testing: the compiler owns its downstream

When the compiler emits Rust, it also emits Rust tests. The tests
compile and run in the target language. If the compiler produces broken
output, `cargo test` on the emitted tests catches it.

## Diagnostics are first-class

Every stage returns `{ value, diagnostics }`. Diagnostics thread through
the pipeline. No diagnostic loss across stage boundaries.

## Value context: how a value is used determines its representation

The emitter must know HOW a value is used, not just WHAT it is.
A `Map<String, String>` that is a constant lookup table, a runtime
data structure, and an algebra witness are the same .dag type but
need different target-language representations.

**ValueContext** is a structural fact precomputed in EmitGraphInfo:

| Context | Meaning | Classification |
|---------|---------|---------------|
| `ConstantData` | Immutable, known at compile time | `data` decl with literal/static body |
| `RuntimeValue` | Heap-allocated, shared at runtime | `let` binding, function return, field |
| `SpecificationWitness` | Structural property, not runtime data | Algebra `fn`-typed fields |
| `CallableValue` | Function type, invokable | `fn`-typed params and fields |

Each target language maps ValueContext to representation:

- **Rust:** ConstantData → `const`/`static`; RuntimeValue → `Rc<T>`;
  SpecWitness → phantom type or omitted; CallableValue → `fn(T)->U`
- **Python:** ConstantData → module-level; RuntimeValue → `T` (GC);
  CallableValue → `Callable`
- **Go:** ConstantData → `var` (package); RuntimeValue → `*T` or value;
  CallableValue → `func`
- **SPICE:** ConstantData → `.param`; RuntimeValue → wire/node
- **English:** ConstantData → table; RuntimeValue → paragraph

**Why this matters:** Without ValueContext, the emitter applies one
strategy everywhere (`Rc<T>` in Rust). This produces:
- E0277: `Rc` in `lazy_static` (needs `Sync`, `Rc` is `!Sync`)
- E0369: `Rc<dyn Fn>` equality (algebra witnesses aren't comparable)
- Per-language bugs that multiply with each new target

**Extension point:** `TypedItemKind` and `TypeSummary` already exist.
ValueContext is computed from item kind + field types + usage analysis
and added to `EmitGraphInfo` in the same precomputation pass.
