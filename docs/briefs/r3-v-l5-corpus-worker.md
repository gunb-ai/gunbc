# R3 Lane 2 — T-Verification-L5-Corpus Worker Brief

**Status:** STANDBY — gates sequentially on Lane 1 (T-V-L4-L7-Direct) corpus existing + Shape A 3-target grounding (Rust + Python + Go). Brief authored at R3 Verification Manager spawn (2026-04-30); converts to dispatch-ready when prerequisites fire.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — Lane 2 of 2 owned lanes (per `r3-structure.md` L108 authority).

**R3 lane authority:** [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" row **T-Verification-L5-Corpus** (L93).

## Scope (this lane)

L5 cross-target equivalence corpus — algebraic equivalence of computational results across Rust / Python / Go on the certification corpus. **Not byte identity**; algebraic equivalence per [`docs/r3-structure.md`](../r3-structure.md) L11 and the PR-D semantic lock at [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md).

L6 form coverage is **explicitly out of scope** — moved to R2-T-Ground-CrossTarget-Meta per [`r3-structure.md`](../r3-structure.md) L92-93 (different input space: per-(form × target) vs per-Behavior; lives as its own substrate-load-time completeness primitive per codex BLOCKING `90220bd97`).

## Dependencies (gates — sequential)

| Dependency | Source | Why |
|---|---|---|
| **Lane 1 corpus exists** | [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md) | Critical path per [`r3-structure.md`](../r3-structure.md) L147: "Verification has its own internal critical path: T-V-L4-L7-Direct → T-V-L5-Corpus (because Corpus's L5 cross-target work consumes Direct's L4 corpus)." |
| **Shape A 3-target grounding** | R2-Grounding-Rust + R2-Grounding-Python + R2-Grounding-Go | L5 is by definition cross-target; needs all 3 emit paths on the same `Dag`. |
| **R2-T-Ground-LanguageSpec** | Grounding Manager continuation | Per-target primitive realization + typed capability edges for comparable observations across Rust / Python / Go. |

## Implementation slices (when dispatch fires)

1. **Slice 1 — corpus seed:** import Lane 1's per-target L4 corpus as the seed for L5 cross-target receipts. One `TestClaim` per corpus program using `TestPredicate::ForAllTargets` (substrate scaffold at `src/v3/std/verification.dag` ~L147; in-file 🟡 dissolution comment respected — emit-scoped use only after Director approval per [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) §slice 2).
2. **Slice 2 — corpus expansion:** grow the corpus along the certification axes (per Verification thesis). Each new program adds one row; no new predicates.
3. **Slice 3 — strict L5 fire:** when corpus is broad enough that `l5_cross_target_consistency` carries non-trivial coverage, fire the strict gate per [`r3-structure.md`](../r3-structure.md) L93.

## Structural acceptance — `.dag` hook

| Gate name | Fixture (proposed) | Suite |
|---|---|---|
| `l5_cross_target_consistency` | `src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag` | `r3_verification_l5_corpus_suite` |

Per [`r3-structure.md`](../r3-structure.md) L56 authority — for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus.

**Stability invariant** (per [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) §"Deferred fixture path"): keep the **declaration name** stable; only update fixture-module path with manager-brief + integration-test co-update in one PR.

## Explicitly out of scope

- **L4 per-target equivalence** — Lane 1 ([`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md)).
- **L6 form coverage** — R2-T-Ground-CrossTarget-Meta (Grounding Manager).
- **Curated corpus authoring at R2** — explicitly post-R2 per [`r2-evaluator-manager.md`](r2-evaluator-manager.md) "primitives only — corpus authoring is post-R2".
- **New `TestPredicate` variants** — `ForAllTargets` already on substrate; INVARIANTS §P1 only for genuinely new facts.

## Cross-refs

- Parent manager: [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md)
- R3 lane row: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-Verification-L5-Corpus (L93)
- Upstream Lane 1: [`docs/briefs/r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md)
- Upstream PR-D semantic lock: [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md)
- Upstream PR-D scaffold: [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)
- L6 reclassification: [`docs/r3-structure.md`](../r3-structure.md) L92-93
- THESIS surface: [`THESIS.md`](../../THESIS.md) §"Tier 3 — Verification from structure" (L5 claim)
