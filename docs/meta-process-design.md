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

## Phasing

### Phase 1: Bootstrap modeling (unblocked, immediate value)

Model stage0 compilation stages in `dsl/gunbc/bootstrap.dag`.
Declare field propagation rules. Automatic default values for
additive fields. Target: adding a field to Node requires only the
.dag change.

Depends on: nothing (emitter default-value support already landing).

### Phase 2: CI pipeline as multi-artifact emission (after Phase 1)

Two sub-phases:

**2a: GitHub Actions extdeps** — model the platform in
`dsl/extdeps/github/actions.dag`. Runners, step types, logging
annotations (`::error::`, `::group::`), artifact upload/download,
matrix strategies. Pure data modeling, same as existing REST/shell
extdeps.

**2b: CI intent + emission** — declare CI gates in
`dsl/gunbc/ci.dag`. Emit the thin YAML shim + .dag-compiled binary.
The binary reads gate declarations, runs checks, reports results
via the GH Actions logging extdep.

Target: adding a new .dag module type automatically creates CI
obligations. The YAML shim never changes.

Depends on: Phase 1 (establishes the pattern), M4 partially
(multi-artifact emission), Track 14 direction (artifact planning).

### Phase 3: Process modeling (future, after Phases 1-2)

Track dependencies as .dag edges. Readiness computation.
Structural milestone completion.

Depends on: Phases 1-2 (established meta-modeling pattern).

## Connection to Existing Work

| Existing work | Enables |
|--------------|---------|
| M4 single emitter | Meta-processes as emission targets |
| PR #404 emitter default values | Additive bootstrap (Phase 1) |
| Track 13 Phase 7 (.dag as target) | Meta-processes validated by the compiler |
| Track 14 (omni-emission) | Multi-artifact: YAML shim + binary from one source |
| KF-3 verification from structure | Meta-process invariants verified automatically |
| `dsl/extdeps/github/` (existing) | REST API model; actions.dag extends to CI platform |
| `dsl/extdeps/shell.dag` (existing) | Shell transport for bootstrap scripts |
| the-gunbai / gunb.ai (prior repos) | Proven pattern: thin YAML shim → .dag binary |

## Done Criterion

**Phase 1:** `scripts/bootstrap.sh` is generated from
`dsl/gunbc/bootstrap.dag`. Adding a field to Node requires zero
manual stage0 edits.

**Phase 2:** `.github/workflows/ci.yml` is a stable thin shim
(< 30 lines) that calls a .dag-compiled CI binary. Adding a new
emission target creates a CI gate in the binary. The YAML never
changes. `dsl/extdeps/github/actions.dag` models the GH Actions
platform.

**Phase 3:** Track dependencies and milestone readiness are computed
from `dsl/gunbc/process.dag`. ROADMAP.md status sections are
generated, not hand-maintained.
