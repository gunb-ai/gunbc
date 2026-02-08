# Minimize Build Pipeline for Tool Targets

**Status**: DONE (2026-02-08)

## Problem

`make gist-recent` compiled the full workspace (26 crates) **4 times** before
the actual 15-crate tool build:

1. `cargo fix --workspace` (preflight-fix) — ALL 26 crates
2. `cargo run -p gunbc-dag --release` (ensure-codegen) — ALL 26 crates (release)
3. `cargo run -p gunbc-dag` (pragma) — ALL 26 crates (dev — separate from #2!)
4. `cargo clippy --all-targets` (lint-upsert) — ALL 26 crates
5. `cargo run -p gunbc-gist` (gist-recent) — 15 crates ← the actual work

Root cause: every tool target depended on `lint-upsert`, which pulled in
`ensure-codegen` + `pragma` + workspace clippy. But the runtime preflight in
each generated binary already handles pragma/clippy/freshness at startup.

## Solution

### Part A — Guardrails

1. **`gunbc-dag/tests/dependency_boundaries.rs`** — cargo_metadata-based dep
   graph assertions: tool crates don't depend on unrelated tools, leaf crates
   have no workspace deps, no upward layer violations.

2. **`tool_targets_use_minimal_prerequisites`** test in render.rs — asserts
   generated CLI tool targets depend on `ensure-codegen`, maintenance targets
   depend on `lint-upsert`.

### Part B — Fix the prerequisite chain

1. Tool target deps: `lint-upsert` → `ensure-codegen` (render.rs)
2. Resource map: `generated_cli` ensure_target → `ensure-codegen` (registry.rs)
3. `ensure-codegen`: removed `preflight-fix` prerequisite
4. Updated all existing test assertions
5. Regenerated Makefile

## Result

`make gist-recent` now triggers at most 2 cargo invocations:
1. `ensure-codegen` — instant on warm cache (binary compiled, manifest fresh)
2. `cargo run -p gunbc-gist` — 15 crates

## What didn't change

- `lint-upsert` definition: still `ensure-codegen + pragma + clippy`
- Maintenance targets (`codegen`, `testgen`, `verify`): still depend on `lint-upsert`
- CI pipeline: unchanged (goes through `make verify`)
- Runtime preflight: unchanged (`ensure_lint_upsert()` at tool startup)
- `preflight-fix`: stays for explicit use

## Files modified

- `gunbc-dag/tests/dependency_boundaries.rs` — new guardrail tests
- `gunbc-dag/src/makegen/render.rs` — tool target deps, ensure-codegen deps, tests
- `gunbc-dag/src/makegen/registry.rs` — generated_cli resource mapping
- `Makefile` — regenerated
