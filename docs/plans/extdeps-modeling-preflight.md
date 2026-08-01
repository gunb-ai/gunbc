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
- **The honesty boundary** (`external_model_scope_decision_kernel_note`): the kernel judges *declared* facts, so it is not yet an admission wall. The mechanical wall today is scope **presence** (the frozen-manifest cover below); subject-content coherence derived from the real module tree is the named enforcement frontier `feature:extdeps-subject-content-derived`.
- **Observations and coverage live downstream** — receipts in the observing product/workflow layer (`DownstreamSupportRoster`, coverage identity symbolic), never `Unobserved` properties authored inside an upstream module.
- **Enrollment is staged** behind `gunbc.extdeps_scope_frontier`: every `dag/extdeps` file is a scope carrier, machinery-exempt, or a row of the frozen legacy manifest (`legacy_extdeps_scope_frontier.tsv`, remove-only, count derived); the live cover witness refuses a file in none of the three, so a new module adopts a scope from birth.
- **The non-precedent counterexample** is `extdeps.browser` (its disposition note names the conflation and the eventual split: `extdeps.automation.playwright` / agnostic interaction shape / transport realization).

## Dissolution trigger (DESIGN §6)

Dissolves when the preflight's decidable half is machine-consumed end-to-end: the ExternalModelScope requirement is promoted from the gunbc.extdeps_scope_frontier staging roster to a v2.lens.mandatory_tag region (compile-grain wall), at which point the subject/coverage questions are enforced by construction and this doc's residue (the genuinely judgment-shaped rows: existing-model search, proposed file map) folds into DESIGN §3's standing text.
