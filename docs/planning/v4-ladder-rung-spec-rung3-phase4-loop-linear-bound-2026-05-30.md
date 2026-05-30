# v4 ladder rung spec — Phase 4 (`loop_linear_bound` × rung 3)

> **Status:** DRAFT — Ladder/Fixture worker (`royal-wolf-898`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2; joint runner spec [`compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md`](compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md) §3 row "rung 3" + §4.4 rung-split.
> **Companion specs (gate vocabulary — do not duplicate):** [`v4-ladder-rung-specs-2026-05-30.md`](v4-ladder-rung-specs-2026-05-30.md) §2; [`v4-ladder-rung-spec-rung3-2026-05-30.md`](v4-ladder-rung-spec-rung3-2026-05-30.md) (PR #4003). **This file is additive**: fixture-particular binding only.
> **Line-number authority:** `origin/main` at `f7f629a30` (2026-05-30, post-#4018). Re-verify if `main` advances.
> **Scope:** Acceptance predicate for **rung 3 only** on `phase4/loop_linear_bound`. Rungs 0–2: [`src/v4/test/claim/loop_linear_bound/rung_0_to_2_three_targets.dag`](../src/v4/test/claim/loop_linear_bound/rung_0_to_2_three_targets.dag). Rung 4: separate PR. Rungs 5–9: Phase 3+.

---

## 1. Scope binding (fixture-particular)

| Field | Value |
| ----- | ----- |
| **Rung** | 3 — round-trip preserved (`.dag` ingest⁻¹) |
| **Phase** | 4 widening (per [`v4-correctness-ladder-2026-05-30.md`](v4-correctness-ladder-2026-05-30.md) §7 Phase 4) |
| **Ladder fixture** | `phase4/loop_linear_bound` (landed #4018) |
| **Fixture module** | `src/v4/test/claim/computation_shapes/loop_linear_bound.dag` |
| **Canonical subject** | `loop_linear_bound_subject` — `ComputationNode { behavior: Loop }` with positional `Transform` body + `Named { name: loop_bound_edge }` measure leaf (same edge discipline as `lens_cost/loop_linear.dag`) |
| **Rung 0–2 roster** | `src/v4/test/claim/loop_linear_bound/rung_0_to_2_three_targets.dag` |
| **Fixture claims (corpus)** | `claim_loop_linear_bound_well_formed`, `claim_loop_linear_bound_equals_refl`, `claim_loop_linear_bound_content_hash_stable` |
| **W1 target** | `dag` only (rung-3 companion §2.1) |
| **Executable W1 round-trip (aux)** | `phase1-aux/dag_round_trip_mvp1` (rung-3 companion §2.4 — diagnostic, not ladder close) |
| **Landed substrate** | Same as rung-3 companion §1 |

**Shape rationale:** Exercises **Loop** + `loop_bound_edge` emit — Phase 4 widening alongside `phase4/branch_dispatch` (#4018).

**W1 / W1b honesty:** Same staging as rung-3 companion §1 (W1 landed; W1b T-36 pending).

---

## 2. Rung 3 acceptance predicate

**Apply companion specs verbatim** for predicates, prerequisites, aux row, and wiring.

**Fixture-specific substitutions:**

| Companion placeholder | This fixture |
| --------------------- | ------------ |
| `phase1/nat_semiring` | `phase4/loop_linear_bound` |
| Module path | `src/v4/test/claim/computation_shapes/loop_linear_bound.dag` |
| RoundTripClaim wedge receipt | `upstream_blocked:claim_loop_linear_bound_module_roundtrip_not_authored` |
| Blocking receipt prefix | `phase4/loop_linear_bound/rung3/…` |
| Ladder `RoundTripClaim` row (TODO) | `src/v4/test/claim/loop_linear_bound/rung_3_*.dag` (NEW — binds `loop_linear_bound_subject` after module-loader) |

**Prerequisite wedge:** No `RoundTripClaim` row on this module post-#4018 → `dag=SKIP`, row `FAIL`, blocking `upstream_blocked:claim_loop_linear_bound_module_roundtrip_not_authored`.

---

## 3. Verdict reporting shape (fixture row)

```text
fixture=phase4/loop_linear_bound
  rung0: FAIL  (dag=… rust=… python=… go=…)
  rung1: FAIL  (rust=…)
  rung2: FAIL  (rust=… python=… go=…)
  rung3: FAIL  (dag=SKIP)
blocking_receipt: upstream_blocked:claim_loop_linear_bound_module_roundtrip_not_authored

fixture=phase1-aux/dag_round_trip_mvp1
  rung3: FAIL  (dag=PASS)
blocking_receipt: phase1-aux/dag_round_trip_mvp1/rung3/dag_roundtrip_fidelity_w1b_unlanded
```

---

## 4. Worker brief triple

```text
fixture=phase4/loop_linear_bound
rung=3
modeling_gap=none (claim authoring) | T-36 only with Modeling DFS worksheet (W1b)
predicate=R3-dag-roundtrip-wave1-ready expected to flip SKIP → PASS|FAIL on phase4/loop_linear_bound
```

---

## 5. Current baseline (post-#4018 main)

| Field | Value |
| ----- | ----- |
| Landed fixture | #4018 |
| `RoundTripClaim` on fixture | **Absent** |
| Expected rung 3 | `FAIL` (`dag=SKIP`), `upstream_blocked:claim_loop_linear_bound_module_roundtrip_not_authored` |

---

## 6. Sign-off (worker)

| Decision | Disposition |
| -------- | ----------- |
| Rung 3 binding on `phase4/loop_linear_bound` | **PROPOSED** |
| Predicate vocabulary | **By reference** — rung-3 companion + ladder specs §2 |
