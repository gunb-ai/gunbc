# R3 Lane 1 W1 Consumer-Side Implementation Spec

**Status:** **POST-W1 LIVE ANCHOR** (Rust / Int slice-1) — implementation record + expansion triggers for Lane 1 L4 W1. **Supersedes** the pre-2026-05-09 “await W1 landing” worker brief; that standby framing was **obsolete** once `test_runner.rs` wired `(rust_emit_output, dag_eval_output)` (see §Current Live State).

**Live-state refresh (2026-05-09):** Rust/Int W1 `DifferentialEquals(rust_emit_output,
dag_eval_output, ProgramOutputBind)` is **landed** in `src/v3/compiler/src/test_runner.rs`
(`eval_differential_equals`, `w1_rust_emit_output_int`, `w1_dag_eval_output_int`) with
integration receipts in `r3_verification_l4_l7_l5_skeleton_test.rs`. This brief still does
not authorize ad-hoc substrate or predicate variants beyond the locked W1 contract.

**Parent lane:** [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md)
and the Lane 1 readiness re-audit in
[`r3-v-l4-l7-direct-readiness-reaudit-post-e5.md`](r3-v-l4-l7-direct-readiness-reaudit-post-e5.md).

**W1 contract authority:** [`r3-pr-e8-w1-output-producer-contract-blocker.md`](r3-pr-e8-w1-output-producer-contract-blocker.md)
from PR #1485. **Implementation authority:** `src/v3/compiler/src/test_runner.rs` — W1 producer
identity dispatch for exactly `rust_emit_output` / `dag_eval_output`, Int stdout carve-out on the
Rust side, `evaluate_body` on the eval side (`w1_dag_eval_output_int`), fail-closed unsupported
pairings (`eval_differential_equals`).

## Current Live State

The Lane 1 L4 fixture lives at
`src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`. It authors
`l4_emit_eval_match` (§1.8 gate #9 canonical `TestClaim.name`) with
`DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind { output_ref: … })`.

The `.dag` module still declares `rust_emit_output` / `dag_eval_output` as `miss_int_lookup()`
**stubs** — by design per #1485 / fixture header: those bodies are not interpreted; the
**runner** supplies both sides. **`NotYetImplemented` does not apply** to this W1 row: the harness
compiles the fixture and asserts `ClaimResult::Pass` for the three certification seeds (see
`r3_verification_l4_l7_l5_skeleton_test.rs`). Pairings outside the W1 contract (e.g. mixed lineage
controls in `r3_verification_l4_emit_eval_mixed_lineage.dag`) remain explicitly deferred with typed
NYI / fail-closed outcomes per `test_runner.rs`.

**Historical Lane-E cost lineage** (`v3_program_cost` / `v2_oracle_cost`) remains a separate
`DifferentialEquals` pairing in the same runner; it does not gate W1.

**Evaluator entry:** `pub fn evaluate_body(...) -> Result<Value, EvalError>` in
`src/v3/compiler/src/lib.rs` is on the path `w1_dag_eval_output_int` uses for `dag_eval_output`.

## W1 Contract Summary

The W1 contract names two acceptable producer-identity routes: transitional
DeclarationRef-name dispatch for exactly `rust_emit_output` /
`dag_eval_output`, or a durable P1 producer role / marker surface. **Slice-1 on
main uses the transitional name-keyed route** in `test_runner.rs`; migrating to
markers is dissolution work, not a consumer rewrite of this fixture shape. It must not
introduce a new predicate, producer enum, or fixture-local authority. The
existing `DifferentialEquals` + `ProgramOutputBind` shape remains the consumer
surface.

The observation domain is likewise contract-bound. Durable W1 should read a
typed observation-channel/value-kind carrier. The narrow slice-1 carve-out may
parse emitted Rust stdout as a single trimmed `Int`, but only for this L4 Int
fixture and only with the W1 dissolution target named. Unknown producer pairs,
unsupported value shapes, failed Rust execution, stdout parse failures,
evaluator errors, and value mismatches must all fail closed with typed
`ClaimResult` outcomes.

## Slice-1 landed targets (authoritative for continuing work)

The following are **already satisfied** on main for Rust / Int W1 slice-1; use them as the
checklist for corpus expansion and multi-target follow-ons (do not re-litigate wiring):

- **Fixture:** `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`.
- **Suite:** `r3_verification_l4_l7_direct_suite` (L4 certification seeds; L7 merges under same suite name at lane close).
- **Claim:** `l4_emit_eval_match`.
- **Predicate row:** keep
  `DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind { output_ref: l4_out })`.
- **Program shape:** keep the existing minimal branch + fold source in
  `TestClaim.source` unless W1's landed producer contract requires a smaller
  fixture. Any fixture source change must preserve the claim name.
- **Eval side:** `dag_eval_output` invokes `evaluate_body` directly over the
  `ProgramOutputBind.output_ref` body using the landed eager strategy. If memo
  identity remains absent, the implementation must name the no-memo eager
  carve-out at the call site.
- **Rust side:** `rust_emit_output` emits the same claim program to Rust,
  compiles/runs it through the W1 path, captures the contract-authorized
  observation channel, and normalizes to the same typed value domain as
  `dag_eval_output`.
- **Comparison:** `DifferentialEquals` compares typed values, not stdout bytes or
  target source text.

Failure taxonomy: emit failure, Rust compile/run failure, observation parse or
normalization failure, evaluator failure, typed value mismatch, and unsupported
producer/value shape must each fail closed with stage attribution. Do not
fallback to success, stdout-byte comparison, or fixture-local lookup behavior.

## Test Infrastructure

Harness: `src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs` —
`OnceLock`-backed `cached_compile`, per-claim and suite runs. **W1 slice-1 expects `Pass`** for
`l4_emit_eval_match`, `r3_verification_l4_emit_eval_false_branch`, and
`r3_verification_l4_emit_eval_nested_branch` under `r3_verification_l4_l7_direct_suite`. Future
negative-path tests should assert structured `ClaimResult` categories without brittle message
substring coupling.

## Re-Engagement / expansion triggers

W1 **Rust/Int slice-1 wiring is landed** — do not re-dispatch a “wait for W1” worker for that
surface. Next Verification-owned triggers:

1. **§Acceptance `l4_emit_eval_match`:** grow the certification corpus until `r3-structure.md`
   §Acceptance coverage matches `docs/r3-program-plan.md` §1.7 corpus-quantified **PASSING** rule
   (ledger authority stays single: §1.8 row + structure §Acceptance).
2. **Additional targets / value kinds:** Python, Go, or non-Int observation domains — require W1
   contract amendments + runner work; until then stay within the landed Int carve-out.
3. **Durable producer markers:** if DeclarationRef-name dispatch is retired, update this brief’s
   contract summary to the substrate marker authority without splitting runner facts across docs.

If a regression shows `ClaimResult::NotYetImplemented` on the **canonical W1 triple** above,
route to R2-Evaluator / runner ownership — that is a live bug, not a spec standby state.

## Non-Claims (§Acceptance / scope boundaries only)

These bound **what this anchor does not certify** — they do **not** retract the **landed** W1 slice-1 facts in §Current Live State above (P1).

- This document does not claim **§Acceptance closure** for `l4_emit_eval_match` (full certification
  corpus per `r3-structure.md` — see `docs/r3-program-plan.md` §1.8 #9 / §1.7).
- This document does not claim W1 beyond the **Rust / Int stdout** slice-1 carve-out.
- This document does not assert `l4_emit_eval_match` is **PASSING** on the §1.8 ledger (only **CONSUMER_LANDED** until corpus-complete).
- This document does not authorize a new substrate role, observation carrier, or
  `TestPredicate` variant.
- This document does not widen W1 beyond the Rust single-target L4 slice-1 row.
- This document does not alter Lane 2 L5 or L7 algebraic-law dispatch.
