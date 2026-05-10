# R3 Bridge Retirement Ledger Zero Audit

**Status:** PROPOSAL — docs-only structural walk of the live
`v3.std.bridge_ledger.bridge_ledger` rows and the production
`bridge_retirement_ledger_zero` TestClaim. No substrate edits, no closure-ledger
authoring, and no new `TestPredicate` variants.

**Authority:** `src/v3/std/bridge_ledger.dag` owns row status; `BridgeLedgerZero`
in `src/v3/std/verification.dag` owns the fold predicate; `docs/r3-structure.md`
L87-L92 names the five bridge gates and the unified ledger-zero gate.

## Grep hygiene (SB5/6 + §3 P4 / V6 alignment)

Phase-2 PM compile correlates merged PR history to ledger / predicate churn using the same **three path anchors** as the SB5/6 bridge appendix on PR [#1804](https://github.com/gunb-ai/gunbc/pull/1804):

1. `src/v3/std/bridge_ledger.dag`
2. `src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag`
3. `src/v3/std/verification.dag`

**Receipt commands** (numeric PR-window filters are PM-pass parameters — not hard-coded here) live in [`docs/r3-design-schedule-2026-05-06.md`](../r3-design-schedule-2026-05-06.md) §3 PB Mgr → **P4 — Verification V6 alignment + grep hygiene**. That subsection ties **§2 V6** (`bridge_retirement_ledger_zero` audit cadence, bold-crane) to PB-side doc hygiene without diluting gate ownership: Verification executes the gate; PB/Substrate retire bridge evidence in owning lanes.

## Summary

The production fixture from PR #1352 is structurally wired:
`src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag` declares
`bridge_retirement_ledger_zero` with predicate
`BridgeLedgerZero { ledger: { decl: bridge_ledger } }`. The runner unwraps the
`BridgeLedgerRef`, fail-closes unless it points at the canonical
`bridge_ledger`, checks the carrier is `List<BridgeLedgerRow>`, and returns
`Pass` iff every row's status constructor is `Retired`.

Current live ledger fold is red: only rows **#1** (`bridge_source_span_file_participation_retired`)
and **#6** (`bridge_exact_string_semantic_patching_residual`) are `Open` in
`bridge_ledger.dag`. Canonical-lens split rows **#3a** and **#3b** are both
**`Retired`** in the live ledger (authorities on each row in
`src/v3/std/bridge_ledger.dag`). Exact-string work uses the same *split shape* as
modeling discipline: narrow lower-helper row **#5** is `Retired`, Row-4 semantic
remainder **#6** stays explicitly `Open`.

## Row Audit

| # | Ledger row | Ledger status | Code state at HEAD | Ratchet / authority | Drift signal |
|---|---|---|---|---|---|
| 1 | `bridge_source_span_file_participation_retired` | `Open` | Open. Audit-packet progress (PR #2150 merged 2026-05-07): row #2 (kernel `Bool` bootstrap patch lookup) **partial** — path-string encapsulated behind `BootstrapAuthorityKey::for_kernel_bool()`, full dissolution awaits row #14 retirement; row #6 (pipeline authority file guard, bootstrap.rs slice) **retired** via typed `BootstrapAuthorityKey::for_pipeline_authority()` + witness-derived spans (`pipeline_authority.rs` stage-binding walk + `compile` arrow lowering remain under separate ownership). Production/lens paths still consult `SourceSpan.file` for participation or filtering: `lens_apply.rs::behavior_source_file`, `reflect_program_dag_nodes_in_file` / `fold_lens_over_reflected_program`, lower's `DIMENSION_STD_AUTHORITY_FILE` gates, and emit `source_filtering.excludes`. | `r3-structure.md` L87; `ROADMAP.md#lens-fold-file-path-semantics`; PR #2150 audit-packet receipt. | None. Ledger `Open` matches code. |
| 2 | `bridge_mark_bootstrap_secret_nominal_opacity_retired` | `Retired` | Retired. `dag.rs::bridge_mark_bootstrap_secret_nominal_opacity_retired` asserts `Secret.nominal_opacity` exists in std, full bootstrap, and without-parse-surface snapshots. No live `mark_bootstrap_secret_nominal_opacity` helper remains. | Rust unit test in `src/v3/compiler/src/dag.rs`; Secret nominal-opacity lineage #1272 / old row authority `PR #937`. | None for status. Authority string is historical but not contradictory. |
| 3a | `bridge_canonical_lens_name_dispatch_pr1183_slice_retired` | `Retired` | Retired in ledger. Narrow PR #1183 dispatch slice; ratchet `canonical_lens_bridge_ratchet_test.rs`. | `src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs`; Director #828 c#4358798673. | None. Ledger `Retired` matches substrate row. |
| 3b | `bridge_canonical_lens_name_patching_residual` | `Retired` | Retired in ledger per canonical-lens name-dispatch closure receipt; authority `docs/briefs/r3-pb-bridge-canonical-lens-name-dispatch-closure.md`. PB-Runtime may still carry transitional `include_str!` / name-keyed surfaces in `test_runner.rs` until interpreter-as-data — track via that brief, not a parallel `Open` ledger row. | Closure brief + `canonical_lens_bridge_ratchet_test.rs` pins; gate #33 lineage. | None. Ledger `Retired` matches substrate row (do not narrate this row as `Open` while the ledger says otherwise). |
| 4 | `bridge_include_str_side_channels_retired` | `Retired` | Retired for the pipeline-authority slice per `bridge_ledger.dag` authority (`l1_5_fixed_point_test.rs` ratchet). `fn compile` still lowers as `ArrowBody::Unparsed`; broader compile-body witness debt is out of scope for this row's retired verdict. | `src/v3/std/bridge_ledger.dag`; `src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs`. | None. Ledger `Retired` matches the landed slice receipt. |
| 5 | `bridge_exact_string_patching_residual_retired` | `Retired` | Retired at PB Tier-2 lower-helper generated-Rust exact-string patch scope (#1014 / #1192). `bridge_lower_helpers_patch_zero_residual_test.rs` ratchets zero residual for the contiguous forbidden token class. | `src/v3/compiler/tests/integration/bridge_lower_helpers_patch_zero_residual_test.rs`. | None. Ledger `Retired` matches the narrow ratchet. |
| 6 | `bridge_exact_string_semantic_patching_residual` | `Open` | Open. Row-4 semantic exact-string patching outside the retired lower-helper slice (e.g. bootstrap `Bool` inhabits patch class, BR-06 non-canonical sentinel splice, infer-helper-driven rewrite classes). | `docs/briefs/r3-v-bridge-row-4-exact-string-deeper-detail-receipt.md`. | None. Ledger `Open` matches remaining class inventory. |

## Code-State Evidence

- #1 source-span participation is still observable in production code through
  file-filtered reflection and emission, not only diagnostics. The row should
  remain `Open` until module / compilation-unit identity and emit-scope carriers
  replace those path checks.
- #2 is backed by an executable Rust unit ratchet over generated snapshots. This
  is a stronger signal than prose status, so `Retired` is grounded.
- #3a/#3b are both **`Retired`** in the live ledger (split landed in substrate;
  see row authorities). PB may still carry transitional name-dispatch / `include_str!`
  surfaces in Rust until interpreter-as-data; that debt is **not** an `Open`
  `bridge_ledger` row — it is scoped by the closure brief on row **#3b**.
- #4 is `Retired` for the pipeline-authority `include_str!` slice at the ledger's
  current authority pointer; broader `ArrowBody::Unparsed` compile-body witness
  debt is tracked outside this row's closed verdict.
- #5/#6 split is the exact-string analogue of the canonical-lens **Q2 class
  split**, but the ledger outcomes differ: canonical-lens **#3a/#3b are both
  `Retired`** at HEAD, while exact-string keeps an explicit **`Open`** remainder
  row **#6** for Row-4 semantic patching until that class retires.

## Production TestClaim Verification

The TestClaim shape is complete for the live carrier:

- Subject is the structural ledger declaration, not an executable program:
  `file_name: "src/v3/std/bridge_ledger.dag"` and
  `BridgeLedgerZero { ledger: { decl: bridge_ledger } }`.
- The current mandatory `source: ""` field remains a known `TestClaim` shape
  limitation; the typed predicate payload is the actual subject.
- `m1_5_verification_test.rs::r3_bridge_retirement_ledger_zero_open_row_count_ratchet`
  compiles the fixture through a performance-only `OnceLock`, runs the suite,
  derives the live open-row names from the bootstrap ledger, and asserts the
  diagnostic names every open row.
- Re-arm trigger is explicit: when `bridge_ledger_open_row_names()` becomes
  empty, change the integration expectation from `Fail` to `Pass` in the same
  PR that flips the last row to `Retired`.

## Binary Ratchet Behavior

`BridgeLedgerZero` is binary by construction:

1. For each row, status constructor equals `BridgeStatus::Retired` => closed.
2. Any other declared `BridgeStatus` constructor (`Open` today) contributes the
   row name to the failure diagnostic.
3. Zero non-retired rows returns `Pass`; one or more returns `Fail`.

The unified `bridge_retirement_ledger_zero` gate should therefore stay red until
the last bridge owner lands its structural retirement receipt and the canonical
`bridge_ledger` row flips to `Retired`.

## Routing

Director-ratified **Q2 split** for canonical-lens rows is **authored in
`bridge_ledger.dag`** at HEAD:

- `bridge_canonical_lens_name_dispatch_pr1183_slice_retired`: **`Retired`**;
  authority `canonical_lens_bridge_ratchet_test.rs` (PR #1183 narrow slice).
- `bridge_canonical_lens_name_patching_residual`: **`Retired`**; authority
  `docs/briefs/r3-pb-bridge-canonical-lens-name-dispatch-closure.md` (gate #33 /
  closure receipt — not an `Open` ledger row).

Exact-string **Q2 split** is the same structural idea with a different ledger
outcome: remainder row **`bridge_exact_string_semantic_patching_residual` stays
`Open`** until Row-4 semantic classes retire.

Canonical-lens and exact-string **Q2 splits** are authored in the substrate
ledger (`src/v3/std/bridge_ledger.dag`); Verification audits
`bridge_retirement_ledger_zero` against the live rows — do not treat prose-only
updates as ledger flips.

**Q2 pattern second instance:** `bridge_exact_string_patching_residual_retired`
(narrow lower-helper slice, `Retired`) + `bridge_exact_string_semantic_patching_residual`
(open Row-4 remainder). Future bridge-retirement work defaults to per-class
enumeration per `feedback_coproduct_dissolution` and
`feedback_state_space_vs_behavioral_invariants`.
