# Task Sheet — Active Work

**Last updated**: 2026-02-25
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**:
- Completed items: `TODO/TODONE/tasks-completed.md`
- Archived lane detail snapshot: `TODO/TODONE/2026-Q1/tasks-archived-lanes-2026-02-25.md`
- Backlog: `TODO/backlog.md`

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

**Scheduling policy (2026-02-25)**: No active lanes. Only remaining item is NF-7 (backlog P1).

## Delivery Lane Summary

| Lane | Status | Remaining |
|------|--------|-----------|
| 1: Mega lane — compile+link hardening | Complete | NF-7 moved to backlog |

---

## Lane 1: Mega Lane — Compile+Link Hardening (Complete 2026-02-25)

**Goal**: Compile+link no-fallback contract for extern funcs/assets. Removal of known fail-open/codegen workaround paths.

All tasks NF-1 through NF-6 complete. NF-7 (lowerer extern func wiring) moved to backlog as P1.
See `TODO/TODONE/tasks-completed.md` for detail.

**What didn't land (carry-forward to backlog)**:
- **NF-7**: Converting shadow `fn` items to `extern func` in DSL files.
- Current blocker: lowerer does not wire `ExternCall` output ports correctly for same-module calls from function bodies, which breaks codegen data flow.
- Design doc complete: `docs/design/v4/externcall-same-module-port-wiring.md`
- Shadow function bodies stay in place with clear documentation for now.
- This is a lowerer limitation, not a design choice.
- Blocks Phases 5-8 of extern bridge elimination (`docs/design/v4/extern-bridge-gap-analysis.md`).
