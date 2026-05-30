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

// W2 extension — Verdict<T> stays monomorphic (preserves verdict_combine/monoid).
// The falsification receipt is two-parameter (S, A) — see §2.3 — so it cannot
// live inside Verdict<T> without existentializing S. Attach at TestClaimRun<S,A>
// level instead (next subsection).
```

`verdict_combine` / `verdict_monoid` semantics unchanged; no carrier edit to `Verdict<T>`.

```dag
// src/v4/compiler/05_eval.dag — TestClaimRun gains falsification slot
data TestClaimRun<S, A> = TestClaimRun {
  cache:         TestClaimCacheReceipt<S>,
  verdict:       Verdict<A>,
  falsification: Option<FalsificationReceipt<S, A>>,  // populated iff verdict == Fail
}
```

Invariant: `falsification.is_some()` ⇔ `verdict` is `Fail`. Lens enforcement candidate (Phase 3+); v1 is a structural convention.

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
| `TestClaimRun.falsification` carrier extension in `05_eval.dag` | ✓ | consult | `Verdict<T>` stays monomorphic; receipt attaches at run level (S generic) |
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
