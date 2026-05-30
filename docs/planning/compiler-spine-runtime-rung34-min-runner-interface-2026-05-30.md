# Compiler Spine × Runtime/TestClaim — minimum runner interface (rungs 3–4)

**Authority:** PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §11.4 item 4 — manager pass before worker code.
**Owners:** Compiler Spine (`smart-stag-871`) + Runtime/TestClaim (`quick-tern-735`).
**Fixture:** `src/v4/test/claim/algebra_laws/nat_semiring.dag` (ratified Phase 1 candidate in §7).
**Out of scope here:** target realization rows, SG-class fixes, global corpus (rung 8), lens PR gates (rung 9).

**Provenance.** §1–§3, §5, §6, §7 originated in Compiler Spine's draft on `session/smart-stag-871`. §4.2 and §4.4 carry Runtime/TestClaim amendments (A1–A4 below); §3 rung 3 fail-closed row is unchanged but §4.4 narrows the rung 3 CI staging.

---

## 1. Purpose

Define the **minimum executable runner** needed to gate **rung 3** (parse–emit–parse round-trip) and **rung 4** (emit runs; output matches `.dag` interpreter eval) on the Phase 1 fixture. Workers may implement only against this interface; Ladder/Fixture owns acceptance predicates that *consume* verdicts from it.

---

## 2. Frozen spine carriers (Phase 2 lock)

No rename or field reshuffle during rung 3–4 work. `GroundedProgramGraph` remains prose-only; implementation name stays **`InferredTree`**.

| Carrier | Location | Frozen shape |
| ------- | -------- | ------------ |
| `InferredTree` | `src/v4/compiler/04_infer.dag` | `{ root: Node, facts: Map<Node, InferredFacts> }` |
| `InferredFacts` | same | `{ grounding: CanonicalGrounding, descent: Witness<TerminationProof> }` |
| `ResolvedTree` | `src/v4/compiler/03_resolve.dag` | unchanged — infer input |
| `TestClaimRun<S, A>` | `src/v4/compiler/05_eval.dag` | `{ cache: TestClaimCacheReceipt<S>, verdict: Verdict<A> }` |
| `TestClaimEvalSubject<T>` | `05_eval.dag` | `{ claim, context, tree: InferredTree, input: TestClaimTypedInput<T> }` |

**Spine pipeline contract** (fixture subjects only; no language-specific realization):

```text
CoreNode
  → normalize → resolve → infer → InferredTree
  → emit(TranslateTo { target }) → TargetSource   // T-10; Target Realization supplies rows
  → eval(tree, interpretation, inputs) → Outcome<RuntimeValue>   // T-22
```

**Change control:** any edit to the table above requires Compiler Spine manager sign-off and Ladder/Fixture rerun of rung acceptance predicates.

---

## 3. Rung acceptance predicates (what "minimum" means)

| Rung | Question | Pass condition (fixture-scoped) | Fail-closed |
| ---- | -------- | ------------------------------- | ----------- |
| **3** | Re-parsing emit reproduces source (up to declared normalization)? | `RoundTripClaim` verdict is **`Pass`** (not `Deferred`) for at least one claim bound to the fixture module | `Deferred` / `Fail` blocks rung 3 close |
| **4** | Does emitted Rust run and match interpreter? | `EqualsClaim` (or dedicated emit-vs-eval claim) verdict **`Pass`** where `actual` comes from **host execution** of emit artifact and `expected` from `eval(InferredTree, …)` on the same subject | fabricated `Pass`, shell string match, host stdout consumed as raw string without `RuntimeValueParse` |

Rung 3/4 claims for the fixture live under `src/v4/test/claim/nat_semiring/` (new module tree per §7 Phase 2 output naming).

---

## 4. Minimum runner surface (joint API)

### 4.1 Spine-owned (T-22 / T-8–T-10)

Already landed; workers extend behavior, not signatures:

```dag
// T-9
infer(resolved: ResolvedTree) -> Outcome<InferredTree>

// T-10
emit(tree: InferredTree, target: TargetModel) -> Outcome<TargetSource>

// T-22 — canonical execution entry
eval(tree: InferredTree, interpretation: InterpretationAlgebra, inputs: Inputs) -> Outcome<RuntimeValue>

run_test_claim(subject: TestClaimEvalSubject<Node>) -> TestClaimRun<Node, RuntimeValue>
```

**Spine debt for rung 3 (blocking):** `run_test_claim_runtime_assert` and `run_test_claim_assert_decided` currently return **`Deferred`** for `RoundTripClaim` (`eval_rejected_roundtrip_deferred`). Minimum runner **requires** a modeled round-trip path: `input Node` → emit (dag target) → parse → normalize → resolve → compare to input under `DagTriviaNormalization` (C5). Spine worker owns this; Runtime does not stub `Pass`.

**Spine non-debt for rung 4:** `EqualsClaim` runtime assert already compares `actual` (root eval) to `eval(tree, …, rhs)` — sufficient for **interpreter-side** half of rung 4.

### 4.2 Runtime-owned (T-38 / host harness) — AMENDED

Minimum additions **outside** emit projection tables. Amendments A1–A4 are the Runtime/TestClaim manager's edits to the Spine draft; rationale follows the table.

| Symbol | Responsibility |
| ------ | -------------- |
| `HostExit` | `Outcome<Witness<ExitOk>>` — typed exit, not raw int (A2) |
| `EmitHostRunReceipt` | `{ target: TargetModel, source: TargetSource, exit: HostExit, stdout_bytes: ByteString, stderr_bytes: ByteString, build_log: BuildLog }` (A1) |
| `RuntimeValueParse` | `(target: TargetModel, bytes: ByteString) -> Outcome<RuntimeValue>` — host stdout deserialization to the spine's typed `RuntimeValue`. Per-target rows supplied by Target Realization (cross-lane consult; this interface only declares the function symbol) (A1) |
| `run_emit_host` | `(target: TargetModel, source: TargetSource, fixture_inputs: Inputs) -> Outcome<EmitHostRunReceipt>` — compile + execute emitted artifact for fixture entrypoint. **Rust-only in W2;** Python/Go rows deferred to Phase 3 (A3) |
| `FalsificationReceipt<A>` | `{ subject: TestClaimEvalSubject<Node>, expected: A, actual: A, host_receipt: EmitHostRunReceipt, divergence: ValueDiff<A> }` — structured artifact returned on every non-`Pass` rung-4 verdict (A4) |
| `run_test_claim_emit_vs_eval` | `(subject: TestClaimEvalSubject<Node>, target: TargetModel) -> TestClaimRun<Node, RuntimeValue>` with the additional invariant: a `Fail` verdict carries a `FalsificationReceipt<RuntimeValue>` in the `Verdict` payload (not just a boolean disagreement) (A4) |
| `run_nat_semiring_rung34_eval` | `() -> CorpusEvalReport` — roster: fixture `RoundTripClaim` (rung 3) + one `EqualsClaim`/`CompilesClaim` emit-vs-eval row (rung 4) |

Corpus aggregation reuses `src/v4/test/claim/workflow/testclaim_corpus_runner.dag` (`CorpusEvalReport`, `corpus_report_tally`). CI gate consumption is split per §4.4.

**Amendment rationale.**

- **A1 (stdout typing).** Spine draft typed `EmitHostRunReceipt.stdout: RuntimeValue`, which silently embeds a deserialization step. Splitting `stdout_bytes` from a `RuntimeValueParse` function makes the parser a named symbol with per-target rows owned by Target Realization, and prevents fabricating typed values from raw bytes.
- **A2 (typed exit).** `HostExit` as `Outcome<Witness<ExitOk>>` — host nonzero exit becomes a modeled `Outcome.Err`, not an opaque int the verdict layer has to reinterpret.
- **A3 (Rust-only W2 scope).** §4.3 places rustc/cargo invocation in Runtime, but the rung 5 cross-target gate is Phase 3, not Phase 2. W2 ships Rust only; Python/Go `run_emit_host` rows are pre-allocated symbols, not implementations, until Phase 3 dispatches.
- **A4 (falsification receipt).** PR #3938 §11.1 row 6 names "falsification verdict receipts" as Runtime/TestClaim authority. A bare `Fail` verdict is not a receipt — `FalsificationReceipt<A>` makes the wrong emit auditable post-hoc and is the artifact Self-host/Release will demand at rung 4 close.

### 4.3 Explicit split (no authority bleed)

| Concern | Compiler Spine | Runtime/TestClaim | Target Realization (consult) |
| ------- | -------------- | ------------------- | ---------------------------- |
| `InferredTree` / infer facts stability | ✓ | consumes only | — |
| Round-trip **semantic** compare in eval | ✓ | — | — |
| `TargetAtomRealization` / type-expression projection | — | — | ✓ |
| rustc / cargo / python driver invocation | — | ✓ | — |
| `RuntimeValueParse` per-target rows | — | declares symbol | ✓ supplies rows |
| `TestClaimRun` / `Verdict` shape | ✓ defines | ✓ wires CI + host | — |
| Falsification receipts for wrong emit | consult | ✓ owns artifact | — |

### 4.4 CI gate staging (rung-split) — AMENDED

The Spine draft proposed a single subset gate `fail_count == 0 and deferred_count == 0`. That deadlocks rung 4 progress: until W1 lands, the rung-3 `RoundTripClaim` returns `Deferred` by construction, so a single combined gate would force W2/W3 to wait on W1.

Split the gate by rung roster, both consuming `corpus_report_tally` over disjoint subsets of `CorpusEvalReport`:

| Gate | Roster subset | Pass predicate |
| ---- | ------------- | -------------- |
| `nat_semiring_rung3_gate` | `RoundTripClaim` rows for the fixture | `fail == 0 && deferred == 0` |
| `nat_semiring_rung4_gate` | emit-vs-eval rows for the fixture | `fail == 0 && deferred == 0` |

Both gates fail-closed independently; rung 4 may green before rung 3 if W2/W3 land first (and vice versa). Combined "rungs 3–4 closed" is the conjunction, evaluated by Ladder/Fixture.

---

## 5. Fixture binding (`nat_semiring`)

**Existing corpus** (`algebra_laws/nat_semiring.dag`): six `EqualsClaim` law rows + one `DiagnosticClaim` falsifier — interpreter-identity checks (`lhs == rhs`), **not** emit/host checks. They remain Tier-1 eval rows; they do **not** satisfy rung 4 alone.

**Phase 2 minimum roster** (authored by Ladder worker, executed via this interface):

1. `claim_nat_semiring_module_roundtrip` — `RoundTripClaim` on module subject (or `dag_round_trip_mvp1`-style structural binding once module-loader lands).
2. `claim_nat_semiring_emit_eval_agrees` — `EqualsClaim` or `CompilesClaim` with `run_test_claim_emit_vs_eval` subject pin.

Until (1) clears `Deferred`, rung 3 is **not closable** regardless of global rustc error histogram.

---

## 6. Worker dispatch shape (post sign-off)

| Order | Owner | Brief |
| ----- | ----- | ----- |
| W1 | Compiler Spine | Implement `RoundTripClaim` eval path (ingest⁻¹ on dag target); no `TargetRealization` edits |
| W2 | Runtime/TestClaim | `EmitHostRunReceipt` + `HostExit` + `RuntimeValueParse` symbol + `run_emit_host` Rust row + `FalsificationReceipt` + `run_nat_semiring_rung34_eval` |
| W3 | Ladder/Fixture | Land `nat_semiring/rung_3_4_*.dag` claims + both CI subset gates (§4.4) on `CorpusEvalReport` tally |

W2 and W3 may proceed in parallel with W1 thanks to the §4.4 rung split; the combined-rungs verdict still requires all three.

**Forbidden:** SG-1/SG-2 histogram workers, full manual corpus (rung 8), or `InferredTree` rename without spine sign-off.

---

## 7. Sign-off

| Party | Status |
| ----- | ------ |
| Compiler Spine (`smart-stag-871`) | **Proposed** (draft authored 2026-05-30) — pending re-ack of §4.2/§4.4 amendments |
| Runtime/TestClaim (`quick-tern-735`) | **Ack with amendments A1–A4 and §4.4 rung split** (this revision) |
| Ladder/Fixture (`keen-crab-361`) | Consumes §3 predicates + §4.4 gates when ratifying Phase 2 |

Amendments: spine carrier freeze (§2) requires both manager acks; acceptance predicates (§3) require Ladder/Fixture ack; Runtime-owned surface (§4.2, §4.4) requires Runtime/TestClaim ack with Target Realization consult on `RuntimeValueParse` per-target rows.
