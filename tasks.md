# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)
**Archive**: `TODO/TODONE/tasks-archive-2026-03-02.md` (40 completed items from earlier lanes)

---

## Three Lanes

| Lane | Doc | Goal | Open Items |
|------|-----|------|------------|
| **1. Type System** | [`TODO/type-system.md`](TODO/type-system.md) | Compositional type coverage — decisions obligate, obligations propagate. WS-1 through WS-7. | 40 items across 7 workstreams |
| **2. Compiler Debt & App Layer** | [`TODO/gunbc-dag-simplification.md`](TODO/gunbc-dag-simplification.md) | Fix compiler gaps that force runtime bridges. Rename/clean gunbc-dag. Each bridge is a compiler lesson. | 17 items across 4 phases |
| **3. SDLC Pipeline** | [`TODO/sdlc.md`](TODO/sdlc.md) | Run the SDLC pipeline end-to-end. This is the objective of all the compiler work. | 19 items across 4 phases |

### Design docs

| Doc | Scope |
|-----|-------|
| [`docs/design/v4/compositional-type-coverage.md`](docs/design/v4/compositional-type-coverage.md) | Type system vision, audit, gaps, workstreams, worked examples |
| [`docs/design/sdlc/domain-modeling-comprehensive.md`](docs/design/sdlc/domain-modeling-comprehensive.md) | SDLC entity/relationship/state machine model |
| [`docs/design/sdlc/production-gap-analysis.md`](docs/design/sdlc/production-gap-analysis.md) | SDLC activation blockers |

### Dependency between lanes

```
Lane 1 (type system)  ──→  Lane 3 (SDLC) uses the type system
Lane 2 (compiler debt) ──→  Lane 3 (SDLC) needs working compilation pipeline
```

Lane 1 and Lane 2 can proceed in parallel. Lane 3 Phase 0 (prove it compiles) can start now — it doesn't need type system improvements, just basic compiler correctness.

**Recommended start order**:
1. Lane 3 Phase 0 (S-1 through S-4) — fix known bugs, prove SDLC compilation
2. Lane 2 Phase 1 (CL-1 through CL-5) — compiler fixes that eliminate bridges
3. Lane 1 WS-1 + WS-3 (no blockers) — in parallel

---

## Completed (archived)

40/40 items complete across earlier lanes:

- **Lane 1: Compiler Pipeline** — 26/26 (C1-C30)
- **Lane 1: Binary Elimination** — 10/10 (A2-A11)
- **Phase 3: Purist Engine** — 4/4 (C28-CT8)
