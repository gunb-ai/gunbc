# R2 PR-C — Reflection Completeness Dissolution Gates

**Status:** PROPOSAL — docs-only structural-gate consumption brief for Evaluator PR-C. This does not reopen the reflection implementation. It names the live PR-C surface, the downstream gates that consume it, and the remaining dissolution conditions before PR-C can be closed as an R2.5/R3 dependency.

## Read First

- [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) — PR-C design lock, landed via [PR #1129](https://github.com/gunb-ai/gunbc/pull/1129).
- [`src/v3/compiler/src/lens_apply.rs`](../../src/v3/compiler/src/lens_apply.rs) — `reflect_program_dag_nodes_in_file`, complete `substrate_reflection` implementation, and PR-E `fold_lens_over_reflected_program` entrypoint; complete reflection implementation landed via [PR #1170](https://github.com/gunb-ai/gunbc/pull/1170).
- [`r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md) — PR-E reflect -> apply slice and deeper fold boundary.
- [`r2-evaluator-manager.md`](r2-evaluator-manager.md) — Evaluator lane status and acceptance gate name.
- [`r2-evaluator-closure-residuals.md`](r2-evaluator-closure-residuals.md) — R2 Release ledger companion for PR-D / PR-E / TC2 deferrals.

## Landed PR-C Surface

PR-C has two landed parts:

1. **Spec authority:** [PR #1129](https://github.com/gunb-ai/gunbc/pull/1129) locked `docs/design-reflection-completeness.md`. Complete reflection means every substrate-declared field on each `Behavior` variant and each reachable nested substrate type projects into `FieldValue`, with no per-consumer narrowing and no execution semantics.
2. **Implementation authority:** [PR #1170](https://github.com/gunb-ai/gunbc/pull/1170) implemented the complete reflection path in `src/v3/compiler/src/lens_apply.rs`. The live entrypoint is `reflect_program_dag_nodes_in_file(program, source_file, id_space)`, backed by the in-file `substrate_reflection` module and unit coverage for optional fields, all five `Behavior` variants, branch-arm totality, loop-bound coproducts, and substrate-conj / sum-payload shape matching.

That means PR-C is **not** an open request to add more reflection fields or rewrite evaluator Rust. Any newly discovered missing structural fact must be routed through the normal substrate-fact-introduction procedure, not patched as a PR-C local exception.

## Live Entry Boundaries

`reflect_program_dag_nodes_in_file` is the PR-C reflection boundary. It filters program nodes by `source_file`, reflects the resulting `Behavior` list into the `FieldValue` carrier, and takes constructor IDs from the supplied `id_space` so the reflected values match the lens program that will consume them.

`fold_lens_over_reflected_program` is the PR-E consumer boundary. It calls `reflect_program_dag_nodes_in_file(program, source_file, lens_program)`, prepends the reflected carrier to user inputs, and delegates to `apply_lens_declaration`. This is the landed reflect -> apply slice, not the full `Lens<C>` / `DimensionReport<C>` fold.

Errors remain fail-closed. Reflection shape failures surface as `LensApplyError::SubstrateReflect`; unsupported lens-body behavior, loop execution, unresolved ports, bad list shape, and other interpreter failures surface through existing `LensApplyError` variants. PR-C must not fabricate a `DimensionReport` or downgrade reflection failures into successful lens output.

## Structural Gates

### Gate 1 — Reflected Program DAG Completeness

The reflected carrier must preserve the `docs/design-reflection-completeness.md` §4 contract:

- `ValueNode`: `id`, `payload`, `result_port`, `span`, `lane2_workflow`.
- `TransformNode`: `id`, full `target` coproduct, `inputs`, `result_port`, `span`.
- `BranchNode`: `id`, `input`, every `paths` entry, branch `result_port`, `span`, `emit_participation`.
- `LoopNode`: `id`, `source`, `init`, `body`, full `bound` coproduct, `result_port`, `span`.
- `BindNode`: `id`, `name`, `result_port`, `params`, `span`, `lane2_workflow`, `emit_participation`.

The current Rust tests in `lens_apply.rs` are implementation evidence for this gate. R2.5/R3 consumers should cite the PR-C spec plus #1170 implementation instead of re-auditing field-by-field in every downstream brief.

### Gate 2 — Lens Fold Consumption Boundary

PR-C ends at the reflected `FieldValue` program carrier. PR-E owns consumption of that carrier by lens declarations.

The landed PR-E slice proves only this bounded path: reflect the program spine, pass it as the first lens argument, and execute the current bounded lens interpreter. The following remain PR-E / PR-B dependencies, not PR-C work:

- Full `Lens<C>` aggregate/fold semantics.
- `DimensionReport<C>` construction and accumulation.
- Lens-instance execution through PB-Runtime-aligned body semantics.
- Any fold-driver behavior that requires runtime `Value`, evaluator frames, strategy, or memoization.

Until those land, PR-C consumers may rely on complete reflection as input, but may not claim full lens application has dissolved.

### Gate 3 — Diagnostic And Fail-Closed Behavior

PR-C preserves the fail-closed boundary:

- Reflection shape mismatches fail as `SubstrateReflect`.
- Lens interpreter gaps fail through explicit `LensApplyError` variants.
- Unsupported loop execution remains `UnimplementedLoopBound` until PR-B supplies evaluator execution semantics.
- The reserved `UnimplementedLensFold` variant stays a PR-E fold-driver marker, not a PR-C residual.

Diagnostic authority follows `docs/design-lens-framework.md` Q6.5: Layer-1 compiler diagnostic kinds remain substrate-owned; lens-local Layer-2 diagnostic kinds are declared by lens instances. PR-C supplies complete structural facts to those lenses. It does not add diagnostic kinds and does not convert reflection failures into lens-local diagnostics by hand.

### Gate 4 — Dissolution Conditions

PR-C can be declared dissolved only when all of the following are true:

1. The `evaluator_lens_application_complete_reflection` acceptance hook is live as a structural `.dag` gate, consuming the #1129 spec and #1170 implementation rather than restating them in prose.
2. PR-E has a live fold path that consumes complete reflection without per-lens Rust projection, and any remaining unsupported fold-driver states are named outside PR-C.
3. PR-B has supplied the evaluator body/runtime semantics needed to retire Rust-side reflection mirrors through evaluator/substrate-fact projection authority, without executing the reflected program during static reflection.
4. R2 Release / R3 consumer briefs cite this gate file or its successor when treating reflection completeness as available input.

The important distinction is status: **complete reflection is landed; dissolution evidence is still being wired.**

## Downstream Consumer Guidance

R3 and R2.5 briefs should use this wording:

- **Available now:** complete static reflection of program `Behavior` nodes into `FieldValue`, per #1129 and #1170.
- **Consume through:** `reflect_program_dag_nodes_in_file` directly, or through `fold_lens_over_reflected_program` when the slice-1 reflect -> apply shape is sufficient.
- **Do not assume yet:** full `Lens<C>` / `DimensionReport<C>` fold, PB-Runtime body execution, witness construction, TC2 strict equality, PR-D cross-target harness execution, or PR-A.3 strategy/memo carriers.
- **STOP+PING if needed:** any carrier shape change, new diagnostic-kind authority, new `TestPredicate` variant, bespoke Rust projection for a single lens, or evaluator execution requirement hidden inside reflection.

## Out Of Scope

- Rust edits.
- New fixtures or `.dag` claims in this slice.
- Substrate carrier declarations.
- PR-A.3 strategy / memoization carriers or tests.
- PR-B evaluator implementation.
- PR-D harness predicates.
- PR-E full fold implementation.

## Closure Handoff

When the `.dag` acceptance hook and closure-ledger wiring land, update:

- `r2-evaluator-manager.md` acceptance row for `evaluator_lens_application_complete_reflection`.
- `r2-evaluator-closure-residuals.md` PR-C note from "dissolution / structural-gate consumption" to the landed gate reference.
- R2 closure ledger rows that treat PR-C as available to R3 consumers.

Until then, PR-C is best described as **implementation landed, structural-gate consumption pending**.
