# R3 Bridge Retirement Ledger Zero Audit

**Status:** PROPOSAL — docs-only structural walk of the live
`v3.std.bridge_ledger.bridge_ledger` rows and the production
`bridge_retirement_ledger_zero` TestClaim. No substrate edits, no closure-ledger
authoring, and no new `TestPredicate` variants.

**Authority:** `src/v3/std/bridge_ledger.dag` owns row status; `BridgeLedgerZero`
in `src/v3/std/verification.dag` owns the fold predicate; `docs/r3-structure.md`
L87-L92 names the five bridge gates and the unified ledger-zero gate.

## Summary

The production fixture from PR #1352 is structurally wired:
`src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag` declares
`bridge_retirement_ledger_zero` with predicate
`BridgeLedgerZero { ledger: { decl: bridge_ledger } }`. The runner unwraps the
`BridgeLedgerRef`, fail-closes unless it points at the canonical
`bridge_ledger`, checks the carrier is `List<BridgeLedgerRow>`, and returns
`Pass` iff every row's status constructor is `Retired`.

Current live ledger fold is red: rows #1, #4, and #5 are `Open`. This audit also
finds one status drift candidate: row #3 currently says `Retired`, but the live
canonical-lens ratchet still pins nonzero bridge surface.

## Row Audit

| # | Ledger row | Ledger status | Code state at HEAD | Ratchet / authority | Drift signal |
|---|---|---|---|---|---|
| 1 | `bridge_source_span_file_participation_retired` | `Open` | Open. Production/lens paths still consult `SourceSpan.file` for participation or filtering: `lens_apply.rs::behavior_source_file`, `reflect_program_dag_nodes_in_file` / `fold_lens_over_reflected_program`, lower's `DIMENSION_STD_AUTHORITY_FILE` gates, and emit `source_filtering.excludes`. | `r3-structure.md` L87; `ROADMAP.md#lens-fold-file-path-semantics`. | None. Ledger `Open` matches code. |
| 2 | `bridge_mark_bootstrap_secret_nominal_opacity_retired` | `Retired` | Retired. `dag.rs::bridge_mark_bootstrap_secret_nominal_opacity_retired` asserts `Secret.nominal_opacity` exists in std, full bootstrap, and without-parse-surface snapshots. No live `mark_bootstrap_secret_nominal_opacity` helper remains. | Rust unit test in `src/v3/compiler/src/dag.rs`; Secret nominal-opacity lineage #1272 / old row authority `PR #937`. | None for status. Authority string is historical but not contradictory. |
| 3 | `bridge_canonical_lens_name_dispatch_retired` | `Retired` | Not fully retired. `canonical_lens_bridge_ratchet_test.rs` pins two canonical-lens `include_str!` constants, two `lens_decl.name.as_deref() == Some(...)` dispatch arms, and two generic name-keyed lookups in `test_runner.rs`. The test text says full retirement waits for PB-Runtime interpreter-as-data or a typed lens-registry carrier. | `canonical_lens_bridge_ratchet_test.rs`; `r2-pb-canonical-lens-bridge-disposition.md`; #1183 narrow slice. | **Drift candidate:** ledger says `Retired`, but live ratchet/code indicate partial/open residual. Route to Substrate Mgr for row-status correction or Director for ratification. |
| 4 | `bridge_include_str_side_channels_retired` | `Open` | Open. `pipeline_authority.rs` explicitly says compile-body cross-check remains suspended because `fn compile` lowers to `ArrowBody::Unparsed`; the prior `include_str!`/file-read side-channel is rejected until a structural compile-body witness exists. | `design-emission-model.md:944`; `pipeline_authority.rs`; PR #1171. | None. Ledger `Open` matches code. |
| 5 | `bridge_exact_string_patching_residual_retired` | `Open` | Open at umbrella scope. The lower-helper sub-slice is retired and ratcheted by `bridge_lower_helpers_patch_zero_residual_test.rs`, but other exact-string patch classes remain. `bootstrap.rs::patch_kernel_bool_boolean_algebra_inhabits` is a live class-5-style residual called from bootstrap paths. | `r3-structure.md` L91; `r2-closure-ledger.md` Tier-2 row; #1014 + #1192 narrow receipt. | None. Ledger `Open` correctly refuses to treat the lower-helper sub-slice as umbrella closure. |

## Code-State Evidence

- #1 source-span participation is still observable in production code through
  file-filtered reflection and emission, not only diagnostics. The row should
  remain `Open` until module / compilation-unit identity and emit-scope carriers
  replace those path checks.
- #2 is backed by an executable Rust unit ratchet over generated snapshots. This
  is a stronger signal than prose status, so `Retired` is grounded.
- #3's live ratchet is a nonzero-count pin, not a zero-residual gate. A
  nonzero-count pin is useful anti-growth evidence, but it does not by itself
  establish ledger retirement.
- #4's authority is explicit in `pipeline_authority.rs`: compile-body drift
  detection is suspended until a structural witness exists. That is a deliberate
  open row, not missing coverage.
- #5 correctly distinguishes the retired lower-helper class from the umbrella.
  The lower-helper ratchet should feed the umbrella; it should not be read as
  the umbrella's only required evidence.

## Production TestClaim Verification

The TestClaim shape is complete for the live carrier:

- Subject is the structural ledger declaration, not an executable program:
  `file_name: "src/v3/std/bridge_ledger.dag"` and
  `BridgeLedgerZero { ledger: { decl: bridge_ledger } }`.
- The current mandatory `source: ""` field remains a known `TestClaim` shape
  limitation; the typed predicate payload is the actual subject.
- `m1_5_verification_test.rs::r3_bridge_retirement_ledger_zero_fixture_reports_open_rows_at_head`
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

Only row #3 needs escalation. This PR does not change the row because status
ownership lives in the substrate ledger. Recommended routing: ask Substrate
Manager / Director whether `bridge_canonical_lens_name_dispatch_retired` should
be corrected back to `Open` until the pinned canonical-lens bridges reach zero,
or whether a newer Director disposition intentionally treats the remaining
ratchet as non-blocking for the ledger. Do not close `bridge_retirement_ledger_zero`
while this drift is unresolved.
