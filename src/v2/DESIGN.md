> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > [docs/architecture.md](../../docs/architecture.md)

# v2 Compiler Design Principles

These are the governing design decisions for the self-hosted compiler.
For current status, phases, and workboards, see [ROADMAP.md](../../ROADMAP.md).
For invariants, see [INVARIANTS.md](../../INVARIANTS.md).

## Core claim

The compiler is a pure transform: `.dag source → backend artifacts`.
Five stages, each a pure function:

```
.dag source → tokenize → parse → resolve → typecheck → emit → output files
```

No interpreter in the pipeline. Interpretation is a downstream concern.

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
