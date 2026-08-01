# Native selected-witness bundle — first executable slice

Status: implementation design for the 2026-08-01 `native-selected-witness-bundle` dispatch. This extends [witness realization](witness-realization-plan.md); it does not replace that lane or change floor scheduling.

## The boundary

The production selector already decides which witness entries are relevant. Native realization must consume that answer instead of recreating selection:

```text
SelectedWitnessPlan
  -> NativeWitnessBundleIdentity(member witness identities, compiler identity, shard policy)
  -> one cached native artifact
  -> one process making direct member-function calls
  -> NativeWitnessBundleExecutionReceipt
```

`std.selected_witness_bundle` owns the carriers and admission laws. `SelectedWitnessPlan.members` is the complete selected population. A member is either `NativeSelected`, with its native realization identity, or `InterpreterFrontier`, with a typed cause, reason, and dissolution trigger. The latter is counted compatibility coverage; it is never silently omitted from the selected set.

## Identity and materialization

`NativeWitnessBundleIdentity` is a structural fingerprint built only through `std.content_hash`. Its ordered inputs are:

1. compiler identity;
2. declared shard policy;
3. each native member's witness identity (`entry`, `function`, subject digest);
4. each member's native realization family and realization digest.

This is the structural fnv1a64 `ContentHash` family, not an OCI or cryptographic digest. The carrier states that boundary explicitly until `feature:content-hash-family-grounded` makes the distinction structural. The host realization continues to add the existing `artifact_realization_digest` over materialized files and build argv and the resolved-build-context identity before trusting a warm marker. Thus a subject, compiler, emitted dispatcher, tool, environment, or Cargo-configuration change cannot serve an old binary.

This first slice chooses `OneSelectedBundle`: the selected subset is three tiny members in one already-proven logic family (`meet`, `join`, `complement`), so a shard boundary would add another process and artifact without isolating a cost or capability difference. The stable-shard arm is modeled for the later whole-selected-set slice; changing policy changes bundle identity.

## Cutover safety

The interpreter remains an equivalence oracle during cutover. A family is native-consumable only when both facts execute:

- every selected member's native verdict equals its interpreted verdict, including failure verdicts;
- at least one deliberately wrong body in the family is rejected by the same interpreter-oracle comparison used for cutover.

`native_witness_bundle_execution_admit` takes the `SelectedWitnessPlan` and current compiler identity at the acceptance boundary. Before inspecting verdicts it refuses a stale compiler, wrong bundle identity, an empty native plan, an incorrect counted interpreter frontier, or any receipt whose ordered `(witness identity, realization family)` roster differs from the plan's complete native roster. It then refuses the first divergence with the member and both typed verdicts. The wrong-body control deliberately preserves the interpreter's canonical `1,1,0` answer while the swapped-arm native bundle returns `0,0,1`; admission must therefore produce `NativeBundleExecutionRefusedDivergence`, never relabel the two answers as an equivalent failure. It also refuses a family with no live discriminating RED and any receipt claiming other than one bundle process. Nothing auto-picks native or interpreted after a divergence.

Bundle admission is a separate check: a bundle compiled under a different compiler identity is `NativeBundleRefusedStaleCompiler`; a bundle whose identity does not equal the current plan is `NativeBundleRefusedPlanIdentity`. Both refuse before execution.

## Executable slice and receipts

`v2.test.execution.native_selected_witness_bundle` reuses `emit_family` and the logic family's single member roster. Its generated Rust `main` invokes all three emitted functions directly and writes one fixed-width verdict value per member. One host invocation therefore corresponds to one artifact process and three direct calls.

The executing witness records:

- cold build + run, then a second-process warm run with `compile_skipped = true`;
- native and interpreted wall durations around the same three-member population, grounded as `NanosecondDuration = Measure<Time, Nano, Nat>`;
- per-member verdict equivalence for the correct bodies;
- the all-wrong-body control, where all three native verdicts fail against canonical interpreter passes and the typed divergence refusal names `meet` first;
- stale-compiler, identity-drift, empty/incomplete member coverage, counted-frontier drift, no-native-realization count, missing-RED, process-count, and native/interpreter-divergence refusals.

The wall observation uses the existing `ObserveElapsedAtSubject` model through the seed's `observed_monotonic_nanos(boundary)` realization. Distinct boundary labels prevent pure-call memoization from collapsing start and finish, but are never identity material. It reads elapsed monotonic time only, never calendar time or identity.

The hand-Rust disposition is authored beside that effect in `std.realization_measurement.elapsed_observation_hand_rust_deferral_note`: it is a bounded `v1-hand-queue-drain` deferral under ROADMAP's “Get hand-written Rust in this repository down to zero” row, and dissolves when the emitted runtime owns monotonic observation and `v1-interpreter-delete` removes the seed arm. This document points to that carrier rather than restating a second receipt.

Local receipt on 2026-08-01 (`CTRL_BUILD_MODE=local`, aarch64 Linux): all 25 bundle witnesses passed in one resolved entry after the fail-closed admission expansion. The correct bundle's cold+warm host witness was 974 ms and the wrong-body cold+warm witness was 946 ms; both logged `compile_skipped=false` then `compile_skipped=true`. The three interpreted correct-member controls totaled 17 ms (7+5+5 ms). Invoking the already-built native artifact directly took 3 ms wall for all three calls in one process; invoking it through the existing warm `cargo run --quiet` host transport took 457 ms and compiled nothing. These name different surfaces deliberately: the direct artifact number is production execution shape, while the larger cached-host number is the current development realization transport retained for the equivalence oracle.

## Deliberate exclusions

- No floor executor or `run_walk` scheduling changes; Dispatch E owns that surface.
- No claim-discovery or affected-set reimplementation; the eventual floor adapter supplies the already-decided `SelectedWitnessPlan`.
- No one-binary-per-function fallback.
- No claim that the full corpus is native-realizable. Every unavailable selected member remains a counted `InterpreterFrontier` row until its named trigger lands.

## Next slice

Adapt the production selected-entry projection to construct `SelectedWitnessPlan`, group native-capable members into the smallest stable shard set justified by measured capability/cost boundaries, and let the floor select this execution kind. After each family's cutover evidence is live, the production runner consumes the native artifact only; interpreter comparison moves to the development/falsifier cadence.
