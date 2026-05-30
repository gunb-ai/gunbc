# v4 ladder rung spec — Phase 4 (`branch_dispatch` × rung 3)

> **Status:** DRAFT — Ladder/Fixture worker (`royal-wolf-898`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2; joint runner spec [`compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md`](compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md) §3 row "rung 3" + §4.4 rung-split.
> **Companion specs (gate vocabulary — do not duplicate):** [`v4-ladder-rung-specs-2026-05-30.md`](v4-ladder-rung-specs-2026-05-30.md) §2 (cell vocabulary, prerequisites, headlines); [`v4-ladder-rung-spec-rung3-2026-05-30.md`](v4-ladder-rung-spec-rung3-2026-05-30.md) (landed via PR #4003 — W1/W1b staged predicates, aux-row convention, P2 constructor rule). **This file is additive**: fixture-particular binding only.
> **Line-number authority:** `origin/main` at `f7f629a30` (2026-05-30, post-#4018). Re-verify with `git show origin/main:<path>` if `main` advances.
> **Scope:** Acceptance predicate for **rung 3 only** on `phase4/branch_dispatch`. Rungs 0–2: [`src/v4/test/claim/branch_dispatch/rung_0_to_2_three_targets.dag`](../../src/v4/test/claim/branch_dispatch/rung_0_to_2_three_targets.dag). Rung 4: companion rung-4 phase4 doc (separate PR). Rungs 5–9: Phase 3+.

---

## 1. Scope binding (fixture-particular)

| Field | Value |
| ----- | ----- |
| **Rung** | 3 — round-trip preserved (`.dag` ingest⁻¹) |
| **Phase** | 4 widening (per [`v4-correctness-ladder-2026-05-30.md`](v4-correctness-ladder-2026-05-30.md) §7 Phase 4) |
| **Ladder fixture** | `phase4/branch_dispatch` (landed #4018) |
| **Fixture module** | `src/v4/test/claim/computation_shapes/branch_dispatch.dag` |
| **Canonical subject** | `branch_dispatch_subject` — `ComputationNode { behavior: Branch }` with two positional `Transform` arms (`branch_dispatch_arm_then`, `branch_dispatch_arm_else`) |
| **Rung 0–2 roster** | `src/v4/test/claim/branch_dispatch/rung_0_to_2_three_targets.dag` — labels `fixture=phase4/branch_dispatch rung=0..2 …` |
| **Fixture claims (corpus)** | `claim_branch_dispatch_well_formed`, `claim_branch_dispatch_equals_refl`, `claim_branch_dispatch_content_hash_stable` |
| **W1 target** | `dag` only (per rung-3 companion §2.1 — emitted-target round-trip is rung 5) |
| **Executable W1 round-trip (aux)** | `phase1-aux/dag_round_trip_mvp1` — same aux row as rung-3 companion §2.4 (not ladder close on this fixture) |
| **Landed substrate** | Same as rung-3 companion §1 (`run_test_claim_round_trip_verdict`, `dag_round_trip_wave1_authorities_ready`, etc.) |

**Shape rationale (emit / ladder):** `nat_semiring` exercises `Conj`/`Atom` algebra subjects; this fixture exercises **Branch** computation-node emit — the Phase 4 widening gap called out in companion spec §1.1 "Widen-after".

**W1 / W1b honesty:** Identical staging to rung-3 companion §1 — W1 proves wave-1 authorities re-derived; W1b emit→ingest bit-identical fidelity is **not** landed (T-36). Row aggregate stays `FAIL` until W1b closes (companion §2.2).

**Out-of-scope (fixture row):** Same global list as rung-3 companion §1, plus: no claim that Branch emit round-trips in rust/python/go (rung 5). No duplicate of companion §2 predicate definitions.

---

## 2. Rung 3 acceptance predicate

**Apply companion specs verbatim** for cell vocabulary (§2.4), W1/W1b staged predicates (`R3-dag-roundtrip-wave1-ready`, `R3-dag-roundtrip-fidelity`), prerequisite chain, aux-row rendering, and TestClaim wiring patterns.

**Fixture-specific substitutions only:**

| Companion placeholder | This fixture |
| --------------------- | ------------ |
| `phase1/nat_semiring` | `phase4/branch_dispatch` |
| `src/v4/test/claim/algebra_laws/nat_semiring.dag` | `src/v4/test/claim/computation_shapes/branch_dispatch.dag` |
| `claim_nat_semiring_module_roundtrip_not_authored` | `upstream_blocked:claim_branch_dispatch_module_roundtrip_not_authored` |
| Blocking receipt prefix | `phase4/branch_dispatch/rung3/…` (e.g. `phase4/branch_dispatch/rung3/dag_roundtrip_wave1_not_ready`) |
| Ladder `RoundTripClaim` row (TODO) | `src/v4/test/claim/branch_dispatch/rung_3_*.dag` (NEW — after module-loader lands; binds `branch_dispatch_subject` or module root per joint spec §5) |
| Host gate script extension | `scripts/v4-phase1-nat-semiring-rung-gate.sh` does **not** yet include `phase4/branch_dispatch` — Phase 4 host transport is a follow-up; matrix labels exist in `rung_0_to_2_three_targets.dag` only |

**Prerequisite wedge (current main expectation):** No `RoundTripClaim { … }` row binds to `v4.test.claim.computation_shapes.branch_dispatch` on `main` post-#4018 — same class of wedge as `nat_semiring` (rung-3 companion §2.3 / §6). `R3-dag-roundtrip-wave1-ready` → **`SKIP`** with `upstream_blocked:claim_branch_dispatch_module_roundtrip_not_authored`; row aggregate **`FAIL`**.

---

## 3. Verdict reporting shape (fixture row)

```text
fixture=phase4/branch_dispatch
  rung0: FAIL  (dag=… rust=… python=… go=…)   # per companion §2 rungs 0–2; substrate gap expected post-#4018
  rung1: FAIL  (rust=…)
  rung2: FAIL  (rust=… python=… go=…)
  rung3: FAIL  (dag=SKIP)
blocking_receipt: upstream_blocked:claim_branch_dispatch_module_roundtrip_not_authored

fixture=phase1-aux/dag_round_trip_mvp1
  rung3: FAIL  (dag=PASS)   # aux diagnostic — per rung-3 companion §2.4
blocking_receipt: phase1-aux/dag_round_trip_mvp1/rung3/dag_roundtrip_fidelity_w1b_unlanded
```

Rung 3 on **`phase4/branch_dispatch`** is the ladder close for this fixture id. Aux row is diagnostic only.

---

## 4. Worker brief triple

```text
fixture=phase4/branch_dispatch
rung=3
modeling_gap=none (claim authoring) | T-36 only with Modeling DFS worksheet (W1b)
predicate=R3-dag-roundtrip-wave1-ready expected to flip SKIP → PASS|FAIL on phase4/branch_dispatch
```

---

## 5. Current baseline (post-#4018 main)

| Field | Value |
| ----- | ----- |
| Landed fixture | #4018 — `branch_dispatch.dag` + `rung_0_to_2_three_targets.dag` |
| `RoundTripClaim` on fixture | **Absent** — wedge |
| Expected rung 3 | `FAIL` (`dag=SKIP`), blocking `upstream_blocked:claim_branch_dispatch_module_roundtrip_not_authored` |
| W1b | Unlanded — when W1 executes, row aggregate stays `FAIL` with `phase4/branch_dispatch/rung3/dag_roundtrip_fidelity_w1b_unlanded` until T-36 |

**Executable receipt anchor:** first PR binding `RoundTripClaim` to this module + extending a phase4-aware host gate script supersedes this baseline.

---

## 6. Sign-off (worker)

| Decision | Disposition |
| -------- | ----------- |
| Rung 3 binding on `phase4/branch_dispatch` | **PROPOSED** (Ladder/Fixture Manager ratification) |
| Predicate vocabulary | **By reference** — rung-3 companion + `v4-ladder-rung-specs` §2 |
| Independent from rung 4 | **Yes** — per joint spec §4.4 |
