# v3/v4 Interim Validation Ledger

This ledger records PR-level dispositions for v4 validation proposals that would otherwise add or preserve v3 hand-authored Rust integration tests. The default under the Rust-to-0 review gate is that new `src/v3/compiler/tests/integration/v4_*_smoke_test.rs` receipts do not land as interim v4 validation.

| PR | Proposed receipt | Disposition | Preferred receipt |
| --- | --- | --- | --- |
| `#3330` | `src/v3/compiler/tests/integration/v4_test_bootstrap_wave0_smoke_test.rs`, a new v3 hand-Rust parse-surface ratchet over `src/v4/lens/testgen.dag`, `src/v4/workflow/bootstrap.dag`, and `src/v4/workflow/ci.dag`. | **Dropped before merge.** This was new interim v4-validation debt, not grandfathered. It is not listed in `EXPECTED_HAND_AUTHORED_TEST`, has no `INVARIANTS.md` P5 row, and is not wired from `src/v3/compiler/tests/integration.rs`. | Use non-census substrate/build receipts: CI `v4` / `v2-compiler compile --source-root src/v4 --target dag` for whole-tree v4 parse/resolve viability, plus the T-19/T-20/T-22/T-24 `.dag` authority PRs and their generated/TestClaim follow-ups for semantic surface checks. |
