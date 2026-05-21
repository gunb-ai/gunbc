# HostModel Option C Runtime Inventory — 2026-05-21

## Decision

Operator-direct Option C decomposes the former `HostModel` umbrella:

- Abstract runtime carriers live in `src/v4/std/runtime.dag`.
- Concrete runtime fact-bundles live in `src/v4/extdeps/runtimes/`.
- No substrate-level umbrella record re-bundles all runtime carriers.

## Current Concrete Runtime Inventory

| Runtime | Status | Concrete file | Notes |
|---|---|---|---|
| v4 evaluator / TestClaim runner | Modeled in this PR | `src/v4/extdeps/runtimes/v4_evaluator.dag` | This is the concrete runtime surfaced by `src/v4/compiler/05_eval.dag` today. It binds `ModelCore` to `ValueRepresentationModel`, `InterpretationAlgebra`, `ExecutionSemantics`, and `ResourceEffectBoundary`. |
| Rust emitted-code execution | Not modeled as a runtime extdep in this PR | None | `src/v4/extdeps/languages/rust.dag` is a language model for grammar/primitives/target serialization. No current v4 file declares a Rust runtime interpretation algebra. |
| POSIX process substrate | Not a full eval runtime | `src/v4/extdeps/process.dag` | This models process invocation/lifecycle facts. It is a resource/process substrate that a future concrete runtime may consume, not itself a concrete evaluator runtime. |

## Non-Speculation Boundary

This PR authors only `v4_evaluator.dag` as a concrete runtime because it is the only runtime shape directly present in the v4 eval/test-claim path. Rust and POSIX process runtime bundles should land when a consumer needs their value representation, interpretation algebra, execution semantics, and resource/effect boundary facts.
