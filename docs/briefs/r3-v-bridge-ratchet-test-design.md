# R3 Bridge-Retirement Ledger-Zero Ratchet Test Design

**Status:** PROPOSAL - design-only, Director-ratified composition. No substrate, runner, or fixture implementation in this slice. The `BridgeLedger` carrier is Substrate-owned and not authored here.

**Owning manager:** R3 Verification Manager.

## Goal

Author the shape for a `bridge_retirement_ledger_zero` `.dag` `TestClaim` that gates the unified Verification audit:

- fails while any Director-ratified bridge row is open;
- passes only when all bridge rows are structurally green;
- never expands the bridge set without Director-approved per-class row enumeration.

Canonical bridge-map authority is [`docs/r3-structure.md`](../r3-structure.md) T-Bridge-Retirement plus [`r3-verification-manager.md`](r3-verification-manager.md) §"Bridge-retirement ledger - current state".

## Director Disposition

Director ratified Option B: structural fold over a forthcoming `BridgeLedger` carrier. The gate is a `.dag` `TestClaim`, not a Rust integration ratchet. It stays red while any row is non-`Retired` and passes only at ledger-zero.

`BridgeLedger` carrier shape is expected to be `{ name, owner, status, authority } x N rows`, with `data Retired` as the status constructor. `N` may exceed the original five bridge categories because exact-string patching splits into per-class rows.

## Per-Bridge Gate Audit

| # | Bridge | Canonical gate name | Current authority / gap |
|---|---|---|---|
| 1 | `SourceSpan.file` participation | `bridge_source_span_file_participation_retired` | Named in `r3-structure.md`; no strict compiler test or `.dag` predicate found. Open/R3-deferred per #1273. |
| 2 | Secret bootstrap nominal opacity | `bridge_mark_bootstrap_secret_nominal_opacity_retired` | Named in `r3-structure.md`; implemented today as a Rust unit test in `src/v3/compiler/src/dag.rs`. Retired by #1272. |
| 3 | canonical lens-name dispatch | `bridge_canonical_lens_name_dispatch_retired` | Retired by R3 gate #33. `canonical_lens_bridge_ratchet_test.rs` now pins the canonical-lens byte constants, lens-name equality arms, and generic lens-name lookups at zero in `test_runner.rs`. |
| 4 | `include_str!` side channels | `bridge_include_str_side_channels_retired` | Named in `r3-structure.md` and `r2-closure-ledger.md`; no strict compiler test or `.dag` predicate found. Open at `pipeline_authority` pending structural compile-body witness. |
| 5a | lower-helper generated-source exact-string patching | `bridge_lower_helpers_patch_zero_residual` | Concrete class from the former umbrella. `bridge_lower_helpers_patch_zero_residual_test.rs` proves this class is zero after #1014/#1192. |
| 5b | other exact-string patching classes | TBD per class | Director chose split rows, not umbrella. Audit found `bootstrap.rs::patch_kernel_bool_boolean_algebra_inhabits` tracked separately as Class 5 Gap 1 in `debt-paydown-synthesis-2026-04-25.md`; it should not be silently absorbed into 5a. No additional `dsl/std/algebra.dag` exact-string bridge row was found in this pass. |

## Standby TestClaim Shape

Author the fixture only after Substrate lands importable `BridgeLedger` / `BridgeLedgerStatus` carriers, or land the text below as the standby shape in the carrier PR:

```dag
module std.r3_verification_bridge_ledger_zero_deferred

import std.verification { BridgeLedgerZero, TestClaim, TestSuite }
import std.bridge_ledger { bridge_retirement_ledger }

data bridge_retirement_ledger_zero: TestClaim = {
  name: "bridge_retirement_ledger_zero",
  source: "",
  file_name: "src/v3/std/bridge_ledger.dag",
  predicate: BridgeLedgerZero {
    ledger: bridge_retirement_ledger
  },
  requires: []
}

data r3_bridge_retirement_ledger_zero_suite: TestSuite = {
  name: "r3_bridge_retirement_ledger_zero_suite",
  claims: [bridge_retirement_ledger_zero]
}
```

`BridgeLedgerZero` is a substrate-fact-introduction candidate, not an autonomously authored predicate here. The `TestClaim.source` field is intentionally empty in this standby shape because the subject is the structural `ledger` declaration, not an executable source program. If the eventual runner requires a non-empty source string, that is a sign the predicate shape is wrong for this carrier and should be redesigned rather than filled with a dummy program. The existing `RatchetZero` variant does not fit today: runner evaluation is hard-coded to `compiler_std_positive_set_ratchet`, not a generic structural fold over rows.

## Required Fold Semantics

1. Fold every `BridgeLedger` row.
2. Pass iff every `status == Retired`.
3. Fail if any row is non-`Retired`; diagnostic must list every non-retired row by `name`, `owner`, `status`, and `authority`.
4. Carry explicit `Open` rows for unresolved bridges (#1/#4 today); an `Open` row is evidence for red, not a skipped predicate.
5. Freeze the Director-ratified row set in the carrier. Any later row expansion is an explicit carrier change, not hidden test logic.

## Substrate-Fact-Introduction Signal

This TestClaim is authored against a not-yet-landed carrier. Structural activation occurs when Substrate lands `BridgeLedger`, status constructors including `Retired`, the canonical ledger value, and the strict fold predicate or equivalent `RatchetZero` generalization. Until then, this remains a standby shape and must not widen existing predicates.

## Non-Goals

- Do not add a new bridge category beyond Director-approved per-class split rows.
- Do not edit `r2-closure-ledger.md`, `r3-structure.md`, or `r3-verification-manager.md`.
- Do not implement the Rust test in this slice.
