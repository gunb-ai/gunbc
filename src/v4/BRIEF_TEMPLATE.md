# v4 — Worker Brief Template

Every worker brief has this shape. It is the structural commitment that prevents v3's failure mode (prose-translation drift across hierarchy layers).

## The shape

```
TASK ID: T-N
FILE: src/v4/<path/to/file>.dag

SCOPE (immutable — declared in the file's header):
  Input:  <type>
  Output: <type>

SUBSTRATE YOU MAY USE (and only these):
  <file>.dag — <concept>
  <file>.dag — <concept>
  ...

SUBSTRATE YOU MAY NOT USE:
  - Any file outside SUBSTRATE YOU MAY USE
  - New files (substrate extension = stop signal)
  - New std/* concepts (substrate extension = stop signal)
  - Hand-Rust shims of any kind

DISCIPLINE (non-negotiable):
  - Pure: no hidden state, no side effects
  - Decidable (INVARIANTS P4): every accepted input terminates
  - Fail-closed (INVARIANTS P3): every invalid input is a typed Diagnostic
  - No __is_X markers, no string sentinels, no escape hatches
  - Cost-of-change: changes here should not ripple to other files
    (if they do, the substrate is wrong — surface, do not work around)

TEST SURFACE:
  src/v4/test/claim/<path>/*.dag — TestClaim data
  Each TestClaim is typed input + expected output, no Rust
  Coverage required: golden path + every Diagnostic path

REFERENCE (study, do not copy):
  <v2 or v3 file path> — prior implementation for context
  <doc anchor> — substrate discipline notes

DEFINITION OF DONE:
  - Implementation present in declared file
  - Compiles via v2 binary (bootstrap)
  - TestClaim suite covers golden + Diagnostic paths
  - WorkerOutput instance authored: declares which substrate fact dissolves
    which prior residual (workflow/worker_output.dag schema)
  - Round-trips through any downstream pipeline stage already implemented

YOUR DECISIONS (the actual modeling work):
  - <decision point 1>
  - <decision point 2>
  - <decision point 3>
  When trade-offs are non-obvious: surface to operator BEFORE deciding

ESCALATION TRIGGERS (stop and surface, do NOT work around):
  - You need a new file
  - You need a new std/* concept
  - You need to split this file into multiple
  - The declared I/O contract is wrong
  - Substrate you may use is insufficient
  - You hit a substrate-design ambiguity that affects the modeling
```

## Why this shape

The shape is the structural fix to v3's failure mode. Every constraint exists because v3 violated it:

- **Immutable I/O contract** — v3 had drift in worker outputs because contracts were prose; here they're declared in the file header before any work begins.
- **Substrate whitelist** — v3 workers reached into adjacent substrate ad-hoc, creating the cascade-edit problem; here the substrate surface is closed.
- **No new files** — v3's paper-shrink V1 (template-relocation to `tools/*.rs.in`) and V2 (module-relocation to `pub mod`) both required adding files; here both are syntactically impossible because the worker has no authority to add files.
- **WorkerOutput instance with `dissolves` field** — v3's "retirement" was a list-length ratchet; here it's a structural predicate the worker declares + the system verifies.
- **Escalation triggers explicit** — v3 had no clear "stop and surface" discipline; here it's a list, and triggering escalation is a feature, not a failure.

## Authoring discipline (for the operator)

When filling in a brief from this template:

1. **The CONTRACT is yours to author once and freeze.** Workers cannot change it. If you can't write the contract precisely, you are not ready to dispatch the task — settle the substrate model first.
2. **The DECISIONS list is the most important section.** The work IS the decisions. List the ones a worker will face. Name the ones that need pre-decided answers vs the ones the worker can decide. The brief succeeds if a worker can finish without escalating; it succeeds even better if necessary escalations happen early.
3. **REFERENCE files are study material, not copy targets.** v2 and v3 files exist to inform modeling, not to be lifted wholesale. The discipline is to understand prior implementations, then model fresh against v4's substrate.
