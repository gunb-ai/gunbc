## What

B-ordered planning retext of ONE field (red_control) of row `roadmap-unprobed-findings-are-unsized` in `dag/gunbc/roadmap/roadmap_authority.dag`. Gap measured at origin/main a2be34842e8bb79c5af513310d1f7cd5ba294baf: the red_control named NO executed control — 0 hits in roadmap_authority.dag for either the brief's control or the substitute actually run.

Per Manager B msg_5c02c3b0 + lane-manager requirement msg_88a82d10: the clause names the substitute as the executed parent-side control and carries the substitution reason VERBATIM from worker neat-seal-10 (msg_00295ee0). Original prose assertions are preserved verbatim; nothing else in the block is touched.

## Self-checks (measured at base a2be34842e8b unless stated)

- diff: one file, 1 insertion / 1 deletion; no new row, no identity, no schema change
- `roadmap_authority.dag :: a_declared_population_item_still_prices_members_times_days` = 0; `... :: a_parent_with_priced_children_is_derived_and_totals_their_work` = 0
- `dag/test/claim/roadmap/roadmap_estimate_derivation_witness.dag` :: brief control = 1, substitute = 1 (`test fn` decl; the file declares `module test.claim.roadmap.roadmap_estimate_derivation_witness`)
- `dag/test/claim/roadmap/roadmap_estimate_unprobed_findings_red_witness.dag` :: brief control = 1, substitute = 0
- citation form `test.claim.<...>::fn` matches in-file precedent (ci_fleet.dag, fabric_executor_class.dag)
- paren balance (quoted strings stripped) = 0
- head 94cea267253048060fe54712ae88d784c5c53b2f gh-api resolved; tree 53f6383e292278e15f05f96be146f4f5aa4583cd

## Constraints honored

Plan content only: `planning_has_one_home` — nothing in the compiler may come to depend on this string; CI parse/typecheck is the automated check. If dag-check or any build step begins INSPECTING the clause beyond parsing, work stops and is reported rather than accommodated. Draft PR per lane flow; review at head requested; no self-merge.
