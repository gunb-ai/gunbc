# Compiler Spine × Runtime/TestClaim — minimum runner interface (rungs 3–4)

**Authority:** PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §11.4 item 4 — manager pass before worker code.
**Owners:** Compiler Spine (`smart-stag-871`) + Runtime/TestClaim (`quick-tern-735`).
**Fixture:** `src/v4/test/claim/algebra_laws/nat_semiring.dag` (ratified Phase 1 candidate in §7).
**Out of scope here:** target realization rows, SG-class fixes, global corpus (rung 8), lens PR gates (rung 9).

**Provenance.** §1–§3, §5, §6, §7 originated in Compiler Spine's draft on `session/smart-stag-871`. §4.2 and §4.4 carry Runtime/TestClaim amendments (A1–A4 below); §3 rung 4 fail-closed row extended per A1. Appendix A is Compiler Spine tree verification.

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
  → eval(tree, interpretation, inputs) -> Outcome<RuntimeValue>   // T-22
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
| `HostExit` | `Outcome<Termination>` — canonical POSIX wait status from `v4.extdeps.posix` (`Exited { code: ExitCode }` \| `Signaled { signal: SignalNum }`); unparseable wait → `Rejected`, not ok-only witness (A2) |
| `EmitHostRunReceipt` | `{ target: TargetModel, source: TargetSource, exit: HostExit, stdout_bytes: ByteString, stderr_bytes: ByteString, build_log: BuildLog }` (A1) |
| `RuntimeValueParse` | `(target: TargetModel, bytes: ByteString) -> Outcome<RuntimeValue>` — host stdout deserialization to the spine's typed `RuntimeValue`. Per-target rows supplied by Target Realization (cross-lane consult; this interface only declares the function symbol) (A1) |
| `run_emit_host` | `(target: TargetModel, source: TargetSource, fixture_inputs: Inputs) -> Outcome<EmitHostRunReceipt>` — compile + execute emitted artifact for fixture entrypoint. **Rust-only in W2;** Python/Go rows deferred to Phase 3 (A3) |
| `FalsificationReceipt<S, A>` | `{ subject: TestClaimEvalSubject<S>, expected, actual, divergence: ValueDiff<A>, evidence: ExecutionEvidence }` — `Host` \| `Interpreter` \| `EvidenceNone` per verdict-surface-contract (A4) |
| `run_test_claim_emit_vs_eval` | `(subject, target) -> TestClaimRun<Node, RuntimeValue>` — **only** public constructor for rung-4 emit-vs-eval `Fail`; must return `Verdict<RuntimeValue>.Fail` with required `falsification: FalsificationReceipt<Node, RuntimeValue>` (`evidence: Host { … }` when host ran) (A4) |
| `run_nat_semiring_rung34_eval` | `() -> CorpusEvalReport` — roster: fixture `RoundTripClaim` (rung 3) + one `EqualsClaim`/`CompilesClaim` emit-vs-eval row (rung 4) |

Corpus aggregation reuses `src/v4/test/claim/workflow/testclaim_corpus_runner.dag` (`CorpusEvalReport`, `corpus_report_tally`). CI gate consumption is split per §4.4.

**P2 enforcement (host + receipt).** `EmitHostRunReceipt.exit` is `Outcome<Termination>` from `v4.extdeps.posix` (`Termination = Exited { code: ExitCode } | Signaled { signal: SignalNum }`) — same carrier as `TerminatedProcessState.termination`, not `Outcome<Witness<ExitOk>>` or bare `Int`. Rung-4 falsification is type-enforced on the **landed** verdict carrier `Verdict<A>` (`std/verdict.dag`): W2 extends `Verdict<RuntimeValue>.Fail` with **required** `falsification: FalsificationReceipt<Node, RuntimeValue>` (subject typing stays on the receipt / `TestClaimRun<Node, RuntimeValue>`, not a second `Verdict` type parameter). `run_test_claim_emit_vs_eval` is the sole public entry that may return rung-4 `Fail` and must populate `evidence: Host { receipt: EmitHostRunReceipt }` when the host path ran (structural fails use `EvidenceNone` / `Interpreter`, still required field).

**Amendment rationale.**

- **A1 (stdout typing).** Spine draft typed `EmitHostRunReceipt.stdout: RuntimeValue`, which silently embeds a deserialization step. Splitting `stdout_bytes` from a `RuntimeValueParse` function makes the parser a named symbol with per-target rows owned by Target Realization, and prevents fabricating typed values from raw bytes.
- **A2 (typed exit, POSIX facts).** `HostExit` as `Outcome<Termination>` (`posix.dag`) — nonzero exit codes and signal termination remain structured `ExitCode` / `SignalNum` facts (P2 facts-flow-forward); only zero exit is `Exited`. Do **not** collapse to `Outcome<Witness<ExitOk>>` or bare `Int` — that drops signaled/nonzero semantics at the host boundary.
- **A3 (Rust-only W2 scope).** §4.3 places rustc/cargo invocation in Runtime, but the rung 5 cross-target gate is Phase 3, not Phase 2. W2 ships Rust only; Python/Go `run_emit_host` rows are pre-allocated symbols, not implementations, until Phase 3 dispatches.
- **A4 (falsification receipt).** PR #3938 §11.1 row 6 names "falsification verdict receipts" as Runtime/TestClaim authority. A bare `Fail` verdict is not a receipt — `FalsificationReceipt<Node, RuntimeValue>` (subject inside the receipt) makes the wrong emit auditable post-hoc and is the artifact Self-host/Release will demand at rung 4 close.

**A4 attachment (W2-kickoff default, both managers; see verdict-surface-contract #3961).** Extend **landed** `Verdict<A>.Fail` in `std/verdict.dag` (today `Fail { actual: Outcome<A> }` only) on the **rung-4 surface only** as:

```dag
// W2 substrate row for emit-vs-eval (this doc's scope) — both type params are concrete:
Verdict<RuntimeValue>.Fail {
  actual: Outcome<RuntimeValue>
  falsification: FalsificationReceipt<Node, RuntimeValue>   // required; not Option
}
```

`Verdict<A>` has a **single** type parameter (`verdict.dag:31-34`); an unbound `S` on `Fail` is not implementable and must not reintroduce `Verdict<S, T>`. When `TestClaimRun<S, A>` has `S ≠ A` (general claims), subject typing stays on `FalsificationReceipt<S, A>` at the **`TestClaimRun` / constructor** boundary — not as a second parameter on `Verdict<A>`. This doc's W2 change is the concrete `Node` / `RuntimeValue` row above; broader `S` variance is a separate `#3961` generalization via `TestClaimRun` pairing, not `Verdict<S, A>`.

P2: a rung-4 `Fail` without a receipt is unrepresentable; structural / interpreter rejects use the same carrier with `evidence: EvidenceNone` (or `Interpreter { … }`). `verdict_fail` helpers that lack host context supply the minimal `EvidenceNone` shell. `run_test_claim_emit_vs_eval` is the API gate for rung 4. Runtime owns `FalsificationReceipt` + `ExecutionEvidence`; Spine owns the `Verdict<RuntimeValue>.Fail` extension for this surface.

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
| Compiler Spine (`smart-stag-871`) | **Ack** §4.2 A1–A4 + §4.4 rung-split (2026-05-30); A4 W2 default: `Verdict.Fail` extension |
| Runtime/TestClaim (`quick-tern-735`) | **Ack** with amendments A1–A4 and §4.4 rung split; A4 W2 default agreed |
| Ladder/Fixture (`keen-crab-361`) | Consumes §3 predicates + §4.4 gates when ratifying Phase 2 |

Amendments: spine carrier freeze (§2) requires both manager acks (**met**); acceptance predicates (§3) require Ladder/Fixture ack; Runtime-owned surface (§4.2, §4.4) requires Runtime/TestClaim ack (**met**).

---

## Appendix A. Tree verification (spot-checks on `main` lineage, 2026-05-30)

Claims in §2–§5 were checked against the v4 tree on branch `session/smart-stag-871` (parent `e332fc27b` + this doc). Use these anchors when reviewing; line numbers drift with edits.

| Claim | Verified | Anchor |
| ----- | -------- | ------ |
| `InferredTree` shape | yes | `src/v4/compiler/04_infer.dag:91-94` |
| `infer` signature | yes | `04_infer.dag:420` |
| `emit(tree, target)` | yes | `05_emit.dag:33-36` |
| `eval` → `Outcome<RuntimeValue>` | yes | `05_eval.dag:1663` |
| `TestClaimRun<S,A>` | yes | `05_eval.dag:452-455` |
| `TestClaimEvalSubject` carries `InferredTree` | yes | `05_eval.dag:421-425` |
| `Verdict<T>` = Pass \| Fail \| Deferred (single type param; **not** `Verdict<S,T>`) | yes | `src/v4/std/verdict.dag:31-34` |
| `TestClaimRun.verdict` is `Verdict<A>` | yes | `05_eval.dag:452-455` |
| `RoundTripClaim` → **Deferred** (rung 3 blocker) | yes | `05_eval.dag:1732-1736`, `1793-1797` |
| `run_test_claim` entry | yes | `05_eval.dag:1826-1849` |
| `run_test_claim` used in manual roster wedge | yes | `eval_runtime_mvp.dag:402` (`run_eval_mvp2_test_claim_route`) |
| T-38 `CorpusEvalReport` + tally | yes | `testclaim_corpus_runner.dag:35-68` |
| Roster is **3 static rows**, not full manual corpus | yes | `manual_corpus_roster.dag:20-24` |
| CI: structural bridge only; verdict exec deferred | yes | `ci.yml:288-290`, `ci.yml:342`; `scripts/v4-testclaim-corpus-gate.sh:9-18` |
| `nat_semiring` = EqualsClaim law rows, not emit/host | yes | `algebra_laws/nat_semiring.dag:106-151` |
| `GroundedProgramGraph` not in `src/v4/` | yes | grep empty — name is design prose only (`docs/design-v4-compiler-homomorphism.md`) |
| `dag_ingest_round_trip` RoundTrip deferred to T-38 | yes | `round_trip/dag_ingest_round_trip.dag:3-4` |

**Corrections surfaced (no spec change required):**

- `run_manual_testclaim_corpus_eval()` assembles **pre-built** `TestClaimRun` data; Phase-2 CI must call modeled drivers or document compile-time-only rows until host runner lands.
- §4.4 supersedes the earlier single-gate note: fixture-scoped **rung-split** gates avoid W2/W3 deadlock on W1 `Deferred`.
