# Post PR1-PR5 Hardcoded Reduction Plan (Items 5-8)

> **Update (2026-03-04)**: For the strict hard-error/no-fallback direction on items 5/7/8, see
> [`compositional-type-coverage.md`](./compositional-type-coverage.md) § WS-7.
> This document remains useful as historical context and broad workstream framing.

## Scope

This document covers only the remaining items:

1. ID brittleness / string-coupled classification (item 5)
2. Compiler lint for report coverage (item 6)
3. Broader deterministic compilation contract (item 7)
4. Layer-1 embedding multi-module semantics (item 8)

Already-addressed items (CI `success: true`, documented classifier TODO, documented layer1 TODO, canonical-json help text) are intentionally excluded.

## Design Principles

1. Prefer structured metadata over string-prefix parsing.
2. Keep one authority per behavior (no duplicate hardcoded lists in CLI + emit + tests).
3. Add checks as analyzers/lints first; only promote to hard errors after migration.
4. Preserve current behavior unless a stricter behavior is explicitly enabled.

## Workstream A: Reduce String-Coupled Classification (Item 5)

### Current brittleness

- Service phase detection in emit uses name prefixes (`service_transport::prepare::...`) in `service_emit.rs`.
- Layer1 handler classification includes module-name string tables in `rust_exec_runtime.rs`.
- `classify_by_name` in `computation.rs` still uses name-prefix heuristics for `load_` / `render_`.

### Proposal A1: Service phase should be structural, not name-based

Use obligation category as canonical phase for service codegen:

- `ServiceTransportPrepare` => prepare
- `ServiceTransportExecute` => execute
- `ServiceTransportParse` => parse

Implementation shape:

- Extend collected symbol metadata in emit to carry service phase (derived from `LoweredOp` obligation).
- Change service emit entrypoints to branch on phase enum, not `raw_name.starts_with(...)`.

Primary files:

- `core/daglang/daglang-emit/src/lib.rs`
- `core/daglang/daglang-emit/src/service_emit.rs`

### Proposal A2: Remove dead/legacy name paths where primitive kind already exists

`content_upsert` expansions are already emitted as `PrimitiveOpKind::{IoPrepareFileRead, IoExecuteFileRead, CompareEquality, IoPrepareFileWrite, IoExecuteFileWrite}`.

Action:

- Remove (or de-prioritize behind fallback path) `content_upsert::...` handling in `classify_by_name`.
- Keep one classification path: primitive kind for content-upsert semantics.

Primary file:

- `core/daglang/daglang-emit/src/computation.rs`

### Proposal A3: Convert module name tables to capability tables (incremental)

For layer1 handler classification, module-name tables become more stable if they map to declared capability profiles rather than string literals.

Phase 1 (low churn):

- Centralize module->handler fallback mapping in one constant map in emit (single source of truth).

Phase 2 (stronger):

- Derive module capability profile from lowered/derived metadata, then classify by profile.

Primary file:

- `core/daglang/daglang-emit/src/rust_exec_runtime.rs`

## Workstream B: Report Coverage Lint (Item 6)

### Goal

Detect drift where pipeline report aggregates omit semantically relevant stages.

### Why not simple name matching

Direct stage-name matching is brittle (`codegen_stage` may be reported as `"codegen"`).

### Proposed lint model

Use data provenance, not label strings:

1. Build `producer_map`: bound variable -> stage that binds it.
2. Detect report-stage stage-result collections by type shape (`List<StageResult>`), not function name.
3. For each `StageResult` entry expression, collect referenced identifiers.
4. Map referenced identifiers back to producer stages via `producer_map`.
5. `covered_stages` = union of producer stages referenced by report entries.
6. `required_stages` = stages (excluding report stage) that produce values consumed by downstream stages (or by report stage directly).
7. Lint on `missing = required_stages - covered_stages`.

This avoids hardcoding `stage_result` / `stage_from_output` names and avoids fragile string-stage-label comparisons.

### Delivery strategy

Do this as a dedicated lint command first (non-breaking):

- `daglang lint-report-coverage <file.dag|dir> [--format text|json]`

Then optionally integrate into `check/compile` as warn/error mode after migration.

Primary files:

- `core/daglang/daglang-typecheck/src/lib.rs` (analysis utilities)
- `core/daglang/daglang-cli/src/main.rs` (new command parse/help)
- `core/daglang/daglang-cli/src/commands.rs` (command dispatch/output)

## Workstream C: Determinism Contract Expansion (Item 7)

### Current baseline

- Canonical-json determinism exists for single-file compile.

### Proposal

Add deterministic tests at higher-complexity surfaces:

1. Canonical-json determinism for `dsl/pipelines/ci.dag`.
2. Canonical-json determinism for directory input (`dsl/`) where applicable.
3. Emit-mode determinism:
   - same run twice -> same `emit_manifest.json` content
   - emitted file list and hash ordering stable

Primary file:

- `core/daglang/daglang-cli/tests/compile_commands.rs`

Secondary checks:

- keep existing parity/determinism tests unchanged.
- add explicit assertions for stable ordering (paths sorted, hashes stable).

## Workstream D: Layer1 Embedding Generalization (Item 8)

### Current brittleness

- CLI post-processes compile output and pushes `src/embedded_makefile.txt` only when module list contains `tools.makegen`.
- Native backends in emit use `is_makegen_module()` string checks.

### Proposal

Move embedding decision and asset requirements into emit/driver contract:

1. Introduce generic embedded asset keys in compile options:
   - e.g. `EmbeddedAssetKey::MakegenContent`
2. Emit runtime declares required assets based on handler usage (not module name checks).
3. Driver injects provided assets once; CLI no longer scans module names.
4. Native backends use same asset-presence contract instead of `is_makegen_module()`.

Net result:

- No CLI hardcoded module checks for layer1 embedding.
- One embedding contract for Rust/Go/C/MIPS.

Primary files:

- `core/daglang/daglang-driver/src/lib.rs`
- `core/daglang/daglang-cli/src/commands.rs`
- `core/daglang/daglang-emit/src/lib.rs`
- `core/daglang/daglang-emit/src/rust_exec_runtime.rs`

## Proposed Implementation Order

1. Workstream C (determinism tests) first, as safety net.
2. Workstream A1/A2 (service phase structuralization + content_upsert cleanup).
3. Workstream D (layer1 embedding contract generalization).
4. Workstream B (report coverage lint command).
5. Optional: Workstream A3 phase-2 capability profile.

## Acceptance Criteria

1. No new hardcoded module-name checks in CLI for embedding.
2. Service codegen phase decisions do not depend on string prefix parsing.
3. CI pipeline canonical-json determinism test passes (two-run byte equality).
4. Report coverage lint returns deterministic output and identifies missing stage coverage via provenance, not string stage labels.
5. Existing compile/emit behavior remains backward-compatible in default mode.

## Open Decisions For Review

1. Report lint severity default:
   - Option A (recommended): separate lint command only (non-breaking).
   - Option B: warning during `check/compile`.
   - Option C: hard error by default.

2. Scope of required stage coverage:
   - Option A (recommended): stages with outputs consumed downstream.
   - Option B: all non-report stages.
   - Option C: explicit allowlist/annotation model.

3. Determinism test scope:
   - Option A (recommended): add CI pipeline canonical-json + emit-manifest stability.
   - Option B: include SDLC pipeline canonical-json in same pass.
