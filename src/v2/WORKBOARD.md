# v2 Compiler Workboard

This is the canonical operational doc for the v2 compiler.

Use this file for:
- current status
- work queue
- parallel workstream planning
- deciding which other doc to read next

Do not use this file for deep design prose or exhaustive audit notes. Keep
those in the docs listed below.

## Current state

As of 2026-03-14:
- The v2 compiler bootstrap path exists and emits a generated Rust crate.
- The generated crate passes `cargo check`.
- `cargo build` and runtime smoke tests exist as slower gates.
- The active team priority is getting the v2 compiler working end-to-end without
  destabilizing the current bootstrap path.

This means the repo needs two things at once:
- one trunk thread fixing whatever blocks the bootstrap path
- several low-conflict side threads that keep momentum without stomping on the
  active `.dag` compiler modules

## Canonical docs

| Doc | Role | Edit when |
|-----|------|-----------|
| `src/v2/WORKBOARD.md` | Canonical entrypoint for current compiler work | Queue, priorities, parallel lanes, doc map change |
| `src/v2/DESIGN.md` | Target architecture and design rules for v2 | The intended compiler shape changes |
| `src/v2/POSTMORTEM.md` | Exhaustive debt ledger and audit evidence | A new finding is proven and needs permanent record |
| `src/v1/SUSTAINABILITY.md` | Root-cause theory across the codebase | A finding changes the sustainability model itself |
| `README.md` | Public repo overview | External-facing overview changes |

### Duplicate doc policy

`DESIGN-v2-compiler.md` overlaps with `src/v2/DESIGN.md`.

From now on:
- `src/v2/DESIGN.md` is the canonical design doc
- `DESIGN-v2-compiler.md` is legacy context only
- new design edits should not be made in both places

## Working rules

### Rule 1: protect the active bootstrap path

Assume these files are the high-conflict trunk until the compiler is stable:
- `src/v2/00_core.dag`
- `src/v2/01_tokenize.dag`
- `src/v2/02_parse.dag`
- `src/v2/03_resolve.dag`
- `src/v2/04_typecheck.dag`
- `src/v2/05_emit.dag`
- `src/v2/06_pipeline.dag`

Broad rewrites in those files should happen in one coordinated thread, not in
parallel cleanup work.

### Rule 2: side threads should be file-disjoint whenever possible

Preferred side-thread areas:
- `src/v2/tests/`
- docs under `src/v2/` and repo root
- narrowly scoped new files
- backlog extraction and task slicing

### Rule 3: design first, bulk refactor later

If a fix touches the backend boundary, type boundary, or emitted crate shape:
- write down the intended boundary here first
- land the design change in `src/v2/DESIGN.md`
- delay the code refactor until it will not collide with bootstrap stabilization

## Parallel work lanes

| Lane | Goal | Safe scope now | Avoid for now |
|------|------|----------------|---------------|
| L0 | Bootstrap stabilization | Minimal fixes in `src/v2/*.dag` plus targeted tests | Broad cleanups, renames, cross-file reshapes |
| L1 | Test hardening | `src/v2/tests/src/lib.rs`, emitted-crate checks, fixture cleanup | Rewriting compiler semantics while adding tests |
| L2 | Backlog extraction | `src/v2/POSTMORTEM.md`, this file, issue slicing docs | Editing compiler code directly |
| L3 | Doc convergence | `README.md`, `src/v2/DESIGN.md`, `DESIGN-v2-compiler.md`, this file | Mixing doc cleanup with semantic code changes |
| L4 | Backend-boundary prep | Design notes, tests, stubs for backend-neutral pipeline API | Landing full multi-backend implementation mid-bootstrap |

## Near-term queue

### P0: keep the bootstrap path working

Definition:
- generated Rust crate stays at least `cargo check` clean
- blocking parser, resolver, typecheck, and emit bugs get fixed first

Scope:
- correctness bugs in `src/v2/*.dag`
- smallest possible fixes
- every fix should add or tighten a test if feasible

### P1: stop losing work to one-thread-only execution

Do in parallel:
- extract actionable tasks from `src/v2/POSTMORTEM.md`
- keep this workboard current
- split tasks by touched files, not by topic names

Definition of done:
- at least 3-5 side-thread tasks always exist that do not require editing the
  same `.dag` compiler file

### P2: restore one canonical backend boundary

Current mismatch:
- design says pre-emit pipeline should be backend-neutral
- implementation currently calls `emit_rust` directly in `src/v2/06_pipeline.dag`

Do now:
- capture the intended interface and migration steps
- add tests or doc notes that make the boundary explicit

Do later:
- land the actual `Backend` parameter and dispatch when the trunk is quieter

### P3: harden emitted-crate verification

Safe parallel tasks:
- clarify which tests are syntax-only, semantic, build, and runtime
- tighten generated-crate assertions
- reduce reliance on ad hoc string checks where a stronger test is available

Primary files:
- `src/v2/tests/src/lib.rs`
- `src/v1/07_emit/daglang-emit/src/v2_crate_emit.rs`

### P4: keep docs converged

Current problem:
- design intent, postmortem findings, and active work status live in separate
  places with no clear entrypoint

Policy:
- status and queue live here
- architecture lives in `src/v2/DESIGN.md`
- exhaustive findings live in `src/v2/POSTMORTEM.md`

## Candidate slices from current backlog

| Slice | Primary files | Parallel with L0? | Notes |
|------|---------------|-------------------|-------|
| Emitted-crate gate cleanup | `src/v2/tests/src/lib.rs`, `src/v1/07_emit/daglang-emit/src/v2_crate_emit.rs` | Yes | Strengthen build/runtime gates without rewriting compiler semantics |
| Postmortem extraction | `src/v2/POSTMORTEM.md`, `src/v2/WORKBOARD.md` | Yes | Turn large findings into assignable file-scoped tasks |
| Doc convergence | `README.md`, `src/v2/DESIGN.md`, `DESIGN-v2-compiler.md` | Yes | Keep entrypoints clean and stop duplicate planning notes |
| Backend-neutral pipeline boundary | `src/v2/06_pipeline.dag`, tests, docs | Usually no | Design is ready to define now; code change should be coordinated |
| Null-coalesce and `for` semantics | `src/v2/02_parse.dag`, `src/v2/05_emit.dag`, tests | No | Semantic fix in hot files; keep on trunk |
| Alias and serde-key fidelity | `src/v2/04_typecheck.dag`, `src/v2/05_emit.dag`, tests | No | Also hot-path work; keep coordinated with trunk |

## Not now

Delay these until the v2 bootstrap path is stable:
- broad v1 cleanup that does not unblock self-hosting
- full multi-backend implementation
- deleting old bootstrap scaffolding without a passing replacement
- mass renames across `src/v2/*.dag`
- large-formatting or style-only rewrites of compiler modules

## Immediate candidate tasks

These are safe examples of work that can proceed without competing for the same
files as the main bootstrap thread.

1. Extract the highest-priority open items from `src/v2/POSTMORTEM.md` into a
   small checklist grouped by touched files.
2. Strengthen emitted-crate tests in `src/v2/tests/src/lib.rs` so current
   invariants fail faster.
3. Document the exact backend-neutral pipeline signature that should replace the
   direct `emit_rust` call in `src/v2/06_pipeline.dag`.
4. Collapse doc duplication by routing all new v2 planning updates to this file
   and all design updates to `src/v2/DESIGN.md`.

## Update protocol

When a new thread starts, record:
- lane
- goal
- touched files
- whether it is safe to run in parallel with L0

When a thread ends, record:
- what was proven
- what files remain hot
- whether the result changed design, backlog, or only local implementation
