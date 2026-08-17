# Dissolution census A — `gunbc.ci_layer_roots` prose markers

**Status:** census complete, measured at `44126ca1de0`, 2026-08-03. No prose deleted, no carriers migrated.

**Authority:** this markdown is now the sole surviving record. It *was* a generated projection of a
revision-pinned TSV observation, and both the TSV and the projection script were deleted 2026-08-16
under the operator ruling to delete anything not actively derived. **It is therefore no longer
regenerable** — the rows below are a dated observation retained as evidence, not a live projection,
and re-deriving them means re-running the census against live `gunbc.ci_layer_roots` rather than
re-running a script.

**Classifier:** `dag-note-prose-census-lexical-v1` — Lexical sentence classifier (same honesty bound as dag-note-prose-census §6); shares are ±10pp.

**Scope note:** Dated observation pinned at the named HEAD. Evidence for selecting dissolution work — not a claim about current main. Re-read live gunbc.ci_layer_roots before acting on any row.

**Scope:** every prose-bearing site inside `dag/gunbc/ci_layer_roots.dag` — the CI floor's single-authority witness-layer, discovery-exclusion, and falsifier-roster carrier (25.3 KiB prose mass per [dag-note-prose-census.md](dag-note-prose-census.md) §1).

**Instrument:** row-level register at `docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.tsv` (263 sites) plus the generated `docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.summary.json`. **Grain key:** a *site* is one `reason`, `dissolve_on`, or `data String` field; a site is *inline prose* when `is_ref=False` (literal string body); template-ref sites (`reason: excl_*`) carry `is_ref=True` and are not prose.

---

## 0. Count reconciliation (brief claimed 135)

Three grains — do not conflate:

| grain | count | bytes | definition |
|---|---|---|---|
| **All inline prose** (every `is_ref=False` site) | **158** | 64.8 KiB (66,329 B) | Full census population |
| — module `*_note` | 24 | 33.0 KiB | Authority essays |
| — `excl_*` shared templates | 21 | 5.6 KiB | Classification-scoped reason/dissolve templates |
| — per-row `reason` / `dissolve_on` on typed rows | **113** | 26.1 KiB | `WitnessExclusionRow` + `RehomedBinWetRow` + `SubstrateLongLaneRow` |
| **Brief grain** (excl. templates — already field-shaped) | **137** | 59.2 KiB | 24 notes + 113 per-row |
| **Brief claimed** (excl. templates + NoConsumer RED-control pair) | **135** | — | 137 − 2 inline fields on `synthetic_orphan_admission_witness_test.dag` |
| Template-ref sites (`reason: excl_*`, not prose) | 105 | — | `is_ref=True` |
| **Total marker sites** (prose + refs) | **263** | — | TSV row count |

Arithmetic check: 24 + 21 + 113 = 158 inline sites.

---

## 1. Structural groups (where prose lives)

Five carriers hold all prose. Three are **already row-typed** with `reason` + `dissolve_on` fields; two are module-level notes/templates.

| structural group | sites | inline prose | role |
|---|---|---|---|
| `WitnessExclusionRow` | 154 | 49 | PATH POLICY roster — pattern + `WitnessConsumerCadence` + reason/dissolve |
| `SubstrateLongLaneRow` | 46 | 46 | Falsifier batch 6 — Class C long-lane hermetic witnesses |
| `RehomedBinWetRow` | 18 | 18 | Falsifier batch 5 — over-budget bin-execution witnesses |
| module `*_note` | 24 | 24 | Authority essays — lane policies, reconciliation receipts |
| `excl_*` shared templates | 21 | 21 | Classification-scoped reason/dissolve templates (§3 nicknaming half-fixed) |

---

## 2. Semantic classes (inline prose, dag-note-prose-census §2)

| class | markers | share | migrates to |
|---|---|---|---|
| **SPEC_NORM** | 119 | 75% | `StandingIntent` row + type/lens material |
| **RECEIPT** | 34 | 22% | event-log row (ages out) |
| **RULING** | 2 | 1% | ruling-register row |
| **XREF** | 2 | 1% | citation edge |
| **EVENT** | 1 | 1% | event-log row |

**Dissolution census finding:** unlike the corpus-wide prose census (69% multi-class notes), `ci_layer_roots` prose is **already field-separated** — reason vs dissolve_on vs module note — so the anemic-serialization problem is structural (String fields on typed rows) not paragraph-level mixing. The payoff is typing the fields, not sentence-splitting.

---

## 3. Sibling censuses

- [dag-note-prose-census.md](dag-note-prose-census.md) — corpus-wide annotation layer (864 KiB)
- [live-read-witness-classification-design.md](live-read-witness-classification-design.md) — supersedes hand exclusion rows when G2+G3 wire (same dissolution trigger as `witness_exclusion_single_authority_reconciliation_note`)
- [module-identity-storage-binding-design.md](module-identity-storage-binding-design.md) — Phase 0(b) admission invariant this carrier implements

**Dissolve-on:** typed annotation carriers land (`StandingIntent`, event log, citation edges) and an annotation-budget lens counts rows — same trigger as dag-note-prose-census.

