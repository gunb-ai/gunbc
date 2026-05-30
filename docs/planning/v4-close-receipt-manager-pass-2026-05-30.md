# Close / Receipt Manager Pass — 2026-05-30

**Manager session:** `sharp-otter-407` (Close/Receipt lane per PR #3938 §11.1).
**Cites:** `docs/planning/v4-correctness-ladder-2026-05-30.md` (PR #3938, merged 2026-05-30 as `b129ce3f2`) §3, §6, §8, §10.0, §11. The operator merge of #3938 without per-decision answers effectively ratified the §8 PM-recommendations; this receipt records that ratification explicitly and adds the policy substance (close grades, anti-shelfware deadlines) within Close/Receipt-lane authority.
**Authority scope (from §11.1):** close predicates, two-axis disposition vocabulary, ladder ↔ questionnaire complementarity, anti-shelfware deadline policy. **No implementation work.**

This doc records the Close/Receipt manager-pass outcomes called for by PR #3938 §11.4 item 1. Where §8 names a decision as operator-authority (D4, D7) or as a substantive policy choice the manager should not unilaterally cement (D5), the pass records a **manager-recommendation** posture, not ratification.

---

## §1. Two-axis disposition vocabulary — RATIFIED

The two-axis vocabulary defined in PR #3938 §10.0 is **adopted as the Close/Receipt-authoritative close-readiness vocabulary** across:

- `docs/v4-close-interrogation.md` (questionnaire, 346 probes)
- `docs/audit/v4-close-interrogation-validation-2026-05-30.md` (PR #3941)
- all future close-related audit, planning, and validation artifacts in `docs/audit/` and `docs/planning/`

Axes (verbatim from §10.0, copied here as the canonical surface):

```text
ship_disposition:
  PROVEN | GAP | NOT_IN_V4 | NOT_PROMISED | OPERATOR_DECISION_REQUIRED

engineering_state:
  SUBSTRATE_PRESENT
  SCAFFOLD_PRESENT
  PARTIAL_GATE_PRESENT
  CENSUS_NOT_RUN
  EXECUTION_NOT_WIRED
  NO_ARTIFACT_FOUND
```

**Closure invariant (mechanical, restated and enforced by this lane):**

> A probe cannot move to `ship_disposition: PROVEN` by adding more substrate declarations or scaffolds. It moves to `PROVEN` only by adding an executable receipt that answers the exact probe and includes the falsification case when one was requested. `engineering_state` is orthogonal to `ship_disposition` and is never a substitute for it.

**Migration of the prior single-axis vocabulary.** The legacy single-axis terms (`PROVEN / WEAK-EVIDENCE / GAP / NOT-CHECKED / OPERATOR-DECISION-REQUIRED / NOT-IN-V4 / NOT-PROMISED`) used by the §0 questionnaire and the 2026-05-30 validation map as follows:

| Legacy | New `ship_disposition` | New `engineering_state` |
| ------ | ----------------------- | ----------------------- |
| `PROVEN` | `PROVEN` | n/a (closure reached) |
| `WEAK-EVIDENCE` | `GAP` | one of `SUBSTRATE_PRESENT` / `SCAFFOLD_PRESENT` / `PARTIAL_GATE_PRESENT` |
| `GAP` | `GAP` | `EXECUTION_NOT_WIRED` or `NO_ARTIFACT_FOUND` |
| `NOT-CHECKED` | `GAP` | `CENSUS_NOT_RUN` |
| `OPERATOR-DECISION-REQUIRED` | `OPERATOR_DECISION_REQUIRED` | as observed |
| `NOT-IN-V4` | `NOT_IN_V4` | n/a |
| `NOT-PROMISED` | `NOT_PROMISED` | n/a |

The most important consequence: **`WEAK-EVIDENCE` does not survive the migration as a ship axis value.** Substrate-present-but-not-gated rows become `ship_disposition: GAP, engineering_state: SUBSTRATE_PRESENT`. This makes the substrate-rich / activation-poor pattern (PR #3938 §3, confirmed by #3941's headline of `0 PROVEN / 346 GAP` with `233 SUBSTRATE_PRESENT / 68 NO_ARTIFACT_FOUND / 45 CENSUS_NOT_RUN`) impossible to misread as "partially close-ready."

**Effective:** all artifacts authored after this receipt lands. PR #3941 already emits under the two-axis vocabulary (see its summary table at `docs/audit/v4-close-interrogation-validation-2026-05-30.md:18` and per-row tables from `:71`); this receipt formalizes that vocabulary as the canonical close-readiness surface for every subsequent artifact rather than overwriting #3941.

---

## §2. Close predicates — RATIFIED

The Close/Receipt lane is the authority for what "closed" means at the receipt boundary. The lane recognizes three distinct close grades; mixing them is the documented failure mode (`DONE` recorded against substrate-only work).

**Axis disambiguation (load-bearing).** The grades `SUBSTRATE_CLOSED` / `GATE_CLOSED` / `RECEIPT_CLOSED` are **lane-level grades** — what a manager reports for a lane of work. They are NOT probe-level `ship_disposition` values. A `GATE_CLOSED` lane grade does NOT promote any probe to `ship_disposition: PROVEN`; the §1 closure invariant (executable receipt + falsification when requested) is the only path to probe-level `PROVEN`. The §2.4 mechanical rule below uses the grades; per-probe rows (e.g. the §1 migration table and the per-probe ledger) use `ship_disposition` × `engineering_state`. The two axes never collapse.

### §2.1 `SUBSTRATE_CLOSED`

A modeling-only close. Substrate types, marks, and worksheets exist; no executable receipt has fired. Allowed only when the lane explicitly carries a follow-on activation work item (see §4 anti-shelfware policy below) and the disposition row reads `ship_disposition: GAP, engineering_state: SUBSTRATE_PRESENT`.

**`SUBSTRATE_CLOSED` is never sufficient for v4-done or for a ladder-rung gate.** A `SUBSTRATE_CLOSED` lane that has no live activation work item is, by definition, shelfware and is reopened by this lane on next audit.

### §2.2 `GATE_CLOSED`

A ladder-rung-style close: a gate fires on PRs against a defined fixture (or fixture set), and the gate has produced at least one passing receipt and at least one falsification receipt (negative-case rejection demonstrated). **Effect on covered probes:** their `engineering_state` advances to `PARTIAL_GATE_PRESENT` (fixture-only) or unmarked (corpus-wide); `ship_disposition` stays `GAP` until corpus widening AND falsification both land per the §1 closure invariant. `GATE_CLOSED` is therefore a lane-progress signal, not a probe-close signal.

### §2.3 `RECEIPT_CLOSED`

The release-grade close. All six TASKS.md:801-815 v4-done predicates hold (pending the §3 D4 disposition). For a single lane: an executable, reproducible, falsification-tested receipt exists, the gate fires on full corpus (not just fixture), and the lane's questionnaire-section probes have all moved to `ship_disposition: PROVEN`.

### §2.4 Mechanical rule

| Lane reports | This lane's stance |
| ------------ | ------------------ |
| `DONE` without naming one of the three grades | rejected; lane must re-report with grade |
| `SUBSTRATE_CLOSED` without a live activation work item | reopened; counts as shelfware |
| `GATE_CLOSED` claim without a falsification receipt | downgraded to `SUBSTRATE_CLOSED` until falsification lands |
| `RECEIPT_CLOSED` claim against fixture only | downgraded to `GATE_CLOSED` until corpus widening lands |

Worker briefs and manager passes use these grades verbatim; PR descriptions cite the grade in the form `Close grade: SUBSTRATE_CLOSED` / `GATE_CLOSED` / `RECEIPT_CLOSED`.

---

## §3. §8 decision dispositions (Close/Receipt-lane recommendations)

| § | Decision | Authority | Pass posture |
| - | -------- | --------- | ------------ |
| D1 | Ladder ontology + #6 gap | Close/Receipt (per §11.3) | **RATIFIED** with #6-gap **Option C** (out of v4 ladder scope; tracked under T-25 refinement substrate). The ladder ↔ questionnaire complementarity claim from D1's modified text is **adopted**: questionnaire stays as probe surface, ladder stays as gate-sequencing surface, both adopt the two-axis vocabulary above. |
| D2 | Fixture-first vs broad-rustc-first | Close/Receipt (per §11.3, secondary Ladder/Fixture) | **RATIFIED** PM-recommendation: small fixture first. Rationale unchanged from §8.D2; the substrate-rich / activation-poor pattern is the documented failure mode this sequencing is designed to break. |
| D3 | Rung 5 as release gate | Ladder/Fixture (primary) | **Deferred** to the Ladder/Fixture manager; this lane has no independent position. |
| D4 | Rung 7 / TASKS.md v4-done definition | **Operator** (TASKS.md is operational authority) | **Manager-recommendation: Option A** (all six TASKS.md:801-815 predicates remain release gate; §7 extends with phases 5+ rather than narrowing the v4-done definition). The Close/Receipt lane cannot ratify a change to operational authority. **Posture: recommendation-pending-operator.** |
| D5 | Anti-shelfware deadline policy | Close/Receipt | **Manager-recommendation; see §4 below.** This lane authors the policy; operator ratifies the deadline-shape. **Posture: recommendation-pending-operator.** |
| D6 | §7 fixture-first phases | Ladder/Fixture (primary) | **Deferred** to the Ladder/Fixture manager. |
| D7 | Phase 0 retrospective ratification | **Operator** (Phase 0 was operator-dispatched) | **Manager-recommendation: ratify.** Phase 0 (PR #3941, merged) produced the realistic distribution under the two-axis vocabulary: `0 PROVEN / 346 GAP`, engineering split `233 SUBSTRATE_PRESENT / 68 NO_ARTIFACT_FOUND / 45 CENSUS_NOT_RUN` (`docs/audit/v4-close-interrogation-validation-2026-05-30.md:63`), confirming substrate-rich / activation-poor at probe granularity; redo brief landed within target window. (An earlier PR #3938 §8.D7 draft cited an obsolete first-pass split of `267 WEAK-EVIDENCE / 42 GAP / 37 NOT-CHECKED` before the redo landed; this receipt supersedes those counts with the merged headline.) **Posture: recommendation-pending-operator (operator merge of #3938 implicitly ratifies).** |

D4, D5, D7 are explicitly recorded as recommendation-pending so that operator sign-off remains the closing act, not this manager pass.

---

## §4. Anti-shelfware deadline policy (D5) — Close/Receipt proposal

Per PR #3938 §11.4, the receipt removes "residual 30-day framing" from the ladder. The ladder itself is not calendar-bound — its gates are state-bound (a rung is met when its predicate fires, not when a clock runs out). However, the **anti-shelfware policy** is structurally deadline-bound: the entire purpose is to dissolve the substrate-rich / activation-poor pattern by making un-activated substrate a tracked, blocking debt.

This lane therefore separates the two:

- **Ladder:** no calendar dates. Rung predicates are the only gate.
- **Anti-shelfware policy:** deadline-bound, but **per lens family**, not a blanket 30-day rule applied to every substrate landing.

### §4.1 Per-lens-family dissolution deadlines

When a substrate PR lands without a same-PR activation, the lane owner files an **activation debt** dashboard work item at merge time. The debt has:

- **Lens family** (e.g., omni-emission, lens self-application, TestClaim execution, target realization, refinement)
- **Activation predicate** (the concrete gate-firing condition that resolves the debt)
- **Dissolution window** (per family, see table below)
- **Blocking scope** (the lens family — further substrate PRs in the same family are blocked until the debt resolves or the operator extends the window with named rationale)

### §4.2 Recommended dissolution windows

| Lens family | Recommended window | Rationale |
| ----------- | ------------------ | --------- |
| omni-emission (rungs 0–2) | 14 days | tightest because the per-fixture gate shape is already designed; landing substrate without firing the gate is a small step away |
| TestClaim execution (rung 4) | 30 days | T-38 runner is the gating dependency; not authored in 14-day windows |
| lens self-application (rung 9) | 45 days | requires lens-on-CI-data wiring; longest acceptable substrate-only window |
| target realization rows (per language) | 21 days | row authoring is mechanical once the parent realization type lands |
| refinement / partiality (rung gap #6) | out of v4 scope per D1 Option C | no v4 deadline; tracked under T-25 |

**Default fallback:** when a substrate PR is filed with no named lens family (or the family is not in the table above), the dissolution window is **30 days**. The default exists so that the absence of a family name does not let shelfware live indefinitely; the per-family table above takes precedence when the family is named.

**Single-substrate single-deadline rule:** a substrate PR may not carry more than one named activation debt. Bundling multiple lens families into one substrate PR forces the worst-case (longest) window to apply to all — explicit incentive against omnibus substrate landings.

**Operator-extension shape:** any window extension is a dashboard work item authored by the operator with named rationale, not a manager-pass act. This keeps the Close/Receipt lane honest: the lane can warn and block, but cannot quietly grant extensions.

### §4.3 Why not blanket 30 days

The §8.D5 PM-recommendation proposed "≤30 days from substrate landing" as a blanket. Two problems:

1. A 30-day window for rung 0–2 (omni-emission) is generous; the fixture-first sequencing in §7 expects those activation gates to fire within a single fixture phase.
2. A 30-day window for rung 9 (lens self-application) is too tight; the wiring work the operator has scheduled is multi-week and a uniform 30-day clock would generate a debt that auto-blocks the family it depends on.

The per-family table above is the smallest deviation from the §8.D5 spirit that survives contact with the actual sequencing.

---

## §5. Ladder ↔ questionnaire complementarity — RATIFIED

This lane formally adopts the §9 framing:

- **Questionnaire** (`docs/v4-close-interrogation.md`, 346 probes) is the **granular probe surface**. Each probe receives a `(ship_disposition, engineering_state)` row.
- **Ladder** (§6 of PR #3938, 9 rungs) is the **gate-sequencing surface**. Each rung is a single predicate that fires on PRs against a defined fixture or corpus.

**Complementarity rule:** every ladder rung names the questionnaire sections it activates verification for (§9.2 cross-map is canonical). A rung that has fired `GATE_CLOSED` against a fixture moves the corresponding questionnaire probes to `PARTIAL_GATE_PRESENT` engineering state; their `ship_disposition` stays `GAP` until the §1 closure invariant is satisfied (executable receipt that answers the probe, plus a falsification receipt where the probe is explicitly adversarial). Corpus-widening from fixture to full corpus is **necessary but not sufficient** for `ship_disposition: PROVEN`; the §1 receipts (and the §5 rung gate where applicable) are also required.

**Closure direction is one-way:** an audit may find that a ladder rung is *not actually firing* against the receipts it claims; in that case, the questionnaire probes are downgraded and the rung's `GATE_CLOSED` is reopened. The questionnaire never grants close on its own — a `PROVEN` probe still requires the relevant rung's gate to be firing AND the §1 receipts to exist.

---

## §6. What this manager-pass changes (and what it doesn't)

**Changes:**

- The two-axis vocabulary (§1) is the canonical close-readiness surface from now on.
- Three close grades (§2) replace bare `DONE` in receipt language; manager passes and worker briefs cite the grade verbatim.
- The per-lens-family deadline table (§4.2) is the Close/Receipt-lane proposal for D5; operator ratification finalizes it.
- The ladder ↔ questionnaire complementarity (§5) is formally adopted.

**Does NOT change:**

- TASKS.md:801-815 — the v4-done definition is **untouched**; D4 is recommendation-pending.
- The §6 ladder itself — D1 is ratified with Option C for the rung-#6 gap, but no rung definitions are altered here.
- PR #3941's existing rows — they already emit under the two-axis vocabulary (per §1 above); this receipt does not retro-edit them, it ratifies the vocabulary they use as the canonical surface going forward.
- Any worker-dispatch decisions — those remain with the per-lane managers (Modeling DFS, Ladder/Fixture, Compiler Spine, Target Realization, Runtime/TestClaim, Self-host/Release).

---

## §7. Dependencies and downstream effects

- **Cites (now merged):** PR #3938 (`b129ce3f2`) §3, §6, §8, §10.0, §11.
- **Unblocks:** Ladder/Fixture manager pass (§11.4 item 3) can cite §2 close grades and §5 complementarity. Modeling DFS manager pass (§11.4 item 2) can cite §1 vocabulary in worksheet rows.
- **Future audit cadence:** this lane re-audits every `SUBSTRATE_CLOSED` lane on a per-window basis (per §4.2 table) rather than on a single calendar clock.

---

## §8. Related artifacts

- `docs/planning/v4-correctness-ladder-2026-05-30.md` (PR #3938) — parent planning doc; this receipt depends on it.
- `docs/v4-close-interrogation.md` — 346-probe questionnaire; future runs use §1 vocabulary.
- `docs/audit/v4-close-interrogation-validation-2026-05-30.md` (PR #3941, merged) — Phase 0 validation; already emits two-axis vocabulary (`ship_disposition` × `engineering_state`).
- `docs/audit/v4-deferral-audit-2026-05-29.md` — names the substrate-rich / activation-poor pattern this receipt's policy targets.
- `src/v4/TASKS.md:801-815` — operational v4-done definition; D4 recommendation-pending here.
