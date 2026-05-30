# v4 ladder rung spec — Phase 4 (`loop_linear_bound` × rung 4)

> **Status:** DRAFT — Ladder/Fixture worker (`royal-wolf-898`), 2026-05-30.
> **Authority:** PR #3938 §11.1; joint runner spec [`compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md`](compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md) §3 "rung 4" + §4.2.
> **Companion specs (do not duplicate):** [`v4-ladder-rung-specs-2026-05-30.md`](v4-ladder-rung-specs-2026-05-30.md) §2; [`v4-ladder-rung-spec-rung4-2026-05-30.md`](v4-ladder-rung-spec-rung4-2026-05-30.md) (#3990); [`v4-ladder-rung-spec-rung3-phase4-loop-linear-bound-2026-05-30.md`](v4-ladder-rung-spec-rung3-phase4-loop-linear-bound-2026-05-30.md) (#4035). **Additive** fixture binding only.
> **Line-number authority:** `origin/main` at `f7f629a30`.
> **Scope:** **Rung 4 only** on `phase4/loop_linear_bound`. Rung 3: #4035. Independent closure per joint spec §4.4.

---

## 1. Scope binding (fixture-particular)

| Field | Value |
| ----- | ----- |
| **Fixture** | `phase4/loop_linear_bound` (#4018) |
| **Module** | `src/v4/test/claim/computation_shapes/loop_linear_bound.dag` |
| **Eval subject** | `loop_linear_bound_subject` (`Loop` + `loop_bound_edge` + `Transform` body) |
| **W2 targets** | `rust` only |
| **Modeled closure** | **Not authored** — no `loop_linear_bound_rung34_eval.dag` (fixture activation gap: modeled closure + runtime-value row roster) |
| **Closure carrier (host)** | Pending — extend phase4 host gate or add `scripts/v4-phase4-*-rung-gate.sh` follow-up |

**Known adjacent modeling debt:** This fixture intentionally exercises the current Loop-bound edge shape. Raw `loop_bound_edge` Symbol-tag is owned by the existing Loop-bound dissolution lane (SG-1 DFS / T-12). Do not treat failures involving `loop_bound_edge` as local rung-4 modeling authority; route Symbol-tag / Loop-bound coordinate findings through that lane. This rung spec only binds the fixture to rung-4 evaluation.

**Out-of-scope:** Rung 3 (#4035). R5–R9. Ad hoc emitter spot-fixes around `loop_bound_edge`.

---

## 2. Rung 4 acceptance predicate

**Apply rung-4 companion §2 verbatim.** Substitute `phase4/loop_linear_bound`, `loop_linear_bound_rung34_runtime_value_rows`, `loop_linear_bound_rung4_gate`, receipt prefix `phase4/loop_linear_bound/rung4/…`.

**Prerequisite SKIP receipts (companion §2.3):**

| Prerequisite not met | Per-cell `SKIP` receipt |
| -------------------- | ----------------------- |
| `run_emit_host_rust` transport not landed | `upstream_blocked:emit_host_transport_not_wired` |
| `R2-rust-compile` not `PASS` | `upstream_blocked:R2-rust-compile` |
| `loop_linear_bound_rung34_runtime_value_rows` empty | `upstream_blocked:loop_linear_bound_rung34_runtime_value_rows_empty` |

**Headline ordering (companion §2.4 — rung-4 execution infrastructure before fixture row population):**

1. If the rung-4 host transport entrypoint cannot run at all, headline: `upstream_blocked:emit_host_transport_not_wired`.
2. Once transport exists, if `R2-rust-compile` is not `PASS`, headline: `upstream_blocked:R2-rust-compile`.
3. Once transport exists and `R2-rust-compile` is `PASS`, if runtime-value rows are empty, headline: `upstream_blocked:loop_linear_bound_rung34_runtime_value_rows_empty`.

On post-#4018 `main`, step 1 is the first blocker (transport not landed). Rung 0–2 substrate roster rows are authored predicates, not executed host verdicts for this headline chain.

**Phase 4 scope note:** This spec binds the fixture to rung 4 only. It does **not** claim Phase 4 widening is complete — Phase 4 still requires rungs 0–6 across the fixture set; rungs 7–9 remain open by design.

---

## 3. Verdict reporting shape

```text
fixture=phase4/loop_linear_bound
  rung4: FAIL  (rust=SKIP python=SKIP go=SKIP)
blocking_receipt: upstream_blocked:emit_host_transport_not_wired
```

**Blocked aggregate:** aggregate `FAIL` means the fixture rung is not yet proven; all per-target cells are `SKIP` because prerequisites are blocked — not because emit/eval was attempted and failed.

Independent of the rung 3 row on this fixture (rung-4 companion §2.4).

---

## 4. Worker brief

```text
fixture=phase4/loop_linear_bound
rung=4
modeling_gap=none for Loop fixture shape (loop_bound_edge Symbol-tag: existing T-12 / SG-1 lane, not this doc)
execution_gap=emit_host_transport + loop_linear_bound runtime-value row roster
predicate=R4-rust-emit-equals-eval expected to flip SKIP → PASS|FAIL
```

Do not dispatch an SG/modeling worker from this doc for Loop-bound Symbol-tag work. Dispatch runner/roster work when upstream transport exists.

---

## 5. Baseline (post-#4018 main)

| `loop_linear_bound_rung34_eval` module | **Absent** — fixture activation follow-up |
| Expected rung 4 | `FAIL` blocked aggregate (all cells `SKIP`); headline `upstream_blocked:emit_host_transport_not_wired` |

---

## 6. Sign-off (worker)

| Decision | Disposition |
| -------- | ----------- |
| Rung 4 on `phase4/loop_linear_bound` | **PROPOSED** |
