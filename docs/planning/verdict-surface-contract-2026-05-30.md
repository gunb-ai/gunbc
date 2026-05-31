# Verdict surface contract — T-38 corpus eval × Phase 1a ci_pipeline interpreter

**Authority.** PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §11.1 cross-lane harmonization, prompted by Compiler Spine's coordination flag on CI overhaul PR [#3959](https://github.com/gunb-ai/gunbc/pull/3959).
**Owners.** Runtime/TestClaim (`quick-tern-735`) — drafts; Compiler Spine (`smart-stag-871`) — co-signs.
**Consumer.** Close/Receipt (`sharp-otter-407`) — receives uniform receipts; never branches on producing lane.
**Out of scope.** Verdict semantics under composition (already in `std/verdict.dag`); CI step scheduling (CI overhaul §6); per-target host invocation (W2 brief).

---

## 1. Why this contract exists

Two execution lanes today produce verdicts that flow to the same receipt surface:

- **T-38 corpus eval** (Runtime/TestClaim): emits `TestClaimRun<Node, RuntimeValue>` over the manual claim corpus and (Phase 2) the `nat_semiring` rung 3–4 roster.
- **Phase 1a ci_pipeline interpreter** (Compiler Spine, PR #3959): emits `TestClaimRun<CiPipeline, CiPipeline>` over CI pipeline well-formed and rejection rows (existing `ci.dag workflow_pipeline_*` pattern).

Close/Receipt must consume both without branching on lane. The risk being closed off here: a parallel CI-only verdict carrier emerging in #3959 that Close/Receipt has to special-case.

**Harmonization rule (forbid):** shell JSON, free-form receipt strings, or CI-only verdict types. **One** `TestClaimRun` family, **one** `Verdict` algebra, **one** `VerdictTally` aggregate.

---

## 2. The unified surface

### 2.1 Execution unit

```dag
data TestClaimRun<S, A> = TestClaimRun {
  cache: TestClaimCacheReceipt<S>,
  verdict: Verdict<A>,
}
```

Existing carrier in `src/v4/compiler/05_eval.dag`. No edits required.

### 2.2 Verdict algebra (with W2 extension)

```dag
// src/v4/std/verdict.dag — current
type Verdict<T>
  = Pass
  | Fail     { actual: Outcome<T> }
  | Deferred { actual: Outcome<T>, diagnostic: Diagnostic }

// W2 extension — Fail's payload is the FalsificationReceipt itself, not a
// product of (actual, optional receipt). Single authority for the observed
// outcome (INVARIANTS P2): the receipt is THE Fail payload, and the receipt
// is the only place observed values, divergence locus, and execution evidence
// live. Verdict gains an S parameter solely to thread the subject type into
// the receipt; S is phantom on Pass / Deferred.
type Verdict<S, T>
  = Pass
  | Fail     { receipt: FalsificationReceipt<S, T> }
  | Deferred { actual: Outcome<T>, diagnostic: Diagnostic }
```

**Why receipt-as-payload (not Option-beside-actual).** Two earlier drafts ran aground:
- Draft A attached `falsification: Option<...>` next to `verdict` on `TestClaimRun`. The product made `Pass + Some` representable — a P2 violation flagged on #3961.
- Draft B lifted the slot into `Fail` as `Fail { actual: Outcome<T>, falsification: Option<FalsificationReceipt<S, T>> }`. That deleted `Pass + Some` but kept *two* authorities for the observed outcome — `Fail.actual: Outcome<T>` and the now-absent `FalsificationReceipt.actual: A` (or, with receipt-as-Some, the receipt's own observed values) — and admitted `Fail + None` as a silent un-instrumented state. Codex flagged this on #3961 acc5789d.

The receipt-as-payload shape collapses both axes: `Fail` has exactly one field (the receipt); the receipt has exactly one observed-outcome slot (`actual: Outcome<A>`, see §2.3 — same `Outcome` wrap the current `Fail.actual` carries, so `Rejected{diagnostics}` failures are preserved, not collapsed to a bare `A`); structural rejects construct a receipt with `evidence: EvidenceNone`; there is no un-instrumented `Fail` state because there is nowhere to *not* construct a receipt. `Deferred` keeps `actual: Outcome<T>` because Deferred is the "could not evaluate" state — its outcome is the wrapped reason, not a falsification, and there is only one authority for it within `Deferred`.

The §2.2-vs-§2.3 `None`-meaning ambiguity (cursor 2026-05-30) is moot under this shape: there is no `None` at the verdict layer. The only "no execution evidence" encoding is `FalsificationReceipt { …, evidence: EvidenceNone }`.

**Monoid impact.** `verdict_combine` / `verdict_monoid` generalize from `<T>` to `<S, T>` with shape unchanged: `Pass` identity, `Fail` absorbs, `Deferred` sticky. When two `Fail`s combine, the winner's `receipt` is the join result's `receipt` (matches the existing "Fail's `actual` wins" semantics — receipt is the single field that now carries everything Fail used to put in `actual`).

```dag
// src/v4/compiler/05_eval.dag — TestClaimRun verdict parameter widens to <S, A>
data TestClaimRun<S, A> = TestClaimRun {
  cache:   TestClaimCacheReceipt<S>,
  verdict: Verdict<S, A>,
}
```

No new slot on `TestClaimRun` itself; the carrier change lives entirely in `std/verdict.dag`.

### 2.3 Falsification receipt (Runtime/TestClaim authority)

```dag
data FalsificationReceipt<S, A> = FalsificationReceipt {
  subject:    TestClaimEvalSubject<S>,
  expected:   Outcome<A>,
  actual:     Outcome<A>,
  divergence: ValueDiff<A>,
  evidence:   ExecutionEvidence,
}

type ExecutionEvidence
  = Host        { receipt: EmitHostRunReceipt }    // rung-4 emit→host (W2)
  | Interpreter { trace:   InterpreterTrace }      // Phase 1a in-substrate (spine)
  | None
```

- **`Host`** — emit-vs-eval rung-4 falsifications carry the full `EmitHostRunReceipt` (per joint runner spec §4.2).
- **`Interpreter`** — ci_pipeline rejections that walked the interpreter carry an `InterpreterTrace` (see §3 typing constraint).
- **`None`** — structural `DiagnosticClaim`-style rows with no execution trace.

The sum keeps `FalsificationReceipt<S, A>` **total over both lanes** while preserving lane-specific evidence: Host receipts are not present on CI rejects, Interpreter traces are not present on rung-4 host failures, and Close/Receipt reads neither — only the tally.

### 2.4 Aggregation

```dag
data CorpusEvalReport = CorpusEvalReport { runs: List<TestClaimRun<Node, RuntimeValue>>, ... }
fn corpus_report_tally(r: CorpusEvalReport) -> VerdictTally { ... }
```

Both lanes converge on `VerdictTally` (`pass`, `fail`, `deferred` counts). Close/Receipt consumes **only** `VerdictTally` plus `test_claim_run_claim → label` for row identity. Source lane is invisible at the consumption boundary.

---

## 3. Typing constraints (spine-flagged, ratified)

- **`InterpreterTrace` must be modeled substrate.** Minimum v1 = `{ pinned_eval_call: EvalPin, diagnostics: List<Diagnostic> }` or a comparable typed shape. **Not** a `String` blob. This keeps Close/Receipt mechanical and lets later lenses (idempotency, ownership) consume traces without parsing.
- **`ValueDiff<A>`** stays minimal v1 = `{ path: NodePath, side: DivergenceSide }` (the *locus* of disagreement, not the full value tree). The full outcomes are already in `expected`/`actual` (both `Outcome<A>`); the diff is only well-defined when both sides are `Accepted` — `Rejected`-vs-`Accepted` divergence is encoded by the outcome shapes themselves, not by `ValueDiff`.
- **`EmitHostRunReceipt`** typed per joint runner spec §4.2 (`stdout_bytes` + `stderr_bytes` + `HostExit` + `build_log`; `RuntimeValueParse` is a separate named symbol).

---

## 3.1 Migration note (PR #3958 transitional shape)

W2 worker PR [#3958](https://github.com/gunb-ai/gunbc/pull/3958) (`keen-raven-290`) shipped against the prior Draft B shape: `Fail { actual: Outcome<T>, falsification: Optional<FalsificationReceipt<S, T>> }`. That implementation was at full review criteria when codex flagged the dual-`actual` authority on this PR's `086d6a91`. Per operator manual-merge policy, the worker's branch may merge as-is — but the collapse to receipt-as-payload (this contract's §2.2) is the **target shape** and lands in a follow-up tightening PR.

**Live-consumer census (verified `main` HEAD, cursor 2026-05-30):** an earlier revision of this note understated the blast radius as "W2 substrate + tests only". That was wrong. `Fail { actual: … }` is pattern-matched in **8 files** across `src/v4/` — `grep -l 'Fail {' src/v4/ --include='*.dag'` returns:

| File | Site count | Lane |
| ---- | ---------- | ---- |
| `src/v4/std/verdict.dag` | 3 | substrate (carrier + `verdict_combine` + `verdict_tally_add`) |
| `src/v4/compiler/05_eval.dag` | 8+ | spine (eval pattern-matches, lines 1721–1817) |
| `src/v4/workflow/ci.dag` | 3 | Phase 1a CI workflow (lines 1094, 1111, 1138) |
| `src/v4/test/claim/workflow/pipeline_rejections.dag` | varies | tests |
| `src/v4/test/claim/manual/diagnostic_assert_eval.dag` | varies | tests |
| `src/v4/test/claim/manual/infer_ground_add_mvp.dag` | varies | tests |
| `src/v4/test/claim/manual/dissolution_subsumption_reverification.dag` | varies | tests |
| `src/v4/test/claim/generated/language_behavior_equivalence.dag` | varies | tests |

**~36 total `Fail {` match sites.** The follow-up touches three substrate authors (`std/verdict.dag` for the carrier change, `05_eval.dag` for spine pattern-match updates, `workflow/ci.dag` for Phase 1a CI pattern-match updates) plus 5 test files plus the `EXPECTED_HAND_AUTHORED_TEST` census row. Bounded, but **not** "W2-only": the spine lane (`05_eval.dag`) and the Phase 1a CI lane (`ci.dag`) both consume the carrier and need synchronized edits. Coordination with Compiler Spine manager (`smart-stag-871`) before the follow-up PR opens is required.

The change is **mechanical** per site because `receipt.actual` is `Outcome<A>` (same wrap the current `Fail.actual` carries — see §2.3) — existing `Outcome<T>` values flow through with no extraction or coercion. Per-site shape: rename `Fail { actual: x }` constructions to `Fail { receipt: build_receipt(actual: x, expected: …, …) }` helpers; rename `Fail { actual: x }` pattern-matches to `Fail { receipt: { actual: x, … } }`; drop the `Option` on the `falsification` slot since the new shape has no Option. The semantic step is preserved by `Outcome<A>`; the mechanical step is the carrier rename. Treat the follow-up as a small multi-file PR (Compiler Spine coordination required per the census above), not a semantic redesign.

---

## 4. Lane responsibility table

| Symbol / change | Spine | Runtime/TestClaim | Notes |
| --------------- | ----- | ----------------- | ----- |
| `Verdict<S, T>` parameter widening + `Fail { receipt: FalsificationReceipt<S, T> }` (receipt-as-payload) in `std/verdict.dag` | ✓ | consult | single observed-outcome authority lives in the receipt; no `Fail.actual`, no `Option` (INVARIANTS P2 / single authority) |
| `verdict_combine` / `verdict_monoid` generalization to `<S, T>` | ✓ | — | shape unchanged; Fail-absorbs carries the winning falsification |
| `TestClaimRun<S, A>.verdict` widens from `Verdict<A>` to `Verdict<S, A>` in `05_eval.dag` | ✓ | consult | mechanical follow-on to the verdict.dag change |
| `FalsificationReceipt<S, A>` type declaration | — | ✓ | std-domain home TBD at W2 (`std/test_claim.dag` candidate) |
| `ExecutionEvidence` sum | — | ✓ | |
| `InterpreterTrace` substrate type | ✓ | consult | spine owns interpreter; Runtime co-signs shape |
| `EmitHostRunReceipt`, `HostExit`, `RuntimeValueParse` (symbol decl), `run_emit_host` (Rust row) | — | ✓ | per joint runner spec §4.2 |
| `ValueDiff<A>` | consult | ✓ | |
| `TestClaimRun<CiPipeline, CiPipeline>` Phase 1a wiring | ✓ | — | existing `workflow_pipeline_*` pattern |
| `TestClaimRun<Node, RuntimeValue>` T-38 + rung-4 wiring | — | ✓ | per joint runner spec §4 + W2 |
| `corpus_report_tally` / `VerdictTally` reuse | — | ✓ | already exists; both lanes consume |
| Close/Receipt consumes `VerdictTally` only — never branches on `S` or `A` | — | — | invariant; Close/Receipt enforces |

---

## 5. What this contract does NOT decide

- **Where `FalsificationReceipt<S, A>` lives.** `std/test_claim.dag` is the working candidate; W2 brief settles it.
- **`Interpreter` evidence shape beyond v1.** Lens/synthesis lanes may demand richer traces later; v1 is `{pinned_eval_call, diagnostics}`.
- **Per-target `RuntimeValueParse` rows.** Target Realization (`keen-heron-687`) owns those; Runtime declares the symbol only.
- **CI gate composition** (rung 3 vs rung 4 split etc.). That is in joint runner spec §4.4, not duplicated here.

---

## 6. Sign-off

| Party | Status |
| ----- | ------ |
| Runtime/TestClaim (`quick-tern-735`) | **Proposed** (this revision) |
| Compiler Spine (`smart-stag-871`) | Pending co-sign on PR; ExecutionEvidence sum + InterpreterTrace-as-substrate already pre-acked in-message |
| Close/Receipt (`sharp-otter-407`) | Consult — confirms `VerdictTally`-only consumption boundary |
| PM (`nimble-dove-733`) | Routes to operator via PR #3959 once both manager sigs present |
