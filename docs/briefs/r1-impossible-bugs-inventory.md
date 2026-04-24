# R1 Impossible-Bugs Inventory

**Landing status:** DRAFT-ONLY. Do not convert this into narrative
until the authoritative rows below are live. This file is not a
readiness ledger: `THESIS.md` owns the committed bug-class list,
`ROADMAP.md` owns R1 gate readiness, and
`docs/thesis/compositional-modeling.md` owns any Parts 1-6
`[live]` proof row used by the narrative.

This is working scaffolding for `T-Demo`'s impossible-bugs suite.
When the R1 authority rows prove the demo claims, the narrative
lands in the story doc and this file becomes a closed receipt.

| Bug class claimed | Scope authority | Current Parts 1-6 proof row in story doc | Authoritative row to cite before narrative |
|---|---|---|---|
| Suboptimal-complexity contract violation | `THESIS.md` "Enumerable impossible-bug classes" tags this `[R1]`; `ROADMAP.md` T-Demo scopes it under `impossible_bug_class_suite_r1`. | None yet. The Part 1 algebra rows prove operation attachment, not complexity-bound rejection. | `ROADMAP.md` T-LaneE gates `complexity_merge_sort_is_nlogn` and `complexity_v3_matches_v2_oracle`. |
| Idempotency-contract violation | `THESIS.md` "Enumerable impossible-bug classes" tags this `[R1]`; `ROADMAP.md` T-Demo scopes it under `impossible_bug_class_suite_r1`. | None yet in the story doc's Parts 1-6 summary. The live lens register says `idempotency.dag` is behaviorally complete, but the story doc still needs the confirmed Part row before narrative. | `docs/v3-lens-capability-register.md` marks `idempotency.dag` behaviorally COMPLETE; `ROADMAP.md` T-Demo owns the R1 demo proof. |
| Transport/type drift | `THESIS.md` "Enumerable impossible-bug classes" tags this `[R1]`; `ROADMAP.md` T-Demo scopes it under `impossible_bug_class_suite_r1`. | None yet. Part 7 is `[target]`; Parts 1-6 do not yet carry a live multi-target boundary-coherence row. | `ROADMAP.md` T-Emit gate `emit_omni_demo_fixtures_green` and T-Demo `fixture_integration_canonical`. |
| Nested-optional flatten | `THESIS.md` "Enumerable impossible-bug classes" tags this `[R2+]`; `ROADMAP.md` T-Demo excludes it from R1 demo scope. | None yet. Part 3 explicitly marks nested-optional flatten `[target]`. | `docs/thesis/compositional-modeling.md` Part 3 gap cites `ROADMAP.md:305` plus DB-11 alias-RHS closure at `ROADMAP.md:231`. |
| Unenumerated effects | `THESIS.md` "Enumerable impossible-bug classes" tags this `[R2+]`; `ROADMAP.md` T-Demo excludes it from R1 demo scope. | None yet. The story doc only cites service-level retry / effect enforcement as target-state adjacent work. | `ROADMAP.md` T-Demo `impossible_bug_class_suite_r1` row is the concrete authority excluding this from R1. Before any narrative claims it `[live]`, a future ROADMAP row must name the post-R1 effect-set gate; DB-18/DB-20 are adjacent workflow-effect receipts, not the full declared-vs-actual effect-set trigger. |
| Unhandled diagnostic paths | `THESIS.md` "Enumerable impossible-bug classes" tags this `[R2+]`; `ROADMAP.md` T-Demo excludes it from R1 demo scope. | None yet. No Parts 1-6 `[live]` row proves Tier 2 totality for division-by-zero, out-of-bounds, or force-unwrap. | `docs/thesis-validation-plan.md` T2.1-T2.4 names the concrete validation blockers: refinement types for division-by-zero / overflow / bounds, plus total runtime-helper completeness for optional extraction. |
