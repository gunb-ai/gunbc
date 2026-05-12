# F-β.1 — effect_enum Migration-Shape Canvas (gate #82)

**Status:** Mgr canvas surfaced for Director ratification per `feedback_substrate_shape_belongs_in_mgr_canvas`. Closes gate #82 `effect_enum_migration_shape_ratified` when Director ratifies. Cluster F F-β phase per `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md`. No predicate-block on F-α (#81 walker port).

**Authority:** Director (zesty-bear-812) routing via PM (deep-wolf-155) msg_a63f5e7e Wave-1 dispatch. Director hint (msg_57631cde): canvas should explicitly audit-cite `docs/design-effect-enumeration-resource-threading.md` §3.2 + §6.2 so ratification can be terse.

**Scope:** Ratify the migration-shape from the current `OperationEffect`-based effect enumeration to the §3.2 thread-through-signature shape. F-β.2 (implementation) follows; 4 sub-phases per Cluster-F plan.

---

## §0. Source-claim audit-citation (per Director hint)

Two paragraphs of pre-existing substrate authority bound this canvas:

### §3.2 (existing authority) — `Operation` carrier already exists

> "The pinning *substrate carrier* already exists: `src/v3/std/services.dag::Operation`:
>
> ```dag
> type Operation {
>   callable: CallableRef          // declaration whose signature carries the resource thread
>   inputs:   Map<String, InputField>
>   endpoint: RestEndpointBinding  // per-target realization (E-9 binding)
> }
> ```
>
> [...] **Resource pinning IS the `callable: CallableRef` field plus the threaded signature on the referenced declaration.** No new top-level carrier is required; the thread-through-signature rule of §2 plus the existing `Operation.callable: CallableRef` provides the pinning surface."
>
> — `docs/design-effect-enumeration-resource-threading.md` §3.2

### §6.2 (existing authority) — Atomic-migration feasibility

> "**`Operation` already exists.** `src/v3/std/services.dag:121` declares the carrier with `callable: CallableRef + inputs: Map + endpoint: RestEndpointBinding` — exactly what §3.2 needs. **No new substrate type.** The PR-β..ω lineage in `services.dag:9-14` is the dispatch frame for the per-extdep population work."
>
> — `docs/design-effect-enumeration-resource-threading.md` §6.2

**Audit grep receipt** (2026-05-12T20:35Z; corrected per cursor APPROVE_WITH_COMMENTS review 10384 — anchored pattern, real output):
```
$ grep -n "^type Operation {" src/v3/std/services.dag
121:type Operation {
$ grep -nE "^type Operation|callable: CallableRef|inputs:.*InputField" src/v3/std/services.dag
102:// identity (`callable: CallableRef`) plus its REST realization +
121:type Operation {
122:  callable: CallableRef
123:  inputs:   Map<String, InputField>
```
Carrier present at line 121 with declared shape. Line 102 is a header-comment substring match (noise; not a structural match). §3.2 + §6.2 audit-citations are verifiable against current main.

---

## §1. Migration shape (canvas recommendation)

Per the §3.2 + §6.2 authority above, the F-β.1 migration shape is:

**Adopt the §3.2 thread-through-signature shape verbatim.** No new top-level carrier. The existing `Operation` carrier at `services.dag:121` is the pinning surface; resource sets are read off the arrow signature on the referenced `CallableRef`.

This is **option (c-trivial)** in the substrate-shape question: the canvas does not propose alternative carrier shapes because the pre-existing authority already locked the shape. The canvas's job here is **ratification routing**, not shape-selection.

## §2. Sub-phase decomposition (F-β.2 implementation)

Per Cluster-F plan, F-β.2 lands in 4 sub-phases (post-ratification):

1. **F-β.2a — resource-threading.** Thread the resource set through arrow signatures of every `Callable` referenced by an `Operation`. Per-callable, isolated, parallelizable.
2. **F-β.2b — metadata retirement.** Retire `OperationEffect` declaration at `src/v3/std/effects.dag:421` + `DeriveOpEffectResult` at `:431`. `derive_op_effect` function dissolves.
3. **F-β.2c — lens body update.** Replace `OperationEffect`-keyed lens reads with `Operation.callable.signature.resource_set` reads. Maintains lens-behavioral-parity.
4. **F-β.2d — `CompositionVerdict.BrokenBy` reshape.** `first_breaker: ElementRef<OperationEffect>` → `ElementRef<Operation>` per §4.1 table.

Each sub-phase is dispatchable as a separate worker; **F-β.2a is the only parallel-across-callables phase**, the rest sequence.

## §3. Practice 4 (coproduct dissolution) discipline

No new sum-type proposed in this canvas. The migration is a structural carrier-shape simplification (parallel record → on-arrow-signature), not a sum-introduction. Practice 4 YELLOW/GREEN/RED classification does not apply; the substrate-shape change is a P5 dissolution (`OperationEffect` dissolves rather than transforms).

## §4. Cost-of-change accounting

Per `INVARIANTS.md` "Cost of Change" — number of files edited to grow the language by one effect-bearing operation:

| State | Files edited |
|---|---|
| Pre-migration (today) | ≥3 (Operation row + OperationEffect record + lens-body consumer) |
| Post-migration (§3.2 shape) | 1 (the `CallableRef` arrow signature only) |

Cost-of-change drops from 3 to 1. This is the **substrate-progress payoff** of F-β: every future effect-bearing operation costs one edit, not three.

## §5. Open questions (none surfaced)

The canvas does not surface open questions because §3.2 + §6.2 authority already ratifies the shape. If Director ratification raises a substantive question, it routes to a separate sub-canvas; this canvas is intentionally narrow.

## §6. Ratification ask

Director ratification on:
- (a) Adopt §3.2 thread-through-signature shape verbatim for F-β.1
- (b) F-β.2 sub-phase decomposition (4-phase per §2 above)
- (c) F-β.2a parallel-across-callables / F-β.2b-d sequential

On ratification, gate #82 closes. F-β.2 sub-phase brief authoring follows (Wave-2 lane).

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-12
**Source**: `docs/design-effect-enumeration-resource-threading.md` §3.2 + §6.2 (pre-existing substrate authority)
