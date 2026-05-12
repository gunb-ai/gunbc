# R2 Modeling Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827; refreshed 2026-04-28 post-#1078 merge; Q1 asymmetric-bound-algebra **merged in PR #1129** at [`docs/design-emission-model.md`](../design-emission-model.md) §"Q1 — `BoundDeclaration` substrate type" — int-lit item consumes the locked disposition). Eligible to spawn pre-R1-close per `r2-structure.md` Transition mechanics step 4 (no technical R1 dependency). NEW manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers (count rose from 6 to 7 with Evaluator added 2026-04-28).
- **Program scope source:** [`docs/r2-structure.md` §"Goals" item 2 (modeling-faithfulness)](../r2-structure.md) + 4th item the rework added (tokenizer charclass phase-2 consumer).
- **Cross-program consumer:** all 4 items consume Substrate Manager carriers. Three dispatch as their T-Substrate dependency lands; Dimensions is dispatchable immediately because the producer audit found the needed phantom-parameter substrate already present. **Int-lit item now consumes PR-PreF Interval<D>** (Q1 lock cascaded 2026-04-28).
- **Demo coordination:** signal item-close to R2 Release Manager (closure ledger; per the **structural-acceptance-per-lane-close discipline** locked in `r2-structure.md` — the demo IS the structural gate, not a separate artifact).
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): self-serve through the 3-step decision procedure when the consumer migration surfaces a substrate gap. If the procedure outcome is "extend substrate," signal Substrate Manager via cross-manager queue rather than authoring substrate locally.

## Program scope (T-Modeling)

R2's Goal 2 — **Modeling-faithfulness dissolution**. Three Tier-1 type-refinement gaps close + tokenizer charclass phase-2 (added as 4th item due to shared T-Substrate ValueBody-list/sum dependency).

| Item | Consumer of | Status (at brief authoring) |
|---|---|---|
| Surface int-literal magnitude at concept layer | T-Substrate cardinality subset (consuming PR-PreF Interval<D>) | AUTHORED — gated on producer readiness / scoped to range-facts consumer work. **Adjacent landing:** T-Cost-Dimension fail-closed symbolic-cost analysis (#1003) — DominateScanAcc conjunctive accumulator; relevant precedent for fail-closed semantics in this consumer. |
| `Secret<T>` nominal-opaque graduation | T-Substrate nominal-opaque subset | AUTHORED — gated on producer readiness. **Producer side advanced:** NominalOpacity carrier **merged in PR #900**; **fail-closed field-projection enforcement merged in PR #937** (NominalOpacityViolation diagnostic + production enforcement before field descent). Day-1 consumer migration: `dsl/std/types.dag` retire bare `Secret = String`; land `where only … may construct` semantics; integration test for unauthorized construction without bootstrap stamp. |
| `Dimension<Carrier>` typed value wrapper with phantom-parameter unit-mismatch enforcement | T-Substrate parametric-algebra subset | AUTHORED — dispatchable immediately; producer audit closed substrate-side. |
| Tokenizer charclass phase-2 | T-Substrate ValueBody-list/sum subset | AUTHORED — gated on producer readiness. **In flight:** scanner-order retype landed pre-cascade (commit `242c65d07` — `feat(v3): retype tokenizer charclass scanner order`). Reclassified R1→R2 per Surface Manager handoff 2026-04-24. |

## Pre-dispatch design lock cadence (consumed; per #1078 lock)

Worker dispatch on the int-lit item gates on **PR-PreF Interval<D>** (Substrate-owned; foundational substrate consolidation). Dimensions item is fully dispatchable now — no cadence dependency. Secret<T> and charclass phase-2 are independent of the cadence.

| Cadence PR | Locks | Consumer (this manager) |
|---|---|---|
| **PR-PreF** *(Substrate-owned)* | `Interval<D>` substrate consolidation — shared parent | int-lit magnitude item (consumes Q1 BoundDeclaration via Interval<Int>) |

**Asymmetric BoundDeclaration match rule (Q1 lock):** target's `Unbounded` universally accepts; target's `ExactInterval(lo,hi)` requires exact range match. Worker briefs MUST consume this without re-litigation when reasoning about int-magnitude reconciliation across targets.

## Owned deliverables (through R2 close)

For each item:
- Worker brief authored (one per item; size S–M).
- Worker dispatched as the corresponding T-Substrate sub-lane lands, except Dimensions which is already dispatchable per its producer audit.
- Migration into the new substrate carrier; structural test demonstrating the impossible-bug class (e.g., int magnitude overflow → compile error; `Secret<T>` no `Show` instance; `Dimension<m>` + `Dimension<s>` → unit-mismatch error).
- Item-close signal to R2 Release Manager (closure ledger + R2 demo per structural-acceptance-per-lane-close).

## Cross-program dependencies

**Produces (none — Modeling consumes carriers, doesn't produce them).**

**Consumes:**
- Substrate Manager — cardinality-for-int-lit subset (now consumes PR-PreF Interval<D>)
- Substrate Manager — nominal-opaque-for-Secret subset
- Substrate Manager — parametric-algebra-for-Dimensions subset
- Substrate Manager — ValueBody-list/sum + std.unicode bootstrap

**Adjacent territory:**
- Tokenizer charclass phase-2 historically R1 T-Sub work; reclassified to R2 per Surface Manager handoff 2026-04-24. R1 retains the charclass phase-1 close; R2 Modeling Manager owns phase-2 (the substrate-capability-gated portion).
- **§6a per-method-metadata bulk migration** (R2 Release Manager-owned, but consumes `MethodContract` substrate at method-call sites in `cost.dag`/`complexity.dag`). Modeling Manager doesn't author this; tracked here only as adjacent.
- **SourceFiltering canonical authority** (#1004 sleek-wren-716) — cross-target drift class closed via independent per-target `excluded_prefixes`. Adjacent to Modeling Manager scope; not owned here, but relevant precedent for cross-target uniformity work.

## Locked design decisions consumed (per #1078 8-question dialogue)

Worker briefs MUST consume these without re-litigation:

- **Q1**: `Interval<D>` shared parent in substrate (PR-PreF prepended); `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent`; asymmetric match rule (target's `Unbounded` universal-accepts; target's `ExactInterval(lo,hi)` requires exact range match). **Consumed by int-lit magnitude item.**
- **Q2 (b3' emission-biased non-violating minimal target modeling)**: relevant only as the framing under which Dimensions item's phantom-unit-mismatch error fires structurally; not directly consumed.
- **Q3 (Cost<Unit>)**: not directly consumed by this manager (cost belongs to LanguageSpec lane in Grounding); referenced only for awareness.
- **Q5**: cardinality is the connectives axis; collapses with PR-PreF Interval<Cardinal>; reinforces Dimensions item's `phantom_unit_mismatch` semantics.

Full disposition: [`docs/r2-structure.md`](../r2-structure.md) §4.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (post-#1078-merge):** brief authoring + cadence sequence locked. Manager spawns once at least one item is dispatchable; pre-R1-close spawn allowed (Dimensions item is dispatchable immediately).
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Modeling worker briefs without Director.
- Dispatches workers against worker briefs as Substrate Manager signals carrier readiness.
- Resolves Modeling-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every T-Modeling worker brief names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; every PR introducing hand-Rust under `src/v3/` fills the per-PR gate.
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1) applies when a consumer migration surfaces a substrate gap; if outcome is "extend substrate," signal Substrate Manager rather than authoring locally.
- **Cross-program signal authority:** carrier-consumption requests → Substrate Manager via cross-manager queue; per-item closure → R2 Release Manager (closure ledger).

## Reporting cadence

- Item-close → R2 Release Manager (closure ledger + demo coordination per structural-acceptance-per-lane-close).
- Cross-program signals (consume Substrate Manager carrier-readiness) → cross-manager queue.
- Blockers + scope changes → Director.
- **Weekly health surfacing to Director:** which items within 1 step of unblocking (Substrate readiness state); which workers fill vs. ready.

## Acceptance — `.dag` gates

Each item closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

- `int_lit_magnitude_overflow_compile_error` — int-literal exceeding target int range produces compile error via Q1 BoundDeclaration asymmetric match (consumes PR-PreF Interval<Int>)
- `secret_t_no_show_instance_compile_error` — `Secret<T>` cannot be shown / serialized at compile time
- `dimension_unit_mismatch_compile_error` — `Dimension<m>` + `Dimension<s>` produces unit-mismatch error via `phantom_unit_mismatch` carrier
- `tokenizer_charclass_phase2_lowers_structurally` — `data ascii_scan_order: List<CharClass> = [...]` lowers structurally via ValueBody-list substrate

## Sub-briefs (authored / pending)

Authored — pre-spawn Director-authored per inbox #828 coordination split; post-spawn manager owns dispatch / refresh:
- [`r2-modeling-int-lit-magnitude-worker.md`](r2-modeling-int-lit-magnitude-worker.md) — gated on T-Substrate cardinality readiness + PR-PreF Interval<D>; scoped to range-facts consumer work.
- [`r2-modeling-secret-graduation-worker.md`](r2-modeling-secret-graduation-worker.md) — gated on T-Substrate nominal-opaque readiness.
- [`r2-modeling-dimensions-phantom-worker.md`](r2-modeling-dimensions-phantom-worker.md) — dispatchable immediately; consumes already-present phantom-parameter substrate.
- [`r2-modeling-tokenizer-charclass-phase2-worker.md`](r2-modeling-tokenizer-charclass-phase2-worker.md) — gated on T-Substrate ValueBody-list/sum readiness.

Pending: none at manager-brief authoring time. Worker briefs may need refresh post-PR-PreF merge to consume Interval<D> directly rather than introduce a new bound type locally.

## Working state (fill on spawn)

Spawn refresh, 2026-04-28 (post-#1078, status-refresh against landed PRs):

- Item status table unchanged in scope; int-lit item now consumes PR-PreF Interval<D> via Q1 lock cascade.
- **Secret<T> producer side substantially advanced:** NominalOpacity carrier PR #900 + fail-closed field-projection enforcement (#937) both landed. Day-1 consumer migration is dispatchable.
- **Tokenizer charclass phase-2 in flight:** scanner-order retype (`242c65d07`) landed pre-cascade.
- **Adjacent precedents landed:** SourceFiltering canonical authority (#1004); T-Cost-Dimension fail-closed analysis (#1003).
- Dimensions remains dispatchable immediately (phantom_params already in substrate).
- Other items continue to gate on Substrate Manager carrier-readiness signals where carriers haven't fully landed.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Modeling Manager"
- Goal authority: `docs/r2-structure.md` §"Goals" item 2
- Substrate dependencies: `docs/briefs/r2-substrate-manager.md`
- Q1-Q5 disposition: `docs/r2-structure.md` §4 + `docs/design-emission-model.md`
- INVARIANTS substrate-fact-introduction procedure: `INVARIANTS.md` §P1
- Originating analyses: PR #745 (P4 int-literal row); ROADMAP post-merge-debt 2026-04-23 thesis-doc surface (Secret<T>, Dimensions); 2026-04-24 ROADMAP amendment (charclass reclassification)
- Thesis-claim disposition: `docs/thesis/r2-r3-thesis-mapping.md`
