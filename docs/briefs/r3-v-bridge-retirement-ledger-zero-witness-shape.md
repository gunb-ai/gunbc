# R3 Bridge Retirement Ledger Zero Witness Shape

**Status:** PROPOSAL — research-only runtime-witness pre-authoring for the
`bridge_retirement_ledger_zero` gate. No substrate edits, no fixture changes,
and no new `TestPredicate` variants.

**Authority:** `src/v3/std/bridge_ledger.dag` owns the `BridgeLedger` carrier
and row statuses; `src/v3/std/verification.dag` owns
`TestPredicate::BridgeLedgerZero`; PR #1395 is the row-by-row structural audit;
PR #1396 is the canonical-lens row split that applied the Q2 per-class pattern.

## Finding

No additional Evaluator runtime witness is required for
`bridge_retirement_ledger_zero`.

This gate is already a substrate fold over the static
`v3.std.bridge_ledger.bridge_ledger` carrier. Its witness is the carrier value
itself: each row has a closed-coproduct `status: BridgeStatus`, and the runner
checks whether every status constructor is `Retired`. Owner programs retire
bridges by landing their structural receipt and updating the substrate row from
`Open` to `Retired`; Verification's gate observes that carrier update.

That makes this gate different from `Lens<C>`-fold claims such as TC1/TC2/TC3
and Free-Consequences. Those claims need runtime Evaluator work because the
witness is produced by applying a lens or evaluating a program. The bridge
ledger-zero claim reads an already-materialized substrate declaration; no
program evaluation, lens fold, or emitted target run is part of the witness.

## Existing Runtime Path

The production fixture
`src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag` already
declares `BridgeLedgerZero { ledger: { decl: bridge_ledger } }`. The structural
subject is the `bridge_ledger` declaration, not an executable program. The empty
`source` field is a current `TestClaim` shape limitation; the typed predicate
payload is the authority-carrying edge.

`test_runner.rs::eval_bridge_ledger_zero` already implements the binary fold:

- unwrap `BridgeLedgerRef { decl }`;
- fail closed unless `decl` is the canonical `bridge_ledger` declaration;
- fail closed unless that declaration is `List<BridgeLedgerRow>`;
- resolve `BridgeStatus::Retired` structurally from the coproduct;
- collect every row whose `status` constructor is not `Retired`;
- return `Pass` only when that collection is empty.

Failure diagnostics name every non-retired row. That diagnostic is the current
runtime witness for the red state: the gate reports the exact rows that still
block ledger-zero.

## Current Carrier State

After PR #1396, the canonical ledger has six rows because the old canonical-lens
row was split by class. Current `Open` rows are source-span participation,
canonical-lens-name patching residual, `include_str!` side channels, and the
exact-string patching residual umbrella. Current `Retired` rows are Secret
nominal opacity and the PR #1183 canonical-lens dispatch slice.

The Q3 discipline remains binary even when the row count grows by class split:
any non-`Retired` row makes the unified claim `Fail`; zero non-`Retired` rows
makes it `Pass`.

## Pass-State Contract

When the last owner program retires its bridge:

1. The owner lands the structural retirement receipt in its natural lane
   (Substrate or PB).
2. The substrate row in `src/v3/std/bridge_ledger.dag` flips from `Open` to
   `Retired` with authority pointing at that receipt.
3. The existing fixture remains unchanged:
   `BridgeLedgerZero { ledger: { decl: bridge_ledger } }`.
4. The integration test
   `m1_5_verification_test.rs::r3_bridge_retirement_ledger_zero_fixture_reports_open_rows_at_head`
   is re-armed in the same PR: instead of expecting `Fail` plus row diagnostics,
   it expects `ClaimResult::Pass`.

That re-arm is not a new witness design. It is the mechanical test-side
acknowledgment that `bridge_ledger_open_row_names()` has become empty and the
already-authored fold has crossed its binary boundary.

## Cross-Program Coordination

Verification owns the ledger-zero audit gate only. It does not retire bridges
and does not author per-row status flips. The coordination path is:

bridge owner program retirement -> substrate row update -> existing
`BridgeLedgerZero` fold -> integration ratchet flips from red to green.

This pre-authored shape therefore has a "no further runtime work" outcome:
unlike the Evaluator-dependent witness streams, the runtime behavior needed for
the future Pass state is already present. Future work should focus on the
bridge-owner receipts and the substrate row updates, then re-arm the integration
expectation in the same PR that flips the final row.
