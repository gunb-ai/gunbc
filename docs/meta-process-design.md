# Meta-Process Design: Compositional Process Modeling

## Thesis

.dag is a language for declaring compositional facts. The compiler's own
processes — bootstrap, CI, testing, release — are processes with
dependencies, stages, and invariants. Model them in .dag.

When the compiler models its own processes, manual coordination
dissolves. Adding a feature automatically updates the bootstrap, CI,
and test pipelines because the dependencies are declared, not
remembered.

## Core Principle: Upsert Semantics

Traditional CI is a hand-maintained list: add a feature, remember to
update the CI config, remember to add the test gate, remember to update
the bootstrap sequence. Forgetting any step is silent — you only
discover the gap when something breaks.

Process modeling in .dag uses **upsert semantics**: adding a new
declared fact (a type, a file, a target, a field) automatically
produces the corresponding process step because the process is
**derived from the declarations**. You can't "forget" to add something
to CI because CI is a consequence of structure, not a separate
maintenance obligation.

## Three Domains

### Domain 1: Bootstrap Pipeline

**Problem:** The bootstrap pipeline is implicit knowledge. Adding
structural changes (new fields, new types on Node) requires manual
stage0 edits and multi-phase bootstrap coordination. PR #404's
`ident: Int` chicken-and-egg is the canonical example — the old
compiler doesn't know about new fields, so regen omits them.

**Target state:** Bootstrap stages are declared facts:

```
dsl/gunbc/bootstrap.dag

type BootstrapStage {
  inputs: List<SourceFile>
  outputs: List<GeneratedFile>
  depends_on: List<BootstrapStage>
  strategy: BootstrapStrategy       // SinglePass | TwoPhase | Additive
}

type BootstrapStrategy = SinglePass | TwoPhase | Additive
```

- Which .dag files feed which stage0 outputs (declared edges)
- Which changes require two-phase bootstrap vs single-pass regen
  (derived from change classification)
- Default value strategy for additive fields (the emitter already
  supports this after the PR #404 work)
- Fixed-point convergence check as a declared pipeline step, not
  a manual `diff`

**Concrete outcome:** Adding a field to Node requires only the .dag
change. The bootstrap pipeline knows what to do.

### Domain 2: CI Pipeline

**Problem:** CI gates are a hand-maintained list in
`.github/workflows/` and ROADMAP.md. Adding a new emission target,
test category, or .dag module requires manual CI updates. The current
CI gate table (ROADMAP.md) is already drift-prone — ratchet values
get stale, new gates get forgotten. Generated YAML is hard to debug.

**Key insight (from the-gunbai / gunb.ai):** YAML is a transport
constraint — GitHub Actions requires it — not the CI logic itself.
The same pattern as REST: you don't generate HTTP, you generate code
that speaks HTTP. The design:

1. **Model GitHub Actions as extdeps** — runners, steps, logging
   commands, artifact upload/download, matrix strategies. These are
   external platform facts, same category as REST endpoints or
   shell transports. Lives in `dsl/extdeps/github/actions.dag`.

2. **Declare CI intent in .dag** — the meta-process code declares
   what gates exist, what they check, when they run. This is the
   actual CI logic, compiled like any other .dag program.

3. **Emit a thin YAML shim** — the minimum YAML that GitHub requires
   (workflow trigger, runner spec, checkout step). The shim calls a
   .dag-compiled binary that IS the CI runner.

4. **The binary does the work** — runs gates, emits GitHub Actions
   logging annotations (`::error::`, `::group::`, etc.) via the
   actions extdep, reports results. Debugging happens in .dag code,
   not in generated YAML.

```
dsl/extdeps/github/actions.dag    -- GH Actions platform model
dsl/gunbc/ci.dag                  -- CI intent declarations

Emitted artifacts (multi-artifact):
  .github/workflows/ci.yml        -- thin YAML shim (< 30 lines)
  target/ci-runner                 -- .dag-compiled binary
```

This is a **multi-artifact emission** (Track 14 direction): one .dag
source produces both the YAML shim and the binary, coordinated by
the artifact plan.

- Each .dag file type has declared testing requirements
- Each emission target has declared cross-language equivalence
  obligations
- Each ratchet has declared gate behavior (blocking vs advisory,
  direction, threshold)
- Adding a feature produces CI coverage automatically

**Concrete outcome:** Adding a new emission target language creates
the cross-language equivalence test gate. The binary gains the gate
logic. The YAML shim doesn't change. No YAML editing ever.

### Domain 3: Development Process (future)

Model tracks, milestones, dependencies, and readiness as .dag
declarations. Track dependencies become edges. Readiness is computed.
Milestone completion is structural (all constituent criteria met).

Lower priority than domains 1 and 2. Completes the picture: the
compiler models everything, including its own development.

## Architecture

Meta-processes use two layers: **extdeps** (platform facts) and
**gunbc/** (compiler-specific process declarations).

```
dsl/extdeps/github/actions.dag   -- GH Actions platform model (runners,
                                     logging annotations, artifact ops)
dsl/gunbc/
  bootstrap.dag                  -- stage compilation pipeline
  ci.dag                         -- CI intent declarations
  process.dag                    -- development track modeling (future)
```

These files compile with the same compiler. The CI pipeline is a
**multi-artifact emission**:

```
dsl/gunbc/ci.dag ──(compile)──▶ .github/workflows/ci.yml  (thin shim)
                               + target/ci-runner           (binary)
```

The shim is a transport artifact (GitHub requires YAML). The binary
is the actual CI logic. This is the same pattern as REST services:
the .dag code declares intent, the emitter produces both the
transport glue and the implementation.

**Key insight:** meta-processes are another emission target. The
compiler already transforms .dag declarations into mechanical output.
Meta-process modeling asks it to transform declarations about itself.
The YAML shim is no different from an HTTP route handler — it's the
platform's required entry point, generated once and stable.

## The Interpreter: Execution as an Emission Target

The v2 compiler has no interpreter — it's a pure compiler that emits
text files. v1 had one (archived). For meta-processes, this creates
a bootstrapping problem: running bootstrap.dag or ci.dag requires
compiling to Rust, building with cargo, then executing. That's
blocked on M3 (working end-to-end emission).

**The fix: bring back execution as an emission target.**

The compiler already has a `Dag` render target that serializes the
typed graph as JSON (`emit_dag_artifact` in `compile.dag`). The
interpreter extends this: instead of serializing the validated IR,
evaluate it directly.

```
.dag source → tokenize → parse → resolve → infer → CX → ownership → emit
                                                                      ↓
                                                       ┌──────────────┼───────────┐
                                                       ↓              ↓           ↓
                                                   emit_rust     emit_python   emit_eval
                                                   (text files)  (text files)  (evaluate)
```

### Why this is tractable

.dag's bounded kernel makes interpretation safe by construction:

- **All programs terminate** — decidable, bounded iteration
- **All data is finite** — Bit/Word64, no unbounded allocation
- **No mutation** — pure values, reference counting sufficient
- **No side effects** — except through declared service boundaries
- **Three iteration primitives** — fold, descend, repeat have
  clear operational semantics

The core evaluator is a tree-walker over the post-validation IR:

1. Function calls — resolve function, evaluate arguments, evaluate body
2. Match expressions — pattern match on Conj/Disj values
3. Let bindings, literals, field access, constructors
4. fold/descend/repeat — bounded iteration over values
5. Service dispatch — shell, REST, etc. via existing transports
6. Pipe expressions — `x |> f(y)` desugars to `f(x, y)`

No GC needed (immutable values, RC). No infinite loop risk.
No mutation semantics. Straightforward.

### What this means for users

Most users don't care about omni-emission. They want:

```
dag compile foo.dag    # validate (type-check, CX, ownership)
dag run foo.dag        # execute directly
```

Emission to Rust/Python/Go is a **performance optimization** for
production deployment — not the primary development workflow. The
interpreter is the default execution path.

### What this means for M5

The interpreter **unblocks all meta-process phases**:

| Without interpreter | With interpreter |
|--------------------|-----------------|
| bootstrap.dag → compile to Rust → cargo build → run | `dag run bootstrap.dag` |
| ci.dag → compile + build + emit YAML + build binary | `dag run ci.dag` |
| Blocked on M3/M4 emission quality | Unblocked immediately |

The thin YAML shim (Domain 2) still makes sense for production CI,
but development and iteration use the interpreter directly.

### Architectural location

The interpreter lives alongside other emission targets:

- `dsl/extdeps/languages/dag/` — already exists (syntax, types,
  emit data for debug output)
- The evaluator is a Rust module in the compiler binary (not .dag —
  it's infrastructure, like the parser)
- Entry point: `RenderTarget::Dag` already dispatches to
  `emit_dag_artifact`. Extend or add `RenderTarget::Eval` that
  invokes the evaluator instead of serializing to JSON
- Service dispatch: evaluate service calls by routing to the
  appropriate transport handler (shell → subprocess, REST → HTTP
  client, etc.)

### Interpreter phases

**I-1: Pure evaluation** — evaluate pure .dag functions (no services).
Covers data declarations, type functions, pure computation. Already
enough to validate meta-process models.

**I-2: Shell service dispatch** — evaluate shell.Exec, cargo.Build,
etc. by invoking subprocesses. Enough for bootstrap.dag and ci.dag.

**I-3: REST service dispatch** — evaluate REST operations by making
HTTP calls. Enough for review.dag and service integration.

**I-4: Full transport coverage** — all declared transports dispatch
to runtime handlers.

## Phasing (revised)

### Phase 0: Interpreter (new, highest priority)

Build the .dag interpreter as `I-1` (pure eval) + `I-2` (shell
dispatch). This unblocks all subsequent phases — meta-processes
can run without compiling to Rust.

Depends on: nothing. The validated IR already exists post-ownership.
The evaluator is a Rust module reading the existing Node/Conj/Disj
representation.

### Phase 1: Bootstrap modeling (unblocked with interpreter)

Model stage0 compilation stages in `dsl/gunbc/bootstrap.dag`.
Declare field propagation rules. Automatic default values for
additive fields. Target: adding a field to Node requires only the
.dag change.

Depends on: Phase 0 I-2 (shell dispatch for cargo/regen commands).

### Phase 2: CI pipeline as multi-artifact emission

Two sub-phases:

**2a: GitHub Actions extdeps** — model the platform in
`dsl/extdeps/github/actions.dag`. Runners, step types, logging
annotations (`::error::`, `::group::`), artifact upload/download,
matrix strategies. Pure data modeling, same as existing REST/shell
extdeps.

**2b: CI intent + emission** — declare CI gates in
`dsl/gunbc/ci.dag`. For development: `dag run ci.dag` executes
gates directly via the interpreter. For production: emit a thin
YAML shim + compiled binary (deferred until M4 multi-artifact
lands).

Target: adding a new .dag module type automatically creates CI
obligations.

Depends on: Phase 0 (interpreter), Phase 1 (establishes the
pattern). Production YAML emission deferred to M4.

### Phase 3: Process modeling (future, after Phases 1-2)

Track dependencies as .dag edges. Readiness computation.
Structural milestone completion.

Depends on: Phases 1-2 (established meta-modeling pattern).

## Connection to Existing Work

| Existing work | Enables |
|--------------|---------|
| `RenderTarget::Dag` + `emit_dag_artifact` | Interpreter extends existing Dag target from JSON dump to evaluation |
| `dsl/extdeps/languages/dag/` (existing) | Syntax, types, emit data already modeled for Dag target |
| M4 single emitter | Production CI binary (Phase 2b, deferred) |
| PR #404 emitter default values | Additive bootstrap (Phase 1) |
| Track 14 (omni-emission) | Multi-artifact: YAML shim + binary from one source |
| KF-3 verification from structure | Meta-process invariants verified automatically |
| `dsl/extdeps/github/` (existing) | REST API model; actions.dag extends to CI platform |
| `dsl/extdeps/shell.dag` (existing) | Shell transport for interpreter service dispatch |
| `dsl/extdeps/cargo.dag` (existing) | Build/Test/Clippy already modeled for shell transport |
| the-gunbai / gunb.ai (prior repos) | Proven pattern: thin YAML shim → .dag binary |

## Done Criterion

**Phase 0:** `dag run hello.dag` executes a pure .dag program.
`dag run bootstrap.dag` runs the bootstrap pipeline via shell
dispatch. No Rust compilation of the .dag program needed.

**Phase 1:** `dag run bootstrap.dag` manages stage0 regeneration.
Adding a field to Node requires only the .dag change.

**Phase 2:** `dag run ci.dag` executes CI gates locally during
development. Production CI uses a thin YAML shim (< 30 lines)
calling the interpreter or a compiled binary.
`dsl/extdeps/github/actions.dag` models the GH Actions platform.

**Phase 3:** Track dependencies and milestone readiness are computed
from `dsl/gunbc/process.dag`. ROADMAP.md status sections are
generated, not hand-maintained.
