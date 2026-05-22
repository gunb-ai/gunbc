# HostModel Option C Runtime Inventory — 2026-05-21

## Decision

Operator-direct Option C decomposes the former `HostModel` umbrella:

- Abstract runtime carriers live in `src/v4/std/runtime.dag`.
- Concrete runtime fact-bundles live in `src/v4/extdeps/runtimes/`.
- No substrate-level umbrella record re-bundles all runtime carriers.

## Current Concrete Runtime Inventory

| Runtime | Status | Concrete file | Notes |
|---|---|---|---|
| v4 evaluator / TestClaim runner | Concrete bundle authored; eval consumes the interpretation slice today | `src/v4/extdeps/runtimes/v4_evaluator.dag` | `src/v4/compiler/05_eval.dag` currently takes `InterpretationAlgebra` directly. This bundle names the concrete v4 evaluator runtime fact-bundle that will bind `ModelCore` to `ValueRepresentationModel`, `InterpretationAlgebra`, `ExecutionSemantics`, and `ResourceEffectBoundary` when the session-layer runtime target wiring lands. |
| Rust emitted-code execution | Not modeled as a runtime extdep in this PR | None | `src/v4/extdeps/languages/rust.dag` is a language model for grammar/primitives/target serialization. No current v4 file declares a Rust runtime interpretation algebra. |
| POSIX process substrate | Not a full eval runtime | `src/v4/extdeps/posix.dag` | This models process invocation/lifecycle facts. It is a resource/process substrate that a future concrete runtime may consume, not itself a concrete evaluator runtime. |

## Non-Speculation Boundary

This PR authors only `v4_evaluator.dag` as a concrete runtime bundle because the current v4 eval/test-claim path already consumes its `InterpretationAlgebra` slice. `V4EvaluatorRuntime` itself is not imported by `05_eval.dag` in this PR; that wiring belongs to the session-layer `RuntimeTarget` / projection step. Rust and POSIX process runtime bundles should land when a consumer needs their value representation, interpretation algebra, execution semantics, and resource/effect boundary facts.
