# v4 ladder rung spec — Phase 4 (`branch_dispatch` × rung 4)

> **Status:** DRAFT — Ladder/Fixture worker (`royal-wolf-898`), 2026-05-30.
> **Authority:** PR #3938 §11.1; joint runner spec [`compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md`](compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md) §3 "rung 4" + §4.2.
> **Companion specs (do not duplicate):** [`v4-ladder-rung-specs-2026-05-30.md`](v4-ladder-rung-specs-2026-05-30.md) §2; [`v4-ladder-rung-spec-rung4-2026-05-30.md`](v4-ladder-rung-spec-rung4-2026-05-30.md) (PR #3990); [`v4-ladder-rung-spec-rung3-phase4-branch-dispatch-2026-05-30.md`](v4-ladder-rung-spec-rung3-phase4-branch-dispatch-2026-05-30.md) (rung 3 on this fixture, #4034). **Additive** fixture binding only.
> **Line-number authority:** `origin/main` at `f7f629a30`. Re-verify if `main` advances.
> **Scope:** **Rung 4 only** on `phase4/branch_dispatch`. Rung 3: #4034. Independent closure per joint spec §4.4.

---

## 1. Scope binding (fixture-particular)

| Field | Value |
| ----- | ----- |
| **Rung** | 4 — emit runs; output matches `.dag` eval |
| **Fixture** | `phase4/branch_dispatch` (#4018) |
| **Fixture module** | `src/v4/test/claim/computation_shapes/branch_dispatch.dag` |
| **Eval subject** | `branch_dispatch_subject` (`Branch` + two `Transform` arms) |
| **Phase** | 4 widening |
| **W2 target set** | `rust` only — python/go pre-allocated `SKIP` (rung-4 companion §2.1) |
| **Closure carrier (modeled)** | **Not authored** — no `branch_dispatch_rung34_eval.dag` on `main` post-#4018 (fixture activation gap: modeled closure + runtime-value row roster; mirror `nat_semiring_rung34_eval.dag` pattern) |
| **Closure carrier (host)** | Pending — extend phase4 host gate or add `scripts/v4-phase4-*-rung-gate.sh` follow-up |

**Out-of-scope:** Rung 3 ( #4034 ). R5–R9. Leaf-model lane (orthogonal).

---

## 2. Rung 4 acceptance predicate

**Apply rung-4 companion §2 verbatim** for `R4-rust-emit-equals-eval`, python/go `SKIP` receipts, prerequisite chain (`R2-rust-compile` = PASS + transport + roster), inner-fault receipts, and forbidden headlines.

**Fixture-specific substitutions:**

| Companion placeholder | This fixture |
| --------------------- | ------------ |
| `phase1/nat_semiring` | `phase4/branch_dispatch` |
| `nat_semiring_rung34_runtime_value_rows` | `branch_dispatch_rung34_runtime_value_rows` (TODO in `branch_dispatch/rung_4_eval.dag` or workflow module) |
| `nat_semiring_rung4_gate` | `branch_dispatch_rung4_gate` (TODO) |
| Blocking receipt prefix | `phase4/branch_dispatch/rung4/…` |
| Phase 3 unallocated SKIP | `phase4/branch_dispatch/rung4/python_emit_host_unallocated_phase3` (and `go_…`) |

**Prerequisite SKIP receipts (companion §2.3):**

| Prerequisite not met | Per-cell `SKIP` receipt |
| -------------------- | ----------------------- |
| `run_emit_host_rust` transport not landed | `upstream_blocked:emit_host_transport_not_wired` |
| `R2-rust-compile` not `PASS` | `upstream_blocked:R2-rust-compile` |
| `branch_dispatch_rung34_runtime_value_rows` empty | `upstream_blocked:branch_dispatch_rung34_runtime_value_rows_empty` |

**Headline ordering (companion §2.4 — rung-4 execution infrastructure before fixture row population):**

1. If the rung-4 host transport entrypoint cannot run at all, headline: `upstream_blocked:emit_host_transport_not_wired`.
2. Once transport exists, if `R2-rust-compile` is not `PASS`, headline: `upstream_blocked:R2-rust-compile`.
3. Once transport exists and `R2-rust-compile` is `PASS`, if runtime-value rows are empty, headline: `upstream_blocked:branch_dispatch_rung34_runtime_value_rows_empty`.

On post-#4018 `main`, step 1 is the first blocker (transport not landed). Rung 0–2 substrate roster rows are authored predicates, not executed host verdicts for this headline chain.

**Phase 4 scope note:** This spec binds the fixture to rung 4 only. It does **not** claim Phase 4 widening is complete — Phase 4 still requires rungs 0–6 across the fixture set; rungs 7–9 remain open by design.

---

## 3. Verdict reporting shape

```text
fixture=phase4/branch_dispatch
  rung4: FAIL  (rust=SKIP python=SKIP go=SKIP)
blocking_receipt: upstream_blocked:emit_host_transport_not_wired
```

**Blocked aggregate:** aggregate `FAIL` means the fixture rung is not yet proven; all per-target cells are `SKIP` because prerequisites are blocked — not because emit/eval was attempted and failed.

Independent of the rung 3 row on this fixture (rung-4 companion §2.4).

---

## 4. Worker brief

```text
fixture=phase4/branch_dispatch
rung=4
modeling_gap=none for Branch fixture shape
execution_gap=emit_host_transport + branch_dispatch runtime-value row roster
predicate=R4-rust-emit-equals-eval expected to flip SKIP → PASS|FAIL
```

Do not dispatch an SG/modeling worker from this doc. Dispatch runner/roster work when upstream transport exists. SG-class work only with Modeling DFS worksheet.

---

## 5. Baseline (post-#4018 main)

| Field | Value |
| ----- | ----- |
| `branch_dispatch_rung34_eval` module | **Absent** — W3 follow-up |
| Expected rung 4 | `FAIL` blocked aggregate (all cells `SKIP`); headline `upstream_blocked:emit_host_transport_not_wired` |

---

## 6. Sign-off (worker)

| Decision | Disposition |
| -------- | ----------- |
| Rung 4 binding on `phase4/branch_dispatch` | **PROPOSED** |
| Vocabulary | **By reference** — rung-4 companion (#3990) |
