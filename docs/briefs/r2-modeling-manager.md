# R2 Modeling Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. NEW manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers.
- **Program scope source:** [`docs/r2-structure.md` §"Goals" item 2 (modeling-faithfulness)](../r2-structure.md) + 4th item the rework added (tokenizer charclass phase-2 consumer).
- **Cross-program consumer:** all 4 items consume Substrate Manager carriers. Three dispatch as their T-Substrate dependency lands; Dimensions is dispatchable immediately because the producer audit found the needed phantom-parameter substrate already present.
- **Demo coordination:** signal item-close to R2 Release Manager (closure ledger + demo coordination).

## Program scope (T-Modeling)

R2's Goal 2 — **Modeling-faithfulness dissolution**. Three Tier-1 type-refinement gaps close + tokenizer charclass phase-2 (added as 4th item due to shared T-Substrate ValueBody-list/sum dependency).

| Item | Consumer of | Status (at brief authoring) |
|---|---|---|
| Surface int-literal magnitude at concept layer | T-Substrate cardinality subset | AUTHORED — gated on producer readiness / scoped to range-facts consumer work |
| `Secret<T>` nominal-opaque graduation | T-Substrate nominal-opaque subset | AUTHORED — gated on producer readiness |
| `Dimension<Carrier>` typed value wrapper with phantom-parameter unit-mismatch enforcement | T-Substrate parametric-algebra subset | AUTHORED — dispatchable immediately; producer audit closed substrate-side |
| Tokenizer charclass phase-2 | T-Substrate ValueBody-list/sum subset | AUTHORED — gated on producer readiness; reclassified R1→R2 per Surface Manager handoff 2026-04-24 |

## Owned deliverables (through R2 close)

For each item:
- Worker brief authored (one per item; size S–M).
- Worker dispatched as the corresponding T-Substrate sub-lane lands, except Dimensions which is already dispatchable per its producer audit.
- Migration into the new substrate carrier; structural test demonstrating the impossible-bug class (e.g., int magnitude overflow → compile error; `Secret<T>` no `Show` instance; `Dimension<m>` + `Dimension<s>` → unit-mismatch error).
- Item-close signal to R2 Release Manager (closure ledger + R2 demo).

## Cross-program dependencies

**Produces (none — Modeling consumes carriers, doesn't produce them).**

**Consumes:**
- Substrate Manager — cardinality-for-int-lit subset
- Substrate Manager — nominal-opaque-for-Secret subset
- Substrate Manager — parametric-algebra-for-Dimensions subset
- Substrate Manager — ValueBody-list/sum + std.unicode bootstrap

**Adjacent territory:**
- Tokenizer charclass phase-2 historically R1 T-Sub work; reclassified to R2 per Surface Manager handoff 2026-04-24. R1 retains the charclass phase-1 close; R2 Modeling Manager owns phase-2 (the substrate-capability-gated portion).

## Pre-spawn vs post-spawn authority

- **Pre-spawn (now, before R1 close):** Director + PM coordinate on brief authoring per inbox #828 split. PM authors the manager skeleton (this file); Director authored the worker-level briefs listed below. Both stop authoring once R2 spawns.
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Modeling worker briefs without Director.
- Dispatches workers against worker briefs as Substrate Manager signals carrier readiness.
- Resolves Modeling-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every T-Modeling worker brief names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance.

## Reporting cadence

- Item-close → R2 Release Manager (closure ledger + demo coordination).
- Cross-program signals (consume Substrate Manager carrier-readiness) → cross-manager queue.
- Blockers + scope changes → Director.

## Sub-briefs (authored / pending)

Authored — pre-spawn Director-authored per inbox #828 coordination split; post-spawn manager owns dispatch / refresh per "Pre-spawn vs post-spawn authority" subsection above:
- [`r2-modeling-int-lit-magnitude-worker.md`](r2-modeling-int-lit-magnitude-worker.md) — gated on T-Substrate cardinality readiness; scoped to range-facts consumer work.
- [`r2-modeling-secret-graduation-worker.md`](r2-modeling-secret-graduation-worker.md) — gated on T-Substrate nominal-opaque readiness.
- [`r2-modeling-dimensions-phantom-worker.md`](r2-modeling-dimensions-phantom-worker.md) — dispatchable immediately; consumes already-present phantom-parameter substrate.
- [`r2-modeling-tokenizer-charclass-phase2-worker.md`](r2-modeling-tokenizer-charclass-phase2-worker.md) — gated on T-Substrate ValueBody-list/sum readiness.

Pending: none at manager-brief authoring time.

## Working state (fill on spawn)

Item status table refreshes here as work lands. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Modeling Manager"
- Goal authority: `docs/r2-structure.md` §"Goals" item 2
- Substrate dependencies: `docs/briefs/r2-substrate-manager.md`
- Originating analyses: PR #745 (P4 int-literal row); ROADMAP post-merge-debt 2026-04-23 thesis-doc surface (Secret<T>, Dimensions); 2026-04-24 ROADMAP amendment (charclass reclassification)
