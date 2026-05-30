# v4 ladder rung spec — Phase 1 (`nat_semiring` × rung 3)

> **Status:** DRAFT — Ladder/Fixture Manager (`zesty-bat-510`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2 (acceptance predicates per rung); joint runner spec [`compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md`](compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md) §3 row "rung 3" + §4.4 rung-split.
> **Companion specs:** [`v4-ladder-rung-specs-2026-05-30.md`](v4-ladder-rung-specs-2026-05-30.md) — rungs 0–2; [`v4-ladder-rung-spec-rung4-2026-05-30.md`](v4-ladder-rung-spec-rung4-2026-05-30.md) — rung 4. **This file is additive**: it does not restate the 9-rung table, fixture ratification, §2.4 cell vocabulary, §3 acceptance rule, or §4 headline rule from the rungs-0-2 spec — those apply verbatim and are cited in-place below.
> **Line-number authority:** `origin/main` at `0ace5f8b7` (2026-05-30, post-#3960 / #3972 / #3993 / #3994 / #3996). Re-verify with `git show origin/main:<path>` if `main` advances.
> **Scope:** Acceptance predicate for **rung 3 only** on `phase1/nat_semiring`. Rungs 0–2 and rung 4 are owned by the companion specs; rungs 5–9 are out of scope (Phase 3+).

---

## 1. Scope binding

| Field | Value |
| ----- | ----- |
| **Rung** | 3 (THESIS standard "Round-trip preserved" — re-parsing emit reproduces source up to declared normalization) |
| **Phase** | 2 (per parent planning doc §7) — rung 3 closure is independent of rung 4 closure per joint spec §4.4 |
| **Ladder fixture** | `phase1/nat_semiring` (ratified — companion spec §1) |
| **Ladder fixture subject** | `src/v4/test/claim/algebra_laws/nat_semiring.dag` |
| **Executable W1 round-trip fixture (aux)** | `phase1-aux/dag_round_trip_mvp1` — module path `v4.test.fixture.dag_round_trip_mvp1`; claim `v4.test.claim.round_trip.dag_ingest_round_trip` (`src/v4/test/claim/round_trip/dag_ingest_round_trip.dag`) |
| **W1 target** | `dag` only (RoundTripClaim ingest⁻¹ on .dag target per joint spec §3) — Phase 2+ targets are rung-5 cross-target territory, not rung-3 |
| **Landed substrate (PR #3960)** | `run_test_claim_round_trip_verdict` (`src/v4/compiler/05_eval.dag:1721-1739`), `eval_round_trip_claim_input_for_verdict` (`05_eval.dag:1705-1717`), `dag_round_trip_wave1_authorities_ready` (`src/v4/extdeps/languages/dag.dag:3168+`), `claim_dag_ingest_round_trip` (`src/v4/test/claim/round_trip/dag_ingest_round_trip.dag:79-83`) |

**Wave-1 scope honesty (load-bearing for §2.2 pass-condition):** the W1 RoundTripClaim verdict landed in #3960 proves **wave-1 lex/grammar/C5-trivia authorities are present and re-derivable from `dag.dag`** (`dag_round_trip_wave1_authorities_ready`). It does **not** yet prove bit-identical emit→ingest fidelity on the fixture — that is the W1b / T-36 follow-up, called out verbatim in the landed claim label ("emit→ingest fidelity W1b") and in `dag_ingest_round_trip.dag:3`. Rung-3 closure on the ladder is staged accordingly (§2.2 cell vocabulary + §6 baseline).

**Out-of-scope for this spec (named so the boundary is auditable):**

- R3 for emitted rust/python/go (round-trip of *emitted* code is rung 5 cross-target territory, not rung 3).
- W1b emit→ingest bit-identical fidelity — gated on T-36 follow-up; this spec carries an explicit `SKIP` cell for it.
- Authoring `claim_nat_semiring_module_roundtrip` on the ladder fixture — gated on module-loader landing per joint spec §5 ("`dag_round_trip_mvp1`-style structural binding once module-loader lands"); §2.3 records the wedge.
- Rung-4 predicate (companion spec).
- Rungs 5–9 (Phase 3+).
- Leaf-model verification predicates (PR #3972 / `docs/planning/v4-leaf-model-verification-2026-05-30.md`) — orthogonal lane; **not rung-3**.

---

## 2. Rung 3 acceptance predicate

**Companion spec §2 (rungs 0–2) defines the cell vocabulary and prerequisite rule.** Apply that contract verbatim:

- Row aggregate ∈ {`PASS`, `FAIL`} only (no row `SKIP`).
- Per-target cells ∈ {`PASS`, `FAIL`, `SKIP`}; `SKIP` when predicate did not execute (upstream not `PASS`, or substrate unavailable).
- Forbidden: label `FAIL` for a predicate that did not run.

### 2.1 Phase 1 / W1 target set

Rung 3 is **single-target** by joint spec §3 ("`.dag` target" only). Emitted-target round-trip is rung 5.

| Target id | Toolchain | Rung 3 |
| --------- | --------- | :----: |
| `dag` | RoundTripClaim eval path via `run_test_claim_round_trip_verdict` (`src/v4/compiler/05_eval.dag:1723-1742`) | ✓ |
| `rust` / `python` / `go` | (rung 5 — cross-target equivalence; **not** rung 3) | — |

The rust/python/go cells **do not exist** on the rung-3 row — unlike rung 4 where they are pre-allocated `SKIP`, rung 3 is structurally `.dag`-only by ladder definition. The matrix renders a single `dag` cell on rung 3.

### 2.2 R3 predicate — staged

**Question:** Does the RoundTripClaim verdict for the ladder fixture's module subject return `Pass` (not `Deferred`, not `Fail`) under the joint runner spec §3 pass condition?

Two stages mirroring the landed substrate's own W1 / W1b split:

| Predicate id | Stage | Pass condition | Fail blocking receipt |
| ------------ | ----- | -------------- | --------------------- |
| `R3-dag-roundtrip-wave1-ready` | W1 (landed) | `run_test_claim_round_trip_verdict` returns `Pass` for a `RoundTripClaim` bound to the ladder fixture module subject — i.e. (a) `eval_round_trip_claim_input_for_verdict` returns `Accepted` (well-formed input) AND (b) `dag_round_trip_wave1_authorities_ready()` returns `true` (lex ∧ grammar ∧ C5 trivia authorities re-derived from `dag.dag`). | `phase1/nat_semiring/rung3/dag_roundtrip_wave1_not_ready` (eval rejected with `eval_rejected_invalid_node` at `05_eval.dag:1711` or `eval_rejected_roundtrip_precondition` at `05_eval.dag:1733`) |
| `R3-dag-roundtrip-fidelity` | W1b (T-36 follow-up) | Emit→ingest **bit-identical** fidelity on the fixture under C5 `DagTriviaNormalization` (whitespace / line-comment / block-comment `DeclaredNormalized` per `dag.dag:3160+`). **Not landed** — `Verdict.Pass` here requires the emit→ingest comparator that #3960's claim label explicitly defers as "emit→ingest fidelity W1b". | `phase1/nat_semiring/rung3/dag_roundtrip_fidelity_w1b_unlanded` |

**Atomic — no per-component sub-predicates** on the row. Inner faults surface in the blocking receipt:

| Inner failure (single carrier) | Required blocking receipt |
| ------------------------------ | ------------------------- |
| `dag_round_trip_lex_ready` false | `phase1/nat_semiring/rung3/dag_lex_not_ready` |
| `dag_round_trip_grammar_ready` false | `phase1/nat_semiring/rung3/dag_grammar_not_ready` |
| `dag_round_trip_normalization_declared` false | `phase1/nat_semiring/rung3/dag_c5_trivia_not_declared_normalized` |
| Input not well-formed (`eval_rejected_invalid_node`) | `phase1/nat_semiring/rung3/dag_roundtrip_input_not_well_formed` |
| Precondition gate (`eval_rejected_roundtrip_precondition`) — composite | `phase1/nat_semiring/rung3/dag_roundtrip_wave1_not_ready` |

**Row aggregate (rung 3):** `PASS` requires **both** stages = `PASS`. While W1b is unlanded, the row aggregate is `FAIL` and the headline blocking receipt is `dag_roundtrip_fidelity_w1b_unlanded` (per the "lowest unresolved upstream" rule in companion spec §2.4) — which is the **honest** rendering: the ladder rung-3 standard ("round-trip preserved") is not yet proven on the fixture, even when W1 readiness flips to `PASS`.

### 2.3 Prerequisite chain

Per companion spec §2.4 prerequisite rule. R3 extends rungs 0–2:

| Predicate | Runs only when |
| --------- | -------------- |
| `R3-dag-roundtrip-wave1-ready` | `R0-dag-parse` = **`PASS`** (companion spec §2.1) AND a `RoundTripClaim` row exists bound to the ladder fixture module subject |
| `R3-dag-roundtrip-fidelity` | `R3-dag-roundtrip-wave1-ready` = **`PASS`** AND W1b emit→ingest comparator landed (T-36) |

If either prerequisite is unmet → **`SKIP`** with one of:

- `upstream_blocked:R0-dag-parse` (companion rung 0 dag cell not `PASS`),
- `upstream_blocked:claim_nat_semiring_module_roundtrip_not_authored` (no `RoundTripClaim` row bound to the **ladder fixture** module — current main has the row on `phase1-aux/dag_round_trip_mvp1`, NOT on `phase1/nat_semiring`; joint spec §5 names the migration as "module-loader lands → bind on `nat_semiring`"),
- `upstream_blocked:R3-dag-roundtrip-wave1-ready` (W1b prerequisite),
- `upstream_blocked:w1b_emit_ingest_comparator_unlanded` (T-36 follow-up).

**Forbidden:** `R3-dag-roundtrip-*` **`FAIL`** when any prerequisite is not **`PASS`** (companion spec §2.4 cell semantics). The empty-roster wedge on the ladder fixture (no `RoundTripClaim` row authored against `nat_semiring` yet) reads as **`SKIP`**, never **`FAIL`**.

### 2.4 Aux fixture rendering (`phase1-aux/dag_round_trip_mvp1`)

The committed `claim_dag_ingest_round_trip` (`src/v4/test/claim/round_trip/dag_ingest_round_trip.dag:79-83`) is the only `RoundTripClaim` row currently executable through `run_test_claim_round_trip_verdict`. It binds to `dag_round_trip_mvp1`, **not** to the ladder fixture. To keep the matrix honest without conflating fixtures, report it on a parallel aux row:

```text
fixture=phase1/nat_semiring
  rung0: PASS | FAIL  (dag=… rust=… python=… go=…)
  rung1: PASS | FAIL  (rust=…)
  rung2: PASS | FAIL  (rust=… python=… go=…)
  rung3: FAIL          (dag=SKIP — upstream_blocked:claim_nat_semiring_module_roundtrip_not_authored)
  rung4: PASS | FAIL  (rust=… python=SKIP go=SKIP)
blocking_receipt: …

fixture=phase1-aux/dag_round_trip_mvp1
  rung3: PASS | FAIL  (dag=…)
blocking_receipt: …
```

The aux render is **diagnostic**, not the rung-3 close on the ladder. The combined "rung 3 closed on the Phase 1 fixture" line is `phase1/nat_semiring` row only.

### 2.5 Verdict reporting shape (rung 3 row addition)

Extend the matrix with one more line per fixture id:

- Rung 3 row aggregate ∈ {`PASS`, `FAIL`} (no row `SKIP`, per companion §2.4).
- Single `dag` cell ∈ {`PASS`, `FAIL`, `SKIP`}.
- Per joint spec §4.4 rung-split, rung 3 is independent from rung 4 — combined "rungs 3–4 closed" is the conjunction, evaluated only when both rows are renderable on the same fixture id.

### 2.6 TestClaim wiring target (worker implementation)

| Pattern | File (line range, `origin/main` @ `0ace5f8b7`) | Use for |
| ------- | --------------------------------------------- | ------- |
| R3 eval path (W1) | `src/v4/compiler/05_eval.dag:1723-1742` (`run_test_claim_round_trip_verdict`) | Sole `Pass` constructor for R3 dag row in W1; runtime variant at `:1745-1768` |
| R3 input admission | `src/v4/compiler/05_eval.dag:1707-1716` (`eval_round_trip_claim_input_for_verdict`) | Inner well-formed check |
| R3 wave-1 readiness gate | `src/v4/extdeps/languages/dag.dag:3168+` (`dag_round_trip_wave1_authorities_ready`) | The pass-condition (b); re-derived from `dag.dag` per P2 |
| Wave-1 axis ready helpers | `src/v4/extdeps/languages/dag.dag:3132,3150,3160` (`dag_round_trip_lex_ready`, `dag_round_trip_grammar_ready`, `dag_round_trip_normalization_declared`) | Inner-fault disambiguation per §2.2 blocking-receipt table |
| Aux fixture | `src/v4/test/fixture/dag_round_trip_mvp1.dag` | Subject of the only executable W1 R3 row today |
| Aux RoundTripClaim row | `src/v4/test/claim/round_trip/dag_ingest_round_trip.dag:79-83` | `phase1-aux/dag_round_trip_mvp1` row source |
| Ladder-fixture RoundTripClaim (TODO) | `src/v4/test/claim/nat_semiring/rung_3_*.dag` (NEW — worker authors after module-loader lands) | Migrates aux row to `phase1/nat_semiring`; joint spec §5 sequencing |
| W1 host gate script | `scripts/v4-phase1-nat-semiring-rung-gate.sh` (NOT YET extended for rung 3) | Worker extends matrix render after first executable R3 row binds to ladder fixture |

**Worker brief triple (required on the W1-extension PR):**

```text
fixture=phase1/nat_semiring
rung=3
modeling_gap=none (claim authoring) | T-36 only with Modeling DFS worksheet approval (W1b fidelity)
predicate=R3-dag-roundtrip-wave1-ready expected to flip SKIP → PASS|FAIL (W1) or
predicate=R3-dag-roundtrip-fidelity expected to flip SKIP → PASS|FAIL (W1b)
```

---

## 3. SG / substrate work acceptance rule — rung 3 row

Extends companion spec §3 table:

| Work type | Accept when | Reject when |
| --------- | ----------- | ----------- |
| New `RoundTripClaim` row | Binds to `phase1/nat_semiring` module subject AND uses `run_test_claim_round_trip_verdict` (no parallel verdict constructor) AND moves R3 cell on `phase1/nat_semiring` from `SKIP` to `PASS`/`FAIL` | Binds to `phase1-aux/dag_round_trip_mvp1` only (already covered); uses an ad-hoc round-trip predicate outside `05_eval.dag` |
| W1b emit→ingest comparator landing | `R3-dag-roundtrip-fidelity` flips `SKIP` → `PASS`/`FAIL` on **both** fixtures | Comparator landed without rung-3 row movement; comparator that doesn't honor C5 `DagTriviaNormalization` |
| Wave-1 readiness substrate edit (lex/grammar/C5) | Re-runs `dag_round_trip_wave1_authorities_ready` and the rung-3 W1 row stays `PASS` (or flips with a named inner-fault receipt) | Silent edit to `dag_round_trip_lex_ready` / `_grammar_ready` / `_normalization_declared` without rung-3 row re-run on aux fixture |

**Forbidden globally for rung-3 PRs:**

- `Verdict.Pass` on R3 via any constructor other than `run_test_claim_round_trip_verdict` (P2 single-authority).
- Eval reading embedded ready atoms from claim input (the W1 substrate's own P2 comment at `dag.dag:3121` and `:3167+` makes this explicit — eval re-derives, doesn't trust).
- Stub `Pass` returns labelled "rung 3" while W1b emit→ingest comparator is unlanded (would falsely close the ladder standard "round-trip preserved").
- Conflating leaf-model verification (PR #3972 lane) with rung 3 — different predicates, different lanes.

---

## 4. "No rustc-clean as headline" — rung 3 extension

Companion spec §4 applies verbatim. Rung 3 adds:

### 4.1 Forbidden headlines (rung 3)

- "RoundTripClaim Pass" as the primary success criterion **without** naming the W1-vs-W1b stage and whether `dag_round_trip_wave1_authorities_ready` re-derived (W1) or emit→ingest bit-identical (W1b).
- "Round-trip preserved" / "T-36 closed" while W1b emit→ingest comparator is unlanded.
- Closing rung 3 on the `phase1-aux/dag_round_trip_mvp1` row alone — does not satisfy the ladder rung-3 standard on `phase1/nat_semiring`.

### 4.2 Required headlines (rung 3)

Primary success statement must be fixture×rung×stage shaped, e.g.:

- `phase1/nat_semiring: rung3 FAIL (dag=SKIP upstream_blocked:claim_nat_semiring_module_roundtrip_not_authored)` — current baseline.
- `phase1-aux/dag_round_trip_mvp1: rung3 PASS (dag=PASS — W1 wave-1 authorities ready)` — W1 aux row.
- `phase1/nat_semiring: rung3 FAIL (dag=PASS W1 wave-1; W1b emit→ingest comparator unlanded)` — post claim-migration but pre-W1b.

### 4.3 Manager dispatch gate (rung 3)

Do not dispatch "author RoundTripClaim row" / "land W1b fidelity" workers without a brief that names:

1. `fixture=phase1/nat_semiring` (NOT aux — aux is already covered).
2. `rung=3` + stage (`W1 wave-1 ready` or `W1b emit→ingest fidelity`).
3. Predicate id from §2.2 expected to flip.
4. P2 single-authority ack: `run_test_claim_round_trip_verdict` is the sole constructor used.
5. (W1b only) Modeling DFS worksheet approval for the emit→ingest comparator landing per companion §4.4 / joint runner spec §6 W1b row.

---

## 5. Spot-check receipts (vs `origin/main` @ `0ace5f8b7`)

Verified 2026-05-30 with `git show origin/main:<path>`.

| Spec claim | Spot-check | Result |
| ---------- | ---------- | ------ |
| R3 W1 verdict path landed | `src/v4/compiler/05_eval.dag:1723-1742` (`run_test_claim_round_trip_verdict`) | **CONFIRMED** — Pass arm requires both well-formed admission AND `dag_round_trip_wave1_authorities_ready` |
| R3 runtime verdict variant landed | `src/v4/compiler/05_eval.dag:1745-1768` (`run_test_claim_round_trip_verdict_runtime`) | **CONFIRMED** |
| Wave-1 authorities readiness gate | `src/v4/extdeps/languages/dag.dag:3168+` (`dag_round_trip_wave1_authorities_ready`) | **CONFIRMED** — conjunction of lex / grammar / C5 trivia ready |
| C5 trivia DeclaredNormalized | `src/v4/extdeps/languages/dag.dag:3160-3166` (`dag_round_trip_normalization_declared`) | **CONFIRMED** — whitespace + line-comment + block-comment |
| W1 wave-1 readiness scope (not full fidelity) | `src/v4/test/claim/round_trip/dag_ingest_round_trip.dag:3` ("emit→ingest fidelity W1b") + `:80` (claim label "wave-1 readiness") | **CONFIRMED** — explicit W1 / W1b split in landed claim |
| Aux fixture committed | `src/v4/test/fixture/dag_round_trip_mvp1.dag` (referenced at `dag_ingest_round_trip.dag:45`) | **CONFIRMED** |
| No `RoundTripClaim` row on `nat_semiring` | `git grep -l RoundTripClaim src/v4/test/claim/algebra_laws/` returns nothing | **CONFIRMED** — wedge per §2.3 |
| Eval P2 (no embedded-atom trust) | `src/v4/extdeps/languages/dag.dag:3121` + `:3167+` ("eval re-derives lex/grammar/C5 from dag.dag only (P2)") | **CONFIRMED** |
| Joint runner spec §3 rung-3 row | `docs/planning/compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md` §3 | **CONFIRMED** — RoundTripClaim Pass (not Deferred) on at least one fixture-bound claim |
| Joint runner spec §4.4 rung-split | same doc §4.4 | **CONFIRMED** — rung 3 may green independently of rung 4 |
| Joint runner spec §5 module-loader migration | same doc §5 ("dag_round_trip_mvp1-style structural binding once module-loader lands") | **CONFIRMED** — aligns §2.3 wedge receipt |

---

## 6. Current baseline (post-#3960 main)

**Ratification-time expectation:**

- `phase1/nat_semiring` rung 3 = **`FAIL`** with `dag=SKIP` (`upstream_blocked:claim_nat_semiring_module_roundtrip_not_authored`) — no `RoundTripClaim` row binds to the ladder fixture module on `main`. Substrate is ready; claim-authoring follow-up is gated on module-loader (joint spec §5).
- `phase1-aux/dag_round_trip_mvp1` rung 3 W1 = **`PASS`** expected (substrate gate satisfiable when `dag_round_trip_wave1_authorities_ready` returns `true` on current main); W1b = **`SKIP`** with `upstream_blocked:w1b_emit_ingest_comparator_unlanded`.

Expected matrix render (ladder fixture; aux fixture rendered separately per §2.4):

```text
fixture=phase1/nat_semiring
  rung3: FAIL  (dag=SKIP)
blocking_receipt: upstream_blocked:claim_nat_semiring_module_roundtrip_not_authored

fixture=phase1-aux/dag_round_trip_mvp1
  rung3: FAIL  (dag=SKIP)
blocking_receipt: upstream_blocked:w1b_emit_ingest_comparator_unlanded
```

(Row aggregate **`FAIL`** when any cell is not `PASS`, including all-`SKIP` rows, per companion §2.4. **Substrate appendix (non-headline):** when the first `RoundTripClaim` row binds to `nat_semiring` AND `dag_round_trip_wave1_authorities_ready` returns `true`, the W1 stage flips `PASS`; the row aggregate still stays `FAIL` until W1b emit→ingest comparator lands — that's the honest ladder standard, not a regression.)

**Executable receipt anchor:** the W1 baseline lands with the first PR that binds a `RoundTripClaim` row to `phase1/nat_semiring` AND extends `scripts/v4-phase1-nat-semiring-rung-gate.sh` with the rung-3 row. This spec records the **expected** baseline; the executed baseline supersedes on first run.

---

## 7. Manager sign-off

| Decision | Disposition |
| -------- | ----------- |
| Rung 3 binding on `phase1/nat_semiring` (ladder fixture, per companion §1) | **RATIFIED** |
| Single `.dag` cell on rung 3 row (no rust/python/go cells — those are rung 5) | **RATIFIED** |
| Staged predicates `R3-dag-roundtrip-wave1-ready` (W1) + `R3-dag-roundtrip-fidelity` (W1b) | **RATIFIED** |
| Row aggregate `FAIL` while W1b unlanded — honest staging vs ladder standard | **RATIFIED** |
| Aux row `phase1-aux/dag_round_trip_mvp1` reported in parallel as diagnostic, not ladder close (§2.4) | **RATIFIED** |
| `run_test_claim_round_trip_verdict` is sole `Pass` constructor (P2 single-authority) | **RATIFIED** |
| Independent gating from rung 4 per joint spec §4.4 | **RATIFIED** |
| Leaf-model verification (PR #3972) is **NOT** rung 3 — separate lane, separate predicate | **RATIFIED** — explicit out-of-scope (§1) |
| Module-loader-blocked migration of `RoundTripClaim` row from aux to ladder fixture | **PENDING** — Runtime/TestClaim or Compiler Spine follow-up per joint spec §5 |
| W1b emit→ingest comparator landing | **PENDING** — T-36 / W1b; gates the `phase1/nat_semiring rung3` row aggregate `PASS` |
| Host gate script + CI matrix extension for rung 3 | **PENDING** — depends on the first executable R3 row binding to the ladder fixture |
