# §1.8 Acceptance-Aggregator Pattern (Pilot)

**Status**: PILOT scaffold (Director-authored 2026-05-12 per Brian exploratory + PM-confirmed cadence message)

**Purpose**: Add an explicit row-class to `docs/r3-program-plan.md` §1.8 for **meta-program clusters** whose closure is the conjunction of multiple constituent gate-IDs already present in the ledger. Resolves the "viz gap" where cluster-level programs (Cluster M = T-Tests-As-Data-Completeness, Cluster F = T-LP-Retirement parity, T-CI-WAD = ci.yml WAD program) have disparate constituent blockers but no single ledger row representing the cluster as a whole.

## Structural shape (kernel-compatible)

Per `feedback_compiler_is_dag_processor` the compiler knows only `Node / Conj / Disj / Cardinality / Bit`. An acceptance-aggregator is structurally:

```
AcceptanceAggregator<cluster> = Conj<gate_id_1, gate_id_2, ..., gate_id_N>
```

The aggregator's `Status` is the meet of constituent statuses under the lattice:

```
DECLARED < CONSUMER_LANDED < PASSING
```

PASSING iff every constituent is PASSING. Any DECLARED makes the meet DECLARED.

This is the SAME shape used informally in §1.5 cluster-rollup notes; this pilot lifts it to a first-class §1.8 row so dashboards/visualizers can render the cluster without parsing prose.

## Row-class additions to §1.8

Add ONE new column to the existing §1.8 table:

| Column | Existing or NEW | Notes |
|---|---|---|
| # | existing | numeric row id (continues from 1..N) |
| Gate ID | existing | for aggregator rows, the cluster-tier program-tag (e.g., `t_ci_wad_full_r3_close`, `cluster_m_tests_as_data_completeness`) |
| Family | existing — extended | NEW family value: `acceptance-aggregator` (alongside `substrate-shape` / `state-check` / `demonstration` / etc.) |
| Owner Lane | existing | the cluster-tier lane (e.g., T-CI-WAD, T-Tests-As-Data-Completeness) |
| Status | existing | derived from `depends_on:` meet; **MUST be computed from constituents, never hand-set** |
| **`depends_on:`** | **NEW** | comma-separated list of constituent §1.8 row-IDs (numeric `#` references, e.g., `#56, #84, #85, #86, #87`) |
| Notes | existing | one-line description + the meet-evaluation rule citation |

### Derivation rule (machine-checkable invariant)

Live §1.8 uses more status values than the abstract 3-value lattice. The derivation rule operates over a coerced status space:

**Coercion table — live §1.8 Status → lattice Status** (applied before meet):

| Live status | Coerced lattice status | Reason |
|---|---|---|
| `PASSING` | `PASSING` | identity |
| `SATISFIED-BY-CONSTRUCTION` | `PASSING` | gate is closed by structural construction; no executable receipt needed; functionally identical to PASSING for closure arithmetic |
| `CONSUMER_LANDED` | `CONSUMER_LANDED` | identity |
| `INTEGRATION_RECEIPT (partial — ε-slice)` (or any `_partial`-suffixed bracketed form) | `CONSUMER_LANDED` | partial-receipt forms close to `CONSUMER_LANDED` per §1.7 corpus-quantified rule (slice receipts ≠ PASSING) |
| `R3-LOAD-BEARING` | **REJECTED — see precondition rule below** | this is scope-metadata ("in R3, not R4-carved"), NOT a closure-progress status; cells with this value must be reframed to expose closure-progress status before aggregator applies |
| `DECLARED` (and `DECLARED through R3` variants) | `DECLARED` | identity |
| `HELD-CANVAS-DEFERRED` | **EXCLUDED — see exclusion rule below** | gate is moved past R3 close; not load-bearing for R3-close arithmetic; MUST NOT appear in any aggregator's `depends_on:` |
| `DEFERRED` (post-R3) | **EXCLUDED** | same as above — post-R3 work; not in aggregator scope |

**Precondition rule (aggregator-readiness)**: An aggregator row can be added ONLY IF every constituent's live Status coerces cleanly into `{DECLARED, CONSUMER_LANDED, PASSING}`. Constituent rows whose Status cell is purely scope-metadata (`R3-LOAD-BEARING` standing alone) are NOT aggregator-ready; their closure-progress status must be inlined into the cell (e.g., `R3-LOAD-BEARING — DECLARED` or `R3-LOAD-BEARING — CONSUMER_LANDED`) so the coercion table can resolve them, OR the row must be split such that scope-metadata and closure-progress are separated columns/rows.

**Exclusion rule**: `DEFERRED` and `HELD-CANVAS-DEFERRED` constituents MUST NOT appear in any aggregator's `depends_on:` list. These gates are explicitly removed from R3-close arithmetic per §1.5 (e.g., `97 enumerated − 1 canvas-deferred {#11} = 96 R3-load-bearing`); including them in an aggregator pollutes the lattice-meet with values the §1.8 ledger has already excluded.

For every row where `Family == acceptance-aggregator`:

1. Parse `depends_on:` as a list of row-IDs.
2. For each row-ID, look up the constituent's `Status` cell.
3. Apply the coercion table to obtain a value in `{DECLARED, CONSUMER_LANDED, PASSING}`.
4. If any constituent's Status fails the precondition rule (purely scope-metadata, e.g., bare `R3-LOAD-BEARING`), the aggregator row is INVALID — fail-close with "constituent #N not aggregator-ready: closure-progress status must be inlined".
5. If any constituent is marked `DEFERRED` or `HELD-CANVAS-DEFERRED`, the aggregator row is INVALID — fail-close with "constituent #N is post-R3-excluded; not eligible for aggregator depends_on:".
6. Otherwise compute the lattice meet over the coerced values: any `DECLARED` → `DECLARED`; otherwise any `CONSUMER_LANDED` → `CONSUMER_LANDED`; else `PASSING`.
7. Assert: aggregator row's `Status` == meet. If mismatch, the ledger is inconsistent — this is a structural ratchet (analogous to per-row invariants in §1.7).

**Why not allow hand-set status**: aggregator rows that drift from their constituents create the "duplicate authority" class openai-pro 2026-05-06 PAUSE_AND_REGROUP flagged. The derivation rule preserves "single canonical view" property.

**Why this is INVARIANTS P2-clean**: the coercion table + precondition + exclusion rules are themselves a single derivation authority; the aggregator's Status is a pure function of constituent Status values under those rules. There is no parallel authority for closure progress — every fact still lives in exactly one §1.8 row.

## Pilot row — T-CI-WAD FULL R3-close

Proposed row to add to §1.8 (pending warm-wolf-698 canvas ratification of substrate-shape options (i)/(ii)/(iii) for gate #56; the AGGREGATOR ROW is shape-stable regardless of which substrate option wins, since the constituent gate-IDs are unchanged):

```
| 9X | t_ci_wad_full_r3_close | acceptance-aggregator | T-CI-WAD | <derived> | depends_on: #56, #<NEW1>, #<NEW2>, #<NEW3>, #<NEW4>. Constituents: ci_workflow_modeled_as_dag + ci_yml_deleted + workflow_emission_target_open_enum_landed + test_cost_dimension_landed + slow_test_exemptions_dissolved. Closes when all 5 PASSING per lattice-meet derivation rule. Brian-elevated to FULL R3-close scope 2026-05-12 per operator directive at gunbc#846. |
```

The 4 NEW constituent rows MUST be added separately to §1.8 with their own families:
- `ci_yml_deleted` → state-check
- `workflow_emission_target_open_enum_landed` → substrate-shape (carrier-shape per WI-1 canvas ratification)
- `test_cost_dimension_landed` → substrate-shape (already exists as cost-dim work; verify row number)
- `slow_test_exemptions_dissolved` → state-check

This pilot ratifies the pattern; PM consumes the template for the §9 update to `docs/scope-t-ci-wad-full-r3.md` (PR #2744) when warm-wolf-698 canvas ratifies the substrate shape.

## Candidate consumers post-pilot

Same row-class generalizes to sibling clusters at low cost:

- **Cluster M** (T-Tests-As-Data-Completeness) — aggregator over §1.8 #84/#85/#86/#87 (per `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md` 3-phase plan); aggregator-ready once constituents' Status cells are coerce-able
- **Cluster F** (T-Lens-Behavioral-Parity) — candidate aggregator over the 4-lens parity rows (#79 complexity, #80 cost, #81 parallelism, #82 effect_enumeration) per Director carve-promotion 2026-05-09. **NOT aggregator-ready at HEAD**: rows #81 and #82 carry bare `R3-LOAD-BEARING` as scope-metadata in the Status cell (not a closure-progress status), and gate #95 / #83 use the same pattern; these rows fail the precondition rule. **Precondition for Cluster F pilot**: rework the affected Status cells to inline closure-progress (e.g., `R3-LOAD-BEARING — DECLARED`) so the coercion table resolves them. Until then, cite Cluster F as a *post-precondition-fix* candidate, not an immediate consumer.
- **Cluster K** — aggregator scope TBD per `docs/audit/r3-cluster-analysis-2026-05-09.md` §2; precondition check required at pilot time
- **T-V2-Retirement** — aggregator over v2 retirement constituent gates; precondition check required at pilot time

PM-tier consumes per cluster on a rolling basis; no big-bang migration required. Each pilot MUST run the precondition + exclusion rules over its candidate constituents before adding the aggregator row.

## Reviewer-rubric notes

If a reviewer (cursor/codex/openai-pro) catches an aggregator row with `Status` inconsistent with its `depends_on:` meet, that is a structural ratchet violation (analogous to a "Documentation Describes Live State" §INVARIANTS finding). Cite the derivation rule above as the authority for the fail-closed verdict.

The aggregator row's purpose is **visualization + dashboard rendering**, NOT a separate closure obligation. The honest-close arithmetic (97 enumerated → 96 R3-load-bearing per §1.5 carve-out math) is unchanged: aggregators consume existing rows without inflating the count. Each aggregator row is a "view" over its constituents, not a new gate.

## Cross-references

- `feedback_compiler_is_dag_processor` — Conj is one of the 5 kernel types; aggregator pattern is kernel-compatible
- `feedback_substrate_principle_audit` — 6-question audit; aggregator passes because the constituents already exist (no new carrier)
- `feedback_parallel_representation_debt` — aggregators MUST be derived, never duplicated; the derivation rule prevents parallel-authority
- `feedback_no_snapshot_integers_in_briefs` — aggregator's `Status` is computed at read time from constituents; never bake snapshot status
- §1.7 corpus-quantified-rule taxonomy — aggregator rows do NOT participate in §1.7 corpus rules; they are pure meet projections
- §1.8 "Status vs Notes (corpus gates)" boilerplate — aggregator rows clarify Status derivation in their Notes column with the citation `<lattice-meet of depends_on:>`
