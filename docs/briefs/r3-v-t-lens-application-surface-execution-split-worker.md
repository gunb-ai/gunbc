# R3 T-Lens-Application-Surface — execution split (substrate vs demonstration)

**Status:** **PRE-AUTH DISPATCH-READY** — separates **carrier / routing execution** from **worked-example demonstration execution** for lane **T-Lens-Application-Surface** (tier-1 queue **#1859**). Cross-program: **Substrate Manager + Verification Manager** per `r3-structure.md` §"Lane structure".

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md).

**Design lock:** [`docs/design-lens-application-surface.md`](../design-lens-application-surface.md).

**Cascade note:** Lane is **Evaluator-gated** and listed **after** T-LBP in the R3 critical-path narrative; **T-LBP option (b)** narrows behavioral-complete work to **complexity + cost** in R3 — parallelism / effect_enum behavioral completeness are **R4-carved**. Demonstration gate **#95** still names `Lens<Iteration-Independence>` **as a lens-application worked example**; execution may proceed **without** waiting for R4 parallelism **lens behavioral completeness** if substrate + evaluator can exercise the **apply_lens** surface per design doc (coordinate Substrate — **no false lane-closure** claim).

## Execution slice A — substrate-shape + routing (Substrate-primary)

| Plan §1.8 # | Gate ID | Pass intent |
| --- | --- | --- |
| 88 | `lens_application_carrier_landed` | `EnforcedApplication<Output, Budget>` + `IntrospectApplication<Output>` |
| 89 | `section_ref_substrate_landed` | `SectionRef` disjoint sum |
| 90 | `lens_enforcement_carrier_landed` | Per-lens `LensEnforcement<Output, Budget>` projection |
| 91 | `enforce_violation_routing_landed` | Enforce-mode routing through `DiagnosticSeverity` (design §3 + **INVARIANTS** C-8) |

**Verification partner:** conformance tests + `TestClaim` shells only after carriers exist — **does not** author top-level carriers (**§P1**).

## Execution slice B — demonstrations (cross-program)

| Plan §1.8 # | Gate ID | Pass intent |
| --- | --- | --- |
| 92 | `complexity_violation_compile_error_demonstrated` | `apply_lens(complexity, …, Enforce)` compile-error worked example |
| 93 | `crdt_cost_basis_demonstrated` | CRDT cost basis via `apply_lens` |
| 94 | `memory_peak_cost_basis_demonstrated` | Memory-peak cost basis |
| 95 | `opt_in_iteration_parallelism_via_lens_application_demonstrated` | Iteration-independence opt-in via lens application |

**Verification-owned:** integration `TestClaim`s, corpus fixtures, diagnostic assertions by shape ([`TESTING.md`](../../TESTING.md)).

## Dependencies

| ID | Dependency | Notes |
| --- | --- | --- |
| A1 | Slice **A** gates green | Hard prerequisite for meaningful Slice **B** demos |
| A2 | Evaluator + lens runtime | Lane row R2-Evaluator gating |
| A3 | Class 2 gap-test narrative | `substrate_gap_function_valued_data_closed` traces through LAS + T-E-P per plan §1.8 — keep receipts aligned (**#828** chain-break discipline) |

## Dispatch triggers

1. Substrate signals **88–91** consumer-ready.
2. Verification schedules **92–95** PRs with finite representative programs (no parallel authority).
3. Escalate substrate-shape questions → **[#1739](https://github.com/gunb-ai/gunbc/issues/1739)**; Verification inbox **[#1740](https://github.com/gunb-ai/gunbc/issues/1740)**.

## STOP+PING

| Item | Discipline |
| --- | --- |
| Demonstrations **before** enforce routing landed | **STOP+PING** — fail-closed semantics undefined |
| Claiming **T-LBP COMPLETE** for carved lenses | **STOP+PING** — **C1/C2** remain R4 |

## Cross-refs

- T-LBP partner (producer + cementing alignment): [`r3-v-t-lbp-narrowed-scope-partner-worker.md`](r3-v-t-lbp-narrowed-scope-partner-worker.md)
- Self-application lane consumer: [`docs/r3-structure.md`](../r3-structure.md) §"Acceptance" T-Lens-Self-Application (uses `EnforcedApplication` timing example)
- Substrate gap class row **#174**: [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8
