# PR-E E9 - Post-W1 Lane 1 consumption readiness delta

**Status:** READINESS DELTA - docs-only. This note updates the E9
cross-target consumption surface after #1417, #1485, and #1499. It does not
authorize L5 corpus execution, target enumeration, new `TestPredicate`
variants, substrate edits, or observation-channel conventions.

**Parent authority:** [`r3-pr-e9-cross-target-harness-consumption-readiness.md`](r3-pr-e9-cross-target-harness-consumption-readiness.md)
and [`../design-cross-target-equivalence.md`](../design-cross-target-equivalence.md).

**W1 authorities:** #1485 records the approved W1 producer-contract carve-outs;
#1499 lands the narrow `DifferentialEquals(rust_emit_output, dag_eval_output,
ProgramOutputBind)` runner path for Rust / Int output under those carve-outs.

## Post-W1 Live State

W1 is no longer a blanket Lane 1 blocker on current `main`:

- `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`
  carries the Rust emit/eval `DifferentialEquals` row.
- `src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs`
  expects that row to pass through the wired W1 path.
- `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_mixed_lineage.dag`
  keeps unsupported mixed producer pairs fail-closed as `NotYetImplemented`.

The W1 path remains intentionally narrow. It is a Rust-only, Int-only,
single-output-bind `DifferentialEquals` slice using transitional producer
identity and stdout normalization authority from #1485 / #1499. It is not a
typed multi-target observation model and does not make `ForAllTargets`
execution strict.

## First Unblocked Post-W1 Slice

The first truly unblocked consumption slice is **Lane 1 / L4 direct
consumption**, not E9/L5 execution:

1. Add or extend a single Rust `DifferentialEquals(rust_emit_output,
   dag_eval_output, ProgramOutputBind)` corpus row using the existing W1
   producer pair.
2. Keep the program inside the currently proven W1 surface: pure, branch-only
   or otherwise evaluator-supported, one named Int output bind, no callable /
   fold / effect dependency unless that evaluator behavior is already live.
3. Keep the oracle as `.dag` eager evaluation through `dag_eval_output`; do not
   use emitted Rust as the semantic authority.
4. Preserve the mixed-lineage `NotYetImplemented` control so future changes do
   not weaken the W1 producer-identity gate.

This slice consumes W1 to grow L4 evidence. It may seed later L5 planning, but
it is not itself a cross-target `ForAllTargets` receipt.

## E9 / L5 Gates Still Closed

Strict E9 consumption remains blocked until these gates are all cited together:

| Gate | Current post-W1 status |
|---|---|
| **LanguageSpec readiness** | Still blocked. W1 runs a Rust producer path; it does not provide target-language capability facts for Rust / Python / Go. |
| **All Shape A targets grounded** | Still blocked. #1499 covers Rust only; Python and Go Shape A emit/run paths are not grounded by W1. |
| **Corpus home** | L4 has a W1 fixture receipt; L5 still needs an approved corpus-home decision before rows are expanded beyond the existing skeleton. |
| **Typed structural observation carrier** | Still blocked for L5. W1's Int stdout parse is an approved transitional carve-out, not the typed observation-channel/value-kind surface required for strict cross-target equality. |
| **`ForAllTargets` runner authority** | Still blocked. The L5 skeleton remains a `NotYetImplemented` scaffold and must not be implemented through raw command strings or target enumeration. |

## STOP+PING Boundaries

Stop and ping rather than implementing E9 if the next slice would require:

- choosing target sets without LanguageSpec / Shape A authority;
- comparing raw stdout bytes, emitted source, diagnostic strings, or command
  exit code as semantic equality;
- adding a new predicate variant, runtime `Value` shape, or substrate carrier
  outside a P1 proposal;
- relocating or expanding the L5 corpus without an approved corpus home;
- treating W1's Rust / Int carve-out as a general observation-channel policy.

## Readiness Verdict

#1499 opens a narrow L4 consumption lane: more Rust / Int
`DifferentialEquals` rows may be considered when they stay inside the current
evaluator and W1 producer surface. E9 itself remains readiness-only for strict
L5 execution. The first post-W1 implementation slice belongs to Lane 1/L4
direct evidence; the first E9/L5 implementation slice is still blocked on
LanguageSpec, Shape A target grounding, corpus-home, and typed observation
authority.
