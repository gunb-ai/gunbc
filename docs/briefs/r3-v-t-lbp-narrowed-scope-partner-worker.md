# R3 T-Lens-Behavioral-Parity — Verification partner brief (option **(b)** narrow scope)

**Status:** **PRE-AUTH DISPATCH-READY** — Verification-side cross-program partner after Director ratification **Q-Lens-Behavioral-Parity-R3-Closeability option (b)** (**#828**). Substrate owns S2 canvas + substrate folds; **this brief** owns Verification receipts that gate **cementing**, **demonstration**, and **register alignment** from the Verification lane.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md).

**Substrate canvas (authority for blocker matrix):** [`docs/briefs/r3-substrate-s2-t-lbp-scope-calibration-canvas.md`](r3-substrate-s2-t-lbp-scope-calibration-canvas.md).

**AMENDED 2026-05-09 — full T-LBP scope IN R3 per Director carve-promotion-IN-R3 ratification at gunbc#846 #issuecomment-4412330468 + (a) at #issuecomment-4412380947**: all 4 lenses (complexity + cost + parallelism + effect_enum) R3-load-bearing within Cluster F per [`docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-f-sequencing-plan-2026-05-09.md). Prior C1/C2/C3 carves DISSOLVED. Register gate **#83** fires for ALL 4 in-R3 lenses (prior narrowed scope DISSOLVED).

**Single authority (INVARIANTS §P2):** Option **(b)** obligation text is **`docs/r3-structure.md`** §"Acceptance" T-Lens-Behavioral-Parity **plus** the matching Summary lane bullet (item **14**) **plus** the §"Lane structure" table row — updated in lockstep. **This brief** elaborates Verification receipts only; it does **not** define lane scope independently of `r3-structure.md`.

**Lane acceptance:** [`docs/r3-structure.md`](../r3-structure.md) §"Acceptance" T-Lens-Behavioral-Parity — gates **#79–#83** in [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8.

## In-R3 obligations (Verification-owned or cross-program)

| Plan gate # | Gate ID | Verification partner role |
| --- | --- | --- |
| 79 | `complexity_lens_behaviorally_complete` | Cementing tests + `ClaimResult` diagnostics; consumes T-E-P producer evidence; aligns witness shapes with [`r3-v-witness-shape-pattern-survey.md`](r3-v-witness-shape-pattern-survey.md) |
| 80 | `cost_lens_behaviorally_complete` | Same — shared producer dependency as complexity (S2 matrix lens 2) |
| 73 | `lens_behavioral_parity_demonstration` | Per **in-R3** lens demo (complexity, cost) matching **frozen** v2-oracle snapshot (plan §1.6 — **not** live v2 consumer); carved lenses **R4** |
| 83 | `lens_capability_register_zero_proxy_zero_stub` | Receipt that register lists **ZERO PROXY / ZERO STUB** for **ALL 4 in-R3 lenses** (complexity + cost + parallelism + effect_enum) per Director carve-promotion ratification 2026-05-09 c#4412330468; prior narrowed-scope C3 framing DISSOLVED |

## Out of R3 (partner discipline)

| Gate ID | Discipline |
| --- | --- |
| `parallelism_lens_behaviorally_complete` | **STOP+PING for R3 closure** — carved **C1**; no Verification PASSING receipt pretending R3 owns this slice |
| `effect_enumeration_lens_behaviorally_complete` | **STOP+PING for R3 closure** — carved **C2** |

## Dependencies

| ID | Dependency | Owner |
| --- | --- | --- |
| L1 | T-E-P Phase 1 producer coverage (`e_p_*` gates) | Substrate / Evaluator |
| L2 | Behavioral lens folds for complexity + cost | Substrate |
| L3 | Frozen snapshot capture **before** v2 retirement | Cross-program (PB + Verification timing) |
| L4 | Capability register updates for narrowed scope | Substrate + Verification audit |

## Dispatch triggers

1. **L1** unblocks complexity/cost producer consumption (S2 matrix lens 1–2).
2. **L3** plan locked — cementing demos won't reattach live v2 test consumers.
3. **Cross-program PR** pattern: Substrate lands lens fold + register rows; Verification lands cementing integration + demonstration `TestClaim` receipts.

## STOP+PING

| Item | Discipline |
| --- | --- |
| Expanding T-LBP back to 4 lenses inside R3 | **STOP+PING** — requires Director revision of option **(b)** |
| Cementing without frozen snapshot discipline | **STOP+PING** — conflicts with `v2_oracle_no_remaining_test_consumers` closure narrative |

## Cross-refs

- Tests-as-data lane (cementing gate D): [`r3-v-tests-as-data-v1-worker.md`](r3-v-tests-as-data-v1-worker.md)
- Free-consequences / witness overlap: [`r3-v-free-consequences-worker.md`](r3-v-free-consequences-worker.md)
- Program plan §10.3 Q-LBP row: [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3
