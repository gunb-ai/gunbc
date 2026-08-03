# Dissolution census A — `gunbc.ci_layer_roots` prose markers

**Status:** census complete, measured at `44126ca1de0`, 2026-08-03. No prose deleted, no
carriers migrated.

**Scope:** every prose-bearing site inside `dag/gunbc/ci_layer_roots.dag` — the CI floor's
single-authority witness-layer, discovery-exclusion, and falsifier-roster carrier (25.3 KiB
prose mass per [dag-note-prose-census.md](dag-note-prose-census.md) §1).

**Instrument:** row-level register at
`docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.tsv` (269 sites) plus
`docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.summary.json`. Classifier is
lexical (same honesty bound as dag-note-prose-census §6); shares are ±10pp.

---

## 0. Count reconciliation (brief claimed 135)

| grain | count | bytes |
|---|---|---|
| **Inline prose markers** (the brief's subject) | **137** | 66.3 KiB |
| — module-level `*_note` / `excl_*` templates | 45 | 29.0 KiB |
| — per-row `reason` / `dissolve_on` on typed rows | 92 | 37.3 KiB |
| Template-ref sites (`reason: excl_*`, not prose) | 111 | — |
| **Total marker sites** (prose + refs) | **269** | — |

The brief's **135** is within the §6 error bar: 137 at this head, delta 2 — plausibly two
notes added since the brief was authored (`bin_witness_wet_per_row_actions_log_observation_gap_note`,
`known_red_probe_note` 2026-08-03 single-authority rewrite) or the brief excluded the two
synthetic RED-control rows (`NoConsumer`, `FixtureExplicitRoster`).

---

## 1. Structural groups (where prose lives)

Five carriers hold all prose. Three are **already row-typed** with `reason` + `dissolve_on`
fields; two are module-level notes/templates.

| structural group | sites | inline prose | role |
|---|---|---|---|
| `WitnessExclusionRow` | 154 | 49 | PATH POLICY roster — pattern + `WitnessConsumerCadence` + reason/dissolve |
| `SubstrateLongLaneRow` | 48 | 46 | Falsifier batch 6 — Class C long-lane hermetic witnesses |
| `RehomedBinWetRow` | 20 | 18 | Falsifier batch 5 — over-budget bin-execution witnesses |
| module `*_note` | 24 | 24 | Authority essays — lane policies, reconciliation receipts |
| `excl_*` shared templates | 21 | 21 | Classification-scoped reason/dissolve templates (§3 nicknaming half-fixed) |

### `WitnessExclusionRow` by `WitnessConsumerCadence`

| classification | rows | inline reason | template-ref reason |
|---|---|---|---|
| `OfflineLocalRecipe` | 44 | 22 | 22 |
| `BinWitnessWet` | 23 | 1 | 22 |
| `FalsifierSubstrateLongLane` | 4 | 0 | 4 |
| `FalsifierRehomedBinWet` | 3 | 0 | 3 |
| `FixtureExplicitRoster` | 2 | 2 | 0 |
| `NoConsumer` | 1 | 1 | 0 (synthetic RED control) |

**Finding:** 52 of 77 exclusion rows (68%) delegate reason/dissolve to shared `excl_*`
templates — the right consolidation move is already half-done. The remaining 25 carry
**substantiated per-row** prose (artifact_store, external_model_scope, stage0_rust_host_observation,
etc.) that cannot collapse without losing discriminating receipts.

---

## 2. Module-note topic groups

Twenty-four module notes (excl. templates) cluster into seven topics:

| topic | notes | bytes | dissolution posture |
|---|---|---|---|
| **Roster-gate / offline exclusions** | 7 | 9.0 KiB | Dissolve when G2 path-intersection + scheduled consumers land (enforcement-coverage, accumulator-copy, identity-captured-navigation, lever-a, whole-tree scaffold) |
| **Witness-class policies** | 7 | 8.2 KiB | Shrink as rows graduate: long-lane, wet-integration, bin-wet, falsifier-self-host, install-media, per-row budget |
| **Authority reconciliation** | 3 | 6.6 KiB | `witness_exclusion_single_authority_reconciliation_note` is the meta-note; dissolves when enforcement-intent registry absorbs roster |
| **Falsifier lanes** | 2 | 4.1 KiB | substrate-long + silent-pick gate notes |
| **Scoped v1 batch** | 2 | 3.8 KiB | Dissolves on `V2ParserOwnsV1Claims` |
| **Admission lane** | 3 | 2.2 KiB | Phase 0(b) offline/fixture/transport posture |
| **Shared templates** | 21 | 5.7 KiB | Already field-shaped; key by `WitnessConsumerCadence` variant |

Largest single row: `known_red_frontier_note` (3.4 KiB) — mostly **EVENT** history that
should become event-log rows, not live spec.

---

## 3. Semantic classes (inline prose, dag-note-prose-census §2)

| class | markers | share | migrates to |
|---|---|---|---|
| **SPEC_NORM** | 119 | 75% | `StandingIntent` row + type/lens material |
| **RECEIPT** | 34 | 22% | event-log row (ages out) |
| **RULING** | 2 | 1% | ruling-register row |
| **XREF** | 2 | 1% | citation edge |
| **EVENT** | 1 | <1% | event-log row |

**Dissolution census finding:** unlike the corpus-wide prose census (69% multi-class notes),
`ci_layer_roots` prose is **already field-separated** — reason vs dissolve_on vs module
note — so the anemic-serialization problem is structural (String fields on typed rows) not
paragraph-level mixing. The payoff is typing the fields, not sentence-splitting.

---

## 4. Dissolution migration buckets (sequencing)

| bucket | population | first move |
|---|---|---|
| **A — row fields** | 92 inline reason/dissolve_on on typed rows | Promote `reason` to typed `Cause` coproduct; `dissolve_on` to `DissolutionTrigger` ref (pattern exists in `gunbc.non_fold_residue`) |
| **B — templates** | 21 `excl_*` + 111 refs | Single `WitnessConsumerCadence → (reason_template, dissolve_template)` table; delete refs |
| **C — module notes** | 24 `*_note` | Split per §3: SPEC→StandingIntent, RECEIPT/EVENT→event log, shrink meta-notes as rows absorb |
| **D — external echo** | 58 cross-file `data String` refs | Consumer update pass when A–C land (79 `.dag` files import `gunbc.ci_layer_roots`) |

**Do not plan deletion:** 0.6% crisp-deletable history (dag-note-prose-census §3) applies
here too — every row binds live floor admission. The rate argument (§4, ~2 KiB/PR append) is
the justification for typed expiry, not a cleanup pass.

---

## 5. Sibling censuses

- [dag-note-prose-census.md](dag-note-prose-census.md) — corpus-wide annotation layer (864 KiB)
- [live-read-witness-classification-design.md](live-read-witness-classification-design.md) —
  supersedes hand exclusion rows when G2+G3 wire (same dissolution trigger as
  `witness_exclusion_single_authority_reconciliation_note`)
- [module-identity-storage-binding-design.md](module-identity-storage-binding-design.md) —
  Phase 0(b) admission invariant this carrier implements

**Dissolve-on:** typed annotation carriers land (`StandingIntent`, event log, citation edges)
and an annotation-budget lens counts rows — same trigger as dag-note-prose-census.
