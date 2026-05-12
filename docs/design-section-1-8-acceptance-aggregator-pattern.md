# §1.9 Acceptance-Aggregator Views (Pilot)

> **Filename note**: this file is named `design-section-1-8-acceptance-aggregator-pattern.md` for historical reasons (original draft targeted §1.8). The substantive section the doc specifies is **§1.9** of `docs/r3-program-plan.md`, NOT §1.8 — the relocation was done per codex REQUEST_CHANGES /api/reviews/9982 to preserve §1.8's single-authority semantics. Filename retained to avoid breaking in-flight review thread anchors.

**Status**: PILOT scaffold (Director-authored 2026-05-12 per Brian exploratory + PM-confirmed cadence message). REVISED 2026-05-12 per codex REQUEST_CHANGES /api/reviews/9982 on PR #2748 — aggregators relocated OUT of §1.8 into a new §1.9 section to preserve §1.8's "canonical closure-authority ledger" single-authority semantics per INVARIANTS P2.

**Sunset condition** (P5 "scaffold" posture per cursor /api/reviews/10013): this pilot doc retires when `docs/r3-program-plan.md` contains §1.9 per the table shape below AND at least one §1.9 view entry (e.g., V1 `t_ci_wad_full_r3_close`) is live in the ledger. At that point this doc's authority is consumed; PM may delete it or fold any remaining cross-references into `r3-program-plan.md` §1.9 itself.

**Purpose**: Provide a derived **view layer** over `docs/r3-program-plan.md` §1.8 that surfaces cluster-tier meta-program closure progress as the lattice-meet of constituent §1.8 rows. Resolves the "viz gap" where cluster-level programs (Cluster M = T-Tests-As-Data-Completeness, Cluster F = T-LP-Retirement parity, T-CI-WAD = ci.yml WAD program) have disparate constituent blockers but no single rendering of the cluster as a whole.

## INVARIANTS P2 framing — why a SEPARATE section, not §1.8 rows

§1.8 is the **canonical closure-authority ledger**. Its row count is the live `97 enumerated / 96 R3-load-bearing` arithmetic per §1.5. Each row is a closure obligation with its own Status, Family, Owner Lane, and §1.7 corpus-quantified rules.

An earlier draft (PR #2748 commit 62e47bfde and prior) proposed adding aggregator rows DIRECTLY to §1.8 with a new `acceptance-aggregator` family value and a `depends_on:` column. codex review /api/reviews/9982 correctly identified two P2 violations in that shape:

1. **Two closure objects for the same fact**: putting aggregator rows in §1.8 alongside their constituents means the same closure progress is now represented in (N constituents + 1 aggregator) rows. Even with "derived, never hand-set" prose discipline, the row shape doesn't structurally prevent treatment as a closure obligation.
2. **Side taxonomy inside the canonical ledger**: an in-§1.8 aggregator row needs special rules ("does not participate in §1.7 corpus rules", "Status is derived not hand-set") that diverge from the canonical row semantics. A canonical ledger row should not need a side taxonomy to explain why its closure logic differs.

**The structural fix** (this revision):

- Aggregators live in **§1.9 of `docs/r3-program-plan.md`** as a SEPARATE table (this doc specifies its shape; PM authors the actual §1.9 section)
- Aggregator entries use a DISTINCT ID namespace: `V1, V2, V3, ...` (or `agg-1, agg-2, ...`) — explicitly **NOT** numeric §1.8 row IDs
- The `97 enumerated / 96 R3-load-bearing` §1.5 arithmetic is computed exclusively over §1.8; §1.9 entries do not appear in that arithmetic and do not constitute closure obligations
- §1.9 entries reference §1.8 row numbers via `depends_on:` (foreign-key style); the §1.8 rows themselves are unmodified by this pilot

This preserves §1.8's single-authority semantics. §1.9 is a derived view; queries that ask "how many R3 closure gates exist?" still parse §1.8 only and ignore §1.9 entirely.

## Structural shape (kernel-compatible)

Per `feedback_compiler_is_dag_processor` the compiler knows only `Node / Conj / Disj / Cardinality / Bit`. An acceptance-aggregator view is structurally:

```
AcceptanceAggregator<cluster> = Conj<§1.8_row_id_1, §1.8_row_id_2, ..., §1.8_row_id_N>
```

The view's `Derived Status` is the lattice-meet of constituent Status values under the lattice `DECLARED < CONSUMER_LANDED < PASSING` (after coercion; see §"Derivation rule" below).

This is the same shape used informally in §1.5 cluster-rollup prose; the pilot lifts it to a typed table form for dashboard/visualizer consumption.

## §1.9 table shape

The §1.9 table sits in `docs/r3-program-plan.md` after the §1.8 ledger and before §2. Its columns are intentionally NOT the same as §1.8's, to make the visual distinction obvious:

| Column | Notes |
|---|---|
| **View ID** | `V1, V2, ...` (or `agg-1, agg-2, ...`) — DISTINCT from §1.8's numeric `#` namespace |
| **View Name** | cluster-tier program-tag (e.g., `t_ci_wad_full_r3_close`, `cluster_m_tests_as_data_completeness`) |
| **Cluster Lane** | the cluster's owner lane (e.g., T-CI-WAD, T-Tests-As-Data-Completeness) |
| **`depends_on:`** | comma-separated list of §1.8 row-IDs the view derives from (e.g., `#56, #84, #85, #86, #87`) |
| **Derived Status** | computed at read time per the derivation rule below; never hand-set; rendered as `<DERIVED>` in committed text since the value rots between authoring and consumption per `feedback_no_snapshot_integers_in_briefs` |
| **Notes** | one-line description of the cluster + brief context |

### Derivation rule (machine-checkable spec)

Live §1.8 uses more status values than the abstract 3-value lattice. The derivation operates over a coerced status space:

**Coercion table — live §1.8 Status → lattice Status** (applied before meet):

| Live status | Coerced lattice status | Reason |
|---|---|---|
| `PASSING` | `PASSING` | identity |
| `SATISFIED-BY-CONSTRUCTION` | `PASSING` | gate is closed by structural construction; no executable receipt needed; functionally identical to PASSING for closure arithmetic |
| `CONSUMER_LANDED + PASSING` (composite) | `PASSING` | composite form (~25 instances in live §1.8, e.g., #86 `program_generator_carrier_landed`) means the gate progressed through CONSUMER_LANDED and reached PASSING; current state is PASSING |
| **Composite rule** `<earlier> + <later>` | coerce to `<later>` | general rule for composite status cells: the `+` is conjunction-of-progression-stages; current state is the most-advanced (rightmost) component. Applies to any future composite forms not enumerated above. |
| `CONSUMER_LANDED` | `CONSUMER_LANDED` | identity |
| `INTEGRATION_RECEIPT (partial — ε-slice)` (or any `_partial`-suffixed bracketed form) | `CONSUMER_LANDED` | partial-receipt forms close to `CONSUMER_LANDED` per §1.7 corpus-quantified rule (slice receipts ≠ PASSING) |
| `R3-LOAD-BEARING` | **REJECTED — see precondition rule below** | this is scope-metadata ("in R3 per carve-promotion-IN-R3 2026-05-09; formerly R4-carved (C1/C2/C3), now DISSOLVED per Director ratification gunbc#846"), NOT a closure-progress status; cells with this value must be reframed to expose closure-progress status before view-membership applies |
| `DECLARED` (and `DECLARED through R3` variants) | `DECLARED` | identity |
| `HELD-CANVAS-DEFERRED` | **EXCLUDED — see exclusion rule below** | gate is moved past R3 close; not load-bearing for R3-close arithmetic; MUST NOT appear in any view's `depends_on:` |
| `DEFERRED` (post-R3) | **EXCLUDED** | same as above — post-R3 work; not in view scope |

**Precondition rule (view-readiness)**: A §1.9 view can be added ONLY IF every constituent's live §1.8 Status coerces cleanly into `{DECLARED, CONSUMER_LANDED, PASSING}`. Constituent rows whose Status cell is purely scope-metadata (`R3-LOAD-BEARING` standing alone) are NOT view-ready; their closure-progress status must be inlined into the §1.8 cell (e.g., `R3-LOAD-BEARING — DECLARED`) so the coercion table can resolve them, OR the §1.8 row must be split such that scope-metadata and closure-progress are separated columns/rows.

**Exclusion rule**: `DEFERRED` and `HELD-CANVAS-DEFERRED` constituents MUST NOT appear in any view's `depends_on:` list. These gates are explicitly removed from R3-close arithmetic per §1.5 (e.g., `97 enumerated − 1 canvas-deferred {#11} = 96 R3-load-bearing`); including them in a view pollutes the lattice-meet with values §1.8 has already excluded from honest-close arithmetic.

**Derivation algorithm** (consumed by dashboards, visualizers, and reviewer-rubric checkers):

1. Parse the view's `depends_on:` as a list of §1.8 row-IDs.
2. For each row-ID, look up the §1.8 row's `Status` cell.
3. Apply the coercion table to obtain a value in `{DECLARED, CONSUMER_LANDED, PASSING}`.
4. If any constituent's Status fails the precondition rule (purely scope-metadata, e.g., bare `R3-LOAD-BEARING`), the view is INVALID — fail-close with "constituent #N not view-ready: closure-progress status must be inlined in §1.8".
5. If any constituent is marked `DEFERRED` or `HELD-CANVAS-DEFERRED`, the view is INVALID — fail-close with "constituent #N is post-R3-excluded; not eligible for view depends_on:".
6. Otherwise compute the lattice-meet over coerced values: any `DECLARED` → `DECLARED`; otherwise any `CONSUMER_LANDED` → `CONSUMER_LANDED`; else `PASSING`. This is the view's `Derived Status` at the moment of evaluation.

There is no stored `Status` value for §1.9 views. The Derived Status is always computed at read time.

## Pilot view — T-CI-WAD FULL R3-close

Proposed §1.9 entry (post-(c-refined) canvas ratification per PR #2749; constituent gate-IDs reflect PM cascade renames including `ci_yml_deleted` → `ci_yml_hand_authority_dissolved`):

```
| V1 | t_ci_wad_full_r3_close | T-CI-WAD | depends_on: #56, #<N1>, #<N2>, #<N3>, #<N4>, #<N5>, #<N6> | <DERIVED> | Brian-elevated to FULL R3-close scope 2026-05-12 per gunbc#846. Constituents: ci_workflow_modeled_as_dag + ci_yml_hand_authority_dissolved + emission_target_open_enum_landed + test_cost_dimension_landed + slow_test_exemptions_dissolved + project_github_actions_landed + ci_uses_affected_set_selection. |
```

The 6 NEW constituent rows MUST be added to **§1.8** (not §1.9) with their canonical families per the existing §1.8 schema:

- `ci_yml_hand_authority_dissolved` → state-check (PM cascade per PR #2744 commit 19a1d8dfc — file presence orthogonal to hand-authority dissolution)
- `emission_target_open_enum_landed` → substrate-shape (gunbc-namespace per (c-refined))
- `test_cost_dimension_landed` → substrate-shape (verify row number against existing cost-dim work)
- `slow_test_exemptions_dissolved` → state-check
- `project_github_actions_landed` → substrate-shape (new; projection function in gunbc namespace per (c-refined))
- `ci_uses_affected_set_selection` → state-check (T-WAD + T-Verification cross-tier; PR #2713 affected-set lens consumed by BinaryShim emitter; Layer 2 path-regex bridge dissolved)

These 6 NEW §1.8 rows participate in the canonical arithmetic (close-gate counts shift accordingly per §1.5). The §1.9 V1 entry is a **derived rendering**, not a 7th closure obligation.

## Candidate §1.9 entries post-pilot

Same view pattern generalizes to sibling clusters at low cost. Each candidate MUST pass the precondition + exclusion rules over its §1.8 constituents before being added to §1.9:

- **Cluster M** (T-Tests-As-Data-Completeness) — view over §1.8 #84/#85/#86/#87 (per `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md` 3-phase plan); view-ready once constituents' Status cells coerce cleanly
- **Cluster F** (T-Lens-Behavioral-Parity) — candidate view over the 4-lens parity rows (#79 complexity, #80 cost, #81 parallelism, #82 effect_enumeration). **NOT view-ready at HEAD**: §1.8 rows #81 and #82 (and #95) carry bare `R3-LOAD-BEARING` as scope-metadata in the Status cell with no closure-progress inlined; these rows fail the precondition. Row #83 (`lens_capability_register_zero_proxy_zero_stub`) is a positive counter-example — its Status cell already reads "**DECLARED — full scope IN R3 (carve-promotion-IN-R3 2026-05-09)**", inlining closure-progress alongside the scope-metadata, so #83 already coerces to `DECLARED` under the precondition rule. **Pre-pilot fix required for #81/#82/#95**: their §1.8 cells must follow #83's pattern and inline closure-progress (e.g., `R3-LOAD-BEARING — DECLARED`) before the Cluster F view can be added.
- **Cluster K** — view scope TBD per `docs/audit/r3-cluster-analysis-2026-05-09.md` §2; precondition check required at pilot time
- **T-V2-Retirement** — view over v2 retirement constituent gates; precondition check required at pilot time

PM-tier consumes per cluster on a rolling basis; no big-bang migration required.

## Reviewer-rubric notes

If a reviewer (cursor/codex/openai-pro) catches a §1.9 view whose stored "Derived Status" diverges from the meet of its current §1.8 constituents, that is a discipline violation — the column MUST be rendered as `<DERIVED>` (or computed live by a tooling layer), never stored as a snapshot value.

If a reviewer catches a §1.9 view referencing a §1.8 row with a `DEFERRED` / `HELD-CANVAS-DEFERRED` / bare-scope-metadata Status, that is a precondition or exclusion violation per the derivation rule above. Fail-closed with a citation.

§1.9 views are **visualization + dashboard rendering**, NOT closure obligations. The honest-close arithmetic (`97 enumerated → 96 R3-load-bearing` per §1.5 carve-out math) is exclusively over §1.8 and is unchanged by any addition or removal of §1.9 entries.

## Cross-references

- `feedback_compiler_is_dag_processor` — Conj is one of the 5 kernel types; aggregator view pattern is kernel-compatible
- `feedback_substrate_principle_audit` — 6-question audit; this pattern passes because it introduces no new substrate carrier
- `feedback_parallel_representation_debt` — views MUST be derived, never duplicated; the §1.9-vs-§1.8 separation + Derived-Status discipline prevents parallel-authority
- `feedback_no_snapshot_integers_in_briefs` — Derived Status is computed at read time; never bake snapshot status in committed text (use `<DERIVED>` placeholder)
- `feedback_audit_adjacent_authority_first` — audit §1.8 + §1.5 + §1.7 authority before authoring §1.9 entries; cite existing parents instead of restating
