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

### End-state file layout

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

This means `06_pipeline.dag` is not the right end-state name. It is not
"stage 6". It is the driver for the whole compiler, so the end-state
name should be `compile.dag` or `driver.dag`.

### Module classes

Not every compiler-adjacent module is a pipeline stage. The end-state
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
  today, not yet driving compilation; end-state is to sit above emit as
  the selector of artifact targets and boundaries
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

### Emitter end state

The scalable end state is not "no target files". The scalable end state
is "no target-owned traversal".

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

The stable end-state model is:

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

At project scope, artifact planning sits above emit. The end-state
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

### Gap analysis: current tree vs end state

This section is the working delta between the intended architecture above
and the current repository. It is intentionally operational: what is
still structurally wrong, how much re-architecture remains, what counts
as done, and what old entrypoints/helpers should be deleted when the new
shape lands.

#### G1. Stage naming and file ownership

Current state:

- `04_reconcile.dag` still carries the old name even though the intended
  role is `infer` / `typecheck`
- `06_pipeline.dag` is the compiler driver, not a sixth numbered stage
- numbered and unnumbered responsibilities are still mixed in the tree

Target state:

- numbered files are only the six core stages
- driver/orchestrator is `compile.dag` (or `driver.dag`)
- analyses and runtime-side modules are unnumbered

Work / re-architecture:

- low-to-medium; mostly renames plus import/test/doc cleanup

Acceptance criteria:

- `04_reconcile.dag` renamed to `04_infer.dag` (or `04_typecheck.dag`)
- `06_pipeline.dag` renamed to `compile.dag`
- no numbered file exists that is not a core lowering stage
- docs/tests/imports use the new names consistently

Deletion / compatibility cleanup:

- delete the old `06_pipeline` name after callers migrate
- delete stale doc language that calls the driver "stage 6"

#### G2. Artifact planning sits above emit

Current state:

- the current top-level compile path still assumes one target for the
  whole compile
- `Artifact.target` is still a `String`
- `artifact.dag` is explicit-plan-only and not yet driving compilation

Target state:

- infer whole typed graph first
- artifact planning chooses backend per artifact
- emit runs once per artifact
- single-target compile becomes a wrapper over a default one-artifact plan

Work / re-architecture:

- medium; top-level interfaces change, but core stages remain intact

Acceptance criteria:

- `Artifact.target` uses typed `Backend`
- `plan_artifacts(...)` runs after infer and before emit in the driver
- `emit_artifact(...)` is the primary emit entrypoint
- `compile(...)` is implemented as a compatibility wrapper around a
  default artifact plan

Deletion / compatibility cleanup:

- delete the assumption that whole-project compile has exactly one target
- delete `String`-typed artifact target fields once callers migrate

#### G3. Shared emit spine vs per-target walkers

Current state:

- `05_emit.dag` has shared helpers and context-building
- Rust, Python, and Go still each own a full expression dispatcher
- Rust still owns a separate TCO dispatcher, and Python/Go each have
  their own TCO walks as well

Target state:

- `05_emit.dag` owns traversal and dispatch
- target adapters own only irreducible per-target lowering
- adding a new backend does not mean adding another whole emitter walk

Work / re-architecture:

- high; this is the main remaining architecture extraction

Acceptance criteria:

- no target adapter owns a full 20-arm `ExprData` dispatcher
- no target adapter owns a separate whole-tree TCO dispatcher
- per-target modules are leaf renderers/adapters only
- shared emit can drive Rust/Python/Go/Dag through one traversal spine

Deletion / compatibility cleanup:

- delete per-target whole-expression dispatchers after shared dispatch lands
- delete duplicate TCO walkers after shared TCO dispatch lands

#### G4. DAG backend and downstream execution

Current state:

- current code still models backends as native emit targets only
- no canonical DAG backend artifact exists yet
- no v2 DAG runtime/interpreter exists yet

Target state:

- `Dag` is a first-class backend
- `targets/dag/adapter.dag` lowers typed graphs to canonical DAG artifacts
- runtime execution happens in `runtimes/dag/*`
- interpretation and JIT are runtime strategies over the same artifact

Work / re-architecture:

- medium-to-high; mostly new implementation, not a rewrite of core stages

Acceptance criteria:

- `Backend` / target enums include `Dag`
- a canonical DAG artifact schema exists
- compiler can emit that artifact
- runtime/interpreter can execute it outside the core compile stages

Deletion / compatibility cleanup:

- delete wording that says compiler output is only native source files
- delete assumptions that execution must happen through Rust/Python/Go

#### G5. Test generation and downstream validation

Current state:

- test generation has not disappeared, but it has narrowed
- Rust still emits generated tests from `mock_response` fixtures
- Python and Go do not currently have equivalent generated-test paths
- test generation is not yet modeled as a backend-/artifact-level shared system

Target state:

- generated tests are a first-class output of artifact emission
- fixtures (`mock_response`) and boundary contracts can produce hermetic tests
- tests are backend-aware but not hardcoded inside one language emitter

Work / re-architecture:

- medium; current Rust-only path should be preserved, then generalized

Acceptance criteria:

- generated tests still exist after emit refactors
- at least one backend path remains capable of hermetic generated tests
- test generation is surfaced at the artifact/emit contract level, not as
  an implementation detail of one emitter file
- long term: Python/Go/Dag can opt into generated test emission through
  the same artifact-level interface

Deletion / compatibility cleanup:

- delete Rust-only ownership of test generation once shared/adapter-level
  test emission exists
- delete stale doc claims that imply multi-backend generated tests already exist

#### G6. Proof and analysis modules are derivations, not side systems

Current state:

- `complexity.dag` is real and currently always-on in the driver
- `ownership.dag` exists as a proof layer
- these modules are still easy to misread as stray pipeline stages or as
  ad hoc side analyses disconnected from emitted outputs

Target state:

- proof/analysis modules remain separate from lowering
- they derive first-class outputs from the same typed graph: proofs,
  reports, or executable validation obligations
- they may run by default, but do not change program meaning unless they
  are explicitly promoted to a transformation stage

Work / re-architecture:

- low-to-medium; mostly interface clarity, output contracts, and
  preserving cost discipline

Acceptance criteria:

- derivation modules are invoked explicitly by `compile.dag`
- derivation modules have bounded/default cost suitable for always-on use
  when enabled
- outputs from derivation modules are first-class in interfaces/docs, not
  hidden diagnostics-only side effects
- docs clearly separate derivation modules from lowering stages

Deletion / compatibility cleanup:

- delete any future tendency to smuggle rewrites into derivation modules
- delete doc language that treats proofs/tests as second-class cleanup

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

Current status in the tree:

- Rust still emits generated tests from `mock_response` fixture data
- this is currently Rust-specific, not a shared artifact-level facility
- preserving and generalizing generated tests is an explicit migration requirement,
  not something to assume is already solved by the new emitter architecture

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

### One graph, many projections

Runtime code, generated tests, proofs, complexity certificates, and
reports should all fall out of the same typed DAG. They are not separate
representation families; they are different projections or roles over
the same program structure.

The intended rule is:

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

That same formula should cover test generation, ownership proofs,
complexity proofs, policy/data-flow proofs, and future mathematical
proofs. The compiler team maintains the derivation rules and capability
tables; individual programs should not require hand-maintained "testing
gap" lists.

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

The intended end-state taxonomy is:

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

### What exists today

Compiler self-tests exist today and are healthy. Examples include:

- parse/type/emit regression tests in `src/v2/tests/src/lib.rs`
- strict pipeline smoke tests
- Go pipeline smoke tests
- compile-all-modules smoke coverage

Generated backend tests exist today only in a narrow Rust-specific form:

- `05_emit_rust.dag` emits `tests/<module>_test.rs` files for service
  operations that contain `mock_response` fixture data
- these tests are generated from `mock_response` / `mock_*` properties

Important current limitation:

- the current Rust-generated tests are closer to fixture scaffolds than
  full execution/assertion tests; they preserve the slot in the
  interface, but they are not yet the end-state validation story

No equivalent generated-test path currently exists for:

- Python backend
- Go backend
- Dag backend
- artifact boundary integration tests
- trace/repro-driven generated tests

### Generated test gap analysis

#### T1. Preserve generated tests as a first-class emit output

Current state:

- generated tests exist only as an implementation detail of Rust emit

Target state:

- generated tests are a first-class output of artifact emission
- at least one artifact can emit both runtime files and test files

Acceptance criteria:

- `ArtifactOutput` or the equivalent emit contract makes room for test files
- emit refactors do not accidentally drop generated tests

#### T2. Upgrade Rust generated tests from scaffold to validation

Current state:

- Rust-generated tests are created from `mock_response` data, but the
  current emitted bodies are minimal scaffold code

Target state:

- generated Rust tests invoke emitted operations/services
- tests assert observable behavior, not just fixture setup
- dry-run/mock paths remain hermetic and credential-free

Acceptance criteria:

- generated Rust tests execute code paths and contain assertions
- generated tests fail meaningfully when emitted behavior regresses

#### T3. Lift test generation to the artifact/back-end interface

Current state:

- test generation is Rust-local

Target state:

- test generation is modeled at the artifact level
- each backend can opt into generated tests through a common interface

Acceptance criteria:

- test generation is described by shared emit/artifact contracts
- per-backend adapters implement only backend-specific rendering details

#### T4. Add artifact boundary integration tests

Current state:

- `artifact.dag` has boundary modeling and proofs, but no generated
  boundary test emission

Target state:

- boundary plans can emit integration tests that validate adapters and
  protocol assumptions at runtime

Acceptance criteria:

- at least one boundary kind can generate a hermetic integration test
- generated boundary tests are tied to explicit boundary declarations

#### T5. Add Dag-runtime conformance tests

Current state:

- no DAG backend/runtime exists yet, so there is no runtime conformance suite

Target state:

- the same typed program can be exercised against Dag runtime and at
  least one native backend with comparable expected behavior

Acceptance criteria:

- conformance tests exist for at least one shared program shape
- differences are explicit and documented, not accidental

#### T6. Trace/repro tests become generated regressions

Current state:

- `trace.dag` defines the contract, but no test-generation path consumes it

Target state:

- captured repro cases can be re-emitted as hermetic tests
- trace/source-map regressions can be preserved automatically

Acceptance criteria:

- at least one repro case can round-trip into a generated regression test

### Deletion / migration guardrails

- do not delete the existing Rust `mock_response` test path until a
  stronger shared/generated replacement exists
- do not claim "compiler emits tests" in a backend-generic sense until
  at least one non-Rust or artifact-level path exists
- do not bury generated-test interfaces inside one target adapter; keep
  the contract visible at the emit/artifact layer

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

## Layer 5: Compiler Composition

Each stage has a well-typed return. The compile pipeline is linear composition.
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
