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

The compiler is a pure transform: `.dag files → backend artifacts`. It
reads source, analyzes it, and lowers the program to a chosen backend.
There is no interpreter stage embedded in the compiler pipeline.
Interpretation, when supported, is a downstream concern: the compiler
may emit a canonical DAG artifact, and a separate runtime/interpreter
may consume that artifact.

```
.dag source → [compiler] → backend artifact → [target runtime / interpreter]
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

The fix: the compiler lowers programs to backend artifacts, and runtime
execution stays outside the compiler. For native targets that artifact
is emitted source files. For a future DAG target it is a canonical DAG
program/bundle. In both cases, the compiler and the runtime are
separate programs with a boundary between them.

## Core principle: types are values, not references

In v2, types are structural values that flow through the pipeline
like any other data. A type is a `Node` — the same unified recursive
structure used for expressions, functions, and all other program
elements.

When a function parameter has type `List<Span>`, the compiler doesn't
store the string `"List<Span>"` and look it up later. It stores a
`Node` with the structural composition inline. The structure IS the
type. No registry. No deferred lookup. No stale copies.

| v1 problem | v2 design |
|---|---|
| TypeId is a string → needs registry | Types are Node values → no registry |
| Cardinality cached on port | Derived from Node structure |
| Parallel fn body evaluator + DAG executor | Compiler emits files, no interpreter |
| `mock_element_expr` enumeration | Emitter walks Node structure |
| `register_core_types()` duplication | Types defined in .dag only |
| String-based classification | Pattern match on typed AST |
| No boundary contracts | Each stage is a typed function |
| Transport config as `Map<String, String>` | Transport dissolved into Node |
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
│Typecheck  │  ModuleGraph → TypedGraph (types resolved to Node)
└────┬──────┘
     ▼
┌─────────┐
│  Emit    │  TypedGraph → backend artifact files
└────┬──────┘
     ▼
  output files (.rs, .py, .dag.json, ...)
```

Five stages, each a pure function. The compiler goes from typed AST
directly to backend artifact files.

For project compilation, artifact planning happens in orchestration
after infer and before repeated per-artifact emit. That planning step is
intentionally not a numbered core stage.

### File layout

Numbers are reserved for the true linear transformation stages of the
compiler. If a file is numbered, it must be one step in the
source-to-files transform and it must be callable as a pure stage.

```
src/v2/
  00_core.dag         -- shared compiler vocabulary and structural types
  01_tokenize.dag     -- source text -> tokens
  02_parse.dag        -- tokens -> parsed modules
  03_resolve.dag      -- parsed modules -> import-resolved graph
  04_infer.dag        -- resolved graph -> typed graph
  05_emit.dag         -- typed artifact/subgraph + target -> backend artifact(s)

  compile.dag         -- compiler driver/orchestrator; wires stages together
  complexity.dag      -- proof/obligation derivation over typed graph; not a core compile stage
  ownership.dag       -- proof/obligation derivation over typed graph; not a core compile stage
  artifact.dag        -- artifact planning/partitioning; sits above per-artifact emit
  trace.dag           -- runtime/source-map trace contracts; side-system
  tools/
    interpret.dag     -- convenience tool: compile to DAG backend, then run it
  runtimes/
    dag/
      interpreter.dag -- executes canonical DAG artifacts; not part of compile pipeline

  targets/
    rust/
      adapter.dag     -- Rust-specific lowering from typed DAG facts to shared emit hooks
    python/
      adapter.dag     -- Python-specific lowering
    go/
      adapter.dag     -- Go-specific lowering
    dag/
      adapter.dag     -- compiler-owned lowering to canonical DAG artifact

dsl/extdeps/languages/
  rust/
    emit.dag          -- declarative Rust syntax/type/runtime/import facts
    types.dag
    runtime.dag
    imports.dag
    ...
  python/
    ...
  go/
    ...
```

The key naming rule:

- numbered files = pure transformation stages only
- unnumbered files = orchestration or adjacent analyses
- `targets/<lang>/adapter.dag` = compiler-owned target lowering
- `dsl/extdeps/languages/<lang>/*` = declarative language facts, not compiler graph walks

`06_pipeline.dag` is the compiler driver, not a sixth numbered stage.
Pending renames (tracked in ROADMAP.md M1):

- `04_reconcile.dag` → `04_infer.dag`
- `06_pipeline.dag` → `compile.dag`

### Module classes

Not every compiler-adjacent module is a pipeline stage. The
classification is:

- Core stages:
  `00_core`, `01_tokenize`, `02_parse`, `03_resolve`, `04_infer`,
  `05_emit`.
  These are the only numbered files. They define the linear
  source-to-backend transform.

- Proof / obligation derivation layers:
  `complexity.dag`, `ownership.dag`.
  These consume the typed graph and derive proofs, executable validation
  obligations, reports, or diagnostics from the same source program.
  They may run by default inside `compile.dag`, but they do not define
  new program meaning and they are not additional lowering stages unless
  they are explicitly promoted into a transformation stage.

- Orchestration / build planning:
  `compile.dag`, `artifact.dag`.
  `compile.dag` wires the core stages and chooses which analyses run.
  `artifact.dag` plans buildable units, backend targets, and boundaries
  after infer and before per-artifact emit. It sits in orchestration,
  not as a numbered core stage.

- Runtime / debugging boundary:
  `trace.dag`, `tools/interpret.dag`, `runtimes/dag/*`.
  These describe execution traces, source remapping, convenience tooling,
  and DAG runtime behavior. They consume backend artifacts or runtime
  events; they are not compile stages.

Current standing of the main non-stage modules:

- `complexity.dag`: real and useful today; an always-on proof/report
  derivation in the current driver, but still conceptually not a lowering
  stage
- `artifact.dag`: honest future orchestration model; explicit-plan-only
  today, not yet driving compilation; target role is to sit above emit
  as the selector of artifact targets and boundaries
- `trace.dag`: honest runtime/debug contract; normalized schema, not part
  of compile-time execution

### Ownership model

The compiler owns target adapters. The language facts do not.

Why:

- the adapter knows about compiler IR (`Node`, typed graphs, scope, emit context)
- the adapter applies target rules to a specific program
- the extdeps language modules should remain declarative and reusable

So the split is:

- `src/v2/targets/<lang>/adapter.dag`:
  compiler-owned meta-domain modeling for code generation
- `dsl/extdeps/languages/<lang>/*`:
  language-owned facts such as keywords, type templates, import syntax,
  runtime templates, ownership/value-semantics tables

Program-dependent decisions stay on the compiler side. Examples include:

- whether a typed DAG shape must be Rc-wrapped in Rust
- how a particular match arm lowers under Rust ownership constraints
- how a runtime bridge call should be shaped for this typed program

Those are not universal Rust facts. They are compiler analyses that use
Rust facts.

### Emitter architecture

The scalable design is not "no target files". It is "no target-owned
traversal".

`05_emit.dag` should own:

- shared per-artifact item traversal
- shared expression dispatch
- shared TCO dispatch
- shared service/resource traversal
- common emit context construction

`artifact.dag` should own:

- partitioning the typed whole-program graph into artifacts
- choosing a backend per artifact
- declaring explicit boundary kinds between artifacts
- driving repeated calls into `05_emit.dag`, once per artifact

Each target adapter should own only irreducible target-specific lowering:

- leaf syntax/rendering hooks
- target-only ownership/calling-convention adjustments
- target-only runtime bridge shaping
- target-only statement/expression forms when truly unavoidable

With that structure, adding a new target does not require adding a new
compiler pipeline stage or another full copy of the emitter. It only
adds:

1. a language fact package under `dsl/extdeps/languages/<lang>/`
2. optionally, a thin compiler-owned adapter under `src/v2/targets/<lang>/`

Simple targets may need only language facts. Harder targets such as Rust
need both a fact package and an adapter.

The DAG target is special:

- it is compiler-owned, not extdeps-owned
- it defines the canonical executable/portable DAG artifact for v2
- it exists so interpretation can be supported without embedding an
  interpreter in the core compiler stages

### Interpretation and DAG backend

Long-term DAG interpretation is supported, but it does not become a new
compiler stage.

The model is:

1. the core compiler pipeline remains:
   `tokenize -> parse -> resolve -> infer -> emit`
2. one backend target is `Dag`
3. `targets/dag/adapter.dag` lowers the typed graph to a canonical DAG
   artifact
4. `runtimes/dag/interpreter.dag` executes that artifact
5. `tools/interpret.dag` is just a convenience wrapper:
   `compile(target: Dag) -> run(interpreter, artifact)`

This preserves the key invariant:

- no interpreter logic is mixed into tokenize/parse/resolve/infer
- no parallel "compile path" vs "interpreter path" inside the core stages
- interpretation is just another downstream consumer of the compiler's
  backend output

If DAG artifacts become first-class, `05_emit.dag` may eventually be
renamed to `05_backend.dag`. The architectural rule stays the same:
stage 5 owns backend lowering; execution stays downstream.

The emitter is pluggable — it takes a typed artifact/subgraph plus a
`Backend` and produces files. Different backends produce different
languages or artifact formats, but the traversal belongs in shared emit,
not in per-target whole-compiler files.

At project scope, artifact planning sits above emit. The target
interfaces are:

```dag
type Backend = Rust | Python | Go | Dag

func infer_sources(sources: List<SourceFile>) -> TypedGraph
func plan_artifacts(typed: TypedGraph, rule: PartitionRule) -> ArtifactPlan
func emit_artifact(typed: TypedGraph, artifact: Artifact) -> ArtifactOutput
func compile_project(sources: List<SourceFile>, rule: PartitionRule?) -> BuildResult
```

Interface consequences:

- `Artifact.target` should be a typed `Backend`, not a `String`
- the compiler should stop treating "one target for the whole compile"
  as the primary long-term interface
- the current single-target flow remains as a compatibility wrapper that
  constructs a default one-artifact plan and then calls per-artifact emit

```dag
type Backend = Rust | Python | Go | Dag

func compile(root: FilePath, backend: Backend) -> CompileResult {
  let sources = discover_and_read(root: root)
  let tokenized = map(sources, s => tokenize(source: s.content))
  let parsed = map(tokenized, t => parse(tokens: t))
  let graph = resolve_modules(modules: parsed)
  let typed = typecheck(graph: graph)
  let plan = default_single_artifact_plan(typed: typed, backend: backend)
  let outputs = plan.artifacts |> map(artifact => emit_artifact(typed: typed, artifact: artifact))
  let files = outputs |> flat_map(output => output.files)
  CompileResult { files: files, diagnostics: typed.diagnostics }
}
```

### Gap analysis

Work items, acceptance criteria, and execution order for closing the gap
between the current tree and the target architecture are tracked in
`ROADMAP.md` (Architecture Migration Workboard, M1–M8). This document
describes the target; the roadmap tracks the path.

### Bootstrap subset

The v2 compiler covers the **gist.dag bootstrap subset**: the 8
Item variants used by gist.dag and its transitive dependencies
(`module`, `import`, `type`, `fn`, `func`, `service`, `resource`,
`data`). The remaining variants are added incrementally.

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

Generated test emission is currently Rust-specific (`mock_response`
fixture data). Generalizing to a shared artifact-level facility is
tracked in `ROADMAP.md` M5.

### Generated test strategy

Generated tests are a day-0 interface concern, not a cleanup task to
append after the emitter stabilizes.

The rule is:

- if something can be proven structurally at compile time, prove it in
  the compiler
- if something depends on runtime behavior, backend semantics, transport
  behavior, or emitted code execution, validate it with generated tests

This keeps the architecture honest:

- compile-time proofs prevent impossible programs from being emitted
- generated tests validate the residual runtime obligations the compiler
  cannot discharge structurally

### Generated test quality bar

Generated tests are only valuable if they assert something the compiler
has not already assumed into existence. "The generated request object
equals the same fields the generator just copied" is not a sufficient
test.

The review bar for generated tests is:

- each generated test must correspond to a real residual obligation, not
  a tautology
- the test must exercise a meaningful boundary:
  emitted request shape, response decoding, lifecycle transition,
  cleanup/delete semantics, provider error handling, or runtime
  behavior
- at least one assertion in the test must be capable of failing because
  the emitted program or integration behavior is wrong
- negative-path tests are required whenever the contract has meaningful
  failure modes
- generated tests should prefer observable outcomes over internal
  implementation details
- if a test only reasserts facts already proven structurally, it should
  be deleted or downgraded to a compile-time proof

For review purposes, generated tests should answer:

- what runtime obligation is this test discharging?
- what concrete regression would cause it to fail?
- is the failure signal legible to a human reviewing CI?
- does this test validate a boundary we actually care about in
  production?

### Guarantee ledger

The system should make it easy to answer "what is tested?" and "what is
guaranteed?" without reading generator internals. That means every
relevant workflow/artifact should be able to emit a readable guarantee
ledger.

One useful vocabulary is:

```dag
type GuaranteeMode
  = CompileTimeProof
  | GeneratedValidation
  | Unsupported

type GuaranteeRecord {
  subject: String
  obligation: String
  mode: GuaranteeMode
  evidence: String?
}
```

The important property is not the exact type shape. The important
property is that a reviewer can see:

- which obligations are proven structurally by the compiler
- which obligations are validated by generated tests
- which obligations are still unsupported or unchecked

This is also how the additive rule becomes visible in practice:

- new proof/test families add new guarantee records or upgrade existing
  obligations from `Unsupported` to `GeneratedValidation` or
  `CompileTimeProof`
- they do not require hand-maintained patch lists for every new unseen
  structure
- the tradeoff is explicit:
  more computation/authoring effort in exchange for stronger safety
  guarantees

If the system cannot produce a readable guarantee ledger, the generated
tests/proofs are not yet first-class enough.

### One graph, many projections

Runtime code, generated tests, proofs, complexity certificates, and
reports should all fall out of the same typed DAG. They are not separate
representation families; they are different projections or roles over
the same program structure.

The rule is:

- the source program carries enough structure to derive runtime artifacts,
  proofs, and executable validation artifacts
- proof/validation modules discharge obligations already implicit in the
  graph; they do not rely on hand-maintained per-program checklists
- anything structurally decidable should produce a proof, witness, or
  report directly from the graph
- anything not structurally decidable should produce executable
  validation artifacts from that same graph
- compile should fail or surface an explicit unsupported obligation when
  a required proof/test cannot be discharged

One useful vocabulary for this is:

```dag
type ProjectionRole = Runtime | Test | Proof | Fixture | Harness | Report

type ObligationStatus
  = StaticallyDischarged
  | RequiresExecutableValidation
  | Unsupported
```

The important invariant is not "tests use a different IR". The important
invariant is that the same typed graph can be projected into:

- runtime artifacts
- generated test artifacts
- proof/report artifacts
- diagnostics about obligations that remain unsupported

The same rule also applies to operational setup workflows. If a provider
requires manual provisioning at a system boundary (for example, an admin
must create an API key in an external dashboard), that boundary should
still be modeled as a typed workflow in `.dag`:

- detect missing or invalid credential state
- produce explicit human/admin instructions
- reconcile the resulting secret into managed secret storage
- validate the credential against the provider
- return a ready handle for downstream workflows

In other words, auth upsert is a workflow, not undocumented setup glue.
The workflow may cross a manual boundary, but everything around that
boundary should still be represented, auditable, and testable in the
same projection model as the rest of the system.

That same formula should cover test generation, ownership proofs,
complexity proofs, policy/data-flow proofs, and future mathematical
proofs. The compiler team maintains the derivation rules and capability
tables; individual programs should not require hand-maintained "testing
gap" lists.

### Open-world derivation rule

Proof/test/complexity generation must be open-world and additive. The
compiler should not depend on a fragile closed list of ad hoc proof
variants that has to be manually extended forever just to keep the
system alive.

The maintenance rule is:

- define proof families in terms of graph structure, typed facts, and
  explicit semantic tags
- derive obligations by walking the same typed graph, not by maintaining
  per-program exception lists
- when new structure is added, extend the local derivation rule for that
  structure or return `Unsupported`
- every proof family must have a total outcome for any typed `.dag`
  graph: discharge it statically, lower it to executable validation, or
  mark it unsupported
- new proof families are additive: they add new projections/witnesses
  without invalidating the projection model itself

One useful vocabulary is:

```dag
type DerivationOutcome
  = ProofWitness
  | ExecutableValidation
  | Unsupported
```

This keeps the system from turning into a dying checklist. The compiler
team maintains reusable derivation rules over the graph algebra; it does
not maintain hand-curated inventories of every shape seen so far.

### Example proof families

These are examples of proof families that should scale over any typed
`.dag` graph once their structural rules exist:

1. Structural typing and boundary compatibility
   For any typed graph, derive witnesses that local node shapes, call
   edges, and artifact boundaries are type-compatible. New node forms add
   local typing/boundary rules; existing graphs do not need bespoke
   maintenance.

2. Complexity / cost certificates
   For any typed graph, derive a cost witness from call structure,
   recursion shape, collection transforms, and method semantics. If a new
   construct participates in cost, it adds one local rule to the cost
   algebra. If it cannot yet be modeled, return `Unsupported` for that
   obligation rather than silently guessing.

3. Ownership / aliasing proofs
   For any typed graph, derive ownership and borrowing obligations from
   container structure, call boundaries, mutation points, and target
   value/reference semantics. The proof is not a hand-maintained list of
   "interesting cases"; it is a projection of the graph plus the target
   semantics tables.

4. Policy / information-flow proofs
   For any typed graph with policy tags or labeled data classes, derive
   whether data can legally flow to each sink. New sources, sinks, or
   transforms add local flow rules; they do not require a global rewrite
   of the proof system.

5. Runtime validation projections
   For any typed graph, residual obligations that cannot be discharged
   statically become executable validation artifacts. The test plan is
   therefore not a separate manual matrix; it is the remainder after
   structural proof derivation.

The common pattern is the same in every case:

- graph structure provides the raw facts
- derivation rules compute proofs or residual obligations
- residual obligations become executable artifacts
- unsupported obligations stay explicit in the bundle/diagnostics

That pattern should work for any `.dag` program shape. What changes over
time is which proof families exist and how strong they are, not the
fundamental interface between source graph, proof derivation, and emitted
artifacts.

Examples of compile-time proofs:

- type resolution and shape compatibility
- artifact boundary schema compatibility
- information-flow / policy proofs
- ownership / complexity proofs

Examples of test-time validation:

- emitted code actually executes
- transport stubs and mock responses behave as expected
- serialization / deserialization round-trips work in the target backend
- backend-specific lowering preserves runtime semantics
- runtime/source-map traces remap correctly

### Test taxonomy

The test taxonomy is:

1. Compiler self-tests
   These are host-side tests for tokenize/parse/resolve/infer/emit and
   the compile driver. They are not generated. They protect compiler
   correctness directly.

2. Generated backend unit tests
   Per-artifact tests emitted alongside backend outputs. These validate
   backend/runtime behavior for one artifact in isolation using hermetic
   fixtures.

3. Generated artifact boundary integration tests
   Tests generated from `artifact.dag` boundary plans. These validate
   adapters, serialization, contracts, and cross-artifact compatibility.

4. Generated runtime conformance tests
   Tests that the same typed program behaves consistently across
   backends/runtimes (for example Rust vs Dag interpreter vs JIT).

5. Generated repro / trace tests
   Tests synthesized from normalized traces or captured repro cases to
   lock in real failures as hermetic regressions.

### Generated test migration

Work items for upgrading generated tests (preserving as first-class
emit output, upgrading Rust scaffolds to validation, lifting to
artifact-level interface, boundary integration tests, DAG conformance,
trace/repro tests) are tracked in `ROADMAP.md` M5.

Guardrails:

- do not delete the existing Rust `mock_response` test path until a
  stronger shared/generated replacement exists
- do not claim "compiler emits tests" in a backend-generic sense until
  at least one non-Rust or artifact-level path exists
- do not bury generated-test interfaces inside one target adapter; keep
  the contract visible at the emit/artifact layer

### Bootstrap path

v1 compiles v2's .dag files → v2 compiles itself → fixed point
(byte-identical stage1 == stage2). Self-compile and fixed point are
done. Remaining bootstrap work (gist E2E, v1 retirement) is tracked
in `ROADMAP.md` Phases 2–3.

### Dependency chain

```
kernel primitives (String, Int, Bool, List, Map)
  ← dsl/std/types.dag (FilePath, NonEmptyStr, SourceSpan, ...)
    ← src/v2/std/core.dag (Token, AST, Node, ExprData)
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

After typecheck, `List<Span>` is a `Node` with structural composition
inline: the list application wraps a product node with name `"Span"`
and its fields. The `name` is metadata for diagnostics and emission —
NOT a lookup key. The structural fields are authoritative.

### Map key constraint

Map types allow arbitrary keys in the AST. The typechecker enforces
the String-key constraint. The AST represents what the programmer
wrote; the typechecker rejects what's invalid.

---

# Target domain models

**This is a target-model specification.** The type algebra and
cardinality model are implemented. The domain-generic extensions
(future domains, physical dimensions) remain design targets.

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
- `Optional` is not a type constructor — it is cardinality on the binding site
- `Field.optional: Bool` becomes `Field.cardinality: Cardinality`
- Type nodes are purely "what" (the set of values), never "how many"
- `List<T>`, `Set<T>`, `Map<K,V>` remain as type constructors via application

## Type representation

Types are `Node` values — the same unified recursive structure used
for expressions, functions, operations, and all other program
elements. There is no separate type IR.

The six type algebra operations (Atom, Product, Coproduct, Application,
Refinement, Reference) are all expressed as `Node` compositions.
`Optional` is not a type constructor — it is cardinality on the
binding site.

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

Types are `Node` values. Predicate, Field, Variant, and Param are
structural compositions within `Node`.

```dag
type Field {
  name: String
  type_node: Node
  cardinality: Cardinality       // replaces optional: Bool
  default_value: Node?
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

Semantic types are defined once in `04_reconcile.dag` and imported by
consumers.

```dag
type ResolvedGraph { modules: List<TypedModule>, ..., diagnostics: List<Diagnostic> }
type TypedModule { module: Module, type_env: TypeEnv }
type TypeEnv { bindings: List<TypeBinding> }
type TypeBinding { name: String, resolved: Node }
```

## Layer 4: Emission Model

The emitter consumes `ResolvedGraph`, produces `List<TextFile>`. It is
the only layer that knows about target languages.

Type rendering walks `Node` structure directly. Built-in type
constructor mapping example (target: Rust):

```dag
fn emit_type_app(name: String, args: List<Node>) -> String {
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

## Layer 5: Compiler Composition

Each stage has a well-typed return. The compile pipeline is linear
composition. Each stage function returns its own well-typed result
directly — no wrapper types.

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
  v2.compiler.compile, v2.tools.codegen
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

Foundational type and IR migrations are complete (canonical type homes,
Node unification, wrapper type elimination). OperationSemantics is
deferred to post-self-hosting.

Remaining migration work is tracked in `ROADMAP.md` (Architecture
Migration Workboard, M1–M8).

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
