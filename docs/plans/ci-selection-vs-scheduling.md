# CI selection vs scheduling — the two orthogonal axes (§1/§4 seam)

> The framing behind the §1 "floor runs the right things" bullets. DESIGN refs: §1 (safety — a wrong answer caught late is paid "at interest"), §2 (parallelization-by-realization, the `std/realization.dag` Schedule/Placement/Width arm), §4 (affected-set selection over the Node dep-graph), §6 (don't build machinery for near-zero residue). Authored from warm-lark-306's analysis (2026-06-21), §1-lead input by quick-ant-298. **Carriers are the authority** — this doc exists only to keep the ROADMAP bullets terse (no dual representation).

## 0. The reject: "expensive → nightly" routes by the wrong axis

The stood-down nightly lane (proud-deer's #5447, closed by operator decision 2026-06-21) routed tests to cadences by **cost**. That fuses two facts that must stay separate, and the fusion is **unsound**: it lets an *affected* test be dropped from the per-PR gate merely because it is slow — so a breaking change merges green and is caught hours later on the nightly run, the §1-safety harm paid "at interest". Cost is a *scheduling* fact; whether a test must gate *this* PR is a *selection* fact. Routing by the wrong axis is the design the operator said we were side-stepping.

## 1. The two axes (they never cross)

|  | **SELECTION** (§4) | **SCHEDULING** (§1) |
| --- | --- | --- |
| Question | does this result depend on what I changed? | given the must-run set, when / where / how-wide? |
| Authority | the **affected set** — transitive closure of changed nodes over the dep graph | cost + hermeticity profile of each test |
| Derived from | a property of the **change** | a property of the **test's resources**, not the change |
| Discipline | sound, **fail-closed** — never skip an affected test | measured — cadence, placement, width |

**The load-bearing rule: cost informs SCHEDULING, never SELECTION.** The answer to "the affected set is expensive" is *"run it faster"* (parallelize + cache — the §2 realization arm), never *"run it later"*.

## 2. The three distinctions the bullets must preserve

1. **Cost may inform scheduling, never selection** — the §1-safety violation, the whole reason #5447 was rejected.
2. **`#5427` run-all is the SOUND BASELINE** (a superset of the affected set, already fail-closed on selection). Affected-set is an **affordability refinement that *shrinks* it — not a correction**. #5427 is step 1, not superseded; affected-set is step 2 on top of it.
3. **Nightly = full-corpus selector-backstop + non-hermetic residue, NOT a slow-test dump** — and the backstop (a) is precisely what makes the step-2 shrink *safe*.

## 3. Per-PR membership: sound baseline → minimal sound set

- **Step 1 — the sound baseline:** run-all-unless-`#[ignore]`d-with-a-written-reason (#5427, fierce-hawk-540). A superset of the affected set; fail-closed on selection by construction (it never skips an affected test). The completeness lens — "no affected test unrun; no un-run test without a written reason" — gates this (the `#[ignore]` residue is the legit §6 unstructurable bit).
- **Step 2 — shrink to the affected set:** the **existing** `v2.lens.affected_set` (compile-time, structural over `Node`/`DependencyView`, git-diff edit locus, fail-closed, CI-enrolled) refines the baseline down to the minimal sound set. This is an **affordability** move *on top of* #5427's soundness, not a replacement for it. Membership stays bounded by **affectedness, not cost**: expensive-but-affected tests run per-PR, their cost absorbed by parallelize + cache.

## 4. Off per-PR cadence: only two things (neither is "expensive")

1. **Non-hermetic residue** — live-network / real-external-service captures that *cannot* gate deterministically regardless of cost. Routed by **determinism**, not expense. Small, irreducible.
2. **The full-corpus selector-backstop** — run *everything* on a slow cadence as soundness insurance that affected-set selection missed no dep edge. This **fail-closes the selector itself**, and is exactly the insurance that lets step 2 shrink the per-PR set safely. It is an audit of *all* tests, **not** a dumping ground for slow ones.

The "expensive" category drains toward ~this residue as #5456 (opt-level=3, debug-amplification fix — restores Pop-A to per-PR) and #5450 (build-once for Pop-B) land. So lane machinery is **residue-gated, built only after measurement shows a genuine irreducible set** (§6 — no machinery for near-zero residue). The scheduled-workflow CI-gen (`gunbc ci` / `ci_spec`) is load-bearing — escalate before editing it.

## 5. Why this unifies the §1 story

Affected-set and parallelization-by-realization are the **same work from two ends**: selection shrinks **what** runs per-PR (to the affected set); realization shrinks **how long** it takes (width / placement / cache / Share). Together they make *"run the affected set per-PR, fast"* the default — which dissolves the need for a cost-based nightly dump entirely. The only residue is (1) non-hermetic and (2) the selector backstop — neither is "expensive"; both are about soundness / determinism.

## 6. The seam, precisely

§4 selection produces the must-run set; §1 scheduling runs **that** set affordably (parallelism / cache) and runs the **complement** on a backstop cadence. Selection decides *membership*; scheduling decides *cadence / placement / width*. Cost lives only on the scheduling side.

## Dissolution trigger (DESIGN §6)

Delete this doc when the per-PR gate is the affected set (step 2 landed on top of #5427's baseline), the nightly cadence is the full-corpus backstop + non-hermetic residue (no cost-based selector), and the completeness lens runs executably — at which point the carriers (the lens + the CI-gen spec) state the policy and this framing is redundant.
