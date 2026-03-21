# v2: Self-hosted compiler

Use this file for target architecture and design rules. Current status
and gap analysis live in `ROADMAP.md`. The sustainability ledger tracking
open violations is `src/v1/SUSTAINABILITY.md`.

## Premise

The v0/v1 compiler exists to eliminate glue bugs in downstream systems,
but the compiler itself is full of the same glue bugs: string-matching,
fabrication fallbacks, parallel representations, deferred computation.
The sustainability ledger (src/v1/SUSTAINABILITY.md) traces 38 symptoms to one
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

| v1 problem | v2 design |
|---|---|
| TypeId is a string → needs registry | Types are TypeExpr values → no registry |
| Cardinality cached on port | Derived from TypeExpr structure |
| Parallel fn body evaluator + DAG executor | Compiler emits files, no interpreter |
| `mock_element_expr` enumeration | Emitter walks TypeExpr structure |
| `register_core_types()` duplication | Types defined in .dag only |
| String-based classification | Pattern match on typed AST |
| No boundary contracts | Each stage is a typed function |
| Transport config as `Map<String, String>` | Typed coproduct per transport kind |
| Emitted code never tested downstream | Compiler emits tests alongside code |

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
and produces files. Different backends produce different languages.

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
```

### Bootstrap subset

The v2 prototype targets the **gist.dag bootstrap subset**: the 8
Item variants actually used by gist.dag and its transitive
dependencies (`module`, `import`, `type`, `fn`, `func`, `service`,
`resource`, `data`). The remaining variants are added incrementally
after bootstrap proves the architecture works.

### Testing: the compiler owns its downstream

v1's critical gap: emitted code is text-checked but never compiled or
run. v2 fixes this by emitting tests alongside code:

```
.dag source → compiler → {program.rs, program_test.rs}
                              │              │
                              ▼              ▼
                          cargo build    cargo test
```

Emitted tests are hermetic by default — they use `mock_response` data
from the DSL source as fixtures. No network, no credentials.

### Bootstrap path

1. **v1 Rust compiler** compiles v2's .dag source files (v1 is the host)
2. **v2 compiler** (running on v1) reads .dag files and emits target code
3. **v2 compiles gist.dag** → emitted files match v1's output
4. **v2 compiles itself** → fixed point = self-hosting

### Dependency chain

```
kernel primitives (String, Int, Bool, List, Map)
  ← dsl/std/types.dag (FilePath, NonEmptyStr, SourceSpan, ...)
    ← src/v2/std/core.dag (Token, AST, Expr, TypeExpr)
      ← src/v2/compiler/*.dag (tokenize, parse, resolve, typecheck, emit)
```

## Decisions

### fn vs func

`fn` = pure function (expression body, no side effects).
`func` = workflow function (may use resources, services, transport).

### Diagnostics are first-class

Every stage returns its primary value with a `diagnostics: List<Diagnostic>`
field alongside it. No diagnostic loss across stage boundaries.

### Error recovery: first-error-halt for bootstrap

Multi-error recovery requires synchronization-token strategy designed
into grammar rules. Deferred — first-error-halt is sufficient for a
batch compiler.

### Python backend: deferred

Deferred until Rust self-hosting is proven. Adding a second backend
before the first works recreates the parallel implementation problem.

### Nominal identity in the typed graph

After typecheck, `List<Span>` becomes:
```
Container { kind: List, element: Product { name: Some("Span"), fields: [...] } }
```
The `name` is metadata for diagnostics and emission — NOT a lookup key.
The structural fields are authoritative.

### MapType key constraint

`MapType { key, value }` allows arbitrary keys in the AST.
The typechecker enforces String-key constraint. The AST represents
what the programmer wrote; the typechecker rejects what's invalid.

---

# Target domain models

**This is a target-model specification, not a description of the
current implementation.** The v2 compiler code does not yet match
these models. Each section notes specific gaps where known.

The pipeline stays linear. The model becomes layered.

## Foundations: Set-Algebraic Type Theory

The type system's semantics are set-theoretic (following `std/types.dag`):

```
Every type T denotes a set ⟦T⟧ of values.
Subtyping is set inclusion:  A <: B  iff  ⟦A⟧ ⊆ ⟦B⟧
```

The type algebra has six operations:

| Type Constructor | Set Operation | Notation |
|-----------------|---------------|----------|
| **Atom** | Generator set | `⟦String⟧ = Σ*` |
| **Product** | Cartesian product | `⟦{a: A, b: B}⟧ = ⟦A⟧ × ⟦B⟧` |
| **Coproduct** | Disjoint union | `⟦A \| B⟧ = ⟦A⟧ ⊔ ⟦B⟧` |
| **Application** | Functor application | `⟦F<A>⟧ = F(⟦A⟧)` |
| **Refinement** | Subset comprehension | `⟦T where P⟧ = {x ∈ ⟦T⟧ \| P(x)}` |
| **Reference** | Indirection | `⟦Named("Foo")⟧ = ⟦Foo⟧` (by lookup) |

These six constructors are **complete** for the language's type system.

The kernel has 8 atoms: `{ String, Int, Bool, Float, Secret, Bytes, Unit, Json }`.
Lattice bounds: `⊤ = Json` (universal), `⊥ = Never` (uninhabited).

### Optionality and cardinality

**Direction: cardinality as the primitive.** Optionality is not a
property of the type — it's a property of how the type is used at a
binding site. `T?` means "type T with cardinality 0..1," not a
different type.

```
Required   : exactly 1 value    (1..1)
Optional   : 0 or 1 value       (0..1)
Many       : 0..n values        (0..n)
AtLeastOne : 1..n values        (1..n)
```

This means:
- `Optional` exits TypeExpr entirely
- `Field.optional: Bool` becomes `Field.cardinality: Cardinality`
- TypeExpr is purely "what" (the set of values), never "how many"
- `List<T>`, `Set<T>`, `Map<K,V>` remain as type constructors via `App`

## Target TypeExpr (6 variants, down from 9)

```dag
type TypeExpr
  = Ref { name: String, span: SourceSpan }
  | Atom { name: String, span: SourceSpan }
  | Product { name: String?, fields: List<Field>, span: SourceSpan }
  | Coproduct { name: String?, variants: List<Variant>, span: SourceSpan }
  | App { name: String, args: List<TypeExpr>, span: SourceSpan }
  | Refine { base: TypeExpr, predicates: List<Predicate>, span: SourceSpan }
```

| Removed | Replaced by | Justification |
|---------|-------------|---------------|
| `Named` | `Ref` | Renamed: it is a reference, not a type |
| `Primitive` | `Atom` | Renamed: these are atoms of the type algebra |
| `Container` | `App` | `List<T>` IS `App("List", [T])` — same set, one representation |
| `MapType` | `App` | `Map<K,V>` IS `App("Map", [K, V])` |
| `Optional` | Cardinality | `T?` is not a type — it's cardinality 0..1 on the binding site |
| `Refined` | `Refine` | Renamed for verb form consistency |
| `TypeApp` | `App` | Renamed: shorter, now the only application form |

## Layer 0: Compiler Primitives

```dag
type SourceSpan { start: Int, end: Int }
type Severity = Error | Warning | Info
type Diagnostic { severity: Severity, message: String, span: SourceSpan?, module_name: String? }
type CompileResult { files: List<TextFile>, diagnostics: List<Diagnostic> }
type TextFile { path: String, content: String }
```

Every pass is a function `Input → (Output × List<Diagnostic>)`.

## Layer 1: Type Algebra

TypeExpr (above), Predicate (unchanged — already clean), TypeBody,
Field, Variant, Param.

```dag
type Field {
  name: String
  type_expr: TypeExpr
  cardinality: Cardinality       // replaces optional: Bool
  default_value: Expr?
  span: SourceSpan
}
```

Cardinality all the way down. The `cardinality` field on Field is
itself a Required binding. The recursion bottoms out at kernel types.

## Layer 2: Normalized Core Syntax

Tokens (unchanged, ~75 variants). AST Items (unchanged — 7 Item
variants). Expressions: ExprData discriminator on Node (21 variants).
The parser performs desugarings (`|>` → MethodCall, `pattern`/`interface` →
FuncDef) — the AST represents core syntax, not every surface spelling.

Transport is a Node where `name` = kind ("rest"/"shell"/"file"/"local"),
`properties` = config as FieldInit entries, `children` = ordered argv
(shell only). Constructor helpers in `00_core.dag`.

## Layer 3: Semantic Model

**Current problem**: `TypedGraph`, `TypedModule`, `TypeEnv`,
`TypeBinding` are defined independently in both typecheck.dag and
emit.dag. `ModuleGraph` types are forward-declared in typecheck.dag.

**Target**: All semantic types live in core.dag, imported by consumers:

```dag
type ModuleGraph { modules: List<ResolvedModule>, diagnostics: List<Diagnostic> }
type TypedGraph { modules: List<TypedModule>, diagnostics: List<Diagnostic> }
type TypedModule { module: Module, type_env: TypeEnv }
type TypeEnv { bindings: List<TypeBinding> }
type TypeBinding { name: String, resolved: TypeExpr }
```

## Layer 4: Emission Model

The emitter consumes TypedGraph, produces List<TextFile>. It is the
only layer that knows about target languages.

Built-in type constructor mapping (replaces separate Container/MapType/Optional arms):

```dag
fn emit_type_app(name: String, args: List<TypeExpr>) -> String {
  match name {
    "List"         => "Vec<" + emit_type(args[0]) + ">"
    "Set"          => "BTreeSet<" + emit_type(args[0]) + ">"
    "Map"          => "BTreeMap<" + emit_type(args[0]) + ", " + emit_type(args[1]) + ">"
    "NonEmptyList" => "Vec<" + emit_type(args[0]) + ">"
    "NonEmptySet"  => "BTreeSet<" + emit_type(args[0]) + ">"
    _              => name + "<" + join(map(args, emit_type), ", ") + ">"
  }
}
```

"Option" is absent — optionality is handled via cardinality on the
binding site.

## Layer 5: Pipeline Composition

Each stage has a well-typed return. The pipeline is linear composition.
Wrapper types (`StageResult`, `TokenizeResult`, `ParseStageResult`) are
eliminated — each stage function already has a well-typed return.

## Concept layering

**The key rule:** A higher layer may name a concept, but it should not
explain it from scratch if a lower layer already can.

```
Concept Layer 0: Universal vocabulary (tautologies)
  v2.std.span, v2.std.lex, v2.std.types, v2.std.http

Concept Layer 1: Domain styles / protocol families
  v2.std.rest, v2.std.resource, v2.std.service, v2.std.backend

Concept Layer 2: Provider / backend facts
  v2.targets.rust, v2.providers.github, v2.providers.gcp

Concept Layer 3: Concrete services / concrete compiler constructs
  v2.providers.github.gists, v2.targets.rust.type_emit

Concept Layer 4: Workflows / tools / pipeline
  v2.compiler.pipeline, v2.tools.codegen
```

How layers intersect pipeline stages:

```
                    concept 0         concept 1         concept 2
                    (tautologies)     (domain styles)   (provider facts)
pipeline:tokenize   lex vocabulary      —                  —
pipeline:parse      syntax, types       —                  —
pipeline:typecheck  types, behavioral   service, resource  —
pipeline:emit       types               backend            rust/python facts
```

Splitting `core.dag` into 5-8 concept modules is not Phase 0 work —
it runs alongside self-hosting. But new concepts go in the right
layer, not appended to core.dag.

## Migration plan

**Phase 0a: Canonical type homes.** Move semantic types to core.dag.
Delete local redeclarations. ~1 session, zero semantic change.

**Phase 0b: TypeExpr consolidation.** Rename `Named` → `Ref`,
`Primitive` → `Atom`, etc. Remove `Container`, `MapType`, `Optional`.
Update parser/typechecker/emitter. ~1-2 sessions.

**Phase 0c: Eliminate wrapper types.** Delete the 11+ named wrapper
types in typecheck.dag and pipeline.dag. Return anonymous records.
~1 session.

**Phase 0d: OperationSemantics (deferred).** Post-self-hosting.

## Open design work

**OD1: Typed parser result shape.** Leading candidate: typed coproduct
(`ParseVal`) replacing `Map` in `PR.val`, ~25 variants. Needs
prototyping against parse.dag's error-propagation patterns.

**OD2: Cardinality representation.** Finite coproduct
(`Required | Optional | Many | AtLeastOne`) vs min/max range. Coproduct
covers all current uses; range is more general.

**OD3: Cardinality in return position.** Options: field on function def,
return-type record, or model as coproduct (`Found | NotFound`).

**OD4: ServiceConfig grounding.** Field set should match actual grammar,
not be speculative.

---

# Direction: domain-generic architecture

Where the sections above track the v2 compiler's design and type
models, this section tracks compatibility with domains beyond software.
The DAG is not inherently a software concept — it models causal
process, which all physical systems obey.

## Core model: DAG as causal process

The DAG models causality: directed edges mean "A must happen before B."
Current flows because voltage was applied. A component is soldered
because it was placed. A signal is amplified because it entered the
gain stage.

Even analog circuits with feedback loops are causal in time — the
output at time t feeds back to affect the input at t+dt. The existing
loop construct (LoopUnpack → body → LoopPack) models exactly this.

SPICE netlists appear undirected, but this is a modeling shortcut for
simultaneous constraint solving, not a reflection of the underlying
physics. The DAG captures the causal reality; backends that need
undirected representations project away the direction at emit time.

## Everything is an interface

The `extdeps/` pattern treats every external system as a composable
interface. This is the universal pattern — every domain (circuit
analysis, simulation, frequency response, physical layout) is an
interface. Interfaces compose. The user writes at whatever level of
intent they choose, and the compiler unfolds through the interface
chain.

```
SOFTWARE                              CIRCUIT (same pattern)
extdeps/github.dag → GitHub API       extdeps/ohm.dag → V=IR relationships
extdeps/shell.dag  → local shell      extdeps/kirchhoff.dag → KVL/KCL
extdeps/make.dag   → Make             extdeps/spice.dag → SPICE simulation
```

The domain models live in `.dag` files — type hierarchies and interface
definitions — the same way `dsl/std/` encodes the type chain
`Classical → Bit → Byte → Word → Int → String`. The circuit
equivalent: `Signal → Voltage → BandlimitedVoltage`.

Multiple domain perspectives (signal flow, physical layout, frequency
response, thermal) are **compiler-derived analyses** over the same
source DAG — not separate user-authored graphs. The user writes one
causal description; the compiler derives the rest.

## Architectural constraints

Rules that preserve optionality for future domains:

**C1: Edges are always causal — backends project as needed.**
The core graph does not need undirected edges. Backends that need
different connectivity models (SPICE nets, layout adjacency) derive
them by projecting the causal DAG at emit time.

**C2: Keep the unfolding machinery generic.**
The type DAG composition, lowering passes, and analysis passes must
not be specialized to software concepts. `Signal → Voltage →
BandlimitedVoltage` should work the same way as `Classical → Bit →
Byte → Word → Int` without compiler changes.

**C3: Do not specialize Domain() refinements.**
`Domain(String)` is intentionally open — it could mean
`"ieee754_binary32"`, `"si_volt"`, or `"spice_bsim4_model"`. If
compile-time dimensional analysis is needed, implement it as a
typecheck pass that interprets domain strings, not as a change to
the `Predicate` enum.

**C4: Do not couple type structure to execution model.**
A `Voltage` type should be usable in both a SPICE netlist (emitted
for simulation) and a Rust program (computed numerically) without
the type definition knowing which backend applies. Today this holds.

**C5: Feedback loops use the loop construct, not special edges.**
Analog feedback and digital feedback both model the same thing:
output at time t influences input at t+dt. The loop construct
handles this. Do not introduce cycle-permitting edges.

**C6: Domain analyses are compiler passes, not user-authored graphs.**
The pass infrastructure must support domain-pluggable analyses — a
frequency response pass, a thermal pass — alongside today's typecheck
and derive passes.

## Future domains (stubs)

These are domains the architecture should support. Implementation
means writing `.dag` interface files and emit backends — not changing
the core graph or execution model.

**Digital hardware (Verilog).** `logic_gate.dag`, `register.dag`,
`clock.dag`, `verilog.dag`. The type system already speaks this
domain: `Classical → Bit → Byte → Word`, `Bit where brand("Clock")`.
Emit backend maps `Callable{Fn}` → `module`, `BinaryOp` → `assign`
or `always @(posedge clk)`, port cardinality → wire widths.

**Analog circuits (SPICE).** `ohm.dag`, `kirchhoff.dag`, `spice.dag`.
Domain type hierarchy: `Signal`, `Voltage`, `Current`, `Impedance`
with `Domain()` refinements and `Brand()` nominal safety. Transient
simulation = outer time-stepping loop × inner Newton-Raphson
convergence loop. Same loop construct, different domain.

**Physical dimensions.** SI base dimensions as compositional types.
Every physical quantity is a vector of 7 integer exponents over base
dimensions. The dimensional equivalent of `bit.dag`'s width constraints.

The full composition parallel:

```
LAYER  SOFTWARE                     CIRCUIT
0      Classical (truth)            BaseDimension (mass/length/time/...)
1      Bit = Classical[width(1)]    Quantity = Float64[dimension]
2      Byte = List<Bit>[8]          Voltage = Quantity[si_volt, brand]
3      Int64 = Word64[signed]       Resistor = {resistance, tolerance, ...}
4      Filesystem (resource)        CircuitNet (resource)
5      file_content_matches (step)  ohm::voltage_from (constitutive law)
6      content_upsert (pattern)     simulate::transient (pattern)
7      github.dag (interface)       spice.dag (interface)
user   "ensure clippy.toml"         "simulate voltage divider"
```
