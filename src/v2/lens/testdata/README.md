# anemia-lens CONFIRM eval corpus

`anemia_confirm_eval_corpus.json` is the **single-authority** ground-truth set for evaluating the
anemia-lens CONFIRM judge (the leaf-side §2 decomposition detector — see DESIGN.md §2, and the
`docs/plans/anemia-lens.md` design doc). It is consumed cross-repo (e.g. `ctrl`'s coherence gate)
via the gunbc submodule at this fixed path — do **not** vendor a copy; point at this file.

Each row is one labelled field site:

| key | meaning |
|---|---|
| `id` | stable row key |
| `field` / `located` | the field and its fully-qualified site |
| `declared_type` | the anemic (or correct) declared type at the site |
| `signal` | which structural signal fires: `A` existing-authority coincidence · `B` closed-set-by-comparison · `C` consumer-cracks-the-leaf |
| `decided_by` | **mechanical split** (the only real split — there is no severity tiering): `deterministic` = decided in `.dag`, never reaches the LLM · `haiku_confirm` = the shrinking residual the haiku CONFIRM judge actually adjudicates |
| `coincides_with` | the existing authority / closed set / structured form the value should ground to |
| `label` | `positive` (a real anemia violation) or `section5-negative` (a §5 case CONFIRM must NOT clear-as-violation incorrectly) |
| `expected_confirm` | ground truth: `REAL` (it is anemia) or `CLEARED` (correctly not-a-violation) |
| `section5_kind` | for negatives, why it is legitimately atomic (`escape-payload`, `process-env`, `cited-sku`, `parse-input`, `pipeline-subject`, `constraint-grammar-residual`) |
| `grounding` | provenance (PR # or audit note) |
| `rationale` | one-line justification |

**One violation class.** There is no hard-block vs soft-warn severity. Everything the lens fires
**blocks**; the `haiku_confirm` rows are a hard blocking dependency run in observe/shadow mode as a
*rollout phase*, not a softer tier. The CONFIRM judge can only ever CLEAR a finding, so the
promotion bar is two-sided: (1) **safety** — zero wrong-clears on `positive` rows; (2) **utility** —
correctly CLEAR the `section5-negative` rows (an over-eager judge that clears everything is useless).
