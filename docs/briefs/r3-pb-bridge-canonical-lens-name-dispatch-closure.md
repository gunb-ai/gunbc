---
status: PROPOSAL
owning_manager: Pure Bootstrap Manager (R2 → R3 continuation)
lane: T-Bridge-Retirement — distributed bridge #3 (`bridge_canonical_lens_name_dispatch_retired`)
authored: 2026-05-06 (neat-bear-351 — PB Mgr cycle #1861 / next-tier queue)
---

# R3 PB — canonical lens-name dispatch closure — worker brief

**Status:** PROPOSAL — **pre-authored closure brief** (dispatch-gated). Aligns with [`docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md): partial retirement + ratchet already landed; **full** `bridge_canonical_lens_name_dispatch_retired` remains **open** until substrate-level unblockers fire.

**Owning manager:** Pure Bootstrap Manager (R3 continuation per [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-Bridge-Retirement distribution map).

**Verification Manager ledger row:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — bridge **#3** canonical lens-name dispatch; ledger-zero pending until structural closure.

## Purpose

Drive **`bridge_canonical_lens_name_dispatch_retired`** to **green** by eliminating the **remaining** runner-side bridge surface in `src/v3/compiler/src/test_runner.rs`:

- **(A)** `include_str!` of canonical lens bytes (`named_function_count.dag`, `complexity.dag`).
- **(B)** `lens_decl.name.as_deref() == Some("cost_of" | "named_function_count")` dispatch arms in `LensOutputEquals`.
- **(C)** Generic `lens_decl.name.as_deref()` name-keyed lens lookup (two sites).

**Non-goal:** claiming closure by swapping one string/path side channel for another. Per disposition §"Precise dependency", PB **must not** replace `include_str!` with an ad hoc path registry without a **structural** lens-identity carrier introduced through [`INVARIANTS.md`](../../INVARIANTS.md) §P1.

## Live authority — disposition consumed

Full gap analysis, P2 cross-`Dag` reflection rationale, and ratchet (`tests/integration/canonical_lens_bridge_ratchet_test.rs`) are **not re-argued here** — read [`r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md).

## Acceptance (`bridge_canonical_lens_name_dispatch_retired` — full close)

**Green** when **all** hold:

1. **Zero** remaining bridge debt in categories **A + B + C** above in `test_runner.rs` (or documented successor module), **without** violating P2 reflection discipline ([`INVARIANTS.md#p2-boundary-discipline`](../../INVARIANTS.md#p2-boundary-discipline)).
2. **`canonical_lens_bridge_ratchet_test`** updated or retired per closure — counts **never increase** per `feedback_ratchet_only_down`; ratchet may tighten to zero targets when bridge is gone.
3. **Verification** records ledger row movement toward **`bridge_retirement_ledger_zero`** in cadence with PB + Verification Manager.

## Dispatch triggers (mechanical)

Pick up implementation **only when** a **single live-state check** shows **at least one** path is real on main:

| # | Trigger | Authority |
|---|---------|-----------|
| T1 | **PB-Runtime interpreter-as-data** ( [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) Item 4 ) reaches a milestone where lens application routes by **typed `DeclarationRef` identity** — dissolving **B** and **C** — *or* | PB + Evaluator program |
| T2 | **Typed lens-registry / cross-`Dag` `DeclarationRef` substrate** lands per §P1 — dissolving **A** without a new string registry bridge — *or* | Substrate Manager |
| T3 | **Pair-authored Verification ledger note** — closure row movement coordinated with Verification Manager | PB + Verification |

Until **T1 or T2** is true, this brief stays **PROPOSAL / queued** — PB may maintain **disposition + ratchet** artifacts only; **no** fake structural resolution.

## STOP conditions

- **String/path registry substitute** for canonical lens bytes **without** T2 — **STOP** (disposition §"Either path is a substrate-level change").
- **Dropping P2 reflection invariant** to delete name dispatch — **STOP**; escalate emission-model / Evaluator if a new reconciliation mechanism is required.
- **Broadened scope** ("delete all lens-name checks") without per-site ledger mapping — **STOP**.

## Non-goals

- Retiring **`bridge_include_str_side_channels_retired`** (bridge #4 — [`r3-pb-bridge-include-str-side-channels-closure.md`](r3-pb-bridge-include-str-side-channels-closure.md)).
- **T-LensProducer-Retirement** sub-gates beyond what Item 4 + Item 5 already own — this brief **consumes** their convergence when triggers align.

## Cross-refs

- Disposition + ratchet authority: [`r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md).
- B4 consumer migration STOP (registry escalation): [`docs/briefs/b4-1-declarationref-consumer-migration-worker.md`](b4-1-declarationref-consumer-migration-worker.md).
- PB Manager acceptance row: [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §`bridge_canonical_lens_name_dispatch_retired`.
- Verification ledger: [`r3-verification-manager.md`](r3-verification-manager.md) bridge table row #3.
