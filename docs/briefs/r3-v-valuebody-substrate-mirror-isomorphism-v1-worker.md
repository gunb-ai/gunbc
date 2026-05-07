# R3 ValueBody — Rust↔substrate `.dag` mirror isomorphism Worker Brief

**Status:** **PRE-AUTH DISPATCH-READY** — brief authored for **Q-ValueBody-Isomorphism** consumer path (`docs/r3-program-plan.md` §10.3). **No strict-fire Implementation dispatch** until §Dependencies clear **and** Director row moves off **OPEN** (scoping: R3-close gate vs explicit post-R3 carry).

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md).

**Naming / disambiguation:**
- **Program plan canvas** — Verification Mgr escalation **item 6** (`docs/r3-program-plan.md` §10.2.1 **V6. ValueBody Rust↔.dag mirror drift / missing isomorphism gate**).
- **Design schedule** — [`docs/r3-design-schedule-2026-05-06.md`](../r3-design-schedule-2026-05-06.md) §**V7 — ValueBody isomorphism gate design** (distinct from schedule §2 **V6** = `bridge_retirement_ledger_zero` audit cadence — see [`r3-v-bridge-retirement-ledger-zero-audit.md`](r3-v-bridge-retirement-ledger-zero-audit.md)).

**Evidence anchor (drift fact):** [`ROADMAP.md`](../../ROADMAP.md) — post-merge debt row **`ValueBody` Rust↔.dag mirror drift; no isomorphism gate** (live Rust `ValueBody` in `src/v3/compiler/src/dag.rs` vs `.dag` mirror in `src/v3/std/substrate.dag`; five runtime variants vs three mirror constructors at HEAD when row was authored).

**Feedback discipline:** `feedback_isomorphism_or_generation_for_mirrors` — **generation-or-isomorphism** test, not unbounded hand-maintained dual taxonomies (**INVARIANTS** §P1); same citation pattern as [`r3-v-pattern-a-rust-dag-isomorphism-v1-worker.md`](r3-v-pattern-a-rust-dag-isomorphism-v1-worker.md) §Scope (out).

## Closure predicate (working name — pending §1.8 enumeration)

| Working gate name | Target transition |
| --- | --- |
| `value_body_substrate_mirror_isomorphism_executable` | **Unlisted → DECLARED** in `r3-structure.md` §"Acceptance" / plan §1.8 **only after** Q-ValueBody-Isomorphism ratifies R3 inclusion and Director assigns canonical gate **#** — until then, treat as **pre-declaration** worker scope only (**INVARIANTS** §P2). |
| **CONSUMER_LANDED** | CI-visible conformance: build-time enum walk, boot-time structural check, or `.dag` `TestClaim` that fails closed on variant/shape skew (exact mechanism ratified with Substrate Mgr). |
| **PASSING** | Consumer green on `main` after mirror parity or single-authority generation retires the drift class. |

## Worker pin (cross-program)

| Preference | Worker | Condition |
| --- | --- | --- |
| **Primary (substrate)** | **quick-crab-830** ([gunbc#1739](https://github.com/gunb-ai/gunbc/issues/1739)) | Carrier / mirror schema / regen contract — **§P1** substrate introduction |
| **Primary (verification harness)** | **bold-crane-790** ([gunbc#1748](https://github.com/gunb-ai/gunbc/issues/1748)) | When bundled with Track A Pattern-A / ledger receipts; else Verification lands harness-only PR after substrate shape exists |
| **Alternate** | **New worker** | Partition per `feedback_idle_workers_dispatchable_directly` |

## Scope (in)

- **Conformance surface** — one mechanical gate that **Rust `ValueBody`**, **substrate `.dag` mirror**, and (if applicable) **regen output** cannot diverge silently (ROADMAP failure mode).
- **Verification-owned** — integration test / `TestClaim` / `build.rs` check **spec** once Substrate picks generation vs mirror-completion; **does not** invent long-lived parallel `.dag` type definitions (**INVARIANTS** §P1).
- **Finite representative witnesses** — start with bootstrap-shaped carriers already stressed by `lens_apply` / `regen_bootstrap_emit` matches (per ROADMAP evidence list), expand only with Director-visible census.

## Scope (out) — STOP+PING

| Item | Discipline |
| --- | --- |
| **Hand-editing `substrate.dag` mirrors to “match” Rust without a gate** | **STOP+PING** — reintroduces silent drift |
| **`TestPredicate` variant owned in Verification PRs** | **STOP+PING** — Substrate owns predicate/carrier introduction |
| **Claiming Pattern-A TC1/TC2/TC3 closure** | **STOP+PING** — orthogonal gates (**#11–#13**); coordinate only if a single substrate PR batches shared reflection infrastructure |

## Dependencies (hard)

| ID | Dependency | Owner | Notes |
| --- | --- | --- | --- |
| D0 | **Q-ValueBody-Isomorphism** row **not OPEN** or Director **ENGAGE** slice scoped | Director + Substrate + Brian | Plan §10.3 — R3 vs post-R3 carry |
| D1 | Mirror **completion** (all Rust variants representable in `.dag`) **or** **generation** from single authority | Substrate | Eliminates dual manual taxonomies |
| D2 | Hook point for conformance (build script, test harness, or runner) agreed with compiler crate boundaries | Substrate + Verification | Fail-closed on skew |
| D3 | `FieldValue` / nested carriers — if ValueBody fix cascades | Substrate | ROADMAP row may expand; keep one gate receipt |

## Dispatch triggers (mechanical)

1. **D0** — Director-visible disposition on Q-ValueBody-Isomorphism (full R3 gate vs phased carry).
2. **D1 + D2** — Substrate receipt on **#1739** (PR or comment) naming the chosen dissolution (mirror parity vs codegen).
3. **Worker available** — quick-crab for substrate slice; bold-crane or substitute for harness PR.
4. **Plan §1.8** — Verification Mgr (or Director PR) lands canonical gate id + row when predicate joins closure ledger.

## Implementation slices (suggested PR shape)

1. **Slice 1 — substrate:** `.dag` mirror + lowering/regen alignment **or** generated mirror body — **no** Verification-only “fix” without D2 hook.
2. **Slice 2 — conformance consumer:** `value_body_substrate_mirror_isomorphism_executable` (working name) **CONSUMER_LANDED** — fails CI on drift.
3. **Slice 3 — ledger / plan:** §1.8 status + ROADMAP row retirement / downgrade when gate **PASSING**; cross-link [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) only if TC rows cite ValueBody prerequisites.

## Cross-refs

- Plan Q row: [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3 **Q-ValueBody-Isomorphism**
- Schedule V7: [`docs/r3-design-schedule-2026-05-06.md`](../r3-design-schedule-2026-05-06.md) §**V7**
- Pattern-A neighbors (different predicate family): [`r3-v-pattern-a-rust-dag-isomorphism-v1-worker.md`](r3-v-pattern-a-rust-dag-isomorphism-v1-worker.md) (structural Dag iso — **not** the same gate)
- Substrate mirror program: `src/v3/std/substrate.dag` (**ValueBody**\* symbols)
- Live carrier: `src/v3/compiler/src/dag.rs` (**`ValueBody`** enum)
