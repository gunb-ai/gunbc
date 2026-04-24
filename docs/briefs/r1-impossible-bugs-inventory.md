# R1 Impossible-Bugs Inventory

**Landing status:** DRAFT-ONLY. Do not convert this into narrative
until the sibling managers confirm their `[live]` sets. A bug class
is `[live]` in `docs/thesis/compositional-modeling.md` only when a
Parts 1-6 row there already proves it; otherwise it stays
`PENDING <manager>`.

This is working scaffolding for `T-Demo`'s impossible-bugs suite.
When all rows that are in R1 scope have confirmed `[live]`
evidence, the narrative lands in the story doc and this file
becomes a closed receipt.

| Bug class claimed | Demo scope | Parts 1-6 `[live]` row that proves it | Inventory status |
|---|---|---|---|
| Suboptimal-complexity contract violation | R1 (`THESIS.md` "Enumerable impossible-bug classes"; `ROADMAP.md` T-Demo `impossible_bug_class_suite_r1`) | None yet. The Part 1 algebra rows prove operation attachment, not complexity-bound rejection. | PENDING Substrate (`T-LaneE`: `complexity_merge_sort_is_nlogn`, `complexity_v3_matches_v2_oracle`) |
| Idempotency-contract violation | R1 (`THESIS.md` "Enumerable impossible-bug classes"; `ROADMAP.md` T-Demo `impossible_bug_class_suite_r1`) | None yet in the story doc's Parts 1-6 summary. The live lens register says `idempotency.dag` is behaviorally complete, but the story doc still needs the confirmed Part row before narrative. | PENDING Substrate confirmation + Release inventory update |
| Transport/type drift | R1 (`THESIS.md` "Enumerable impossible-bug classes"; `ROADMAP.md` T-Demo `impossible_bug_class_suite_r1`) | None yet. Part 7 is `[target]`; Parts 1-6 do not yet carry a live multi-target boundary-coherence row. | PENDING Surface (`T-Emit`: `emit_omni_demo_fixtures_green`) + Release fixture |
| Nested-optional flatten | R2+ (`THESIS.md` "Enumerable impossible-bug classes") | None yet. Part 3 explicitly marks nested-optional flatten `[target]`. | PENDING Surface (`T-Sub`: type-alias `where`) + later cardinality substrate owner |
| Unenumerated effects | R2+ (`THESIS.md` "Enumerable impossible-bug classes") | None yet. The story doc only cites service-level retry / effect enforcement as target-state adjacent work. | PENDING Substrate / future effect-system owner |
| Unhandled diagnostic paths | R2+ (`THESIS.md` "Enumerable impossible-bug classes") | None yet. No Parts 1-6 `[live]` row proves Tier 2 totality for division-by-zero, out-of-bounds, or force-unwrap. | PENDING future Tier 2 owner |
