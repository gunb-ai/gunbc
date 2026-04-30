# R3 Bridge-Retirement Ledger-Zero Ratchet Test Design

**Status:** PROPOSAL - design-only. No compiler/test implementation in this slice. Implementation waits on Director ratification of the composition shape below.

**Owning manager:** R3 Verification Manager.

## Goal

Author the shape for a `bridge_retirement_ledger_zero_test` ratchet that gates the unified Verification audit:

- fails while any of the five named bridge rows is open;
- passes only when all five bridge retirements are structurally green;
- never expands the bridge set without Director scope-change approval.

Canonical bridge-map authority is [`docs/r3-structure.md`](../r3-structure.md) T-Bridge-Retirement plus [`r3-verification-manager.md`](r3-verification-manager.md) §"Bridge-retirement ledger - current state".

## Proposed Test Location

Implementation should mirror existing source-ratchet precedent:

`src/v3/compiler/tests/integration/bridge_retirement_ledger_zero_test.rs`

Wire through `src/v3/compiler/tests/integration.rs` beside `bridge_lower_helpers_patch_zero_residual_test.rs` and `canonical_lens_bridge_ratchet_test.rs`.

60s discipline applies; the actual scan should remain small and deterministic.

## Per-Bridge Gate Audit

| # | Bridge | Canonical gate name | Current authority / gap |
|---|---|---|---|
| 1 | `SourceSpan.file` participation | `bridge_source_span_file_participation_retired` | Named in `r3-structure.md`; no strict compiler test or `.dag` predicate found. Open/R3-deferred per #1273. |
| 2 | Secret bootstrap nominal opacity | `bridge_mark_bootstrap_secret_nominal_opacity_retired` | Named in `r3-structure.md`; implemented today as a Rust unit test in `src/v3/compiler/src/dag.rs`. Retired by #1272. |
| 3 | canonical lens-name dispatch | `bridge_canonical_lens_name_dispatch_retired` | Named in `r3-structure.md` and `r2-pb-canonical-lens-bridge-disposition.md`; current test is `canonical_lens_bridge_ratchet_test.rs`, which pins partial bridge surface growth. It is not a zero/retired predicate yet. |
| 4 | `include_str!` side channels | `bridge_include_str_side_channels_retired` | Named in `r3-structure.md` and `r2-closure-ledger.md`; no strict compiler test or `.dag` predicate found. Open at `pipeline_authority` pending structural compile-body witness. |
| 5 | `patch_lower_helpers_*` residual | Umbrella: `bridge_exact_string_patching_residual_retired`; narrow ratchet: `bridge_lower_helpers_patch_zero_residual` | `r3-structure.md` names the umbrella. `bridge_lower_helpers_patch_zero_residual_test.rs` proves only the lower-helper slice is zero; broader exact-string patching remains out of scope. |

## Composition Choice Requiring Ratification

Two viable shapes exist. This brief does not choose between them.

### Option A - AND over per-bridge predicates

The ratchet encodes a fixed list of five gate names and requires each per-bridge predicate to report green.

Pros: simplest implementation; matches current scattered authority; makes the five-bridge cap explicit.

Cons: only bridge #2 has a strict retired test today; #3/#5 are partial ratchets; #1/#4 are docs gates only.

### Option B - structural fold over a `BridgeLedger` carrier

Substrate/Verification authors a small ledger carrier containing the five rows: `{ name, owner, status, authority }`. The ratchet folds the carrier and passes only when every row is `Retired`.

Pros: single structural authority; separates "row exists" from "row retired"; makes silent bridge-set expansion harder.

Cons: requires carrier authoring before the test can be strict; may need `INVARIANTS.md` §P1 review if modeled as `.dag` data.

## Recommended Implementation After Ratification

1. Freeze the five bridge rows from `r3-structure.md`; fail if the materialized row count is not exactly five.
2. Require a strict retired authority per row. #2 can reference/migrate the current Rust unit test; #3/#5 partial ratchets stay non-retired until their umbrella gates fire; #1/#4 stay open until owner programs land strict predicates.
3. Add a negative fixture/state test proving any open row keeps `bridge_retirement_ledger_zero` red.
4. Add the final positive path only when all five rows are structurally retired.

## Open Questions

- Should `bridge_retirement_ledger_zero` be a `RatchetZero`-style `.dag` `TestClaim`, or a Rust integration ratchet over a checked-in ledger fixture?
- Should gate #5 use the umbrella name from `r3-structure.md` or split lower-helper zero from broader exact-string patch retirement in the ledger schema?
- Do bridges #1 and #4 need new owner-authored strict predicates before the unified ratchet can exist, or can the unified test carry explicit `Open` rows until then?

## Non-Goals

- Do not add a sixth bridge from this worker branch.
- Do not edit `r2-closure-ledger.md`, `r3-structure.md`, or `r3-verification-manager.md`.
- Do not implement the Rust test in this slice.
