# R4 ctrl-migration — Phase 1.5 subsystem receipt trail

**Status**: skeleton (2026-05-12). Operational SoT for the §6 staged-debt throttle + Wave-2 fanout gate of the ctrl/ → .dag migration program (project plan: [`docs/r4-ctrl-dag-migration-project-plan.md`](../r4-ctrl-dag-migration-project-plan.md); Mgr brief: [`docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md`](../briefs/r4-ctrl-migration-subsystem-modeling-manager.md)).

**Ownership** (per Verification Mgr `deep-badger-38` ratification `msg_5f8db22f` + `msg_6faaf178` 2026-05-12):
- **Subsystem-Modeling Mgr** (`merry-newt-448`) owns row inserts on Phase 1.5 merge.
- **Verification Mgr** (`deep-badger-38`) owns column semantics + flips `parity_passed` when the named gate is green.
- **Emission-Targets Mgr** (`deep-ibex-326`) authors the Phase 3 source-fact authority + render-projection PRs whose merge flips `phase3_emission_landed`.
- **Substrate Mgr** owns Phase 1 algebra substrate whose merge flips `algebra_landed` for algebra-consumer rows.

**Single SoT discipline**: this is the **only** ledger for the Phase 1.5 staged-debt budget. Do not fork sibling files. If a JSON mirror is later needed for scripts, it lives in this same directory as a derived artifact.

---

## Column semantics

| column | type | flipped by | evidence required |
|---|---|---|---|
| `subsystem_id` | string | row insert (Subsystem-Modeling Mgr) | snake_case name matching `dsl/ctrl/<subsystem>.dag` |
| `catalog_row` | int | row insert | project-plan §3 row number (1–16) |
| `algebra_landed` | bool | Substrate Mgr (or N/A for non-consumers) | Phase-1 substrate PR URL + merge SHA, OR `N/A` for rows not consuming algebra |
| `phase15_pr_merged` | bool | Subsystem-Modeling Mgr | modeling PR URL + merge SHA |
| `phase3_emission_landed` | bool | Emission-Targets Mgr | emission PR URL + merge SHA + canonical digest-declaration ref path + projection module path |
| `parity_passed` | bool | Verification Mgr | gate_id string + CI job URL OR in-tree test path |
| `open_receipt_debt` | derived | computed | `phase15_pr_merged ∧ ¬(phase3_emission_landed ∧ parity_passed)` |

**Dispatch-pause gate (Subsystem-Modeling Mgr polls before any new Phase 1.5 worker dispatch)**: `count(rows where open_receipt_debt = true) ≥ 3` ⇒ **pause** new Wave-1 / Wave-2 dispatch until catch-up (Phase-3 landings or parity flips reduce count below 3). Director-confirmed wording: STAGED Phase 1.5 subsystem merges without both emission-target landing and parity must not accumulate past 2 open debts.

**Wave-1-trio gate**: Wave-2 fanout unblocked only when a designated trio-anchor row has all four `*_landed` / `*_passed` booleans `true`. Current trio anchor: catalog #8 (PR digests).

---

## Ledger

Cells use compact `bool · evidence` form. `—` denotes not-yet-applicable (e.g. `algebra_landed` for non-consumer rows). Empty evidence = column not yet flipped.

| subsystem_id | catalog_row | algebra_landed | phase15_pr_merged | phase3_emission_landed | parity_passed | open_receipt_debt |
|---|---|---|---|---|---|---|
| `pr_digests` | 8 | `—` (non-consumer) | `false` | `false` | `false` | `false` |

(Wave-1 trio anchor — first row inserted on placeholder; flips begin once PR #2777 + #2775 merge and Wave-1 worker spawns + lands `dsl/ctrl/pr_digests.dag`.)

---

## Row-insert protocol

When a Phase 1.5 worker PR merges, Subsystem-Modeling Mgr appends a row (or flips `phase15_pr_merged` if the row was pre-inserted at brief-authoring time, as with `pr_digests` above):

```
| <subsystem_id> | <catalog_row> | <algebra_landed bool · PR URL @SHA OR `—`> | true · <modeling PR URL @SHA> | false | false | true |
```

`open_receipt_debt` is recomputed on every row change. When `phase3_emission_landed ∧ parity_passed` flips, `open_receipt_debt` flips to `false` and the staged-debt count decreases.

## Parity-flip protocol

Verification Mgr flips `parity_passed` to `true` after the named parity-harness gate (per the trio's Phase-3 PR) is green on main. Evidence cell records `gate_id` + CI job URL or in-tree test path.

---

## Cross-references

- Project plan §6 (staged-debt throttle): [`docs/r4-ctrl-dag-migration-project-plan.md`](../r4-ctrl-dag-migration-project-plan.md)
- Mgr brief §"Staged-debt budget" + §"Wave-1-trio checkpoint": [`docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md`](../briefs/r4-ctrl-migration-subsystem-modeling-manager.md)
- Verification Mgr ratification: dashboard messages `msg_5f8db22f` (interface) + `msg_6faaf178` (column refinement), both 2026-05-12 from `deep-badger-38`
