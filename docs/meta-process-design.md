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
get stale, new gates get forgotten.

**Target state:** CI gates are derived from .dag declarations:

```
dsl/gunbc/ci.dag

type CIGate {
  name: String
  command: String
  blocking: Bool
  derives_from: CIGateSource
}

type CIGateSource =
  | RatchetGate { metric: String, direction: RatchetDirection }
  | TestSuiteGate { test_pattern: String }
  | FreshnessGate { generated_from: List<SourceFile> }
```

- Each .dag file type has declared testing requirements
- Each emission target has declared cross-language equivalence
  obligations
- Each ratchet has declared gate behavior (blocking vs advisory,
  direction, threshold)
- Adding a feature produces CI coverage automatically

**Concrete outcome:** Adding a new emission target language creates
the cross-language equivalence test gate. Adding a new ratchet
metric creates the CI check. No YAML editing.

### Domain 3: Development Process (future)

Model tracks, milestones, dependencies, and readiness as .dag
declarations. Track dependencies become edges. Readiness is computed.
Milestone completion is structural (all constituent criteria met).

Lower priority than domains 1 and 2. Completes the picture: the
compiler models everything, including its own development.

## Architecture

Meta-processes live in `dsl/gunbc/`:

```
dsl/gunbc/
  bootstrap.dag    -- stage compilation pipeline
  ci.dag           -- CI gate derivation
  process.dag      -- development track modeling (future)
```

These files compile with the same compiler. They produce:
- Shell scripts (bootstrap automation, CI steps)
- Configuration (GitHub Actions YAML — eventually)
- Validation (is the CI config complete? are bootstrap stages
  consistent?)

**Key insight:** meta-processes are another emission target. The
compiler already transforms .dag declarations into mechanical output.
Meta-process modeling asks it to transform declarations about itself.

## Phasing

### Phase 1: Bootstrap modeling (unblocked, immediate value)

Model stage0 compilation stages in `dsl/gunbc/bootstrap.dag`.
Declare field propagation rules. Automatic default values for
additive fields. Target: adding a field to Node requires only the
.dag change.

Depends on: nothing (emitter default-value support already landing).

### Phase 2: CI gate derivation (after Phase 1)

Model CI gates as declared facts. Derive gate commands from project
structure. Target: adding a new .dag module type automatically
creates CI obligations.

Depends on: Phase 1 (establishes the pattern), M4 partially
(emit to shell for gate scripts).

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
| KF-3 verification from structure | Meta-process invariants verified automatically |

## Done Criterion

**Phase 1:** `scripts/bootstrap.sh` is generated from
`dsl/gunbc/bootstrap.dag`. Adding a field to Node requires zero
manual stage0 edits.

**Phase 2:** `.github/workflows/ci.yml` gates are derived from
`dsl/gunbc/ci.dag`. Adding a new emission target creates a CI gate
with no YAML editing.

**Phase 3:** Track dependencies and milestone readiness are computed
from `dsl/gunbc/process.dag`. ROADMAP.md status sections are
generated, not hand-maintained.
