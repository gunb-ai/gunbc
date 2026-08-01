# extdeps modeling preflight — one subject per upstream module

> Operator-directed (verdict on gunbc#7556, 2026-08-01). Normative source: DESIGN §3 → "External upstream decomposition". The typed rows live in `gunbc.plans.extdeps_modeling_preflight` `extdeps_modeling_preflight_rows`; this table is rendered from them.

## The preflight

Before any new extdeps work, the agent should return this table:

| Question | Required answer |
| --- | --- |
| Independently governed upstream subjects | Enumerated by name |
| Shared standard/interface concepts | Enumerated separately |
| Product-/engine-specific facts | Assigned to their upstream modules |
| Local execution observations | Assigned downstream |
| Consumer policy or coverage | Assigned downstream |
| Exact version/build authority | Named for each upstream |
| Existing model searched | Symbols and modules listed |
| Proposed file map | One subject per implementation module |

No implementation starts until the table is coherent.

## How the answers are checked

- **One subject per module** is the `ExternalModelScope` contract (`extdeps.external_authority` — one symbolic subject `DeclarationRef` plus one-or-more citations; exact revisions live on build/release/fact rows, never on the module scope). `external_model_scope_decision` is the pure decision kernel over declared fact rows: a product-/engine-specific fact row attributed to a subject other than the module's declared subject is `ForeignSubjectRow`, and consumer coverage state stored in an upstream module is `ConsumerCoverageInUpstream`.
- **The honesty boundary** (`external_model_scope_decision_kernel_note`): the kernel judges *declared* facts, so it is not yet an admission wall. The mechanical enforcement today is scope **presence** at a declared grain (the staged frontier below — per-PR for the hermetic half, wet-lane for the live half); subject-content coherence derived from the real module tree is the named enforcement frontier `feature:extdeps-subject-content-derived`.
- **Observations and coverage live downstream** — receipts in the observing product/workflow layer (`DownstreamSupportRoster`, coverage identity symbolic), never `Unobserved` properties authored inside an upstream module.
- **Enrollment is staged** behind `gunbc.extdeps_scope_frontier`, per-PR merge-gated in both directions (codex `review 46258` / `review 46281`): hermetic — every rostered path resolves on disk, the three rosters stay disjoint, carrier files byte-declare the scope, RED controls live; diff-grain — `tools.extdeps_scope_placement_gate` (wet cheap-floor gate) refuses any added `dag/extdeps` `.dag` file outside carriers plus machinery and any frozen-manifest row naming a post-freeze-added file, with diff failure/truncation as refusals. A new module can never join the frozen manifest (remove-only, count derived); the wet live-cover witness remains the whole-population re-verification, and the storage-grain frontier dissolves into the `v2.lens.mandatory_tag` region promotion.
- **The non-precedent counterexample** is `extdeps.browser` (its disposition note names the conflation and the eventual split: `extdeps.automation.playwright` / agnostic interaction shape / transport realization).

## Dissolution trigger (DESIGN §6)

Dissolves when the preflight's decidable half is machine-consumed end-to-end: the ExternalModelScope requirement is promoted from the gunbc.extdeps_scope_frontier staging roster to a v2.lens.mandatory_tag region (compile-grain wall), at which point the subject/coverage questions are enforced by construction and this doc's residue (the genuinely judgment-shaped rows: existing-model search, proposed file map) folds into DESIGN §3's standing text.
