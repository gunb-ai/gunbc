# v4 ladder rung spec — Phase 1 (`nat_semiring` × rung 4)

> **Status:** DRAFT — Ladder/Fixture Manager (`zesty-bat-510`, successor to `keen-crab-361`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2 (acceptance predicates per rung); joint runner spec [`compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md`](compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md) §3 row "rung 4" + §4.2 A1–A4 (host-run + falsification receipts).
> **Companion specs:** [`v4-ladder-rung-specs-2026-05-30.md`](v4-ladder-rung-specs-2026-05-30.md) — rungs 0–2 on the same fixture; [`v4-ladder-rung-spec-rung3-2026-05-30.md`](v4-ladder-rung-spec-rung3-2026-05-30.md) — rung 3 (W1 / W1b staged) on the same fixture (landed via PR #4003). **This file is additive**: it does not restate the 9-rung table, fixture ratification (§1 there), §2.4 cell vocabulary, §3 acceptance rule, or §4 headline rule — those apply verbatim and are cited in-place below.
> **Line-number authority:** `origin/main` at `7facb1934` (2026-05-30, post-#3958). Re-verify with `git show origin/main:<path>` if `main` advances.
> **Scope:** Acceptance predicate for **rung 4 only** on `phase1/nat_semiring`. **Rung 3 is out of scope** here — see the rung-3 companion spec for the `RoundTripClaim`-based predicate (cites PR #3960 substrate, NOT PR #3972 leaf-model verification — those are orthogonal lanes). Rung 4 is independently closable per joint spec §4.4 (rung-split gates).

---

## 1. Scope binding

| Field | Value |
| ----- | ----- |
| **Rung** | 4 (THESIS standard "emit runs; output matches `.dag` eval") |
| **Fixture** | `phase1/nat_semiring` (ratified — see companion spec §1) |
| **Fixture subject** | Same module path: `src/v4/test/claim/algebra_laws/nat_semiring.dag` |
| **Phase** | 2 (per parent planning doc §7) — rung 4 closure is independent of rung 3 closure |
| **Target set (W2)** | `rust` only — Python/Go pre-allocated, deferred to Phase 3 per joint spec §4.2 A3 |
| **Closure carrier (modeled)** | `nat_semiring_rung4_gate(report: CorpusEvalReport) -> Bool` (`src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:56-63`) |
| **Closure carrier (host gate)** | Pending W3 — `nat_semiring_rung34_runtime_value_rows: List<TestClaimRun<Node, RuntimeValue>>` is empty (`nat_semiring_rung34_eval.dag:23`); §6 baseline records the wedge as **SKIP**, not **FAIL** (§2.5). |

**Out-of-scope for this spec (named so the boundary is auditable):**

- Rung 3 round-trip predicate — owned by the rung-3 companion spec (cites PR #3960 RoundTripClaim eval path; PR #3972 is leaf-model verification, orthogonal lane).
- R4 Python / Go cells (W2 ships Rust only; rows pre-allocated per joint spec §4.2 A3).
- R5 cross-target equivalence (Phase 3+).
- R6 post-emit algebraic law re-check (Phase 3+).
- Roster row authoring — `nat_semiring_rung34_runtime_value_rows` fill is Runtime/TestClaim follow-up (W3 wedge per joint spec §4.4).

---

## 2. Rung 4 acceptance predicate

**Companion spec §2 (rungs 0–2) defines the cell vocabulary and prerequisite rule.** Apply that contract verbatim:

- Row aggregate ∈ {`PASS`, `FAIL`} only (no row `SKIP`).
- Per-target cells ∈ {`PASS`, `FAIL`, `SKIP`}; `SKIP` when predicate did not execute (upstream not `PASS`, or transport/emit unavailable).
- Forbidden: label `FAIL` for a predicate that did not run.

### 2.1 Phase 1 / W2 target set

| Target id | Toolchain | Rung 4 |
| --------- | --------- | :----: |
| `rust` | `tools/emit_host_runner` (cargo build + run on emitted Rust); `RuntimeValueParse` per Target Realization row | ✓ |
| `python` | (pre-allocated; Phase 3) | — |
| `go` | (pre-allocated; Phase 3) | — |

Python/Go cells exist in the matrix as **`SKIP`** with blocking receipt `phase1/nat_semiring/rung4/<target>_emit_host_unallocated_phase3` until Phase 3 dispatches `run_emit_host_<target>` rows (joint spec §4.2 A3). They are **not** `FAIL`.

### 2.2 R4 predicates

**Question:** Does the host-executed emit artifact produce the same `RuntimeValue` as the interpreter `eval` on the same fixture subject?

| Predicate id | Target | Pass condition | Fail blocking receipt |
| ------------ | ------ | -------------- | --------------------- |
| `R4-rust-emit-equals-eval` | `rust` | A `TestClaimRun<Node, RuntimeValue>` row exists in `nat_semiring_rung34_runtime_value_rows` (`nat_semiring_rung34_eval.dag:23`) for the fixture subject, was constructed via `run_test_claim_emit_vs_eval` (joint spec §4.2 — sole public constructor for rung-4 emit-vs-eval `Fail`), evidence is `Host { receipt: EmitHostRunReceipt }` (`src/v4/std/test_claim_falsification.dag:23-26`), `receipt.exit` ≡ `Accepted { value: Holds { value: ExitOk { code: 0 } }, diagnostics: None }` (`src/v4/std/host_run.dag:30,35-43`), `receipt.logical_run` is `Accepted { value: HostLogicalRun { stdout: HostRunStdout { … } }, … }`, host-stdout parsed via `RuntimeValueParse` for `TargetModel { target: rust }` is `Outcome.Accepted`, and the resulting `Verdict<RuntimeValue>` is `Pass` (`src/v4/std/verdict.dag:38-41`). | `phase1/nat_semiring/rung4/rust_emit_equals_eval_failed` |
| `R4-python-emit-equals-eval` | `python` | (pre-allocated; not executed in W2) | (not used until Phase 3) |
| `R4-go-emit-equals-eval` | `go` | (pre-allocated; not executed in W2) | (not used until Phase 3) |

**Atomic — no per-stage sub-predicates.** Rung 4 is the conjunction of host-exit-ok ∧ stdout-parsable ∧ value-equals-eval, expressed as a single `Verdict.Pass`. Sub-failures surface in the **blocking receipt**, not as additional matrix cells:

| Inner failure (single carrier) | Required blocking receipt |
| ------------------------------ | ------------------------- |
| `emit_host_transport_not_wired` (W3 not landed) — `Verdict.Deferred` | `phase1/nat_semiring/rung4/rust_emit_host_transport_not_wired` |
| `HostExit` `Rejected` (setup/build failure) | `phase1/nat_semiring/rung4/rust_host_setup_failed` |
| `HostExit` `Accepted { Violates }` (non-zero / signaled) | `phase1/nat_semiring/rung4/rust_host_exit_violates` |
| `logical_run` `Rejected` (stdout cap / capture failure) | `phase1/nat_semiring/rung4/rust_host_logical_run_rejected` |
| `RuntimeValueParse` `Rejected` (un-parsable stdout bytes for `target: rust`) | `phase1/nat_semiring/rung4/rust_runtime_value_parse_rejected` |
| Parsed `RuntimeValue` ≠ `eval(InferredTree, …)` outcome | `phase1/nat_semiring/rung4/rust_emit_equals_eval_failed` |

The matrix renders one cell per target; the receipt names the inner fault. This preserves the companion spec §2.4 contract (no row `SKIP`, single receipt for each `FAIL`).

### 2.3 Prerequisite chain

Per the companion spec §2.4 prerequisite rule. R4 extends rungs 0–2:

| Predicate | Runs only when |
| --------- | -------------- |
| `R4-rust-emit-equals-eval` | `R2-rust-compile` = **`PASS`** **AND** `run_emit_host_rust` transport landed (i.e. `emit_host_transport_not_wired` dissolved per `src/v4/compiler/emit_host.dag` 🟡 marker) **AND** `nat_semiring_rung34_runtime_value_rows` populated with the Rust emit-vs-eval row (W3) |

If any prerequisite is not met → **`SKIP`** with one of:

- `upstream_blocked:R2-rust-compile` (rung 2 Rust not `PASS`),
- `upstream_blocked:emit_host_transport_not_wired` (W3 transport pending),
- `upstream_blocked:nat_semiring_rung34_runtime_value_rows_empty` (W3 roster wedge).

**Forbidden:** `R4-rust-emit-equals-eval` **`FAIL`** when any prerequisite is not **`PASS`**. The empty roster (`nat_semiring_rung34_eval.dag:23`) does **not** read as closure — `nat_semiring_rung34_report_has_evidence` is `false` and `nat_semiring_rung4_gate` returns `false` (`nat_semiring_rung34_eval.dag:36-42,56-63`), which is the modeled fail-closed wedge.

### 2.4 Verdict reporting shape (rung 4 row addition)

Extend the companion spec §2.4 matrix with one more line:

```text
fixture=phase1/nat_semiring
  rung0: PASS | FAIL  (dag=… rust=… python=… go=…)
  rung1: PASS | FAIL  (rust=…)
  rung2: PASS | FAIL  (rust=… python=… go=…)
  rung4: PASS | FAIL  (rust=… python=SKIP go=SKIP)
blocking_receipt: <predicate id> | <upstream_blocked:…> | none
```

Rung 4 is reported **independently** of rung 3 per joint spec §4.4 rung-split. The combined "rungs 3–4 closed" line is the **conjunction**, evaluated by Ladder/Fixture **only when both rows are renderable**; until rung 3 spec lands (gated on PR #3972) the rung-4 row stands alone and the conjunction line is omitted.

### 2.5 TestClaim wiring target (worker implementation)

| Pattern | File (line range, `origin/main` @ `7facb1934`) | Use for |
| ------- | --------------------------------------------- | ------- |
| Rung-3/4 corpus eval entry | `src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:26-32` | `run_nat_semiring_rung34_eval` aggregator |
| Rung-4 gate | `src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:56-63` | `nat_semiring_rung4_gate` — consumes `CorpusEvalReport` tally |
| Rung-4 row carrier (empty wedge) | `src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:23` | `nat_semiring_rung34_runtime_value_rows: List<TestClaimRun<Node, RuntimeValue>>` — W3 populates |
| Emit-vs-eval constructor | `src/v4/compiler/emit_host.dag` — `run_test_claim_emit_vs_eval` | Sole public constructor for rung-4 `Verdict.Fail`; must populate `evidence: Host { receipt: EmitHostRunReceipt }` when host ran |
| Host receipt carriers | `src/v4/std/host_run.dag:30,35-43` (`HostExit`, `HostLogicalRun`, `EmitHostRunReceipt`) | `R4-rust-emit-equals-eval` pass-condition fields |
| Falsification receipt | `src/v4/std/test_claim_falsification.dag:23-26,29-35` (`ExecutionEvidence`, `FalsificationReceipt<Subj,A>`) | Verdict `Fail` payload (required, not optional per joint spec §4.2 A4 for the W2 Rust row) |
| Verdict extension | `src/v4/std/verdict.dag:38-41` (`Verdict<S,T>.Fail { actual, falsification: Optional<FalsificationReceipt<Subj,T>> }`) | Wire-up landed; W2 default of `Optional<…>` is met by populated `Present { receipt }` for emit-vs-eval rows |
| Executable host boundary | `tools/emit_host_runner/src/lib.rs` | Rust row implementation under W3 transport wiring |

**Worker brief triple (required on the W3 roster PR, mirroring companion spec §2.5):**

```text
fixture=phase1/nat_semiring
rung=4
modeling_gap=none (W3 roster fill); SG-class only with Modeling DFS worksheet approval
predicate=R4-rust-emit-equals-eval expected to flip from SKIP → PASS|FAIL
```

---

## 3. SG / substrate work acceptance rule — rung 4 row

Extends companion spec §3 table:

| Work type | Accept when | Reject when |
| --------- | ----------- | ----------- |
| Runtime/TestClaim transport wiring | `R4-rust-emit-equals-eval` flips `SKIP` → `PASS` or `FAIL` on `phase1/nat_semiring` (i.e. dissolves `emit_host_transport_not_wired` and populates `nat_semiring_rung34_runtime_value_rows`) | Roster row added without `run_test_claim_emit_vs_eval` constructor (forbidden by joint spec §4.2 A4); transport landed without rung-4 cell movement |
| SG-class fix touching emit | Same as companion §3 (must move a Phase 1 cell — rung 0/1/2/4) | Same |
| New host-row Phase 3 target (Python/Go) | Cell graduates from `SKIP` (Phase 3 unallocated) to executed predicate on this fixture | Adds target row without `run_emit_host_<target>` model |

**Forbidden globally for rung-4 PRs** (per joint spec §4.2 A1/A4 + companion spec §4):

- Pass-fabrication: returning `Verdict.Pass` for emit-vs-eval without going through `run_test_claim_emit_vs_eval`.
- Treating host stdout as raw `String` and matching it (any path bypassing `RuntimeValueParse`).
- `Fail` without `falsification: Present { receipt }` on the emit-vs-eval surface (encoding "no receipt" as `Present { …, evidence: EvidenceNone }` is the W2 single-absence violation — `Absent` is the only no-receipt encoding per `src/v4/std/verdict.dag:32-41`).

---

## 4. "No rustc-clean as headline" — rung 4 extension

Companion spec §4 applies verbatim. Rung 4 adds:

### 4.1 Forbidden headlines (rung 4)

- "host run green" / "emit runs" as the primary success criterion **without** the equals-eval predicate naming.
- "Verdict.Pass on emit-vs-eval" without the host-receipt + RuntimeValueParse provenance in the PR body.

### 4.2 Required headlines (rung 4)

Primary success statement must be fixture×rung×target shaped, e.g.:

- `phase1/nat_semiring: rung4 PASS (rust); python=SKIP go=SKIP (Phase 3 unallocated)`
- `phase1/nat_semiring: rung4 FAIL (rust=FAIL — rust_runtime_value_parse_rejected); python=SKIP go=SKIP`
- `phase1/nat_semiring: rung4 FAIL (rust=SKIP upstream_blocked:emit_host_transport_not_wired)` — W3 wedge state.

### 4.3 Manager dispatch gate (rung 4)

Do not dispatch "wire emit_host" / "fill rung-4 roster" workers without a brief that names:

1. `fixture=phase1/nat_semiring`
2. `rung=4`
3. `predicate=R4-rust-emit-equals-eval` (sole executed rung-4 predicate in W2)
4. Joint runner spec §4.2 A4 ack on the PR body — explicit confirmation that `run_test_claim_emit_vs_eval` is the sole constructor used.

---

## 5. Spot-check receipts (vs `origin/main` @ `7facb1934`)

Verified 2026-05-30 with `git show origin/main:<path>`.

| Spec claim | Spot-check | Result |
| ---------- | ---------- | ------ |
| `HostExit.outcome: Outcome<Witness<ExitOk>>` | `src/v4/std/host_run.dag:30` (`HostExit { outcome }`) + `:35-43` (success arm: `Accepted { Holds { ExitOk { code: 0 } } }`) | **CONFIRMED** — typed-exit phase boundary intact (joint spec §4.2 A2) |
| `EmitHostRunReceipt` carries `target / source_text / exit / logical_run / stderr_bytes / build_log` | `src/v4/std/host_run.dag:46-53` | **CONFIRMED** — `stdout_bytes` named as `logical_run: Outcome<HostLogicalRun>` (joint spec §4.2 A1 stdout typing) |
| `ExecutionEvidence = Host \| Interpreter \| EvidenceNone` | `src/v4/std/test_claim_falsification.dag:23-26` | **CONFIRMED** |
| `FalsificationReceipt<Subj, A>` shape | `src/v4/std/test_claim_falsification.dag:29-35` | **CONFIRMED** — subject typing on receipt, not `Verdict<S,T>` |
| `Verdict<S,T>.Fail { actual, falsification: Optional<…> }` | `src/v4/std/verdict.dag:38-41` | **CONFIRMED** — W2 `Optional` shape; emit-vs-eval rows use `Present { receipt }` |
| `nat_semiring_rung34_runtime_value_rows` empty wedge | `src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:23` | **CONFIRMED** — empty `List<TestClaimRun<Node, RuntimeValue>>`; comment names W3 backlog |
| `nat_semiring_rung4_gate` fail-closed on empty roster | `src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag:36-42,56-63` | **CONFIRMED** — `report_has_evidence` returns `false`, gate returns `false` |
| `emit_host_transport_not_wired` substrate stub | `src/v4/compiler/emit_host.dag:64,79-86` | **CONFIRMED** — 🟡 dissolution marker bound to W3 (`run_emit_host_rust` wiring) |
| `tools/emit_host_runner` executable boundary | `tools/emit_host_runner/src/lib.rs:1-50` | **CONFIRMED** — host-process boundary; `HOST_BUILD_TIMEOUT`, `HOST_RUN_TIMEOUT`, byte cap |
| Joint runner spec §3 rung-4 row | `docs/planning/compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md` §3 | **CONFIRMED** — pass condition aligned with §2.2 here |
| Joint runner spec §4.4 rung-split (rung 4 independent) | same doc §4.4 | **CONFIRMED** — rung 4 may green before rung 3 |

---

## 6. Current baseline (post-#3958 main)

**Ratification-time expectation:** R4-rust-emit-equals-eval is **`SKIP`** because `nat_semiring_rung34_runtime_value_rows` is empty (W3 wedge) and `emit_host_transport_not_wired` has not been dissolved. The empty-roster fail-closed wedge is **not** rung-4 `FAIL` — it is `SKIP` with `upstream_blocked:emit_host_transport_not_wired` (or `…_rows_empty` once transport lands but rows are still absent).

Expected matrix render:

```text
fixture=phase1/nat_semiring
  rung4: FAIL  (rust=SKIP python=SKIP go=SKIP)
blocking_receipt: upstream_blocked:emit_host_transport_not_wired
```

(Row aggregate is **`FAIL`** when any cell is not `PASS` — including all-`SKIP` rows, per companion spec §2.4. Python/Go `SKIP` receipts are `phase1/nat_semiring/rung4/<target>_emit_host_unallocated_phase3`; the **headline blocking receipt** is the lowest unresolved upstream — `emit_host_transport_not_wired`.)

Once W3 lands (transport wired + Rust row in `nat_semiring_rung34_runtime_value_rows`):

- Expected first executed result: `rust=FAIL` with `phase1/nat_semiring/rung4/rust_emit_equals_eval_failed` (or one of the inner receipts in §2.2) — substrate emit gap, not spec gap.
- Operator/CI surface this as **expected substrate gap signaling**, not a regression.

**Executable receipt anchor:** this baseline is recorded on this draft; the first **executed** baseline lands with the W3 PR (Runtime/TestClaim follow-up) and supersedes this row.

---

## 7. Manager sign-off

| Decision | Disposition |
| -------- | ----------- |
| Rung 4 binding on `phase1/nat_semiring` | **RATIFIED** (this doc) |
| `R4-rust-emit-equals-eval` acceptance predicate (§2.2) | **RATIFIED** |
| Phase 1 / W2 target set = `rust` only; Python/Go = `SKIP` (Phase 3 unallocated) | **RATIFIED** per joint spec §4.2 A3 |
| Prerequisite chain (§2.3) — R4-rust requires R2-rust=PASS + transport + roster | **RATIFIED** |
| No-rustc-clean headline extension to rung 4 (§4) | **RATIFIED** |
| Independent gating from rung 3 per joint spec §4.4 | **RATIFIED** |
| Rung 3 acceptance predicate | **DEFERRED** — gated on PR #3972 (Compiler Spine W1 `RoundTripClaim` verdict shape); follow-up spec lands once #3972 merges |
| W3 roster fill (`nat_semiring_rung34_runtime_value_rows`) | **PENDING** Runtime/TestClaim follow-up (joint spec §4.4 W3) — not this lane's authority |
| Host gate script + CI matrix extension for rung 4 | **PENDING** — depends on W3; Ladder/Fixture wires `nat_semiring_rung4_gate` consumption into `scripts/v4-phase1-nat-semiring-rung-gate.sh` matrix once W3 produces a first executed verdict |
