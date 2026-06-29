# anemia-lens CONFIRM eval corpus

`anemia_confirm_eval_corpus.json` is the **single-authority** ground-truth set for evaluating the
anemia-lens CONFIRM judge (the leaf-side §2 decomposition detector — see DESIGN.md §2). It is consumed
cross-repo (e.g. `ctrl`'s coherence gate) via the gunbc submodule at this fixed path — do **not** vendor
a copy; point at this file. The design rationale lives in the anemia-lens design doc (PR #5302, not yet
on `main`).

**What this corpus is.** A fixed, labelled **decision-eval**: each row is one
`(declared_type, signal, coincides_with)` input plus the correct verdict (`expected_confirm`). The judge
is scored by **reconstructing its input from these row fields** — the harness does **not** re-bind
`located` against the live DAG. That distinction matters: the audited positive sites have **since been
grounded** by the extdeps cleanup program (that's the whole point — the cleanup succeeded), so the live
field at `located` now carries `grounded_now`, not `declared_type`. The merged grounding **is** the
ground-truth evidence that the `positive`/`REAL` label is correct; it is not a live finding against
current `main`.

Each row is one field site:

| key | meaning |
|---|---|
| `id` | stable row key |
| `field` | the field/param name |
| `located` | its **current** fully-qualified site — resolves in-tree (real module.Type.field, or fn param) |
| `declared_type` | the **audit-time anemic** type the CONFIRM judge is shown (the reconstructed input) |
| `grounded_now` | the type that has **since replaced** `declared_type` at `located` (positives), the current legitimately-atomic type for the extensible/constraint negatives, or `null` for plain atomic negatives |
| `signal` | which structural signal fires: `A` existing-authority coincidence · `B` closed-set-by-comparison · `C` consumer-cracks-the-leaf |
| `decided_by` | **mechanical split** (the only real split — there is no severity tiering): `deterministic` = decided in `.dag`, never reaches the LLM · `haiku_confirm` = the residual the haiku CONFIRM judge adjudicates |
| `coincides_with` | the existing authority / closed set / structured form the value should ground to |
| `label` | `positive` (a real anemia violation) or `section5-negative` (a §5 case CONFIRM must NOT clear-as-violation incorrectly) |
| `expected_confirm` | ground truth: `REAL` (it is anemia) or `CLEARED` (correctly not-a-violation) |
| `section5_kind` | for negatives, why it is legitimately atomic (`escape-payload`, `process-env`, `cited-sku`, `parse-input`, `pipeline-subject`, `constraint-grammar-residual`) |
| `grounding` | provenance of the label (PR # or audit note) |
| `rationale` | one-line justification |

**One violation class.** There is no hard-block vs soft-warn severity. Everything the lens fires
**blocks**; the `haiku_confirm` rows are a hard blocking dependency run in observe/shadow mode as a
*rollout phase*, not a softer tier. The CONFIRM judge can only ever CLEAR a finding, so the promotion
bar is two-sided: (1) **safety** — zero wrong-clears on `positive` rows; (2) **utility** — correctly
CLEAR the `section5-negative` rows (an over-eager judge that clears everything is useless).
