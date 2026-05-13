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
| `algebra_landed` | `bool \| "—"` | Substrate Mgr (or `—` for non-consumers) | Phase-1 substrate PR URL + merge SHA, OR `—` for rows not consuming algebra. `—` is **not** a placeholder for "unknown" — it is a structural assertion that this subsystem has no algebra-consumer dependency; see §"N/A semantics" below for gate semantics. |
| `phase15_pr_merged` | bool | Subsystem-Modeling Mgr | modeling PR URL + merge SHA |
| `phase3_emission_landed` | bool | Emission-Targets Mgr | emission PR URL + merge SHA + canonical digest-declaration ref path + projection module path |
| `parity_passed` | bool | Verification Mgr | gate_id string + CI job URL OR in-tree test path |
| `open_receipt_debt` | derived | computed | `phase15_pr_merged ∧ ¬(phase3_emission_landed ∧ parity_passed)` |

**`N/A` semantics** (per operator review codex finding #1 2026-05-12 commit `a6bd5f56`): the marker `—` on `algebra_landed` means **the row is non-consumer of the Phase-1 algebra substrate** and therefore the algebra prerequisite **does not apply per-row**. In every gate computation below, `algebra_landed ∈ {true, —}` is treated as satisfied; **only `false` is unsatisfied**. This makes algebra a **per-row receipt** (not a global gate) and admits a non-consumer trio anchor without misclassification.

**Dispatch-pause gate (Subsystem-Modeling Mgr polls before any new Phase 1.5 worker dispatch)**: `count(rows where open_receipt_debt = true) ≥ 3` ⇒ **pause** new Wave-1 / Wave-2 dispatch until catch-up (Phase-3 landings or parity flips reduce count below 3). Director-confirmed wording: STAGED Phase 1.5 subsystem merges without both emission-target landing and parity must not accumulate past 2 open debts. **Note**: `open_receipt_debt` does NOT reference `algebra_landed` — algebra status does not contribute to staged-debt.

**Wave-1-trio gate**: Wave-2 fanout unblocked only when a designated trio-anchor row satisfies `algebra_landed ∈ {true, —}` ∧ `phase15_pr_merged = true` ∧ `phase3_emission_landed = true` ∧ `parity_passed = true` — i.e. all four columns "satisfied" under the `N/A`-counts-as-satisfied semantics above. Current trio anchor: catalog #8 (PR digests).

---

## Ledger

Cells use compact `bool · evidence` form. `—` denotes not-yet-applicable (e.g. `algebra_landed` for non-consumer rows). Empty evidence = column not yet flipped.

| subsystem_id | catalog_row | algebra_landed | phase15_pr_merged | phase3_emission_landed | parity_passed | open_receipt_debt |
|---|---|---|---|---|---|---|
| `pr_digests` | 8 | `—` (non-consumer) | `true` · PR #2838 commit `b9f8a075f32904039c3e83a5efe0a34a920b4fe0` | `true` · PR #2832 commit `6ce22a1fa970becacf717fe1c2e2f164bee421a3` (projection: `dsl/gunbc/digest_render.dag`; source facts: `dsl/extdeps/github/pulls.dag`; std render primitives: `dsl/std/render.dag`) | `true` · gate_id `r4_ctrl_wave1_catalog8_trio_parity` · witnessed green on main `ee66560e55fd7d374b70b950f603e2a0e5260175` · in-tree: `ctrl_pr_digests_dag_smoke_test::ctrl_pr_digests_dag_tokenizes_and_matches_expected_surface`; `parse_stage4_prep::handwritten_parser_accepts_gunbc_digest_render_dag` (`src/v3/compiler/tests/integration/ctrl_pr_digests_dag_smoke_test.rs`, `src/v3/compiler/tests/integration.rs`) | `false` |

(Wave-1 trio anchor — catalog #8; `dsl/ctrl/pr_digests.dag` + Phase-3 digest projection landed; charter #2775 + Mgr brief #2777 merged 2026-05-12; parity gate `r4_ctrl_wave1_catalog8_trio_parity` green 2026-05-13.)

---

## Row-insert protocol

When a Phase 1.5 worker PR merges, Subsystem-Modeling Mgr appends a row (or flips `phase15_pr_merged` if the row was pre-inserted at brief-authoring time, as with `pr_digests` above):

```
| <subsystem_id> | <catalog_row> | <algebra_landed bool · PR URL @SHA OR `—`> | true · <modeling PR URL @SHA> | false | false | true |
```

(Example row shows `open_receipt_debt = true` immediately after `phase15_pr_merged` flips while Phase-3 + parity are still false — adjust booleans to match the live subsystem.)

`open_receipt_debt` is recomputed on every row change. When `phase3_emission_landed ∧ parity_passed` flips, `open_receipt_debt` flips to `false` and the staged-debt count decreases.

## Parity-flip protocol

Verification Mgr flips `parity_passed` to `true` after the named parity-harness gate (per the trio's Phase-3 PR) is green on main. Evidence cell records `gate_id` + CI job URL or in-tree test path.

---

## Named parity gate — `r4_ctrl_wave1_catalog8_trio_parity` (Wave-1 catalog #8 trio anchor)

**Structural parity (both green on `main`):**

1. `ctrl_pr_digests_dag_smoke_test::ctrl_pr_digests_dag_tokenizes_and_matches_expected_surface` in `src/v3/compiler/tests/integration/ctrl_pr_digests_dag_smoke_test.rs` — lexer + structural needles for `dsl/ctrl/pr_digests.dag`.
2. `parse_stage4_prep::handwritten_parser_accepts_gunbc_digest_render_dag` in `src/v3/compiler/tests/integration.rs` — handwritten parser accepts `dsl/gunbc/digest_render.dag`.

**Filter commands:** `cargo test -p v3-compiler --test integration ctrl_pr_digests_dag_tokenizes_and_matches_expected_surface` and `cargo test -p v3-compiler --test integration handwritten_parser_accepts_gunbc_digest_render_dag`.

**Out of scope for this gate_id (explicit):** execution or `TestClaim` consumption of `digest_render_expectation_receipts` in `dsl/gunbc/digest_render.dag` — those rows remain structural/doc receipts until a dedicated harness evaluates them (non-blocking for Wave-1 trio structural closure per Director + review thread 2026-05-13).

---

## Cross-references

- Project plan §6 (staged-debt throttle): [`docs/r4-ctrl-dag-migration-project-plan.md`](../r4-ctrl-dag-migration-project-plan.md)
- Mgr brief §"Staged-debt budget" + §"Wave-1-trio checkpoint": [`docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md`](../briefs/r4-ctrl-migration-subsystem-modeling-manager.md)
- Verification Mgr ratification: dashboard messages `msg_5f8db22f` (interface) + `msg_6faaf178` (column refinement), both 2026-05-12 from `deep-badger-38`
