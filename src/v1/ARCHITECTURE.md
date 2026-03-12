# Target Architecture

**Status:** Draft — captures intent and rationale. Will evolve as the
refactor progresses. Current state is documented alongside target state
so we can track drift.

---

## Two products, one pipeline

The system produces two things from `.dag` source files:

1. **Compiled binaries** — native executables (Rust/Go/C) generated from
   the lowered IR. The compiler emits source code; a host compiler builds
   it. This is Branch A.

2. **Interpreted execution** — the lowered IR is executed directly by an
   interpreter. No code generation. This is Branch B.

Both branches share the compiler pipeline (parse → typecheck → lower).
They diverge at the artifact boundary: the compiler produces
`VerifiedDag<LoweredOp>`, and each branch consumes it differently.

```
.dag source
    ↓
COMPILER: parse → resolve imports → typecheck → lower
    ↓
VerifiedDag<LoweredOp>  ← THE ARTIFACT
    ↓                         ↓
Branch A: emit → cargo    Branch B: interpret directly
```

The artifact is the contract. The compiler and interpreter share the
artifact format (`gunbc-ir` types) and the evaluator (`daglang-eval`).
They share nothing else. The compiler does not know about transport.
The interpreter does not know about parsing or typechecking.

---

## Stage contracts

Every stage must pass this test: "can you describe what it does in one
sentence, without saying 'and also'?"

### Compiler stages

| Crate | Purpose | Input | Output | Deps (internal) | Side effects | Failure mode |
|-------|---------|-------|--------|-----------------|-------------|-------------|
| `daglang-syntax` | Parse `.dag` source into AST | `&str` (source text) | `SourceFile` (AST) | (none) | None | `ParseError` with span |
| `daglang-resolve` | Discover files, build import graph | filesystem paths | `ModuleGraph` | syntax | Reads filesystem (via `Vfs` trait) | `ResolveError` |
| `daglang-typecheck` | Validate types, produce typed project | `ModuleGraph` | `TypedProject` | syntax, resolve, contract, ir | None | `Verdict<TypedProject>` |
| `daglang-lower` | Lower typed AST to graph IR | `TypedProject` | `VerifiedDag<LoweredOp>` | syntax, typecheck, contract, ir, eval | None (const fold via eval) | `Verdict<LowerResult>` |
| `daglang-derive` | Extract metadata from graph IR | `Dag<LoweredOp>` | `DerivedArtifacts` | ir, lower | None | Infallible (structural walk) |
| `daglang-emit` | Generate target-language source code | `Dag<LoweredOp>` + `DerivedArtifacts` | `EmissionBundle` | ir, expr, syntax, typecheck | None | `EmitError` |
| `daglang-driver` | Orchestrate the compiler pipeline | filesystem paths + options | `CompileOutput` | all compiler stages | Reads filesystem (via resolve) | `Verdict<CompileOutput>` |

### Shared crates

| Crate | Purpose | Deps (internal) | Notes |
|-------|---------|-----------------|-------|
| `gunbc-ir` | DAG, Node, Port, Edge, Value types | infra, contract | The artifact format |
| `gunbc-infra` | Hashing, resource IDs, manifests | (none) | Leaf crate |
| `daglang-contract` | Verdict, Diagnostic, spans | (none) | Leaf crate |
| `daglang-eval` | Evaluate expressions to values | ir | One implementation of all pure ops |

### Interpreter stages

| Crate | Purpose | Input | Output | Deps (internal) | Side effects | Failure mode |
|-------|---------|-------|--------|-----------------|-------------|-------------|
| `gunbc-interp` | Interpret graph IR: pure → eval, I/O → transport | `VerifiedDag<LoweredOp>` + config | `InterpretResult` | ir, eval, exec, transport | I/O via transport only | `InterpretError` |
| `gunbc-exec` | Schedule DAG nodes in topological order | `Dag<T: Executable>` + config | execution log + outputs | ir | None (scheduling only) | `ExecError` |
| `gunbc-lib-transport` | Execute shell, HTTP, filesystem I/O | `TransportRequest` | `TransportResponse` | ir, exec | **The I/O boundary** | `TransportError` |

### Constraints on the contract table

- `gunbc-interp` accepts `VerifiedDag<LoweredOp>` only. It does not
  build graphs, load caches, or compile DSL source. Those are
  orchestration concerns that belong in the driver or a runtime
  orchestrator above the interpreter.
- `gunbc-exec` is a generic scheduler. It does not know about
  `LoweredOp`. It schedules any `Dag<T: Executable>`.
- `gunbc-lib-transport` is the only crate that performs direct I/O.
  Everything else is pure or delegates to transport.
- Every compiler stage returns `Verdict<T>` or `Result<T, Error>`.
  No `eprintln` + continue. No silent fallbacks. No fabricated defaults.

---

## Rationale: why these stages, not others

### Why separate the evaluator from the lowerer?

The evaluator (`eval.rs`) interprets `LoweredExpr` trees to produce
`Value` results. It serves two callers:

- **Compiler** (const folding): `build_data_values()` evaluates `data`
  declarations at compile time. Like `rustc` calling MIRI for const eval.
- **Interpreter** (runtime): fn bodies, pure operations, collection ops.

The evaluator is the interpreter's core logic. Hosting it inside the
compiler (`daglang-lower`) forces the interpreter to depend on the
compiler. Extracting it into `daglang-eval` lets both the compiler and
interpreter depend on it as a shared library, without depending on each
other.

### Why no resolver?

The current resolver (`gunbc-resolve`) converts `LoweredOp` → `DynOp`
by wrapping each lowered operation in a Rust struct implementing
`Executable`. This is pure translation overhead — the evaluator already
handles every pure operation. The resolver exists because the executor
expected trait objects, not because resolution is a meaningful semantic
step.

In the target architecture, the interpreter dispatches on `LoweredOp`
directly: pure ops → evaluator, transport → transport layer. No
translation step, no separate op structs, no duplication.

### Why is the interpreter not part of the compiler?

The compiler's job is to produce `VerifiedDag<LoweredOp>`. Running that
artifact is a separate concern — different dependencies, different
failure modes, different performance characteristics. Mixing them
produces the entanglement documented in the postmortem: the evaluator
trapped inside the lowerer, the resolver depending on the driver, the
interpreter transitively depending on the emitter.

Separation means:
- The compiler can evolve (new syntax, new type features) without
  touching the interpreter.
- The interpreter can evolve (new transport backends, better scheduling)
  without touching the compiler.
- Each can be tested independently with clear contracts.

### Why keep `gunbc-exec` as a generic scheduler?

`gunbc-exec` provides: topological scheduling, dry-run interception,
progress tracking, execution logging, CI context. These are generic DAG
execution concerns. `gunbc-interp` provides the `LoweredOp`-specific
dispatch (pure → eval, transport → I/O). Keeping them separate means
the scheduler can be reused for other DAG types (test mocks, scripted
DAGs) without pulling in the interpreter's dependencies.

---

## Type-intrinsic operations (replacing the pipe method registry)

### The problem

The codebase has a manual registry (`PIPE_METHOD_REGISTRY`) listing 21
pipe methods with their signatures. This registry is consumed by the
typechecker (to derive callable contracts), the emitter (to generate
code), and the evaluator (to interpret at runtime). Adding or removing a
method requires updating multiple sites — the same class of disconnect
that caused the builtin callable failures.

### Why these operations are not "extern"

`fold`, `chars`, `sort_by`, etc. are not external functions that happen
to be implemented in Rust. They are **type-intrinsic operations** —
capabilities that exist because the types exist. `fold` is intrinsic to
`List` the same way `+` is intrinsic to `Int`. You don't declare `+` as
`extern func`; it comes with the type.

### The split

**Type-intrinsic** (~8 operations): Operations that require access to
value internals — iterating list elements, accessing string characters,
sorting, byte-level operations. The evaluator implements these directly
as part of what the built-in types *mean*.

| Type | Intrinsic operations |
|------|---------------------|
| `List<T>` | `fold`, `append`, `sort_by` |
| `String` | `chars`, `split`, `join`, `starts_with`, `ends_with` |
| `Any` | `to_bytes`, `to_json`, `hash` |

**DSL-expressible** (~13 operations): Operations writable as `fn` items
using `fold` and other intrinsics. These are normal functions — no
special treatment.

```dag
fn map(xs: List<A>, f: fn(A) -> B) -> List<B> {
  xs |> fold(init: [], f: fn(acc, item) { acc |> append(f(item)) })
}

fn filter(xs: List<A>, f: fn(A) -> Bool) -> List<A> {
  xs |> fold(init: [], f: fn(acc, item) {
    if f(item) { acc |> append(item) } else { acc }
  })
}

fn any(xs: List<A>, f: fn(A) -> Bool) -> Bool {
  xs |> fold(init: false, f: fn(acc, item) { acc or f(item) })
}
```

### What changes

- **Parser**: Parses `|> name(args)` generically. No `PipeMethod` enum
  at parse time. Produces `Expr::Pipe(expr, name, args)`.
- **Typechecker**: Resolves pipe method names from the AST (DSL `fn`
  definitions for the ~13) and from built-in type intrinsics (the ~8).
  No manual registry.
- **Evaluator**: Handles intrinsics as part of type evaluation (like
  `+` for `Int`). DSL-defined methods go through fn body evaluation.
- **Emitter**: Generates code per intrinsic per target language. DSL-
  defined methods are emitted as normal fn bodies.
- **Deleted**: `PIPE_METHOD_REGISTRY`, `PipeMethod` enum,
  `builtin_callable_contracts()` pipe method derivation.

---

## Data values: embed in the DAG, not as a sidecar

### The problem

DSL `data` declarations are compile-time constants evaluated by the
lowerer. Currently they produce a `HashMap<String, serde_json::Value>`
carried alongside the DAG as a separate output. Every handoff point in
the system must thread this sidecar. Forgetting one handoff caused 658
test failures ("data declarations invisible to evaluator").

### The fix

Embed data declarations as literal nodes in the DAG, the same way the
lowerer already handles data references in service call arguments
(creating `CallLiteralSource` nodes). Fn bodies that reference data
declarations get input ports wired to literal nodes. The DAG is
self-contained — one artifact, one input to the interpreter.

```
Before (sidecar):
  Lowerer produces: VerifiedDag<LoweredOp> + HashMap<String, Value>
  Interpreter receives: two inputs, threaded independently

After (embedded):
  Lowerer produces: VerifiedDag<LoweredOp>  (data as literal nodes)
  Interpreter receives: one input
```

`data_values` as a separate HashMap, `evaluate_fn_body_with_data()`,
`CachedCompileData.data_values`, and the `#[serde(default)]` cache
compat hack all disappear.

---

## Expression IR layering

`LoweredExpr` and related types (`LoweredFnBody`, `LoweredBinOp`,
`LoweredMatchArm`) are defined in `daglang-lower` but consumed by the
evaluator. Extracting the evaluator requires these types to be
independently accessible.

**Near-term** (Phase 6): Create `daglang-expr` containing the
expression types. Dependencies: `gunbc-ir` and `daglang-syntax` (for
`PipeMethod`, `BinOp` enum reuse). This is Option B — smallest blast
radius.

**Target state**: `daglang-expr` defines its own IR-level enums
(`ExprBinOp`, `ExprPipeMethod`) independent of `daglang-syntax`. The
lowerer maps from syntax-level to IR-level during lowering. The
evaluator and interpreter have zero transitive dependency on the parser.
This is the principled layering — each stage's output format is
independent of its input format.

The near-term compromise is intentional: getting the evaluator extracted
is higher priority than perfecting the enum layering. The layering fix
is small and mechanical once the crate boundary exists.

---

## Current state vs. target state

### What exists today

```
daglang-syntax          (leaf)
daglang-contract        (leaf)
gunbc-infra             (leaf)
    ↓
gunbc-ir                (infra, contract, delegate-macros)
    ↓
gunbc-exec              (ir)
    ↓
gunbc-primitives        (infra, ir, exec)          ← DELETING
gunbc-lib-transport     (ir, exec)
    ↓
daglang-lower           (contract, syntax, typecheck, ir)
                        contains eval.rs            ← EXTRACTING
    ↓
daglang-derive          (lower, contract, ir)
daglang-emit            (derive, lower, syntax, typecheck, ir)
    ↓
daglang-driver          (all compiler stages)
    ↓
gunbc-resolve           (9 deps, reimplements eval) ← DELETING
```

### Target state

```
LEAVES:
  daglang-syntax, daglang-contract, gunbc-infra

SHARED:
  gunbc-ir              (infra, contract)
  daglang-eval          (ir)                        ← NEW (from lower)

COMPILER:
  daglang-typecheck     (syntax, resolve, contract, ir)
  daglang-lower         (syntax, typecheck, contract, ir, eval)
  daglang-derive        (ir, lower)
  daglang-emit          (ir, lower, syntax, typecheck)
  daglang-driver        (orchestrates compiler stages)

INTERPRETER:
  gunbc-interp          (ir, eval, exec, transport) ← NEW (from resolve)
  gunbc-exec            (ir)
  gunbc-lib-transport   (ir, exec)
```

Changes:
- `gunbc-resolve` (9 deps) → `gunbc-interp` (4 deps, no compiler deps)
- `gunbc-primitives` → deleted (ops in `daglang-eval`)
- `eval.rs` → extracted from `daglang-lower` to `daglang-eval`
- `PIPE_METHOD_REGISTRY` → deleted (type-intrinsics + DSL fn items)
- `data_values` sidecar → embedded as literal nodes in DAG
- `builtin_callable_contracts()` manual entries → deleted (DSL fn items)

---

## Invariant: fail closed

Every compiler and interpreter stage either succeeds fully or fails with
a structured diagnostic. No silent degradation.

- `eprintln!` is banned in library crates (enforced via `clippy.toml`).
- Unknown types are hard errors, not fallbacks to `Json`/`Opaque`.
- Missing mock types are hard errors, not fallbacks to `Json(Null)`.
- Wiring failures in the lowerer are hard errors, not `continue`.
- Placeholder file rendering in testgen is removed — compile failure is
  a failure, not a "skipped" result.

See `src/v1/README.md` "No hacks or fallbacks" for the coding standard.
This architecture enforces it structurally: with one evaluator and one
execution path, there are fewer places where fallbacks can hide.
