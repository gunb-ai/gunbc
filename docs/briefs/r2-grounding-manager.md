# R2 Grounding Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. Migrates content from [`grounding-manager.md`](grounding-manager.md) (which archives on R2 promotion).

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers (alongside Substrate, Modeling, Impossible-Bugs, Pure Bootstrap, R2 Release; cross-program coordination via Director).
- **Program scope source:** [`THESIS.md §"Tier 1 — Structural correctness — Grounding completeness"`](../../THESIS.md) + [`ROADMAP.md §"Post-R1 Program — Grounding Completeness"`](../../ROADMAP.md).
- **Cross-program consumer:** Engine sharpened-(b) full pilot enumeration consumes Substrate Manager's ValueBody-list/sum + std.unicode bootstrap carrier (T-Substrate sub-lane #4). Coordinate via R1 `Cross-manager notifications queued` brief pattern.
- **Demo coordination:** signal lane-close to R2 Release Manager (closure ledger + demo coordination per `docs/r2-structure.md`).

## Program scope (T-Ground)

R2's Goal 1 — **Grounding Completeness**. Target-side primitive types (Rust / Python / Go) structurally declared in `.dag` rather than table-driven; coercion via algebraic-inhabitance search; Track-13 dissolution. Inherits from `ROADMAP.md §"Post-R1 Program — Grounding Completeness"`.

The one true critical path in R2: `Pilot → Rust → Engine → Tests → Dissolve`. Python and Go run as fill-queue alongside the critical path.

## Owned deliverables (through R2 close)

| Lane | Size | Status (at brief authoring) | Description |
|---|---|---|---|
| T-Ground-Pilot | S | DONE (PR #765 merged 2026-04-25) | Toy inhabitance-search engine for Rust integer family + bool + Unit; routing-stability tests demonstrating parity with current table lookup. |
| T-Ground-Rust | M | NOT YET AUTHORED — listed under Sub-briefs Pending below; gated on pre-spawn Director scope refinement per inbox #828. (Pilot PR #765 + Engine Phase 1 typestructure PR #788 are separate dispatched lanes — see those rows; the prior "DISPATCHED" status here was a parenthetical leak from the Engine row's loader-close parking note.) | Rust target-spec primitive declarations end-to-end. Critical path because Engine blocks on layers 1–3 populated. |
| T-Ground-Engine | M | DISPATCHED (Phase 1 typestructure landed PR #788; Phase 2 implementation pending substrate) | Engine that consumes target-spec primitives + does inhabitance-search; dependent on `ValueBody::List` substrate (Substrate Manager dependency). |
| T-Ground-Tests | M | NOT YET AUTHORED | L4 verification of routing correctness via routing-stability tests + algebra-satisfaction certification. |
| T-Ground-Dissolve | M | NOT YET AUTHORED | Track-13 dissolution: delete `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` once Engine carries the load. Final critical-path step. |
| T-Ground-Python | M | FILL QUEUE (parallel after Pilot) | Python target-spec primitive declarations; not Engine-blocking. |
| T-Ground-Go | M | FILL QUEUE (parallel after Pilot) | Go target-spec primitive declarations; not Engine-blocking. |

## Cross-program dependencies

**Produces (none — Grounding doesn't produce carriers other managers consume).**

**Consumes:**
- **Substrate Manager — ValueBody-list/sum + std.unicode bootstrap carrier**: Engine sharpened-(b) full pilot enumeration via symbolic walk of `rust_pilot_primitives: List<RustPrimitive>` is gated on this. Substrate Manager signals readiness via cross-manager queue; Grounding Manager dispatches Engine sharpened-(b) consumer migration on receipt.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (now, before R1 close):** Director + PM coordinate on brief authoring per inbox #828 split. PM authors the manager skeleton (this file); Director authors any worker-level briefs not yet existing per the manager's "Pending" sub-briefs list. Both stop authoring once R2 spawns.
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Ground sub-briefs without Director.
- Dispatches workers against T-Ground sub-briefs.
- Resolves T-Ground-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every T-Ground worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; every PR introducing hand-Rust under `src/v3/` fills the per-PR gate.

## Reporting cadence

- Lane-close → R2 Release Manager (closure ledger).
- Cross-program signals (e.g., Engine sharpened-(b) substrate readiness consumed) → cross-manager queue.
- Blockers + scope changes → Director.
- Weekly health surfacing to Director: which lanes within 1 step of unblocking, which workers fill vs. ready.

## Sub-briefs (authored / pending)

Authored:
- T-Ground-Pilot (PR #765, merged)
- T-Ground-Engine Phase 1 typestructure (PR #788, merged)

Pending — pre-spawn Director scope refinement per inbox #828 coordination split (may absorb these into one bundled R2 worker brief or split per lane); post-spawn manager-authored autonomously per "Pre-spawn vs post-spawn authority" subsection above:
- T-Ground-Rust full implementation
- T-Ground-Engine Phase 2 implementation (gated on substrate)
- T-Ground-Tests
- T-Ground-Dissolve (Track-13 cleanup)
- T-Ground-Python
- T-Ground-Go

## Working state (fill on spawn)

Lane status table refreshes here as work lands. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Grounding Manager"
- Migrating from: `docs/briefs/grounding-manager.md` (archives on R2 promotion)
- Pre-spawn engineering: `docs/briefs/grounding-pilot-receipt.md`, `docs/briefs/t-ground-engine-substrate-escalation.md`
