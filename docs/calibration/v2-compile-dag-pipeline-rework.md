# Calibration — v2 `compile.dag` pipeline orchestrator rework

**Session:** keen-ibex-355  
**Status:** investigation / draft PR — operator decision pending  
**Authority:** working v2 compiler (`v2-compiler compile --source-root src/v4`)

## What landed in the draft PR

1. **Pipeline-as-data** — `PipelineStageKind`, `PipelineGatePolicy`, `PipelineStageSpec`, and `data v2_compile_pipeline` declare stage order and gate calibration. Stage order is readable from the data list, not only from `compile_sources` call sequence.
2. **Gate policy functions** — `pipeline_gate_policy` / `pipeline_gate_blocks_emission` replace repeated `filter + count > 0` blocks. `StageComplexity` is `SurfaceOnly` (non-blocking); all other listed stages are `BlockingOnErrors`.
3. **Terse v4-style file header** — Scope / Owns / Consumes / Status / Anchor (Practice 9).
4. **Coproduct tags** — `PipelineStageKind`, `PipelineGatePolicy` marked 🟢; `CompileResult` product in `00_core.dag` flagged 🟡 for Outcome migration (owned by core worker).

## CALIBRATION — gate policy (blocking vs surface)

| Stage | Policy | Rationale |
|-------|--------|-----------|
| tokenize, parse, resolve | Blocking | Frontend bundle halts on parse/resolve errors |
| normalize | Blocking | Bare-container / arity errors must not reach infer |
| infer | Blocking | Type errors must not reach emit (`source_audit` ratchet) |
| complexity | **SurfaceOnly** | Violations surfaced; emission not blocked (CX ratchet) |
| ownership | Blocking | Early Detection invariant |
| artifact-plan, emit | Blocking | Plan/emit failures return empty files |

## What can be killed / cleaned up via modeling (dissolution inventory)

| Finding | Lines (approx) | Disposition | Dissolution |
|---------|----------------|-------------|-------------|
| Hand-rolled stage sequencing | `compile_sources` / `compile_to_resolved` | 🟡 gated | Fold over `v2_compile_pipeline` once stage I/O carriers are uniform (`CompileStage { consumes, produces }` like `bootstrap.dag`) |
| Dag JSON serializer in orchestrator | ~570 (`json_*`, `serialize_*`, `emit_dag_artifact`) | 🔴 dissolve-now candidate | Move to `extdeps/formats/json.dag` structural walk; orchestrator calls `emit_dag_artifact` only |
| `front_end_sources` composite | tokenize+parse+resolve in one fn | 🟡 gated | Split when sibling workers freeze per-stage signatures |
| `emit_artifact` RenderTarget match | 4 arms, distinct RHS | 🟢 | Legitimate per-target dispatch (not template-hole) |
| `ownership_diagnostics` SharedError-only arms | small | 🟢 | Legitimate projection of `OwnershipDecision` coproduct |
| `CompileResult` product in `00_core.dag` | cross-file | 🟡 | Migrate to `Outcome`-style carrier with `00_core` worker |
| Duplicate pipeline bodies | compile_sources vs compile_to_resolved | 🟡 | Shared `ValidatedGraphBundle` + fold-driven runner when types align |

## Template-hole scan (compile.dag)

- `emit_artifact` / `RenderTarget`: 4 arms, 4 distinct backends — **legitimate** (ratio 1.0).
- `pipeline_gate_policy`: 9 arms mapping to 2 policies — **mild category split**; acceptable for calibration (data `v2_compile_pipeline` is the authority; match is projection).

## Compiler homomorphism (Practice 10)

- **Not yet applied** to the Dag JSON block: hand-rolled structural recursion over `Node` / `ExprData` should become a `fold` over `std/node.dag` once a substrate catamorphism exists.
- **Applied** at gate boundaries: single `pipeline_gate_blocks_emission` instead of six copy-pasted filter blocks.

## Sibling coordination

Per-stage modules (`01_tokenize` … `05_emit`, `complexity.dag`) own stage bodies. This PR does **not** change their signatures. Escalate signature changes to PM (`sunny-wolf-435`).

## Operator constraints (this PR)

- **No Rust hand-edits** — only `.dag` + calibration doc. `src/v2/tests/src/source_audit.rs` was reverted after PM clarification; infer gate keeps the legacy `type_errors` filter spelling for the existing ratchet until operator approves a test update.
- **No stage0 regen in this PR** — self-compile blocked on `generated.method_template_projection` (known v2 gap); stage0 freshness is a follow-on when the dedicated worker lands.

## Recommended next steps (operator)

1. Decide whether v2-first modeling migration is the program direction (vs waiting for v4 T-9/T-10 impl).
2. If yes: schedule JSON serializer extraction + `front_end_sources` split as follow-on workers.
3. Land `CompileResult` → `Outcome` on `00_core.dag` in the same train as any orchestrator fold refactor.
