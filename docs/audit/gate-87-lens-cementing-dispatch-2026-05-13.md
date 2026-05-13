# Gate 87 Lens Cementing Dispatch

Date: 2026-05-13
Owner session: `bright-carp-672`
Work item: `node://adhoc-b75b3d90-3d0`

## Scope

Gate `lens_cementing_test_discipline_complete` is the test-discipline closure
for the lens-completeness invariant. Its enumeration surface is every
`LensRegistryEntry` in `src/v3/compiler/regen.dag`; lenses outside that registry
remain governed by the separate Band-C / register ratchets.

The concrete closure surface is:

- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`
- `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/std/verification.dag` `lens_capability_register_rows`
- `docs/v3-lens-capability-register.md`

## Dispatched Sub-Items

1. `node://adhoc-40a34592-6d6` — audit `regen.dag`
   `LensRegistryEntry` inventory against the PB-B-1 runner table.

   Acceptance: every registry name has exactly one
   `t_r3_gate_87_cementing_regen_<name>.dag` harness and exactly one
   `R3_GATE_87_CEMENTING_REGEN_SUITES` runner-table row; additions fail closed
   through `r3_gate_87_regen_lens_registry_names_match_fixture_inventory`.

2. `node://adhoc-e0184df5-0e5` — land frozen v2-oracle
   `DifferentialEquals` / `LensOutputEquals` receipts for behaviorally complete
   lenses with concrete v2 counterparts.

   Acceptance: every `LensCapabilityBehavioralComplete` row with a non-`N/A`
   v2 counterpart has matching `.dag` and, where needed, temporary Rust receipt
   entries wired through `cementing_band_c_v2_complete_receipts`, with witness
   shape matching the v2-counterpart column. Any new or expanded Rust receipt
   under `src/v3/compiler/tests/**` must carry the same-PR P5 hand-Rust receipt
   required by `INVARIANTS.md` §"Dispatch-Discipline Mechanisms" and §"SG-0
   hand-authored integration test receipts": exactly one checkable receipt
   (deleted file/scaffold, SG-0 census shrink, or explicit deferral naming a
   lane plus concrete `ROADMAP.md` row), with the matching
   `sg0_census_test.rs` census entry when a new hand-authored test path is added.

3. `node://adhoc-e04ebf89-738` — pin v3-native, `N/A`, and helper-scope rows
   with explicit `Compiles` placeholders plus Rust receipts where the Band-C
   rule requires a temporary pin.

   Acceptance: helper rows such as `infer_helpers`, `lower_helpers`, and
   `variant_payload` are deliberately marked as placeholders with dissolution
   triggers, and Rust pin receipts cover the narrow contracts not yet expressible
   as full `.dag` lens-output witnesses. Each Rust pin receipt must either reuse
   an existing SG-0/P5 receipt row for that exact path or add the same-PR
   `INVARIANTS.md` receipt plus census update described above; placeholder text
   alone is not an acceptable deferral.

4. `node://adhoc-80274b5f-3fa` — verify final dispatch closure across
   `cementing_dispatch.dag`, the PB-B-1 runner, and receipt ratchets.

   Acceptance: `CementingDispatchMatchesProjection`,
   `r3_gate_87_cementing_regen_lens_suites_pass_through_runner`, and
   `r3_gate_87_lens_cementing_regen_receipts_test` agree on the same inventory;
   drift between register rows, `regen.dag`, on-disk harnesses, and temporary
   Rust modules fails closed.

## Non-Goals

- Do not broaden gate 87 to lenses absent from `regen.dag`.
- Do not use GitHub sub-issues for the decomposition; dashboard work items are
  the dispatch mechanism.
- Do not treat `Compiles` placeholders as behavioral completion for rows whose
  register entry requires a frozen v2-oracle witness.
- Do not add or expand hand-written Rust tests for this gate without the P5
  receipt required for `src/v3/compiler/tests/**` hand-Rust.
