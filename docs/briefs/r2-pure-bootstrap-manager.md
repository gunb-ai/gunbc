# R2 Pure Bootstrap Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. Migrates content from [`pure-bootstrap-zero-manager.md`](pure-bootstrap-zero-manager.md) (which archives on R2 promotion); **scope narrowed** per the gate-deferral resolution in PR #827.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers.
- **R1 vs R2 boundary:** R1 owns Pure Bootstrap census-reduction work via T-PB-A / T-PB-B lanes per ROADMAP single authority on gate semantics (target = 0 per `docs/design-pure-bootstrap-zero.md` LIVE 2026-04-25). **R2 Pure Bootstrap Manager owns post-R1 PB program work that survives R1 close — not a duplicate of R1's census-reduction lanes.**
- **Cross-program coordination:** B4's §0.7 file-preference rank carrier touches PB territory; coordinate with Substrate Manager.

## Program scope (T-PB; post-R1 only)

**Does NOT own:**
- T-PB-A non-test hand-Rust census reduction (R1 lane work per ROADMAP).
- T-PB-B Rust-authored test census reduction (R1 lane work per ROADMAP).
- `lens_producer_files_remaining` priority slice (R1 T-PB-A gate per PR #752).

**Owns (post-R1 program work):**

| Lane | Size | Status (at brief authoring) | Description |
|---|---|---|---|
| Tier 3 mirror dissolutions: termination | M | NOT YET AUTHORED | `DescentEvidence`, `PositiveDescentAmount`, `ProportionalDivisor`, `ShrinkFactor`, `evidence_rank`, `merge_evidence` Rust mirror at `dag.rs:628-790` dissolves as v3 lowers + evaluates `.dag` runtime values. Tier 3 #10 from #810. |
| Tier 3 mirror dissolutions: computation | M | NOT YET AUTHORED | `SizeBound`, `RecursionShape`-related Rust mirror at `dag.rs:839-915` dissolves with same substrate dependency. |
| Tier 3 mirror dissolutions: induction | M | NOT YET AUTHORED | `RecursionShape`, `InductiveField`, `SubValueRelation` Rust mirror at `dag.rs:916-980` dissolves with same substrate dependency. |
| Tier 3 mirror dissolutions: effect-carrier | S | NOT YET AUTHORED | `src/v3/compiler/src/dag/effects.rs` (216 LOC) + `compose_operation_effects` in `workflow_idempotency.rs` (105 LOC). Mechanical PB dissolution once self-hosting reaches it. Tier 3 #12 from #810. |
| `kernel_algebra_profile` mirror dissolution | M | GATED (waiting on Substrate Manager `ValueBody::Map` carrier; future T-Substrate sub-lane) | Map-shaped (not list/sum); requires `ValueBody::Map` substrate work. Substrate Manager future sub-lane. |
| Tier 2 `patch_lower_helpers_*` retirement | S | NOT YET AUTHORED (queued for PB-Tier1 priority hint per #810 §5) | If survives R1, PB Manager retires `patch_lower_helpers_generated_type_alias_refinement` once generated `lower_helpers` can emit refinement field natively. |
| Post-R1 emergent dissolutions | varies | NOT YET MATERIALIZED | Catch-all for new PB work that surfaces post-R1 (new mirror dissolutions discovered during R2; new Rust scaffolds inadvertently introduced and needing dissolution). |

## Cross-program dependencies

**Produces:** none (PB consumes substrate, doesn't produce carriers other managers consume).

**Consumes:**
- **Substrate Manager — `ValueBody::Map` carrier** (future T-Substrate sub-lane): unblocks `kernel_algebra_profile` mirror dissolution.
- **Substrate Manager — B4 §0.7 file-preference rank carrier**: touches PB territory; coordinate.
- **R1 close**: T-PB-A / T-PB-B census-reduction work completes per R1 gate authority. PB Manager spawns post-close to own everything else.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (now, before R1 close):** Director + PM coordinate on brief authoring per inbox #828 split. PM authors the manager skeleton (this file); Director authors any worker-level briefs not yet existing per the manager's "Pending" sub-briefs list. Both stop authoring once R2 spawns.
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all post-R1 PB sub-briefs without Director.
- Dispatches workers against post-R1 PB sub-briefs.
- Resolves PB-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every PB worker brief names dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; per-PR gate applies to all hand-Rust dispatches.

## Reporting cadence

- Lane-close → R2 Release Manager (closure ledger).
- Cross-program signals (consume Substrate carrier-readiness) → cross-manager queue.
- Blockers + scope changes → Director.

## Sub-briefs (authored / pending)

Authored:
- Pre-R1 PB program briefs (in `pure-bootstrap-zero-manager.md`, archives on R2 promotion); content migration here covers post-R1 deliverables only.

Pending — Director-authored per coordination on inbox #828:
- Tier 3 mirror dissolution worker briefs (4: termination, computation, induction, effect-carrier)
- `kernel_algebra_profile` worker brief (gated on Substrate Manager `ValueBody::Map`)
- Tier 2 `patch_lower_helpers_*` retirement worker brief (if survives R1)

## Working state (fill on spawn)

Lane status table refreshes here as work lands. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Pure Bootstrap Manager"
- Migrating from: `docs/briefs/pure-bootstrap-zero-manager.md` (archives on R2 promotion)
- Self-hosting design: `docs/design-pure-bootstrap-zero.md` LIVE 2026-04-25
- Tier 3 source: `docs/briefs/debt-paydown-synthesis-2026-04-25.md` items #10 + #12
- ROADMAP single authority on gate semantics: `ROADMAP.md §"Lane acceptance — .dag gates"`
