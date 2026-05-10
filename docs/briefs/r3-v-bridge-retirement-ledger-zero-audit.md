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

Current live ledger fold is red: rows #1 (`bridge_source_span_file_participation_retired`),
#3b (`bridge_canonical_lens_name_patching_residual`), and #6
(`bridge_exact_string_semantic_patching_residual`) are `Open`. The canonical-lens
and exact-string rows use the Director-ratified split pattern: a narrow `Retired`
slice row plus an open residual class row.

## Row Audit

| # | Ledger row | Ledger status | Code state at HEAD | Ratchet / authority | Drift signal |
|---|---|---|---|---|---|
| 1 | `bridge_source_span_file_participation_retired` | `Open` | Open. Audit-packet progress (PR #2150 merged 2026-05-07): row #2 (kernel `Bool` bootstrap patch lookup) **partial** — path-string encapsulated behind `BootstrapAuthorityKey::for_kernel_bool()`, full dissolution awaits row #14 retirement; row #6 (pipeline authority file guard, bootstrap.rs slice) **retired** via typed `BootstrapAuthorityKey::for_pipeline_authority()` + witness-derived spans (`pipeline_authority.rs` stage-binding walk + `compile` arrow lowering remain under separate ownership). Production/lens paths still consult `SourceSpan.file` for participation or filtering: `lens_apply.rs::behavior_source_file`, `reflect_program_dag_nodes_in_file` / `fold_lens_over_reflected_program`, lower's `DIMENSION_STD_AUTHORITY_FILE` gates, and emit `source_filtering.excludes`. | `r3-structure.md` L87; `ROADMAP.md#lens-fold-file-path-semantics`; PR #2150 audit-packet receipt. | None. Ledger `Open` matches code. |
| 2 | `bridge_mark_bootstrap_secret_nominal_opacity_retired` | `Retired` | Retired. `dag.rs::bridge_mark_bootstrap_secret_nominal_opacity_retired` asserts `Secret.nominal_opacity` exists in std, full bootstrap, and without-parse-surface snapshots. No live `mark_bootstrap_secret_nominal_opacity` helper remains. | Rust unit test in `src/v3/compiler/src/dag.rs`; Secret nominal-opacity lineage #1272 / old row authority `PR #937`. | None for status. Authority string is historical but not contradictory. |
| 3a | `bridge_canonical_lens_name_dispatch_pr1183_slice_retired` | Pending substrate split; ratified target `Retired` | Retired at narrow PR #1183 scope. The specific dispatch path covered by #1183 is treated as closed by Director-ratified split. | PR #1183 dispatch path; `canonical_lens_bridge_ratchet_test.rs` narrow ratchet; Director #828 c#4358798673. | None after Substrate Mgr authors the split. The prior single-row drift is resolved by class enumeration, not by treating all canonical-lens-name patching as retired. |
| 3b | `bridge_canonical_lens_name_patching_residual` | Pending substrate split; ratified target `Open` | Open. `canonical_lens_bridge_ratchet_test.rs` pins two canonical-lens `include_str!` constants, two `lens_decl.name.as_deref() == Some(...)` dispatch arms, and two generic name-keyed lookups in `test_runner.rs`. Dissolution trigger: PB-Runtime interpreter-as-data or a typed lens-registry carrier. | Broader exact-string canonical-lens-name class; Director #828 c#4358798673. | None after Substrate Mgr authors the split. Until then, the live single substrate row remains coarser than the ratified class model. |
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
- #3's live ratchet is a nonzero-count pin, not a zero-residual gate. Director
  ratified splitting the narrow PR #1183 retired path from the broader open
  canonical-lens-name patching class, so the anti-growth evidence feeds the open
  residual row instead of falsely closing the whole class.
- #4 is `Retired` for the pipeline-authority `include_str!` slice at the ledger's
  current authority pointer; broader `ArrowBody::Unparsed` compile-body witness
  debt is tracked outside this row's closed verdict.
- #5/#6 split mirrors the canonical-lens pattern: #5 closes the narrow
  lower-helper exact-string patch class; #6 tracks remaining Row-4 semantic
  exact-string patching classes until each lands its own retirement receipt.

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

Director ratified Option 2: sub-class split per the Q2 pattern. Substrate Mgr
authors the substrate-row split per the ratified row structure in #828
c#4358798673:

- `bridge_canonical_lens_name_dispatch_pr1183_slice_retired`: `Retired`;
  authority is the PR #1183 dispatch path / narrow ratchet.
- `bridge_canonical_lens_name_patching_residual`: `Open`; authority is the
  broader exact-string canonical-lens-name class with two `include_str!`
  constants, two `lens_decl.name.as_deref()` arms, and two generic name-keyed
  lookups. Dissolution trigger is PB-Runtime interpreter-as-data or a typed
  lens-registry carrier.

Canonical-lens and exact-string **Q2 splits** are authored in the substrate
ledger (`src/v3/std/bridge_ledger.dag`); Verification audits
`bridge_retirement_ledger_zero` against the live rows — do not treat prose-only
updates as ledger flips.

**Q2 pattern second instance:** `bridge_exact_string_patching_residual_retired`
(narrow lower-helper slice, `Retired`) + `bridge_exact_string_semantic_patching_residual`
(open Row-4 remainder). Future bridge-retirement work defaults to per-class
enumeration per `feedback_coproduct_dissolution` and
`feedback_state_space_vs_behavioral_invariants`.
