# v4 ladder rung spec — Phase 4 (`field_patch_monoid` × rung 4)

> **Status:** MANAGER RATIFIED — Ladder/Fixture Manager (`wise-seal-69`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2; joint runner spec `docs/planning/compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md` §3 (rung 4 row) + §4.2 A1–A4 (host-run + falsification receipts).
> **Companion specs (inherited verbatim, NOT restated):**
> - `docs/planning/v4-ladder-fixture-phase4-field-patch-monoid-2026-05-30.md` (#4028) — fixture ratification, gate vocabulary inheritance pointer.
> - `docs/planning/v4-ladder-fixture-phase4-field-patch-monoid-rung3-2026-05-30.md` (#4036) — rung-3 binding on the same fixture (independent of rung 4 per joint spec §4.4).
> - `docs/planning/v4-ladder-rung-spec-rung4-2026-05-30.md` (#3990) — **rung 4 predicate shape** (§2.2 atomic predicate + inner-failure receipts, §2.3 prerequisite chain, §2.5 verdict reporting, §3 SG rule, §4 headline rule). All §2/§3/§4 contracts from #3990 apply verbatim to `phase4/field_patch_monoid` with the fixture id substituted.
> - `docs/planning/v4-ladder-rung-specs-2026-05-30.md` (#3946) — §2.4 cell vocabulary (PASS|FAIL|SKIP, `upstream_blocked:*`).
> **Scope:** Bind rung 4 to `phase4/field_patch_monoid`. This doc adds only the fixture-particular binding + baseline rendering; the predicate, prerequisite chain, and headline rules are inherited from #3990.

---

## 1. Scope binding

| Field | Value |
| ----- | ----- |
| **Rung** | 4 (THESIS standard "emit runs; output matches `.dag` eval") |
| **Fixture** | `phase4/field_patch_monoid` (Branch-using; ratified #4028) |
| **Fixture subject** | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag` |
| **Target set (W2)** | `rust` only — Python/Go pre-allocated `SKIP`, deferred to Phase 3 per joint spec §4.2 A3 (mirrors #3990 §2.1) |
| **Inherited predicate** | `R4-rust-emit-equals-eval` (predicate id namespaced `phase4/field_patch_monoid/rung4/*`; semantics per #3990 §2.2 — atomic conjunction of host-exit-ok ∧ stdout-parsable ∧ value-equals-eval, expressed as `Verdict<RuntimeValue>.Pass` from `run_test_claim_emit_vs_eval`) |
| **Pre-allocated cells** | `R4-python-emit-equals-eval` = **SKIP** with receipt `phase4/field_patch_monoid/rung4/python_emit_host_unallocated_phase3`; `R4-go-emit-equals-eval` = **SKIP** with receipt `phase4/field_patch_monoid/rung4/go_emit_host_unallocated_phase3` (mirrors #3990 §2.1) |
| **Closure carrier (modeled)** | `field_patch_monoid_rung4_gate(report: CorpusEvalReport) -> Bool` — **PENDING** authoring (mirror `nat_semiring_rung4_gate` at `src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:56-63`; not load-bearing for spec ratification, only for the wiring PR) |
| **Closure carrier (host gate)** | **PENDING** W3 — `field_patch_monoid_rung34_runtime_value_rows` empty until Runtime/TestClaim follow-up; §6 baseline records the wedge as **SKIP**, not **FAIL** (mirrors #3990 §6) |

**Out-of-scope (same as #3990 §1):** rung 3 round-trip (companion #4036); R4 Python/Go cells executed (rows pre-allocated only); rung 5 cross-target equivalence; rung 6 post-emit algebraic law re-check; roster row authoring (W3 wedge).

---

## 2. Rung 4 predicate (inherited from #3990 §2)

**No predicate redefinition.** #3990 §2.1–§2.6 apply verbatim with `phase1/nat_semiring` substituted by `phase4/field_patch_monoid`. The matrix renders three target cells on the rung 4 row (`rust` executable, `python`/`go` pre-allocated SKIP); the row aggregate is `FAIL` while any cell is not `PASS` (#3946 §2.4).

**Fixture-particular note (Branch-using shape):** the rung-4 predicate is shape-agnostic — `run_test_claim_emit_vs_eval` compares host-executed emit output against interpreter `eval` output, both as `RuntimeValue`. The fixture's primary control shape (`if/else` in `fp_ok`, coproduct discrimination over `FieldPatch { Inherit | Override }`) is exercised by the emit pipeline regardless of the rung-4 contract — branch-shape gaps in emit will surface as `RuntimeValue` divergence (`rust_emit_equals_eval_failed`) or stdout-parse failure (`rust_runtime_value_parse_rejected`), not as new rung-4 predicates. This makes `phase4/field_patch_monoid` a useful diagnostic surface for branch-emit regressions even though the rung-4 row shape is unchanged.

---

## 3. Prerequisite chain (inherited from #3990 §2.3)

Identical to #3990 with predicate ids re-namespaced:

| Predicate | Runs only when |
| --------- | -------------- |
| `R4-rust-emit-equals-eval` | `R2-rust-compile` = **`PASS`** on `phase4/field_patch_monoid` (companion fixture ratification §2 inherits master spec rungs 0–2) AND `run_emit_host_rust` transport landed AND `field_patch_monoid_rung34_runtime_value_rows` populated with the Rust emit-vs-eval row (W3) |
| `R4-python-emit-equals-eval` | (Phase 3 — pre-allocated `SKIP` in W2) |
| `R4-go-emit-equals-eval` | (Phase 3 — pre-allocated `SKIP` in W2) |

**`upstream_blocked:*` receipts** mirror #3990 §2.3 verbatim with this fixture id (`upstream_blocked:R2-rust-compile`, `upstream_blocked:emit_host_transport_not_wired`, `upstream_blocked:claim_field_patch_monoid_rung4_row_not_authored`).

---

## 4. SG / substrate work acceptance rule (inherited from #3990 §3)

Applies verbatim with `phase4/field_patch_monoid` substituted. New `TestClaimRun<Node, RuntimeValue>` rows must bind to the fixture module subject AND use `run_test_claim_emit_vs_eval` (sole `Verdict.Pass` constructor for rung 4, P2 single-authority) AND move the R4 rust cell on `phase4/field_patch_monoid` from `SKIP` to `PASS`/`FAIL`. `run_emit_host_rust` transport landing must flip `R4-rust-emit-equals-eval` on this fixture **as well as** `phase1/nat_semiring` and the other Phase-4 fixtures (acceptance is per-fixture).

---

## 5. "No rustc-clean as headline" (inherited from #3990 §4)

Headlines must be fixture×rung×stage shaped. Forbidden: closing rung 4 on `phase4/field_patch_monoid` via a row bound to `phase1/nat_semiring` (or vice versa); `Verdict.Pass` headline without naming the host receipt chain (exit ∧ stdout ∧ value); `rung4 PASS` headline while `R4-rust-emit-equals-eval` is `SKIP` (Verdict.Deferred).

Required headline shapes:

- `phase4/field_patch_monoid: rung4 FAIL (rust=SKIP python=SKIP go=SKIP) — blocking_receipt: upstream_blocked:emit_host_transport_not_wired` — current baseline (transport not wired; row did not execute).
- `phase4/field_patch_monoid: rung4 FAIL (rust=PASS python=SKIP go=SKIP) — W2 rust emit-vs-eval Pass; blocking_receipt: phase4/field_patch_monoid/rung4/python_emit_host_unallocated_phase3` — post-W3, pre-Phase-3 widening.
- `phase4/field_patch_monoid: rung4 PASS (rust=PASS python=PASS go=PASS) — emit-vs-eval Pass across targets` — full close (Phase 3+).

---

## 6. Spot-check receipts (vs `origin/main` @ `c84728b14`, post-#4036)

| Spec claim | Spot-check | Result |
| ---------- | ---------- | ------ |
| Fixture ratified | `docs/planning/v4-ladder-fixture-phase4-field-patch-monoid-2026-05-30.md` (merged #4028) | **CONFIRMED** |
| Rung-3 spec ratified | `docs/planning/v4-ladder-fixture-phase4-field-patch-monoid-rung3-2026-05-30.md` (merged #4036) | **CONFIRMED** — independent gating per joint spec §4.4 |
| Fixture module on main | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag:1-179` — 10 `CompilesClaim` rows; no `TestClaimRun<Node, RuntimeValue>` row | **CONFIRMED** — wedge per inherited #3990 §2.3 |
| Rung-4 predicate template | `docs/planning/v4-ladder-rung-spec-rung4-2026-05-30.md` (#3990) | **CONFIRMED** — §2.2 atomic predicate + inner-failure receipts applied here verbatim |
| Rung-4 closure carrier exemplar | `src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:23,56-63` (`nat_semiring_rung34_runtime_value_rows` + `nat_semiring_rung4_gate`) | **CONFIRMED** — structural template for the (PENDING) `field_patch_monoid_rung34_eval.dag` follow-up |
| `run_test_claim_emit_vs_eval` sole `Pass` constructor | Joint spec §4.2 + #3990 §2.2 | **CONFIRMED** (inherited authority) |
| `run_emit_host_rust` transport status (🟡 unwired) | `src/v4/compiler/emit_host.dag` 🟡 marker per #3990 §2.3 | **CONFIRMED** — same wedge as `phase1/nat_semiring` |

---

## 7. Current baseline

**Ratification-time expectation:** identical wedge to `phase1/nat_semiring` per #3990 §6 with this fixture id. No `TestClaimRun<Node, RuntimeValue>` row binds to `field_patch_monoid` on main; `run_emit_host_rust` transport is 🟡 unwired (W3 follow-up); `R4-rust-emit-equals-eval` did not execute. Python/Go cells pre-allocated `SKIP`.

```text
fixture=phase4/field_patch_monoid
  rung4: FAIL  (rust=SKIP python=SKIP go=SKIP)
blocking_receipt: upstream_blocked:emit_host_transport_not_wired
```

Executable baseline supersedes on first run after the W3 `run_emit_host_rust` transport lands AND the first rung-4 roster row binds to `phase4/field_patch_monoid`. Same staging pattern as `phase1/nat_semiring`: rust cell flips `SKIP → PASS` on a successful emit-vs-eval run; row aggregate stays `FAIL` until Phase 3 widening flips Python/Go cells to `PASS`.

---

## 8. Manager sign-off

| Decision | Disposition |
| -------- | ----------- |
| Rung 4 binding on `phase4/field_patch_monoid` per #3990 §2 contract | **RATIFIED** |
| Target set: `rust` executable in W2; `python`/`go` pre-allocated `SKIP` (Phase 3) | **RATIFIED** (inherited from #3990 §2.1) |
| Atomic `R4-rust-emit-equals-eval` predicate; inner faults via blocking receipt | **RATIFIED** (inherited from #3990 §2.2) |
| `run_test_claim_emit_vs_eval` is sole `Verdict.Pass` constructor (P2 single-authority) | **RATIFIED** (inherited from #3990 §3) |
| Independent gating from rung 3 per joint spec §4.4 | **RATIFIED** |
| `run_emit_host_rust` transport landing on `phase4/field_patch_monoid` | **PENDING** — W3 follow-up; gates the row aggregate progression |
| `field_patch_monoid_rung34_eval.dag` closure carrier (mirror `nat_semiring_rung34_eval.dag`) | **PENDING** — wiring follow-up; not load-bearing for ratification |
| Phase 3 Python/Go widening on `phase4/field_patch_monoid` | **PENDING** — Phase 3 per joint spec §4.2 A3 |

**Wave alignment:** W2.5 maturation for v0.1.1; not Jun 1 gating. Coordinates with royal-wolf-898 rung-3/4 specs for `phase4/branch_dispatch` (#4034) + `phase4/loop_linear_bound` (#4035) under the same role-node.
