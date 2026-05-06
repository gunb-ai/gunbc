# R3 T-Lens-Application-Surface — execution split (substrate vs demonstration)

**Status:** **PRE-AUTH DISPATCH-READY** — separates **carrier / routing execution** from **worked-example demonstration execution** for lane **T-Lens-Application-Surface** (tier-1 queue **#1859**). Cross-program: **Substrate Manager + Verification Manager** per `r3-structure.md` §"Lane structure".

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md).

**Design lock:** [`docs/design-lens-application-surface.md`](../design-lens-application-surface.md).

**Cascade / carve (INVARIANTS §P2):** Internal LAS cascade in [`docs/design-lens-application-surface.md`](../design-lens-application-surface.md) §**7** / §**9** requires each worked example’s lens to be **behaviorally substantive**. Under Director **option (b)** + [`docs/r4-carve-out-routing.md`](../r4-carve-out-routing.md) **C1**, **`parallelism_lens_behaviorally_complete`** is **R4-carved**, and **`opt_in_iteration_parallelism_via_lens_application_demonstrated` (#95) carves with it** — **no #95 PASSING in R3** (demo §4.4 needs parallelism lens parity per design §4.4). **R3** executes substrate **88–91** + demos **92–94** only after **complexity+cost** T-LBP completeness + register **C3**; **#95** schedules against **R4** parallelism landing.

## Execution slice A — substrate-shape + routing (Substrate-primary)

| Plan §1.8 # | Gate ID | Pass intent |
| --- | --- | --- |
| 88 | `lens_application_carrier_landed` | `EnforcedApplication<Output, Budget>` + `IntrospectApplication<Output>` |
| 89 | `section_ref_substrate_landed` | `SectionRef` disjoint sum |
| 90 | `lens_enforcement_carrier_landed` | Per-lens `LensEnforcement<Output, Budget>` projection |
| 91 | `enforce_violation_routing_landed` | Enforce-mode routing through `DiagnosticSeverity` (design §3 + **INVARIANTS** C-8) |

**Verification partner:** conformance tests + `TestClaim` shells only after carriers exist — **does not** author top-level carriers (**§P1**).

## Execution slice B — demonstrations (cross-program)

### B1 — **R3** (after complexity+cost T-LBP + slice **A**)

| Plan §1.8 # | Gate ID | Pass intent |
| --- | --- | --- |
| 92 | `complexity_violation_compile_error_demonstrated` | `apply_lens(complexity, …, Enforce)` compile-error worked example |
| 93 | `crdt_cost_basis_demonstrated` | CRDT cost basis via `apply_lens` |
| 94 | `memory_peak_cost_basis_demonstrated` | Memory-peak cost basis |

### B2 — **R4 (C1)** — blocked on `parallelism_lens_behaviorally_complete`

| Plan §1.8 # | Gate ID | Pass intent |
| --- | --- | --- |
| 95 | `opt_in_iteration_parallelism_via_lens_application_demonstrated` | Iteration-independence opt-in via lens application (**design** §4.4 — requires **parallelism** lens substantive semantics; **carves with C1** per `r4-carve-out-routing.md`) |

**Verification-owned:** integration `TestClaim`s, corpus fixtures, diagnostic assertions by shape ([`TESTING.md`](../../TESTING.md)).

## Dependencies

| ID | Dependency | Notes |
| --- | --- | --- |
| A1 | Slice **A** gates green | Hard prerequisite for meaningful Slice **B** demos |
| A2 | Evaluator + lens runtime | Lane row R2-Evaluator gating |
| A3 | Class 2 gap-test narrative | `substrate_gap_function_valued_data_closed` traces through LAS + T-E-P per plan §1.8 — keep receipts aligned (**#828** chain-break discipline) |
| A4 | **`parallelism_lens_behaviorally_complete` (R4 C1)** | **Gate #95 only** — demos **92–94** do **not** wait on parallelism parity |

## Dispatch triggers

1. Substrate signals **88–91** consumer-ready.
2. Verification schedules demos **92–94** when **complexity+cost** T-LBP + slice **A** clear; schedules **#95** only after **R4** parallelism parity (**C1**) — **no parallel authority**.
3. Escalate substrate-shape questions → **[#1739](https://github.com/gunb-ai/gunbc/issues/1739)**; Verification inbox **[#1740](https://github.com/gunb-ai/gunbc/issues/1740)**.

## STOP+PING

| Item | Discipline |
| --- | --- |
| Demonstrations **before** enforce routing landed | **STOP+PING** — fail-closed semantics undefined |
| Claiming **T-LBP COMPLETE** for carved lenses | **STOP+PING** — **C1/C2** remain R4 |
| **`#95` PASSING under R3 thesis close** | **STOP+PING** — **R4 (C1)** per `r4-carve-out-routing.md` + design §7 |

## Cross-refs

- T-LBP partner (producer + cementing alignment): [`r3-v-t-lbp-narrowed-scope-partner-worker.md`](r3-v-t-lbp-narrowed-scope-partner-worker.md)
- Self-application lane consumer: [`docs/r3-structure.md`](../r3-structure.md) §"Acceptance" T-Lens-Self-Application (uses `EnforcedApplication` timing example)
- Substrate gap class row **#174**: [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8
