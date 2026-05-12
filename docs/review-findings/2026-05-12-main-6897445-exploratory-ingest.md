# Exploratory review ingest — main@6897445

**Source:** [ChatGPT gunbc review thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/6a01c575-2afc-83ea-8e17-9162912bccab)  
**Backend:** gpt-5-5-pro  
**Analyzed tree:** `main@6897445b874f1831468f27c871c00f5b23d7ded2`  
**Ingested against:** `2878c5d7772c59642aca8b5b1296cd4da9b391ea`  
**Scope reported by analysis:** `dsl/std/*.dag`, `src/v2/tests/src/*.rs`, `dsl/extdeps/`, `THESIS.md`, `INVARIANTS.md`, `MODELING.md`, `ROADMAP.md`

This file is archival review-ingest context. Operational routing lives in `ROADMAP.md` under "Post-merge debt (2026-05-12 exploratory analysis at `main@6897445`)".

## Thesis frame

The analysis used the project thesis as the review oracle: a `.dag` program is the dependency graph, declared causes and dependencies should be structurally coherent before emission, and bounded data/iteration/composition are load-bearing. The cited rubric mapped to `INVARIANTS.md` P2 single authority, P3 no fabricated fallback, P4 bounded decidability, P5 active bridge dissolution, and `MODELING.md` M2/M5/M6/M7/M8.

## Reconciled findings

| # | Finding | Ingest disposition |
|---|---|---|
| 1 | Parser hard-coded prelude alias table | Already tracked: `ROADMAP.md` course correction for `PRELUDE_BARE_RHS_ALIAS_IDENTS` dissolution. |
| 2 | `post_emit_verifier` unbounded host-process execution and full output capture | Novel specific row added to `ROADMAP.md`. |
| 3 | `complexity.dag` lets `SameArgumentCall` report successful `UnknownCost` on the descent-operand path | Novel specific row added to `ROADMAP.md`. |
| 4 | Bootstrap load-order facts live in `build.rs` | Partly tracked by Pure Bootstrap and duplicate-authority rows; no separate row added here. |
| 5 | `behavior_result_port` repeated across consumers | Already tracked under shared substrate projection / behavior result-port consolidation. |
| 6 | `Encoding` hand-rolls `BoundedLattice` instance | Already tracked under hand-rolled lattice / bounded-lattice rows. |
| 7 | `DescentEvidence` hand-rolls `BoundedLattice` instance | Already tracked under hand-rolled lattice / bounded-lattice rows. |
| 8 | `CompositionVerdict` is first-breaker monoid but not modeled as one | Novel algebraic framing row added to `ROADMAP.md`. |
| 9 | Target primitive grounding uses `algebra: String` | Already tracked under target primitive carrier / `InhabitantDecl` string rows. |
| 10 | Generic `Lookup<T>` exists but constructors are monomorphized | Already tracked as generic lookup landed with monomorphic constructors remaining. |
| 11 | `std.http_path` duplicated inside v3 `effects.dag` | Already tracked in duplicate-module / embedded mirror rows. |
| 12 | `List<T>` has two live authority shapes | Already tracked by `src/v3/std` vs `dsl/std` convergence. |
| 13 | Anthropic schema mirrored in `dsl/extdeps` and `src/v3/std` | Already tracked as provider/API mirror multiplication risk. |
| 14 | `Certainty` is non-isomorphic between std primitives and v3 complexity | Novel exact type-duplication row added to `ROADMAP.md`. |

## Novel rows checked against current tree

### 1. `post_emit_verifier` bounded-process gap

`src/v3/compiler/src/post_emit_verifier.rs` still documents that `run_post_emit_verifier` invokes a declared command and collects stdout/stderr (`:22-25`). The live implementation builds `Command`, appends the source path, and calls `.output()` (`:171-198`), then converts the full buffers into owned strings (`:200-202`) and carries them in `WrongExitCode` / `PolicyViolation` (`:107-117`, `:204-227`).

The verifier spec is structural enough for command, args, exit code, and output policy, but not for wall timeout, I/O cap, streaming/discard policy, or setup/spawn/exit failure taxonomy. This is separate from the already landed bounded `ExecuteCommand` direction.

### 2. `SameArgumentCall` complexity fail-closed contradiction

`src/v3/lenses/complexity.dag` says no-descent recursion should align with `cost.dag`: `SameArgumentCall` should be `Miss`, not successful `UnknownCost` (`:186-190`). That holds for the `per_call_descent_operand_port -> None` path (`:190-208`).

The descent-operand path still calls `summary_from_iter_bound(pattern_to_iter_bound(...))` (`:237-253`), `pattern_to_iter_bound(SameArgumentCall)` returns `UnknownCost` (`:255-273`), and `summary_from_iter_bound(UnknownCost)` returns `Hit(conservative_unknown_summary(...))` (`:275-278`). That permits a successful complexity summary for the exact case the comment says should fail closed.

### 3. `Certainty` split

`dsl/std/primitives.dag` declares `Certainty = Proven | Amortized | Expected | Conservative` and uses it in `PrimitiveContract.certainty` (`:32-39`). `src/v3/lenses/complexity.dag` declares a separate `Certainty = Proven | Conservative` (`:65-70`).

The complexity design doc argues the lens-local two-variant concept is intentional until another consumer needs it. The current tree already has a same-named std primitive certainty with four variants, so the live state needs either a distinct name plus projection or a shared authority. Otherwise `Amortized` / `Expected` cannot map losslessly into the complexity lens certainty.

### 4. `CompositionVerdict` first-breaker monoid

`dsl/std/effects.dag` defines `CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker }` and implements `compose_effects` by filtering non-idempotent effects and selecting the first (`:146-161`). The v3 copy uses `ElementRef<OperationEffect>` and still computes `first_breaker_ref` then returns `BrokenBy` or `IdempotentComposition` (`src/v3/std/effects.dag:454-506`).

`dsl/std/algebra.dag` already declares `Monoid<T>` as `op` plus `identity` (`:110-115`). The effect composition law has the shape of a first-success / first-failure monoid: identity is `IdempotentComposition`; `BrokenBy(x)` absorbs on the left; `IdempotentComposition` yields the right-hand verdict. The current implementation keeps that algebra as local procedural list logic.

## Highest-value follow-up

The top implementation candidates are:

1. Give `PostEmitVerifier` the same bounded host-spawn policy shape as `ExecuteCommand`, then make `run_post_emit_verifier` consume it.
2. Make `SameArgumentCall` structurally unable to enter the descent-operand path, or change complexity's `pattern_to_iter_bound` to return `Lookup<SymbolicCost>` and `Miss` for `SameArgumentCall`.
3. Rename/project the two `Certainty` concepts or consolidate them under a single authority.
4. Declare `Monoid<CompositionVerdict>` or generic `First<T>` / `FirstFailure<T>` and express `compose_effects` as a fold over that witness.
