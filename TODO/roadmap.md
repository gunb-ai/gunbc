# Roadmap: SDLC in Pure DSL

**Target**: SDLC pipeline running holistically in `.dag` — issue intake through
close, claim-based workers, multi-provider adapters, all modeled and executed from DSL.

**Timeframe**: 8-10 weeks (April/May 2026)

**Principle**: Model first. Delete Rust along the way. Every phase ships something
that works — no speculative buildout.

---

## Where we are (Feb 25, 2026)

**DSL readiness for SDLC**: High. All 8 core interfaces designed and proven
(claim store, outcome ledger, signal store, artifact store, issue provider, agent
provider, credential provider, LLM provider). State machines, stage dispatch, retry
budget enforcement — all working in DSL.

**What exists**:
- 2,149 lines of SDLC DSL code (pipelines, stages, providers, profiles)
- 8 interface contracts fully specified
- Stage handlers with LLM integration (design, review, implementation)
- File-based claim store and outcome ledger providers

**What's blocking**:
- NF-4 (collapse link phase) — in progress
- NF-7 (same-module extern func wiring) — blocks shadow→extern conversion
- fold() runtime — broken at resolution, workaround via for-loop
- Pipeline dispatch not wired into workflow catalog
- Profile binding incomplete for real provider backends

**What can be deleted now**:
- `core/tool-registry` + `core/tool-registry-macros` (dead crates)
- `daglang-emit/src/lower_mips.rs` (2,155 lines unused MIPS lowering)
- Orphaned SDLC Rust stubs (spec_builders, dangling resolve tests)

---

## Phase 1: Compiler hardening (Weeks 1-2)

Complete the NF track. This is prerequisite to everything else — it makes the compiler
fail-closed on missing symbols and eliminates the fallback surfaces that cause silent bugs.

### Tasks

| ID | Task | Size | Deps |
|----|------|------|------|
| NF-4 | Collapse link phase into compile-time resolution. Delete Backend trait, link(), SymbolTable, OpRef, IntrinsicOp. | M | In progress |
| NF-5 | Delete fallback surfaces: passthrough controls, stub asset fallbacks, module-name dispatch heuristics. | M | NF-4 |
| NF-6 | Determinism contract: compile receipt digests, CI determinism gates, deterministic diagnostic ordering. | M | NF-5 |
| CL-1 | Delete dead crates: `core/tool-registry`, `core/tool-registry-macros`. Remove from workspace. | S | -- |
| CL-2 | Delete `daglang-emit/src/lower_mips.rs` (2,155 lines). Remove module declaration. | S | -- |
| CL-3 | Delete orphaned SDLC Rust: spec_builders sdlc fns, dangling resolve.rs pipeline tests. | S | -- |

### Deliverable
Compiler fails hard on missing externs. Zero fallback paths. ~4,500 lines of dead
code deleted.

---

## Phase 2: NF-7 + fold() fix (Weeks 2-3)

Unblock the two features that every downstream phase needs.

### Tasks

| ID | Task | Size | Deps |
|----|------|------|------|
| NF-7 | Fix lowerer ExternCall same-module port wiring. Register extern func as callable endpoints.