---
status: PROPOSAL
owning_manager: Pure Bootstrap Manager (R2 → R3 continuation)
lane: T-Bridge-Retirement — distributed bridge #4 (`bridge_include_str_side_channels_retired`)
authored: 2026-05-06 (neat-bear-351 — PB Mgr cycle #1861 / Director #846 pre-auth queue)
---

# R3 PB — `include_str!` side-channel closure (pipeline_authority) — worker brief

**Status:** PROPOSAL — **pre-authored closure brief** (dispatch-gated). Does **not** authorize merging `include_str!` removal until **dispatch triggers** fire. Aligns with Director pre-auth brief-queue discipline: authoring is **not** merge; triggers gate worker pickup.

**Owning manager:** Pure Bootstrap Manager (R3 continuation per [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-Bridge-Retirement distribution map).

**Verification Manager ledger row:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — bridge **#4** `include_str!` side channels; **`bridge_include_str_side_channels_retired`** remains **open** until this closure lands structurally.

## Purpose

Close **`bridge_include_str_side_channels_retired`** for the **pipeline stage-order / compile-body authority** site by replacing **`include_str!`-mediated source identity** with a **substrate query surface** that reads the same facts **`PipelineStageBinding`** consumers need — **without** introducing a new runtime file-IO side channel.

**Non-goal:** wholesale grep-delete of every `include_str!` in the repo under one PR. This brief scopes the **PB-owned bridge class** called out in [`docs/design-emission-model.md`](../design-emission-model.md) Finding #12 / line ~944 and [`docs/r3-structure.md`](../r3-structure.md) T-Bridge-Retirement row (bridge #4).

## Live authority — open disposition (PR #1171)

Per [`docs/design-emission-model.md`](../design-emission-model.md) ~944:

- **`pipeline_authority`** reads pipeline stage order **structurally** from **`PipelineStageBinding`** rows in the `Dag`.
- **`fn compile`** remains **`ArrowBody::Unparsed`**, so **compile-body stage order is not yet a lowered substrate fact**.
- **PR #1171 (2026-04-29)** **suspended** the prior **`reconcile_with_compile_body`** path rather than swapping **`include_str!`** for **runtime file IO**.

**Corollary:** PB **must not** “close” this bridge by **reading `pipeline.dag` bytes from disk at runtime** as a substitute for `include_str!` — that repeats the **side-channel** failure mode #1171 explicitly rejected.

## Acceptance (`bridge_include_str_side_channels_retired` — scoped slice)

**Green for this site** when **all** hold:

1. **No `include_str!`** of **`pipeline.dag`** at any callsite at the **compile-pipeline authority boundary** (NOT file-tree-locality `src/` only). Scope-locked 2026-05-08 (PB Mgr `warm-dove-618` + Verification Mgr `wise-bear-525` concur, citing Director Option 1 multi-site umbrella ratchet precedent at #828 c#4401659641 and `feedback_substrate_principle_audit` "substrate facts close all-or-nothing"). At lock time the in-scope active set is `src/v3/compiler/src/pipeline_authority.rs` (already zero active, doc-comment only) **and** `src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs:12` (active `include_str!("../../pipeline.dag")` — rewrite to consume the structural witness is **part of this dispatch**, not a separate worker).
2. **Stage-order / compile-body facts** required by pipeline authority consumers are obtainable from **structured Dag data** (existing `PipelineStageBinding` discipline **and/or** a **derived lowered compile-body witness** once authored — substrate/evaluator coordination).
3. **Ratchet test**: a CI assertion that
    ```
    grep -rE 'include_str!\([^)]*pipeline\.dag' src/v3/compiler/
    ```
    returns **zero matches** at HEAD post-retirement (file-path-suffix predicate, ungameable-by-relocation under `src/v3/compiler/`). Catches the production callsite, the test callsite surfaced 2026-05-08, and any future relocation that re-introduces the pattern. Locked pre-merge per Director Option 1 precedent.

**Out-of-scope** (do not double-count under §1/§3):
- `src/v3/compiler/build.rs` `collect_dag_entries(..., &["pipeline.dag"])` — build-time filename enumeration, not source-text-as-string consumption.
- `pipeline_compile_body_remains_unparsed_blocking_structural_retirement` test — tracked under `bridge_source_span_file_participation_retired` (bridge #1) per `docs/briefs/r3-v-bridge-row-1-sourcespan-deeper-detail-receipt.md:82`.
- `bootstrap.rs` `PIPELINE_AUTHORITY_FILE` slice — already retired in PR #2150 per `docs/briefs/bridge-retirement-audit-sourcespan-family.md:85`.

Full **ledger-zero** remains Verification Manager audit until **all five** named bridges in the distribution map are green — this brief is **only** bridge #4’s PB closure slice for the **`pipeline_authority`** lineage.

## Dispatch triggers (mechanical)

Pick up implementation **only when** a **single live-state check** shows **both**:

| # | Trigger | Authority |
|---|---------|-----------|
| T1 | **Substrate / lowering** lands a **structural compile-body witness** OR an approved **derivation** path such that compile-stage order is a **Dag fact** without unparsed-body guessing — **or** Director records an explicit **narrow exception** revising #1171’s suspension rationale | Substrate Manager + Director / emission-model |
| T2 | **`pipeline_authority.rs`** change is **pair-authored** with a **Verification** ledger note (closure row movement) | PB + Verification Manager cadence |

Until **T1** is true, this brief stays **PROPOSAL / queued** — PB may **pre-author** tests and refactors that **do not** fake structural facts.

## STOP conditions

- **Runtime file IO swap** for `include_str!(…pipeline.dag…)` without **T1** — **STOP** (repeats #1171 rejection pattern).
- **New `TestPredicate` / carrier invention** — route **`INVARIANTS.md`** §P1 to Substrate Manager; PB does not invent substrate facts to satisfy the gate.
- **Broadened scope** (“delete all `include_str!` in compiler”) without per-site ledger mapping — **STOP**; violates bridge distribution discipline.

## Non-goals

- Resolving **`compile` unparsed-body** semantics globally (owned by substrate/lowering program — this brief **consumes** the witness once it exists).
- Closing **`bridge_canonical_lens_name_dispatch_retired`** (bridge #3 — LensProducer / PB-Runtime row).
- Closing **`bridge_source_span.file` participation** (bridge #1 — Substrate-deferred per [`docs/r3-structure.md`](../r3-structure.md)).

## Cross-refs

- Emission-model authority + #1171 disposition: [`docs/design-emission-model.md`](../design-emission-model.md) ~944 (`include_str!` retirement / `pipeline_authority`).
- R3 lane acceptance row: [`docs/r3-structure.md`](../r3-structure.md) §"T-Bridge-Retirement" — `bridge_include_str_side_channels_retired`.
- Verification ledger snapshot: [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) bridge table row #4.
- PB Manager scope bridge roll-up: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §closure identifiers + sub-briefs list.
- PR #1171 thread (historical suspension context): GitHub `gunb-ai/gunbc` PR **1171**.
