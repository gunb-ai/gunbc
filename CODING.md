# Coding — gunbc

**Philosophy.** Google C++-style functional/imperative. Data +
pure functions, not objects with hidden state. Every function
reads as `input → output`. Interfaces are precise contracts a
reader can understand without reading the body.

This document covers **Rust implementation style** in
`src/v3/compiler/src/`. For the concepts the compiler **models**,
see `MODELING.md`. For the invariants the compiler **proves**,
see `INVARIANTS.md`. For the shape of tests, see `TESTING.md`.

## Five principles

### 1. Pure functions by default

Most functions in `src/v3/compiler/src/` are structurally pure:
`(input, dag: &Dag, ...) -> Output`. No global mutation. No
hidden state. Given the same inputs, return the same output.

"Pure" in Rust-with-&mut terms:

- `fn f(&self, x) -> Y` — pure reader over `self`, no mutation.
- `fn f(&mut self, x) -> Y` — acceptable when `self` is an
  accumulator whose final state is the caller's intended
  output (the compiler's `Dag::new()` + `push_*` pattern is
  this shape). The function is still logically
  `(old_self, x) → new_self` — Rust's borrow checker is the
  optimizer.
- `fn f(x) -> Y` (free function) — preferred when the body
  doesn't reference `self`. Don't hang a function off a type
  unless the function genuinely needs that type's invariants.

**Impurity is an escape hatch, not a default.** If a function
reads a global, mutates shared state, prints to stderr, or
reaches the filesystem, it should be named in a way that makes
that obvious (`write_*`, `emit_to_*`, `log_*`), and it should
live at the edges of the system — the bootstrap, `main`,
CLI wrappers, the regen binaries.

### 2. Data + free functions, not objects

Most compiler state is **data structures** (`Dag`, `Declaration`,
`Node`, `OperationEffect`, `SymbolicCost`, …) that are operated
on by **free functions** (`compile_to_dag`, `analyze_workflow`,
`cost_of`, `emit_rust_module`, …). Types don't own the behavior;
behavior operates on the types.

This is the shape the lenses enforce: a lens IS a pure function
`fn analyze(dag: &Dag) -> Report`. It doesn't live as a method
on `Dag`. The lens is external; `Dag` stays a data structure.
`feedback_lenses_not_passes` names the same principle:
*"analyses are lenses over physics; zero heuristics."*

**Avoid:** deep method chains on a god object, trait hierarchies
that encode domain concepts, builder patterns with fluent
interfaces.

**Prefer:** a small number of data types with many free
functions that consume them. Grouping by **concept** (the
shape of the data) rather than by **actor** (a class that
"does" things).

### 3. Clear interfaces

A function's signature should tell the reader what it does
without reading the body:

- Names describe the mapping, not the mechanism
  (`cost_of`, not `compute_cost_via_walk`).
- Input types describe exactly what's needed, nothing more
  (`fn f(dag: &Dag, port: PortId)`, not
  `fn f(ctx: &FullCtx)` when only `dag` and `port` matter).
- Output types are structured (`CostLookup`, not `Option<i64>`;
  `CompositionVerdict`, not a `bool` with a sibling error).
- Error paths are in the type, not in comments or undocumented
  panics (`Result<T, StructuredError>`, `Option<T>` when
  absence is meaningful, typed diagnostic enums).

**Anti-signal:** a function taking `&mut Everything` and
returning `()` is usually hiding what it does. Split it.

### 4. Explicit dependencies

A function's inputs include every piece of state it reads. No
globals, no thread-locals, no "pick up config from an
environment variable." If a function needs a `Dag`, the caller
passes a `&Dag`. If it needs a target language, the caller
passes a `TargetLanguageId`. The signature IS the dependency
list.

This makes the code testable by construction. The unit test
constructs a minimal instance of each input, calls the
function, asserts on the output. No setup hooks, no fixture
frameworks.

Acceptable exceptions (impure, documented):
- **Bootstrap**: `Dag::new()` loads std files at initialization
  time; the list of files is static. Localized to one place.
- **Build scripts**: `build.rs` has legitimate filesystem and
  stdout side effects by definition.
- **Regen binaries**: `src/bin/regen_*.rs` write files to disk;
  that's their job.
- **Test harness**: `cached_compile_to_dag` holds a module-level
  `OnceLock` map for per-test-binary amortization. Documented
  as a performance cache, not an expressive dependency.

### 5. Small and composable

Functions do one thing. If a function does two things, it's
two functions. Composition happens at the call site; the
callee doesn't know who called it.

- Function bodies over ~50 lines are a smell. Either the
  function is doing too much, or its input/output types are
  under-specified and the body is compensating with ad-hoc
  unpacking.
- Module files over ~500 lines are a smell for the same
  reason — they typically bundle multiple distinct behaviors
  that would read better as separate modules.
- `impl` blocks over ~20 methods on a single type are a smell:
  the type is accreting responsibilities that don't all
  belong to its invariants.

## What "pure" means in gunbc Rust

| Rust shape | Purity status |
|---|---|
| `fn f(x: T) -> U` (free function, no globals) | pure |
| `fn f(&self, x: T) -> U` (method, reader only) | pure |
| `fn f(&mut self, x: T) -> U` (method, accumulator) | pure-by-borrow — caller threads the accumulator |
| `fn f(x: &mut T) -> U` (caller's data mutated) | pure-by-borrow — same shape, free-function form |
| `fn f(x: T)` returning `()` with `println!`/`writeln!` | impure (side-effecting) — should be at the edge |
| `fn f(x: T) -> U` reading a `static mut` | impure — avoid |
| `fn f(x: T) -> U` reading a `LazyLock<Mutex<...>>` cache | impure — documented amortization only |

**A good rule of thumb:** if you can replace `&mut T` with
`(T) -> T` and the code still reads naturally, the function
was always pure. The `&mut` is a Rust-imposed borrow-checker
shape, not a semantics change.

## Interface design conventions

**Return structured carriers, not primitives.** `cost_of(port)
-> CostLookup` (where `CostLookup = FoundCost(i64) |
MissingCost`) is better than `cost_of(port) -> i64` with 0
meaning "missing." The typed carrier forces the caller to
distinguish. See `feedback_fail_closed_discipline` (C-8).

**Fail-closed at boundaries.** A function that can't produce
its declared output returns a diagnostic or `None` — never a
silent default. `feedback_fail_closed_is_boundary`.

**Typed handles over raw ids.** A function taking `ParamRef` is
better than a function taking `(NodeId, usize)` with a comment
"slot must be < member.params.len()". The typed handle carries
the witness. `feedback_state_space_vs_behavioral_invariants`.

**Accessors named for what they return, not how they compute.**
`port_of(bool_ref) -> PortId` is better than
`extract_port_from_witnessed_ref(bool_ref) -> PortId`. The
body is the implementation; the name is the contract.

**Names are namespaces, not aliases.** See
`feedback_naming_is_aliasing` — a `TypeAlias` declaration is a
namespace into the substrate, not a new type. Treat Rust
wrappers (newtypes) the same way: they carry the witness, they
don't duplicate the underlying structure.

## Anti-patterns

### Builder patterns with mutable fluent interfaces

```rust
// ❌ fluent mutable builder
let result = Builder::new()
    .with_flag(true)
    .with_option(x)
    .with_context(ctx)
    .build()?;
```

Most "builders" are struct literals in disguise. Prefer:

```rust
// ✅ data constructor
let config = EmitConfig {
    flag: true,
    option: x,
    context: ctx,
};
let result = emit(&config)?;
```

The struct-literal form makes dependencies explicit, omits no
fields by accident, and doesn't hide invariants in method
ordering.

### Trait hierarchies for domain concepts

```rust
// ❌ trait for a domain concept
trait CostLens {
    fn analyze(&self, dag: &Dag) -> CostReport;
    fn bound_for(&self, port: PortId) -> Option<CostBound>;
    // ...10 more methods
}
impl CostLens for LinearCostLens { ... }
impl CostLens for PolynomialCostLens { ... }
```

Traits are for **interfaces the language needs** (`Clone`,
`PartialEq`, `Iterator`), not for domain concepts. Domain
concepts are **data + free functions**: `SymbolicCost` is a
data type; `cost_of`, `dominates`, `sequential` are free
functions over it. Polymorphism is pattern-matching on variants,
not dynamic dispatch.

### Hidden state

```rust
// ❌ function with hidden dependency
static LOADED_SPEC: LazyLock<Mutex<Option<TargetSpec>>> = ...;
fn emit_rust(dag: &Dag) -> Result<String, EmitError> {
    let spec = LOADED_SPEC.lock().unwrap().clone().expect("spec loaded");
    // ...
}
```

The spec is a dependency. Pass it in:

```rust
// ✅ explicit dependency
fn emit_rust(dag: &Dag, spec: &TargetSpec) -> Result<String, EmitError> {
    // ...
}
```

### God functions / god modules

```rust
// ❌ one function does five things
fn emit_rust_module(dag: &Dag) -> Result<String, EmitError> {
    // 300 lines: build indexes, walk declarations, render types,
    // render functions, render values, invoke rustfmt, apply
    // correction style, emit diagnostics
}
```

Each of those is a distinct behavior. Compose them at the call
site:

```rust
// ✅ small composable pieces
let indexes = RealizationIndexes::build(dag)?;
let declarations = render_declarations(dag, &indexes)?;
let functions = render_functions(dag, &indexes)?;
let formatted = format_rust(&declarations, &functions)?;
emit_rust_module(dag, &indexes, &declarations, &functions)
```

The top-level function is a pipeline stitch; each stage has
clear input → output.

### Hidden panic surface

```rust
// ❌ unwrap inside library code
fn find_cost(dag: &Dag, port: PortId) -> i64 {
    dag.nodes().iter()
        .find(|n| n.id() == port.node_id())
        .unwrap()  // panics on unknown port
        .cost
        .unwrap()  // panics on missing cost
}
```

Panics are contract violations, not error paths. Library code
returns `Result` or `Option`. Only `main` and test code
`unwrap()`.

## When impurity is acceptable

The compiler's edges necessarily interact with the outside
world. Impurity is fine in these specific places, and should
be clearly labeled:

| Location | Kind of impurity | Labeling |
|---|---|---|
| `src/v3/compiler/build.rs` | filesystem, codegen | rustdoc header |
| `src/v3/compiler/src/bin/regen_*.rs` | filesystem writes | rustdoc header |
| `src/v3/compiler/src/bin/cli.rs` | stdin/stdout | rustdoc header |
| bootstrap inside `Dag::new()` | loads std files at init | single-call-site; std files frozen at build |
| `tests/integration/common/cached_compile.rs` | `LazyLock<Mutex>` cache | explicit: "per-test-binary amortization" |

Everywhere else — library code, lenses, substrate walks,
emitters, inference, lowering — should be pure in the Rust
sense described above.

## Related

- `MODELING.md` — what to model in `std/`
- `INVARIANTS.md` — invariants the compiler enforces
- `TESTING.md` — test discipline (this document is the
  production-code twin)
- `THESIS.md` — what the compiler is for
