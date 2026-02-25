# Task Sheet — Active Work

**Last updated**: 2026-02-25
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**: `TODO/TODONE/tasks-completed.md`
**Backlog**: `TODO/backlog.md`

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Active Roadmap

Two design docs define all current work. Tasks below are the next actions
from each; see the docs for full dependency graphs and task tables.

| Doc | Scope | Link |
|-----|-------|------|
| **Foundation Cleanup** | Dead code, NF-7, extern bridge elimination (9→0), compiler features | `docs/design/v4/foundation-cleanup.md` |
| **SDLC Implementation** | Pipeline activation, provider stubs→real, local+cloud e2e | `docs/design/sdlc/implementation-roadmap.md` |

### Dependency between docs

```
foundation-cleanup                  sdlc-implementation
┌──────────────┐                    ┌──────────────────┐
│ FC-CL ────── │ (no deps)          │ SDLC-1:6 ─────── │ (no deps)
│ FC-NF7 ───── │──────────────────▶ │ SDLC-7 (verify)  │
│ FC-P6, FC-P7 │ (parallel)         │ SDLC-8 (e2e)     │
│ FC-CF ────── │ (parallel w/ P6)   │ SDLC-CD (cloud)  │
│ FC-P8 ────── │ (last externs)     └──────────────────┘
└──────────────┘
```

SDLC activation (SDLC-1:6) can start immediately — no foundation dependency.
FC-NF7 unblocks extern bridge deletion steps but not SDLC pipeline work.

---

## Next Actions — Foundation Cleanup

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-CL1 | Delete `core/tool-registry` + `core/tool-registry-macros` | S | Pending | — |
| FC-CL2 | Delete orphaned SDLC Rust stubs | S | Pending | — |
| FC-CL3 | Remove stale `languages.rs` dead_code rule | S | Pending | — |
| FC-NF7 | Lowerer extern func same-module port wiring | L | Pending | — |

After these: promote FC-P6 + FC-P7 (parallel).

## Next Actions — SDLC Implementation

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-1 | Register SDLC in workflow catalog | M | Pending | — |
| SDLC-5 | Local SignalStore provider (file-based) | M | Pending | — |
| SDLC-6 | Local ArtifactStore provider (file-based) | M | Pending | — |

After SDLC-1: promote SDLC-2, SDLC-3, SDLC-4.

---

## Archived Lanes

### Lane 1: Compile+Link Hardening (Complete 2026-02-25)

All tasks NF-1 through NF-6 complete. NF-7 carried forward as FC-NF7.
Detail: `TODO/TODONE/tasks-completed.md`.
