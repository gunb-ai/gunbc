# Demand minimization in DESIGN §2

**Status:** analysis only. This document recommends one supervised authority edit and ends with its exact **NOT APPLIED** source copy; it does not edit `gunbc.design_document`, `DESIGN.md`, or any projection.

**Reviewed public revision:** `gunb-ai/gunbc@4f556684c3` (the ROOT-1 merge). This is DEMAND-0 of the operator-ratified serial program; the edit lands in DEMAND-1, which starts from the exact main revision this PR merges into, and **this plan is deleted in that same DEMAND-1 PR** — its one consumer is the supervised edit, and retaining it afterward would leave a second prose representation of the live authority. No `HandAuthoredDocBind` is added at any stage.

**Origin:** operator stance, 2026-08-30 — caching/memoization often obscures modeling or workflow defects: a value is computed, thrown away, and demanded again; a cache added at the second ask makes the redundant workflow cheap instead of deleting it, so the defect stops ranking for repair. The reviewing authority's ruling (side chat, 2026-08-30) found the coherent form of this stance **already exists structurally** as `std.materialization_ladder`, so this edit is the *externalization of an existing structural authority as a §2 consequence*, not new doctrine.

## What the substrate already carries

`std.materialization_ladder` decides everything on one axis: **when demand plurality is knowable, and whether what was knowable was prepared for** (operator state law, 2026-07-09):

- Demands whose least common ancestor is a shared-state frame are `AuthoredDuplication` — rewireable, therefore an error; the prescribed repair is `Share`, **never a cache**. `group_redundant_verdict` returns this before provider selection, so a cache provider structurally cannot discharge it.
- Demands separated by an isolation boundary cannot be rewired; materialization is an obligation at the LCA, discharged only by a covering provider — absence, narrow scope, and existence-keying are typed refusals.
- Declared replay or recurrence (`ReplayedFrame`, `UnboundedSiblingsFrame`) obligates preparation before the repeated demand.
- One undeclared pure demand is `AcceptedSingleRecompute`; a cache there is dead weight.
- Nature gates ride on top: fresh effects are never memoized; a world read memoizes only under a declared staleness envelope (`RefusedUnmodeledWorldRead` otherwise).

`test.claim.materialization_ladder_witness` executes both sides: two asks in one shared-state job stay red *even when an sccache provider is supplied*; the same computation across isolated jobs discharges legitimately.

What DESIGN carries today: §2's redundancy law implies the stance but never draws the cache consequence; §3's single-authority law governs a cache's *purity* (`cache_impurity` covers keying and cold/warm identity); §5's absorbing-fallback paragraph carries the adjacent mechanism-locates-the-defect move. The operative sentence exists once, subject-scoped, in the five-minute-CI authority: "no semantic fact recomputed whose inputs did not change" / "values are cached while the world that gives them meaning is rebuilt." No section states the general law.

## Recommendation: one §2 paragraph, and nothing else

**Insertion point:** `gunbc.design_document.section_2_blocks`, immediately after the horizontal/deep two-direction list and before the "net concepts must not grow" paragraph. The opening paragraph establishes why redundancy is bad; the list explains how it is removed; this paragraph supplies the missing consequence for *repeated computation*.

**Explicitly ruled out, per the same ruling (do not reopen without new evidence):**

- **No §3 paragraph.** "A cache is a second representation" is false unqualified: a sound cache is a *derived materialization* of one canonical fact. §3 governs its purity; §2 decides whether the repeated demand should exist. The four-way classification stands: derived materialization (legitimate) / independent semantic authority (§3 violation) / bad key or divergent cold-warm (`cache_impurity`) / cache installed where the demand should have been rewired (`AuthoredDuplication`).
- **No §6 bullet.** "Continuously interrogate the process" is the imprecise form of the precise §2 question — *where does demand plurality become knowable, and could the first value have been carried to the later consumer?* §6 already carries bare-minimum-cost and root-cause-to-the-language-layer; a cache bullet would restate them.
- **No new recurring-failure-mode row.** The failure class exists as `std.materialization_ladder.AuthoredDuplication` — typed identity, total decision fold, prescribed repair, discriminating red, positive control. A prose sibling (`cache_masking_redundant_ask` or similar) would give one semantic defect a second authority and a second recognition rule — the nicknaming defect inside the doctrine that teaches against it. `absorbing_fallback` stays unamended: substituted ignorance and cheapened unnecessary demand share a consequence (the deficit's frequency zeroed), not a mechanism. A knowingly added cache over rewireable duplication is discussable today as author-side `unmarked_workaround` with the semantic verdict `AuthoredDuplication`; a new row becomes eligible only if a masking mechanism is found that *cannot* be represented as `AuthoredDuplication` under the correct frame/demand topology — and the first repair would then extend the materialization authority, not mint prose.

**Three distinct questions the paragraph must keep separate** (collapsing them is the known review failure): demand minimization (should the second ask exist?), cache purity (does the cache faithfully denote the fact? — §3/`cache_impurity`), and cache economics (is serving cheaper than recomputing? — gunbc#9721 measured a compliant, correctly-serving cache costing *more* than recompute; admission requires a measured same-subject `serve < recompute`, so "cache" is not a synonym for "optimization").

## Evidence classing

- `std.materialization_ladder.AuthoredDuplication` + `test.claim.materialization_ladder_witness` — canonical mechanism and structural control.
- The disk-tier repeat-resolve lane and CI's repeated world-reconstruction (five-minute-CI authority, `docs/plans/five-minute-ci-gate-design.md`) — concrete specimens of the second-ask defect.
- gunbc#9721 — economic-admission boundary control, not a masking specimen.

## FrameDemand consumer inventory (current, at the reviewed revision)

Producers that project real subjects into `FrameDemand` today: `gunbc.ci_materialization` (one demand per `RunStep` script of the modeled CI workflow) and `v2.compiler.materialization_carriers` (the compiler's own declared memo demands — `parse_table_memo_demand`, `compile_stage_memo_demand`). Everything else that exercises the ladder is fixture-authored. `v2.lens.duplicate_computation` is registered but not enrolled, self-declares `WallAfterGrounding`, and reads argv token shapes rather than the compiled tree — its grounding (a computation identity arbitrary expressions can carry) is the parked shared root `ComputationIdentity → FrameDemand projection → DemandNature/effect/staleness projection`, ratified to the ROOT/DEMAND program with **activation separately scheduled**; nothing in this edit activates it, and the paragraph must remain true while that projection does not exist (it teaches the discipline; it claims no wall).

## §6 terminal-architecture consumption test

The terminal consumer is the DEMAND-1 supervised edit, which consumes the copy below substantially unchanged into `gunbc.design_document.section_2_blocks` and regenerates `DESIGN.md` (the only projection this edit touches — `docs/design-ledgers.md` is unchanged because no roster changes). This document then deletes in that same PR; the merged DEMAND-0 PR and git history retain the analysis.

## DEMAND-1 transaction (for the record; identical discipline to ROOT-1)

Apply the copy to `section_2_blocks`; regenerate via `tools.generated_artifact_gate.main_wet` with a repo-built `gunbc` from the exact DEMAND-1 tree; run under a readable process-private cgroup bound (the proven route: fresh srv2 clone, `docker --memory=16g`) or an explicit `GUNBC_MEMORY_BUDGET_BYTES` **derived from what the slot can actually deliver** — measured 2026-08-31: a budget above the BuildBuddy VM's real capacity is OOM-killed at rc=137 with the kill masked by any `| tail` pipe, while a smaller budget completes via cap-bounded eviction; treat `HostBudgetUnreadable` as a correct line stop. Expected generated diff: `DESIGN.md` only, exactly one new paragraph. Gate: green CI and reviewing-authority exact-head approval, then merge.

## Exact proposed `gunbc.design_document` insertion text — NOT APPLIED

Insert in `section_2_blocks`, immediately after the two-direction list (`ul`) and before the "net concepts must not grow" paragraph:

```dag
    p(text: "**Minimize the demand graph before materializing its answers.** A repeated computation is first evidence about why the same semantic fact is demanded again, not yet a cache obligation. When several demands share state at their least common ancestor, the repetition is authored duplication: carry, rewire, or share the first value — caching the later request only makes the redundant workflow cheap and suppresses the signal that would make it rank for deletion. Materialize only where recurrence survives as a real boundary — isolated consumers, declared replay, or later execution over unchanged inputs — and then only as a derived value keyed by the complete declared inputs. One demand recomputes. A materialization is admitted only when serving it is measured cheaper than recomputing it. The structural authority is `std.materialization_ladder`; a cache may discharge unavoidable recurrence, and may never excuse authored duplication."),
```
