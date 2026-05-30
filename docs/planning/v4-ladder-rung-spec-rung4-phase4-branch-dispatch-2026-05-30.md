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
| **Closure carrier (modeled)** | **Not authored** — no `branch_dispatch_rung34_eval.dag` on `main` post-#4018 (W3 wedge; mirror `nat_semiring_rung34_eval.dag` pattern) |
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

**Prerequisite SKIP receipts (companion §2.3 — two states, one headline):**

| Prerequisite not met | Per-cell `SKIP` receipt | When it applies |
| -------------------- | ----------------------- | --------------- |
| `run_emit_host_rust` transport not landed | `upstream_blocked:emit_host_transport_not_wired` | **Current main** — lowest unresolved upstream; **headline** blocking receipt for §3 matrix |
| `branch_dispatch_rung34_runtime_value_rows` empty (W3 roster wedge) | `upstream_blocked:branch_dispatch_rung34_runtime_value_rows_empty` | After transport lands but emit-vs-eval row not yet populated |
| `R2-rust-compile` not `PASS` | `upstream_blocked:R2-rust-compile` | Rung 2 rust cell not `PASS` |

**Headline rule (companion §2.4):** use the **lowest** unmet prerequisite in the table above. On post-#4018 `main`, that is always `upstream_blocked:emit_host_transport_not_wired` — not `…_rows_empty`, until transport is dissolved.

---

## 3. Verdict reporting shape

```text
fixture=phase4/branch_dispatch
  rung4: FAIL  (rust=SKIP python=SKIP go=SKIP)
blocking_receipt: upstream_blocked:emit_host_transport_not_wired
```

Independent of rung 3 row on this fixture (rung-4 companion §2.4).

---

## 4. Worker brief triple

```text
fixture=phase4/branch_dispatch
rung=4
modeling_gap=none (W3 roster + transport) | SG-class only with Modeling DFS worksheet
predicate=R4-rust-emit-equals-eval expected to flip SKIP → PASS|FAIL
```

---

## 5. Baseline (post-#4018 main)

| Field | Value |
| ----- | ----- |
| `branch_dispatch_rung34_eval` module | **Absent** — W3 follow-up |
| Expected rung 4 | `FAIL` (all cells `SKIP`); headline `upstream_blocked:emit_host_transport_not_wired` |

---

## 6. Sign-off (worker)

| Decision | Disposition |
| -------- | ----------- |
| Rung 4 binding on `phase4/branch_dispatch` | **PROPOSED** |
| Vocabulary | **By reference** — rung-4 companion (#3990) |
