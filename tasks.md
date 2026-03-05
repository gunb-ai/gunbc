# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)
**Archive**: `TODO/TODONE/tasks-archive-2026-03-02.md` (40 completed items from earlier lanes)

---

## Three Lanes

| Lane | Doc | Goal | Open Items |
|------|-----|------|------------|
| **1. Type System** | [`TODO/type-system.md`](TODO/type-system.md) | Compositional type coverage — decisions obligate, obligations propagate. WS-1 through WS-7. | 29 open + 11 done across 7 workstreams |
| **2. Compiler Debt & App Layer** | [`TODO/gunbc-dag-simplification.md`](TODO/gunbc-dag-simplification.md) | Fix compiler gaps that force runtime bridges. 10 accidental bridges → delete. Each has specific files/LOC to remove. | 10 bridges + app layer cleanup |
| **3. SDLC Pipeline** | [`TODO/sdlc.md`](TODO/sdlc.md) | Run the SDLC pipeline end-to-end. Phase 0 (prove compilation) is a **hard gate**. | 10 done + 9 in progress across 5 phases |

### Design docs

| Doc | Scope |
|-----|-------|
| [`docs/design/v4/compiler-densification-roadmap.md`](docs/design/v4/compiler-densification-roadmap.md) | Prioritized roadmap: kill interpreter, hermeticity, dual-encoding, service codegen |
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
1. Lane 3 Phase 0 (S-1 through S-4) — fix known bugs, prove SDLC compilation. **Hard gate.**
2. Lane 2 "Delete immediately" (bridges 4, 5, 10) — trivial cleanup, ~300 LOC deleted
3. Lane 2 "Compiler fixes" (bridges 1-3, 8-9) — each has specific deletion targets + grep verification
4. Lane 1 WS-1 + WS-3 (no blockers) — in parallel with Lane 2

**Operating principles** (from retrospective):
- Prove before building on top. No Phase N+1 work until Phase N is green.
- Each task names what gets **deleted** and a `grep` command to verify deletion.
- No intermediate abstractions. Go a→f directly.
- `@annotation` is never the final state — go straight to structural blocks.
- Check `Cargo.toml` dependency graphs before moving code between crates.

---

## Completed (archived)

40/40 items complete across earlier lanes:

- **Lane 1: Compiler Pipeline** — 26/26 (C1-C30)
- **Lane 1: Binary Elimination** — 10/10 (A2-A11)
- **Phase 3: Purist Engine** — 4/4 (C28-CT8)
