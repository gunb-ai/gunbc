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

// W2 extension — bind falsification *inside* Fail so the receipt is structurally
// reachable only from Fail (INVARIANTS P2: illegal states unrepresentable).
// Verdict gains an S parameter solely to thread the subject type into the receipt;
// S is phantom on Pass / Deferred.
type Verdict<S, T>
  = Pass
  | Fail     { actual: Outcome<T>, falsification: Option<FalsificationReceipt<S, T>> }
  | Deferred { actual: Outcome<T>, diagnostic: Diagnostic }
```

**Why inside `Fail`, not beside `Verdict`.** An earlier draft attached `falsification: Option<...>` next to `verdict` on `TestClaimRun`. That product made `Pass + Some` representable — falsifying a passing run is nonsense, but the type permitted it. Lifting the slot into the `Fail` variant deletes the `Pass + Some` state at the type level: `Pass` has no falsification slot, so it cannot carry one.

`Fail + None` is **legal** and **meaningful**, and is the intended encoding for the cases enumerated under `evidence: None` in §2.3 — structural `DiagnosticClaim`-style rejections that decided `Fail` without performing host execution or walking the interpreter, so no execution-evidence artifact exists to attach. The `Option` is therefore not redundant: `Some(receipt)` is "this Fail produced an execution-evidence receipt"; `None` is "this Fail is structural, no receipt is owed". A lens may later require `Some` for the rung-4 host roster specifically (where `None` *would* be fabricating), but that is a per-roster predicate, not a global type-level constraint. The P2 win here is single-axis: `Pass + Some` is unrepresentable; `Fail + None` is admissible by design.

**Monoid impact.** `verdict_combine` / `verdict_monoid` generalize from `<T>` to `<S, T>` with shape unchanged: `Pass` identity, `Fail` absorbs, `Deferred` sticky. When two `Fail`s combine, the winner's `falsification` is the join result's `falsification` (matches the existing "Fail's `actual` wins" semantics; receipt is carried along its sibling field).

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
  expected:   A,
  actual:     A,
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
- **`ValueDiff<A>`** stays minimal v1 = `{ path: NodePath, side: DivergenceSide }` (the *locus* of disagreement, not the full value tree). The full values are already in `expected`/`actual`.
- **`EmitHostRunReceipt`** typed per joint runner spec §4.2 (`stdout_bytes` + `stderr_bytes` + `HostExit` + `build_log`; `RuntimeValueParse` is a separate named symbol).

---

## 4. Lane responsibility table

| Symbol / change | Spine | Runtime/TestClaim | Notes |
| --------------- | ----- | ----------------- | ----- |
| `Verdict<S, T>` parameter widening + `Fail.falsification` slot in `std/verdict.dag` | ✓ | consult | receipt is reachable only via `Fail` — illegal states unrepresentable (INVARIANTS P2) |
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
