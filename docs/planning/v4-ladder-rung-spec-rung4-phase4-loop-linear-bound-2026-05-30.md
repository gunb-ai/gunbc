# v4 ladder rung spec — Phase 4 (`loop_linear_bound` × rung 4)

> **Status:** DRAFT — Ladder/Fixture worker (`royal-wolf-898`), 2026-05-30.
> **Authority:** PR #3938 §11.1; joint runner spec [`compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md`](compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md) §3 "rung 4" + §4.2.
> **Companion specs:** [`v4-ladder-rung-specs-2026-05-30.md`](v4-ladder-rung-specs-2026-05-30.md) §2; [`v4-ladder-rung-spec-rung4-2026-05-30.md`](v4-ladder-rung-spec-rung4-2026-05-30.md) (#3990); [`v4-ladder-rung-spec-rung3-phase4-loop-linear-bound-2026-05-30.md`](v4-ladder-rung-spec-rung3-phase4-loop-linear-bound-2026-05-30.md) (#4035). **Additive** only.
> **Line-number authority:** `origin/main` at `f7f629a30`.
> **Scope:** **Rung 4 only** on `phase4/loop_linear_bound`.

---

## 1. Scope binding (fixture-particular)

| Field | Value |
| ----- | ----- |
| **Fixture** | `phase4/loop_linear_bound` (#4018) |
| **Module** | `src/v4/test/claim/computation_shapes/loop_linear_bound.dag` |
| **Eval subject** | `loop_linear_bound_subject` (`Loop` + `loop_bound_edge` + `Transform` body) |
| **W2 targets** | `rust` only |
| **Modeled closure** | **Not authored** — no `loop_linear_bound_rung34_eval.dag` (W3 wedge) |

---

## 2. Rung 4 acceptance predicate

**Apply rung-4 companion §2 verbatim.** Substitute `phase4/loop_linear_bound`, `loop_linear_bound_rung34_runtime_value_rows`, `loop_linear_bound_rung4_gate`, receipt prefix `phase4/loop_linear_bound/rung4/…`.

**Wedge:** `upstream_blocked:loop_linear_bound_rung34_runtime_value_rows_empty` until W3 populates emit-vs-eval row for `loop_linear_bound_subject`.

---

## 3. Verdict reporting shape

```text
fixture=phase4/loop_linear_bound
  rung4: FAIL  (rust=SKIP python=SKIP go=SKIP)
blocking_receipt: upstream_blocked:emit_host_transport_not_wired
```

---

## 4. Worker brief triple

```text
fixture=phase4/loop_linear_bound
rung=4
predicate=R4-rust-emit-equals-eval expected to flip SKIP → PASS|FAIL
```

---

## 5. Baseline (post-#4018 main)

No `loop_linear_bound_rung34_eval` module — rung 4 **`SKIP`** wedge per rung-4 companion §6.

---

## 6. Sign-off (worker)

| Decision | Disposition |
| -------- | ----------- |
| Rung 4 on `phase4/loop_linear_bound` | **PROPOSED** |
