# Test decomposition — W/C/F cut (measured CI head)

**Status:** AUTHORITY for the cut · signed 2026-07-25 (session `bright-newt-31`).
**Parent lane:** CI two-tier placement ([ci-two-tier-placement-redesign.md](ci-two-tier-placement-redesign.md)) — placement answers *where* (`PrTier | Gauntlet`, 5s rule); this cut answers *what shape* a row must take to become PR-eligible.
**Related:** [cost-risk-benefit-floor-model.md](cost-risk-benefit-floor-model.md) (deferred risk stays on an enrolled cadence).

This doc is the worklist, not a parallel placement axis. It dissolves when every Class C row on the measured head has landed its fixture/cadence split (or been reclassified F with a named cost-shape owner), and dark `long/` files are classified only at Gauntlet enrollment.

---

## 1. Scope honesty

- Covers the **measured head** — rows actually costing CI today.
- Dark `~59` `long/` files cost nothing yet: classify W/C/F **one by one as Gauntlet enrolls them**, each with a per-row budget. No pre-audit sweep while unscheduled.
- Every slow batch-4 row audited here is `SubstrateInputsOnly` — pure computation, zero host effects. Almost none of the CI-expensive tests are integration tests.

---

## 2. Placement vs shape (binding)

| Axis | Owns | Does not own |
|---|---|---|
| **Placement** (`PrTier \| Gauntlet`) | *where* a row runs; 5s warm admission | reshaping a claim |
| **Decomposition (this cut)** | *how* a Class C row becomes PR-eligible (mechanism RED on fixtures; corpus grain on enrolled cadence) | a second placement enum |

Class C work *feeds* D3 admission; it is not a nickname for Gauntlet.

---

## 3. Criteria (authoring + review bar)

1. **Wet/external** — subprocess, host, or network *is the subject* → large is honest; control by cadence + kill-at-budget, never per-PR. (**Class W**)
2. **Corpus-totality claim** — a wall over the live tree → split: mechanism REDs on fixtures per-PR; corpus grain on an *enrolled* cadence (named consumer — falsifier / Gauntlet). Corpus grain is irreducible but relocatable. (**Class C**)
3. **Pure and fixture-shaped but still slow** → cost-shape defect; bare-minimum-cost says fix it; decomposition is the wrong tool and absorbs the defect's signal. (**Class F**)
4. **Attribute before decomposing, always.** Decomposing a defect-inflated test makes the red disappear while the defect compounds — absorbing-fallback wearing test-hygiene clothes. The 53→141s caution generalizes.
5. Aggregated keystones over pure data (the #7202 shape — 85 families in seconds) are large in *lines*, not *time* — not a target.

---

## 4. Class W — wet/integration (do not decompose)

Large is honest; controls already in motion (falsifier cadence, kill-at-deadline, sccache health).

| row | cost (order) | note |
|---|---|---|
| `namespace_import_closure` (emit+cargo self-host) | ~21m | claim *is* emit+cargo |
| `self_host_logic_behavioral` | ~34m | same |
| `resolution_divergence` (silent-pick gate) | ~707s | whole-corpus wet resolve; nightly batch-4 |
| real-execution host witnesses | ms–s | fine on per-PR wet smoke |

**Disposition:** leave shape; place by measured cost (Gauntlet when >5s).

---

## 5. Class C — computational breadth (decomposition worklist)

Pure computation sized "over the whole corpus" where the *mechanism* does not need the breadth. Template: **mechanism RED on fixtures per-PR; corpus grain on enrolled cadence.**

| row | cost | fixture-grain per-PR version | corpus grain |
|---|---|---|---|
| `dag_compile_clean_perturb_receipts` | 141s | RED arms (`perturb_unresolved_import`, `perturb_optional_skew`) on a 3-module fixture closure in <1s | live-tree / full gate → cadence. *Attribute first* if 53→141s is #7205 residual (then reclassify F). |
| `wave1_gate1_d_census` | 141s | census mechanism over a fixture module set | corpus census → cadence (re-home already ruled) |
| `compile_clean_shard` totality + roster | 15s + 7s | shard-selection logic on fixtures | totality *claim* is corpus-grain — relocate to cadence, do not shrink |
| provenance producer ×3 | 40s | fixture walls stay per-PR (ruled) | live grain → nightly |
| `parse_binding_fidelity` | 13s (long); fixture REDs on PR path | **Verified-in-tree** (#7227): 3 cross-file/fail-closed in `parse/` via `*_test.dag` walk; 2 module-ingest in `long/` + `commit_workflow`. Notes corrected. | enrolled |
| `floor_skip_discovery` (289s), `cross_shard_seam` (183s) | re-homed | same template when they return | already on falsifier rehomed roster |

**Accept (per Class C row):** (a) per-PR mechanism RED green-by-execution under the 5s fast-lane (or bin per-row budget); (b) corpus grain enrolled on a named cadence consumer (not orphaned); (c) attribute receipt if cost was defect-inflated — else Class F hand-off.

---

## 6. Class F — fixture-shaped, slow by defect (out of decomposition)

Decomposing buys nothing; fix the cost-shape. Shrinks Classes C and F together.

| row | cost | note |
|---|---|---|
| `emit_host_classical_not_ingested` | ~13s | already on `infer_fixture`; pays per-entry v2 pipeline (~11s reconcile/1-module) + re-entrant-compile / memo-drop (`wasted_ms≈174s`) — #7205 residual class |
| wave1 `ingested_bind` / `loop` rows | 27–29s | same family (eval-by-execution per construct) |

**Disposition:** route to cost-shape / #7205 residual lane. Explicit **non-work** for Class C children.

---

## 7. Future enrollment rule (dark `long/`)

When Gauntlet (or falsifier wet cadence) enrolls a `long/` file:

1. Measure warm cost.
2. Classify W / C / F with a one-line attribute receipt.
3. Assign per-row budget.
4. Only Class C enters this worklist.

---

## 8. Dissolution

This doc retires when:

- every §5 Class C row is Accept-green or reclassified with owner;
- §6 Class F rows have a named cost-shape owner (not this lane);
- §7 rule is cited from the Gauntlet enrollment path (two-tier D3 / falsifier roster notes).

Authority then collapses into `CiSpec` placement rows + enrolled cadence rosters — no parallel ledger.
