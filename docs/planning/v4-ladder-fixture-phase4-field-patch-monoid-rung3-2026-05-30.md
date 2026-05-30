# v4 ladder rung spec — Phase 4 (`field_patch_monoid` × rung 3)

> **Status:** MANAGER RATIFIED — Ladder/Fixture Manager (`wise-seal-69`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2; joint runner spec `docs/planning/compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md` §3 (rung 3 row) + §4.4 (rung-split).
> **Companion specs (inherited verbatim, NOT restated):**
> - `docs/planning/v4-ladder-fixture-phase4-field-patch-monoid-2026-05-30.md` (#4028) — fixture ratification (§1), gate vocabulary inheritance pointer (§2), SG acceptance (§4), headline rule (§5).
> - `docs/planning/v4-ladder-rung-spec-rung3-2026-05-30.md` (#4003) — **rung 3 predicate shape** (§2.2 W1/W1b staging, §2.3 prerequisite chain, §2.5 verdict reporting, §3 SG rule, §4 headline rule). All §2/§3/§4 contracts from #4003 apply verbatim to `phase4/field_patch_monoid` with the fixture id substituted.
> - `docs/planning/v4-ladder-rung-specs-2026-05-30.md` (#3946) — §2.4 cell vocabulary (PASS|FAIL|SKIP, `upstream_blocked:*`).
> **Scope:** Bind rung 3 to `phase4/field_patch_monoid`. This doc adds only the fixture-particular binding + baseline rendering; the predicate, prerequisite chain, and headline rules are inherited from #4003.

---

## 1. Scope binding

| Field | Value |
| ----- | ----- |
| **Rung** | 3 (THESIS standard "round-trip preserved") |
| **Fixture** | `phase4/field_patch_monoid` (Branch-using; ratified #4028) |
| **Fixture subject** | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag` |
| **W1 target** | `dag` only (per #4003 §1 — emitted-target round-trip is rung 5) |
| **Inherited predicate** | `R3-dag-roundtrip-wave1-ready` + `R3-dag-roundtrip-fidelity` (predicate ids namespaced `phase4/field_patch_monoid/rung3/*`; semantics per #4003 §2.2) |
| **Inherited substrate (PR #3960)** | `run_test_claim_round_trip_verdict` (`src/v4/compiler/05_eval.dag:1721-1739`), `dag_round_trip_wave1_authorities_ready` (`src/v4/extdeps/languages/dag.dag:3168+`) — same constructors as #4003 |

**Out-of-scope (same as #4003 §1):** R3 for emitted rust/python/go (rung 5); W1b emit→ingest bit-identical fidelity (T-36); rung-4 predicate (companion); leaf-model verification (orthogonal lane).

---

## 2. Rung 3 predicate (inherited from #4003 §2)

**No predicate redefinition.** #4003 §2.1–§2.6 apply verbatim with `phase1/nat_semiring` substituted by `phase4/field_patch_monoid`. The matrix renders a single `dag` cell on the rung 3 row (#4003 §2.1); the row aggregate is `FAIL` while W1b is unlanded regardless of W1 cell state (#4003 §2.2 — "row aggregate `PASS` requires both stages = `PASS`").

**Fixture-particular note (Branch-using shape):** the round-trip predicate is shape-agnostic — `run_test_claim_round_trip_verdict` operates on `Node`/`Atom`/`Conj`/`Edge` carriers, not on control-flow constructors. `if/else` in `fp_ok` (`field_patch_monoid.dag:33`) and `FieldPatch` coproduct discrimination are surface-level surface for the fixture's algebraic laws; the rung-3 round-trip only cares whether re-parsing the fixture module reproduces its `.dag` source under C5 trivia normalization. Branch shapes do not add new round-trip predicates.

---

## 3. Prerequisite chain (inherited from #4003 §2.3)

Identical to #4003 with predicate ids re-namespaced:

| Predicate | Runs only when |
| --------- | -------------- |
| `R3-dag-roundtrip-wave1-ready` | `R0-dag-parse` = **`PASS`** on `phase4/field_patch_monoid` AND a `RoundTripClaim` row exists bound to the fixture module subject |
| `R3-dag-roundtrip-fidelity` | `R3-dag-roundtrip-wave1-ready` = **`PASS`** AND W1b emit→ingest comparator landed (T-36) |

**`upstream_blocked:*` receipts** mirror #4003 §2.3 verbatim (`upstream_blocked:R0-dag-parse`, `upstream_blocked:claim_field_patch_monoid_module_roundtrip_not_authored`, `upstream_blocked:R3-dag-roundtrip-wave1-ready`, `upstream_blocked:w1b_emit_ingest_comparator_unlanded`).

---

## 4. SG / substrate work acceptance rule (inherited from #4003 §3)

Applies verbatim with `phase4/field_patch_monoid` substituted. New `RoundTripClaim` rows must bind to the fixture module subject AND use `run_test_claim_round_trip_verdict` AND move the R3 cell on `phase4/field_patch_monoid` from `SKIP` to `PASS`/`FAIL`. W1b comparator landing must flip `R3-dag-roundtrip-fidelity` on this fixture **as well as** `phase1/nat_semiring` (acceptance is per-fixture).

---

## 5. "No rustc-clean as headline" (inherited from #4003 §4)

Headlines must be fixture×rung×stage shaped. Forbidden: closing rung 3 on `phase4/field_patch_monoid` via a row bound to `phase1/nat_semiring` (or vice versa); `RoundTripClaim Pass` headline without naming W1-vs-W1b stage; `rung3 PASS` headline while W1b unlanded.

Required headline shapes:

- `phase4/field_patch_monoid: rung3 FAIL (dag=SKIP) — blocking_receipt: upstream_blocked:claim_field_patch_monoid_module_roundtrip_not_authored` — current baseline.
- `phase4/field_patch_monoid: rung3 FAIL (dag=PASS) — W1 wave-1 ready; blocking_receipt: phase4/field_patch_monoid/rung3/dag_roundtrip_fidelity_w1b_unlanded` — post claim-migration, pre-W1b.
- `phase4/field_patch_monoid: rung3 PASS (dag=PASS) — W1 wave-1 ready; W1b emit→ingest fidelity proven` — full close (post-W1b landing on this fixture).

---

## 6. Spot-check receipts (vs `origin/main` @ `af7f1fe1c`, post-#4028)

| Spec claim | Spot-check | Result |
| ---------- | ---------- | ------ |
| Fixture ratified | `docs/planning/v4-ladder-fixture-phase4-field-patch-monoid-2026-05-30.md` (this branch's predecessor commit, merged #4028) | **CONFIRMED** |
| Fixture module on main | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag:1-179` — 10 `CompilesClaim` rows; no `RoundTripClaim` row | **CONFIRMED** — wedge per inherited #4003 §2.3 |
| Rung-3 predicate template | `docs/planning/v4-ladder-rung-spec-rung3-2026-05-30.md` (#4003) | **CONFIRMED** — §2.2 W1/W1b staging applied here verbatim |
| W1 verdict substrate | `src/v4/compiler/05_eval.dag:1721-1739` (`run_test_claim_round_trip_verdict`) | **CONFIRMED** — sole `Pass` constructor (P2 single-authority) |
| W1 wave-1 readiness gate | `src/v4/extdeps/languages/dag.dag:3168+` (`dag_round_trip_wave1_authorities_ready`) | **CONFIRMED** — applies fixture-agnostically |
| No `RoundTripClaim` row bound to `field_patch_monoid` | `git grep -n 'RoundTripClaim' src/v4/test/claim/algebra_laws/field_patch_monoid.dag` returns no `RoundTripClaim { … }` data row | **CONFIRMED** — wedge per §3 prerequisite |

---

## 7. Current baseline

**Ratification-time expectation:** identical wedge to `phase1/nat_semiring` per #4003 §6. No `RoundTripClaim` row binds to `field_patch_monoid` on main; `R3-dag-roundtrip-wave1-ready` did not execute.

```text
fixture=phase4/field_patch_monoid
  rung3: FAIL  (dag=SKIP)
blocking_receipt: upstream_blocked:claim_field_patch_monoid_module_roundtrip_not_authored
```

Executable baseline supersedes on first run after the W3 module-loader work (per #4003 §6 / joint spec §5) unblocks `RoundTripClaim` row authoring on `field_patch_monoid`. Same staging pattern: cell flips `SKIP → PASS` on a successful W1 run; row aggregate stays `FAIL` until W1b emit→ingest comparator lands.

---

## 8. Manager sign-off

| Decision | Disposition |
| -------- | ----------- |
| Rung 3 binding on `phase4/field_patch_monoid` per #4003 §2 contract | **RATIFIED** |
| Single `.dag` cell on rung 3 row (no rust/python/go cells; rung 5 territory) | **RATIFIED** (inherited from #4003 §2.1) |
| Staged predicates `R3-dag-roundtrip-wave1-ready` + `R3-dag-roundtrip-fidelity` | **RATIFIED** (inherited from #4003 §2.2) |
| Row aggregate `FAIL` while W1b unlanded — honest staging vs ladder standard | **RATIFIED** (inherited from #4003 §2.2) |
| `run_test_claim_round_trip_verdict` is sole `Pass` constructor (P2 single-authority) | **RATIFIED** (inherited from #4003 §3) |
| Independent gating from rung 4 per joint spec §4.4 | **RATIFIED** |
| W1b emit→ingest comparator landing on `phase4/field_patch_monoid` | **PENDING** — T-36; gates the row aggregate `PASS` |
| Host gate script + CI matrix extension for rung 3 on `phase4/field_patch_monoid` | **PENDING** — depends on the first executable R3 row binding to this fixture |
