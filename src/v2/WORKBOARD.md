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

As of 2026-03-15 (post Track A/B/C integration):
- 10 v2 modules (~9,600 lines). All parse with zero diagnostics.
- Generated Rust crate passes `cargo check`, `cargo build`, and `cargo test`.
- 81 active tests pass, 4 ignored (3 slow cargo gates pass, 1 evaluator stack overflow).
- Emission architecture split: target-agnostic core + Rust/Python renderers.
- Type system is honest: no `unit_type()` fabrication, no sentinel values.
- v1 codegen has TCO for self-tail-recursive functions (tokenize_loop is iterative).

### Completed tracks (2026-03-15)

| Track | What | Files |
|-------|------|-------|
| A | Emission split: `05_emit.dag` → core + `05_emit_rust.dag` + `05_emit_python.dag`. `RenderTarget = Rust \| Python`. Pipeline dispatches via `emit_target`. | 00_core, 05_emit, 05_emit_rust (new), 05_emit_python (new), 06_pipeline |
| B | Kill fabrication bugs: `lookup_field_type` → `TypeExpr?`, resolve sentinels → `Option`/panic, `serde_json::Value` fallbacks → `compile_error!`, `ImportNames` sum type. | 00_core, 02_parse, 03_resolve, 04_typecheck, 05_emit |
| C | Tail-call optimization: `Stmt::Loop/Continue/Break` in code_ir, detection + transformation in fn_codegen, all renderers updated. | v1 code_ir, fn_codegen, render_rust, render_go, render_c, testgen |

### Integration findings

- **S82:** Flattened function namespace caused `lookup_func_sig` collision → fixed
- **S83:** Re-entrant evaluator stack overflow on 11 .dag files → self-hosting eliminates
- **S84:** v2 emitter has no TCO pass → critical for self-hosting

## Canonical docs

| Doc | Role | Edit when |
|-----|------|-----------|
| `src/v2/WORKBOARD.md` | Canonical entrypoint for current compiler work | Queue, priorities, parallel lanes, doc map change |
| `src/v2/DESIGN.md` | Target architecture and design rules for v2 | The intended compiler shape changes |
| `src/v2/PERFORMANCE.md` | Time/space complexity audit of all v2 pipeline stages | A perf finding is proven or a structural fix lands |
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

### P0: v2 emitter TCO pass (S84 — CRITICAL for self-hosting)

Track C added TCO to v1's `fn_codegen.rs`. But the v2 emitter
(`05_emit_rust.dag`) does not perform this transformation. When v2
compiles itself, the generated Rust will stack-overflow on recursive
functions like `tokenize_loop`.

Scope:
- New module (e.g. `04b_tco.dag`) that operates on typed IR between
  typecheck and emit
- Detect self-tail-recursive functions (all self-calls in tail position)
- Rewrite to loop+reassign+continue structure
- Per-target renderers emit `loop {}` (Rust) / `while True:` (Python)
- Must handle the common patterns: accumulator-style recursion,
  state-machine recursion (tokenize_loop, parse_*)

### P1: recursive TypedExpr (PERFORMANCE.md P1)

The emitter re-runs `infer_expr` on raw `Expr` subtrees because
`TypedExpr` only carries a type at the root, not recursively. This is
both a correctness risk and the #1 performance bottleneck.

Scope:
- Change `TypedExpr` to carry type info at every subexpression
- Typecheck produces a fully-typed tree in one pass
- Emit reads types directly — no `infer_expr` import
- Affects: 04_typecheck.dag, 05_emit.dag, 05_emit_rust.dag, 05_emit_python.dag

### P2: eliminate O(n²) patterns (PERFORMANCE.md P2–P3)

- Replace `concat(acc, [x])` accumulation with reverse-build-then-reverse
- Index resolve lookups by module name (currently linear scan)
- Fix `check_duplicate_modules` quadratic check

Scope: 01_tokenize.dag, 02_parse.dag, 03_resolve.dag

### P3: Python emission tests (Track D)

Track A added `05_emit_python.dag` but no tests validate the output.

Scope:
- Emit .py from a fixture via `emit_target(typed, Python)`
- Validate syntax (`python -m py_compile` or `ast.parse`)
- Run any emitted Python test suite
- Add to ignored test suite (slow gate)

### P4: namespace collision guard (S82)

`compile_all_modules()` should detect and reject duplicate function
names across modules. Currently silent — last-loaded module wins.

Scope: `src/v2/tests/src/lib.rs` — add a check in `compile_all_modules()`

### P5: harden emitted-crate verification

- Clarify which tests are syntax-only, semantic, build, and runtime
- Tighten generated-crate assertions
- Add Python crate verification (Track D)

### DONE (completed 2026-03-15)

- ~~P2 (old): restore one canonical backend boundary~~ — Track A: `RenderTarget`, `emit_target` dispatch
- ~~Track A: emission architecture split~~ — merged
- ~~Track B: kill fabrication bugs~~ — merged
- ~~Track C: tail-call optimization in v1 codegen~~ — merged
- ~~S82: lookup_func_sig name collision~~ — fixed

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
