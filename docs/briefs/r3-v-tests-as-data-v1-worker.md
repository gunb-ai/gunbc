# R3 T-Tests-As-Data-Completeness — unified dispatch-ready worker brief (V4)

**Status:** **PRE-AUTH DISPATCH-READY** — consolidates lane closure gates into one worker-facing dispatch artifact (tier-1 queue **#1859**). **No substrate edits** in this brief; carrier introduction stays **§P1** Substrate-owned.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md).

**Lane authority:** [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" + §"Acceptance" — **T-Tests-As-Data-Completeness**.

**Readiness audit (gap vs HEAD):** [`docs/briefs/r3-v-tests-as-data-completeness-readiness-audit.md`](r3-v-tests-as-data-completeness-readiness-audit.md) — census counts, gate A–D snapshots; **this brief** is the **dispatch overlay** (triggers, slices, STOP+PING).

**Design lock (read-only):** [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md).

## Closure gates (lane — single worker coordinates)

| Gate ID | Canonical name | Role |
| --- | --- | --- |
| — | `every_rust_test_ports_to_dag_or_generated` | Facet-3 close: hand-authored Rust test partition → **0** per design §1.1 |
| — | `forall_exists_quantifier_substrate_landed` | Mathematical `ForAll` / `Exists` over program families (**not** `ForAllTargets` emission shim) |
| — | `program_generator_carrier_landed` | `ProgramGenerator` / quantified claim substrate |
| — | `lens_cementing_test_discipline_complete` | Every **in-R3** `.dag` lens with behavioral-complete intent has cementing module vs frozen v2 oracle |

**Demonstration sibling (plan §1.8):** `tests_as_data_demonstration` — at least one Rust test ports to `.dag` `TestClaim` and executes (early signal; **not** a substitute for gate row closure).

## Worker pin

| Preference | Worker | Condition |
| --- | --- | --- |
| **Primary** | **bold-crane-790** (**#1748**) or **cool-heron-521** when SB slice active | R2-Evaluator + TestClaim runtime precondition per lane row |
| **Alternate** | **New worker** | Partition per `feedback_idle_workers_dispatchable_directly` |

## Scope (in)

- **SG-0 census honesty** — [`sg0_census_test.rs`](../../src/v3/compiler/tests/integration/sg0_census_test.rs) list stays reconciled to tree (readiness audit §2).
- **Migration slices 2–5** from readiness audit §6 — predicate-class mapping → generated target tests → quantifiers → facet-3 zero census.
- **Cross-lane receipts** — Lane 1↔2 import contract ([`r3-v-lane1-lane2-corpus-identity-import-spec.md`](r3-v-lane1-lane2-corpus-identity-import-spec.md)); cementing alignment with **T-LBP** narrow scope ([`r3-v-t-lbp-narrowed-scope-partner-worker.md`](r3-v-t-lbp-narrowed-scope-partner-worker.md)).

## Scope (out) — STOP+PING

| Item | Discipline |
| --- | --- |
| **Inventing `ProgramGenerator` / quantifier variants in Verification PRs** | **STOP+PING** — §P1 substrate introduction only |
| **Dropping census entries without port or explicit carve-out** | **STOP+PING** — debt visibility discipline |
| **Claiming lens cementing COMPLETE while T-LBP rows PROXY/STUB for in-scope lenses** | **STOP+PING** — register authority [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md) |

## Dependencies (hard)

| ID | Dependency | Owner |
| --- | --- | --- |
| T1 | R2-Evaluator test execution + `TestClaim` runner stable | Evaluator |
| T2 | Emission path for `TestPredicate` → generated target tests (design §1.3 Path B) | Substrate + Verification |
| T3 | Frozen v2-oracle snapshot infrastructure for cementing | Cross-program (see T-LBP partner brief) |
| T4 | Quantifier + generator carriers | Substrate (**§P1**) |

## Dispatch triggers

1. **T1** green — continuation **[#1743](https://github.com/gunb-ai/gunbc/issues/1743)** receipts.
2. **T2** slice scoped — first port lands **#1276** / Verification inbox signal as appropriate.
3. **Sub-issue** under Verification inbox **#1740** (or bold-crane **#1748** when Track A) + Director workflow.

## Implementation slices (suggested PR sequence)

1. **V4-a — Census + mapping:** refresh readiness audit §1; extend predicate coverage map (audit §6 slice 2).
2. **V4-b — First executable port:** satisfy `tests_as_data_demonstration` + shrink SG-0 net (audit §6 slice 3).
3. **V4-c — Quantifiers + generator:** land when **T4** clears (audit §6 slice 4).
4. **V4-d — Facet-3 close:** census **0** + cementing discipline satisfied (audit §6 slice 5).

## Cross-refs

- Slice-1 census: [`r3-v-tests-as-data-slice1-census-reconciliation.md`](r3-v-tests-as-data-slice1-census-reconciliation.md)
- Witness patterns: [`r3-v-witness-shape-pattern-survey.md`](r3-v-witness-shape-pattern-survey.md)
- TESTING / DB-15: [`TESTING.md`](../../TESTING.md), [`docs/design-test-infra.md`](../design-test-infra.md)
