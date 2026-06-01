# P5 Worksheet — Bootstrap-evaluator corpus runtime (option ii)

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`).
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §3.5, §6.5 #1, §8.5 #3; upward debt `node://adhoc-f8699326-d69`
> **Predicate:** P5 strict — `src/v4/TASKS.md` "Definition of v4-done" bullet **TestClaim suite passes** (runtime execution, not authoring-time surface only)
> **Operator routing:** option **(ii)** authorized as WORKSHEET 2026-06-01T01; option **(i)** (M1 cargo-clean emitted-subset execution) remains the patient-wait path — **out of scope** for this worksheet
> **Implementation handoff:** Runtime/TestClaim Manager (`neat-hawk-413`) post-§8; coordinates with Compiler Spine on bootstrap binary entry only

### Status (single authority — no contradiction)

| Layer | State |
| ----- | ----- |
| **Worksheet** | **READY-FOR-WORKER-DISPATCH** — §8 closed 2026-06-01 (`proud-fox-405`) |
| **Prerequisites on `origin/main`** | P5 Layer 1+2 CLOSED (#4115 structural bridge); T-38-PR2 verdict SURFACE (#4120); `v4-testclaim-corpus-eval.sh` positive-Y host transport; `testclaim_corpus_runner.dag` + `manual_corpus_eval.dag` authoring witnesses |
| **Implementation dispatch** | **Authorized** after worksheet on `main` — Runtime/TestClaim Manager (`neat-hawk-413`) + Compiler Spine bootstrap entry |

---

## Mechanical dispatch rule

> **No P5 runtime-execution implementation worker may land until this worksheet is complete and Modeling DFS Arbiter–approved (§8 checklist).**
>
> Acceptance is **§4 falsification rows (runtime verdict authority)**, not Layer-2 structural receipt retention, grep-verified const witnesses, or M1 error-count movement.

---

## §10.0-adapted worksheet

```text
Runtime-gate class:        P5-STRICT-RUNTIME (T-38 per-row TestClaimRun execution)
Representative failure:    CI host receipt reports
                             "execution_status": "authoring_time_verdict_surface"
                           — scripts/v4-testclaim-corpus-eval.sh verifies upstream compile +
                             item-registry presence + source-grep of witness_manual_corpus_gate_closed
                           but does NOT execute T-22 eval per corpus row at CI time. The modeled
                           witness_manual_corpus_gate_closed folds const data rows from
                           manual_corpus_node_runtime_value_rows; compile does not gate on Bool truth.
                           P5 strict ("TestClaim suite passes") remains OPEN per TASKS.md + dep graph §3.5.

Immediate local patch:
  - Treat T-38-PR2 verdict surface + authoring-time witness as P5 GREEN.
  - Extend grep/substring structural checks in v4-testclaim-corpus-eval.sh.
  - Path (i): cargo test / run emitted M1 Rust for manual corpus only (patient-wait; not option ii).
  - Add more `data run_*: TestClaimRun = run_test_claim(...)` const rows without runtime re-eval.

Why forbidden:
  - Preserves authoring-time const runs as co-authority for "suite passes" (P2 violation).
  - Path (i) binds verdict authority to M1 emit-Rust host execution — parallel authority to bootstrap
    evaluator per SELF_HOSTING §1 (substrate .dag is authoritative; Rust is bootstrap seed).
  - Grep-verified witnesses are fabrication-adjacent (M5): witness present in source ≠ verdict executed.
  - Extending shell structural receipts calcifies the negative-Y host transport P5 Layer 2 dissolved.

DFS path:
  std/ authority (CONSUME — no new parallel verdict types):
    - v4.std.runtime — EvalContext, RuntimeValue, InterpretationAlgebra
    - v4.std.verdict — Verdict, VerdictTally, aggregate_verdicts
    - v4.std.verification — TestClaim, test_claim_label
  extdeps/runtime authority:
    - v4.extdeps.runtimes.v4_evaluator — v4_evaluator_runtime_id, v4_evaluator_runtime_wave1()
  workflow authority:
    - v4.workflow.bootstrap — bootstrap_projection_inputs.runtime_model (pin = v4_evaluator_runtime_wave1)
  compiler stage (CONSUME — do not fork eval):
    - v4.compiler.eval — run_test_claim, eval_test_claim_subject, TestClaimEvalSubject, TestClaimRun
  test-claim workflow (AMEND — runner is the positive authority):
    - v4.test.claim.workflow.testclaim_corpus_runner — CorpusEvalReport, roster fold
    - v4.test.claim.manual.manual_corpus_roster — interim explicit roster (dissolve-on T-19 reflection)
    - v4.test.claim.workflow.manual_corpus_eval — gate witness must read RUNTIME-produced report
  CI binding (AMEND host transport only):
    - v4.workflow.ci — TestClaimCorpusEvalCommand + ci_upsert_testclaim_corpus_eval_*
    - scripts/v4-testclaim-corpus-eval.sh — thin bootstrap-binary invoke; dissolve structural grep gate
  existing scaffold/dissolution notes:
    - testclaim_corpus_runner.dag L4–5, manual_corpus_eval.dag L7–11, ci.dag L145–146 — forward clause
      names bootstrap-evaluator corpus runtime; amend on implementation PR (forbidden: strip 🟡 without amend)

Deepest unsound boundary:
  run_manual_testclaim_corpus_eval() packages pre-authored const TestClaimRun rows
  (manual_corpus_node_runtime_value_rows) instead of invoking run_test_claim at execution time
  with evaluator_pin bound to bootstrap's v4_evaluator_runtime_wave1() projection.

Systemic fix (single-authority fact):
  BootstrapEvaluatorCorpusRuntimeEval — modeled execution plan that, for each rostered claim:
    (1) builds TestClaimEvalSubject via eval_test_claim_subject(claim, context, tree,
        evaluator_pin: v4_evaluator_runtime_id) where context derives from
        bootstrap_projection_inputs.runtime_model (v4_evaluator_runtime_wave1);
    (2) invokes run_test_claim(subject) at RUNTIME (bootstrap fixed-point binary or
        evaluator harness — host transport is invoke-only);
    (3) folds results into CorpusEvalReport (existing type — no parallel report authority).
  Canonical declaration home: v4.test.claim.workflow.testclaim_corpus_runner (amend
  run_manual_testclaim_corpus_eval → execute_manual_testclaim_corpus_via_bootstrap_evaluator
  or equivalent — one entry symbol for CI host transport).
  CI receipt authority: runtime-produced CorpusEvalReport + manual_corpus_gate(report) evaluated
  at execution time (not source-grep of witness_manual_corpus_gate_closed).

Non-goals:
  - Path (i) M1 emitted-Rust cargo-test as sole runtime authority for this gate (separate patient-wait track)
  - Re-litigating P5 Layer 2 positive-Y CiUpsertStep structural bridge (#4115 CLOSED)
  - Full manual/*.dag item-registry reflection (T-19) — interim explicit roster retained with dissolution mark
  - T-22 cache-hash feature gate (T22-EVAL-CACHE-HASHES) — not required for corpus verdict execution
  - Duplicating eval logic outside 05_eval.dag (hand-rolled derived operation — Practice 10)
  - Claiming P5 GREEN on authoring_time_verdict_surface receipt shape after runtime lands

Falsification probe:
  See §4 table — (F1)–(F6) mandatory for implementation PROVEN.

Metric allowed only as secondary:
  manual corpus row count; CI wall-clock — report after F1–F6, not acceptance.
```

---

## §1 Single-authority fact

| Field | Value |
| ----- | ----- |
| **Fact name** | `BootstrapEvaluatorCorpusRuntimeEval` — runtime T-22 eval of rostered manual claims pinned to bootstrap's `v4_evaluator_runtime_wave1()` |
| **Negative authority (retire as pass condition)** | Authoring-time const `data run_*: TestClaimRun` rows as the sole corpus verdict source; `execution_status: authoring_time_verdict_surface` as CI pass; source-grep of `witness_manual_corpus_gate_closed` without runtime Bool evaluation |
| **Positive authority (consume + amend)** | `run_test_claim` + `eval_test_claim_subject` in `v4.compiler.eval`; `CorpusEvalReport` fold in `testclaim_corpus_runner.dag`; `v4_evaluator_runtime_id` pin from `v4.extdeps.runtimes.v4_evaluator`; `bootstrap_projection_inputs.runtime_model` from `v4.workflow.bootstrap` |
| **Host transport posture** | Thin invoke of bootstrap fixed-point binary (or staged evaluator test binary) that executes the modeled runner entry and emits structured JSON receipt — **must not** re-own `gunbc compile` loops or substring structural proofs as acceptance |
| **Explicitly NOT** | A parallel `CorpusRuntimeReport` type; a second eval interpreter; M1-emitted-Rust verdict execution as the authority named by this worksheet |

### 1.1 Option (ii) vs option (i) (routing receipt)

| Path | Authority | This worksheet |
| ---- | --------- | -------------- |
| **(i) M1 emitted subset** | Host runs cargo test / binary over generated `src/v4_*.rs` | **Out of scope** — patient-wait; does not dissolve SELF_HOSTING evaluator authority |
| **(ii) Bootstrap evaluator** | Bootstrap fixed-point + `v4_evaluator` runtime executes `.dag` claims via T-22 | **In scope** — operator-authorized 2026-06-01T01 |

### 1.2 Layering vs P5 structural bridge (#4115)

| Layer | Closed by | This worksheet |
| ----- | --------- | -------------- |
| Layer 2 structural | #4115 — positive-Y `CiUpsertStep`, shell bridge deleted | **Prerequisite only** — do not regress |
| Layer 3 runtime | **This worksheet** | Closes `node://adhoc-f8699326-d69` when F1–F6 PROVEN |

### 1.3 Existing symbols (read-only in worksheet PR)

```text
run_manual_testclaim_corpus_eval          # amend body — runtime eval, not const-row repackage
manual_corpus_node_runtime_value_rows     # interim roster — claims/subjects, not pre-baked runs
manual_corpus_gate / witness_manual_corpus_gate_closed  # must close on RUNTIME report
v4_evaluator_runtime_id / v4_evaluator_runtime_wave1
bootstrap_projection_inputs.runtime_model
ci_upsert_testclaim_corpus_eval_execution # host binding — amend transport only in impl PR
```

---

## §2 Composition boundary

| Edge | Authority | Use |
| ---- | --------- | --- |
| T-22 eval primitives | `v4.compiler.eval` | `run_test_claim`, `eval_test_claim_subject` — sole eval authority |
| Bootstrap runtime pin | `v4.workflow.bootstrap` + `v4.extdeps.runtimes.v4_evaluator` | `evaluator_pin` + `EvalContext` from `runtime_model` |
| P5 Layer 2 CI | `v4.workflow.ci` + #4115 worksheet | Existing `TestClaimCorpusEvalCommand` rows — amend inputs/transport only |
| Verdict surface (interim) | `manual_corpus_eval.dag` | Retained until runtime receipt supersedes; amend witness binding in impl PR |

**Forbidden:** Importing `compute_fabric` / `cache_interface` catalogs into runner — corpus runtime does not need new cache vocabulary for first close.

---

## §3 Forbidden patterns (implementation PR grep discipline)

| Pattern | Why forbidden |
| ------- | ------------- |
| `execution_status.*authoring_time` as final CI pass | Layer 3 not closed |
| Host-owned `gunbc compile` in `v4-testclaim-corpus-eval.sh` | P5 Layer 2 dissolved compile authority |
| New `Corpus*Report` parallel to `CorpusEvalReport` | P2 duplicate authority |
| Per-claim shell `cargo test` over `src/v4_*.rs` as **this gate's** authority | Path (i); not option (ii) |
| `data run_*: TestClaimRun = run_test_claim(...)` added to close gate without runtime harness | Authoring-time co-authority |
| Stripping 🟡 on runner/ci.dag without structural-slice → runtime-slice amendment text | Dissolution discipline |

---

## §4 Falsification table (worker PROVEN rows)

| ID | Probe | Receipt |
| -- | ----- | ------- |
| F1 | CI JSON receipt `execution_status` is `runtime_verdicts` (not `authoring_time_verdict_surface`) | Implementation PR attaches host receipt JSON + CI step summary |
| F2 | Runtime `CorpusEvalReport` produced by executing modeled runner entry (not repackaging const rows) | Hermetic test or integration log: entry invoked via bootstrap/evaluator binary; `manual_corpus_node_runtime_value_rows` supplies **claims**, not pre-built `TestClaimRun` verdicts |
| F3 | Subjects carry `evaluator_pin == v4_evaluator_runtime_id` | Structural grep or `.dag TestClaim` over eval subjects in implementation PR |
| F4 | `manual_corpus_gate(report)` evaluated at **runtime** on produced report closes CI step | Bool true in receipt; fail-closed on any Fail/Deferred row |
| F5 | Host transport does **not** treat source-grep of `witness_manual_corpus_gate_closed` as sole pass | `rg` on script: no pass exit solely from witness line presence without runtime eval |
| F6 | Dissolution marks amended on `testclaim_corpus_runner.dag`, `manual_corpus_eval.dag`, `ci.dag` L145–146 | Mark text names runtime-slice trigger; forbidden: strip 🟡 without amend |

---

## §5 Landing order (implementation — not worksheet-only PR)

```text
1. Model execute_manual_testclaim_corpus_via_bootstrap_evaluator (or amend run_manual_*) in
   testclaim_corpus_runner.dag — runtime run_test_claim per roster row, v4_evaluator pin.
2. Amend manual_corpus_eval.dag witness to bind runtime-produced report (or sibling runtime witness).
3. Wire bootstrap/evaluator harness entry (Compiler Spine + Runtime/TestClaim) — binary invokes (1).
4. Amend scripts/v4-testclaim-corpus-eval.sh — invoke harness; emit F1 receipt; delete structural-only pass.
5. Amend ci.dag / dissolution comments per §1.4 pattern from P5 structural worksheet.
6. Attach §4 PROVEN rows; cite worksheet in PR body.
```

**Lane split:** Runtime/TestClaim Manager owns 1–2–4–6; Compiler Spine owns 3 (bootstrap binary entry) in coordination.

---

## §6 Downstream worker brief (dispatch after §8)

Implement `BootstrapEvaluatorCorpusRuntimeEval` per §1–§5 on `main` after this worksheet merges. Do not claim P5 strict GREEN until F1–F6 PROVEN and `src/v4/TASKS.md` "TestClaim suite passes" receipt is attached. Coordinate with keen-heron-687 (TR) only on shared `Target*` facts if roster claims pull cross-target realization — not per this worksheet's core path.

**Worksheet-only PR non-goals:** `05_eval.dag` eval logic changes (consume only), `src/v2/`, generated `src/v4_*.rs`, bootstrap fixed-point proof landings beyond harness entry.

---

## §7 Non-goals

- Path (i) M1 emitted-Rust runtime as authority for this gate
- P5 Layer 2 re-litigation or shell bridge restoration
- Full T-19 item-registry reflection for manual corpus
- CI evaluator-runtime migration (#4091 YAML→evaluator) — coupled long-pole, not blocking F1–F6 for corpus slice
- SG-class emitter worksheets

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Single-authority fact: `BootstrapEvaluatorCorpusRuntimeEval` (runtime `run_test_claim` via bootstrap evaluator pin — §1)
- [x] Option (ii) vs (i) routing honored — (i) explicit non-goal (§1.1)
- [x] Distinct from P5 Layer 2 structural bridge (#4115) (§1.2)
- [x] Spot-fix forbidden: authoring-time const runs, grep witness pass, path (i) as this gate (§3)
- [x] Falsification table §4 (F1–F6) accepted
- [x] Landing order §5 + lane split accepted (Runtime/TestClaim + Compiler Spine)
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`, Modeling DFS Arbiter per #4137 §11.2)

---

## Related artifacts

- `docs/planning/v4-p5-structural-bridge-replacement-worksheet-2026-05-30.md` — Layer 2 (CLOSED)
- `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` — §3.5, §8.5 #3
- `src/v3/SELF_HOSTING.md` — bootstrap substrate authority
- `src/v4/test/claim/workflow/testclaim_corpus_runner.dag` — runner amend target
- `src/v4/workflow/bootstrap.dag` — `bootstrap_projection_inputs.runtime_model`
- `scripts/v4-testclaim-corpus-eval.sh` — host transport amend target
- `src/v4/TASKS.md` §T-38 — close condition cross-ref
