# R3 Pattern-A — Rust Dag isomorphism first executable slice Worker Brief

**Status:** **PRE-AUTH DISPATCH-READY** — brief authored ahead of runtime triggers (pre-auth queue **#1859**). **No strict-fire Implementation dispatch** until §Dependencies clear.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — Pattern-A cluster under Lane 1 (`docs/r3-structure.md` §"Acceptance" T-V-L4-L7-Direct).

**Research / producer framing:** [`docs/briefs/r3-v-reflected-dag-structural-assertion-analysis.md`](r3-v-reflected-dag-structural-assertion-analysis.md) — Director producer-first disposition (**#828**): Substrate-owned `Lens<DagShapeReport>` (or equivalent) projects reflected / emitted Dag into a structural **shape report**; **`BinaryDimensionReportEquals`** with the **shape-report modifier** is the comparison shell — **no** parallel predicate authority (**INVARIANTS** §P1).

**Program plan (single operational authority):** [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3 **Q-PAFS** — Path **C** (RustDagIsomorphism before TC1) is **not** the default policy ordering; **this brief** only elaborates **gate #14** per §1.8 and `r3-structure.md` §"Acceptance". **Does not** override §10.3 (**INVARIANTS** §P2).

## §1.8 closure predicate (this slice)

| Gate ID | Gate name (canonical) | Target transition |
| --- | --- | --- |
| **#14** | `rust_dag_isomorphism_executable` | **DECLARED → CONSUMER_LANDED** when executable `TestClaim` + runner path land per this brief; **PASSING** when strict-fire evaluates green on CI. |

Canonical Pass body (archive authority): `r3-structure.md` §"Acceptance" — structural Dag equivalence between Rust-emitted / reflected Dag authority and `.dag` source Dag. **Engineering route:** two comparable **shape-report** carriers feeding the unified binary predicate (research brief §"Producer Plus Comparison Shell"), unless Director revises the acceptance prose.

## Worker pin (Verification Mgr partition)

| Preference | Worker | Condition |
| --- | --- | --- |
| **Primary** | **bold-crane-790** ([gunbc#1748](https://github.com/gunb-ai/gunbc/issues/1748)) | Same Track A pin as TC2/TC3 when §Dependencies + staffing allow. |
| **Alternate** | **New worker** | If bold-crane saturated — substitute per `feedback_idle_workers_dispatchable_directly`. |

## Scope (in)

- Executable **gate #14** slice: `TestClaim` + runner wiring once Substrate shape-report producer + unified predicate modifier exist.
- **Representative programs:** finite, named set (bootstrap mirrors / lockstep schema checks called out in research §"Migration Audit" — start with **strong** fits such as carrier-shape tests before anthropic cross-source schema).
- Verification-owned: integration receipts, fixture naming, strict-fire diagnostics **by shape** ([`TESTING.md`](../../TESTING.md)).

## Scope (out) — STOP+PING

| Item | Discipline |
| --- | --- |
| **Parallel `RustDagIsomorphism` predicate family** | **STOP+PING** — consumer instance on unified shell only (research §"RustDagIsomorphism Adjacency"). |
| **Hand-maintained Rust↔.dag mirrors** | **STOP+PING** — generation-or-isomorphism discipline (`feedback_isomorphism_or_generation_for_mirrors`); Substrate owns producer evolution. |
| **Folding TC1 η / TC2 / TC3 into this PR** | **STOP+PING** — separate gates **#11–#13**; coordinate only if a single substrate PR batches shared predicate evolution (then sequence only). |

## Dependencies (hard)

| ID | Dependency | Owner | Notes |
| --- | --- | --- | --- |
| R1 | **`Lens<DagShapeReport>`** (or equivalent structural projection) | Substrate | Producer-first #828 ratification |
| R2 | **Shape-report role** on unified `BinaryDimensionReportEquals` | Substrate | Modifier lands via §P1 |
| R3 | **Reflection / compile-to-dag** path stable enough to extract both sides of comparison | Substrate + Evaluator | parity with research §"Reflection-Completeness Residual" awareness |
| R4 | **Coverage-shape ratification** (which declarations / rows are in-scope for v1 strict-fire) | Director + Verification | named finite harness before PASSING |

## Dispatch triggers (mechanical)

1. **R1 + R2** land — receipts linked from Substrate inbox **[#1739](https://github.com/gunb-ai/gunbc/issues/1739)** / Evaluator **[#1743](https://github.com/gunb-ai/gunbc/issues/1743)** as appropriate.
2. **R4** named — Director-visible finite representative set.
3. **Worker available** — bold-crane (or substitute).
4. **Sub-issue** under **#1748** + `addSubIssue` + inbox pointer.

## Implementation slices (suggested PR shape)

1. **Slice 1 — substrate receipt:** shape-report producer + predicate modifier green on representative input (no ledger PASSING claim until Slice 2).
2. **Slice 2 — executable `TestClaim`:** `rust_dag_isomorphism_executable` integration **Pass**.
3. **Slice 3 — ledger / doc:** §1.8 status + cross-link [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) if bundle row is touched.

**Substrate lands first** on any new carrier — Verification does not invent `DagShapeReport` schema.

## Cross-refs

- Research: [`r3-v-reflected-dag-structural-assertion-analysis.md`](r3-v-reflected-dag-structural-assertion-analysis.md)
- TC1 neighbor (different gate / Branch B hold): [`r3-v-pattern-a-tc1-v1-worker.md`](r3-v-pattern-a-tc1-v1-worker.md)
- TC2 / TC3 neighbors: [`r3-v-pattern-a-tc2-v1-worker.md`](r3-v-pattern-a-tc2-v1-worker.md), [`r3-v-pattern-a-tc3-v1-worker.md`](r3-v-pattern-a-tc3-v1-worker.md)
- Plan §2.1 Pattern-A cluster: [`docs/r3-program-plan.md`](../r3-program-plan.md) §"§2.1 Pattern A executable"
