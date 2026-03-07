# SDLC Ambient + Intellectual Pipeline Roadmap

**Status**: Implementation in progress
**Date**: 2026-03-06
**Purpose**: Canonical end-to-end roadmap from current SDLC state to:
1. production SDLC,
2. ambient trusted operation,
3. intent-driven intellectual workflows,
4. language-level lifecycle control.

**Related docs**:
- `TODO/sdlc.md`
- `docs/design/sdlc/mega-modeling-design.md`
- `docs/design/sdlc/domain-modeling-comprehensive.md`
- `docs/design/sdlc/ambient-feedback-model.md`
- `docs/design/modeling/intellectual-pipeline-kernel.md`
- `docs/design/horizon/h12-managed-lifecycle-control.md`

## 1. Document Contract

This document is the assignment surface for the longer program beyond Day 1 SDLC
activation. It exists to:

1. pin down design decisions early,
2. record the resolved design decisions and any remaining non-blocking assumptions,
3. break the work into parallelizable tracks with concrete acceptance criteria,
4. reduce improvisation during implementation.

If task execution conflicts with the design decisions below, this document should
be updated first.

## 2. Current Position

Use [TODO/sdlc.md](../../../TODO/sdlc.md) as the current implementation tracker.
As of 2026-03-06, the branch position is:

1. SDLC compilation path proven.
2. unit-test dry run proven.
3. local real-mode design-stage path partially live.
4. full cloud/ambient operation not yet proven.
5. generalized intellectual-kernel work: DSL types and mapping DONE, runtime proof pending.
6. managed lifecycle control: parser + types + ensure_absent DONE, lower/codegen pending.

The practical consequence:

- the substrate is real,
- Track B (ambient feedback) has types, interface, 3 providers, ingestion/response logic,
- Track C (intellectual kernel) has inquiry types, kernel artifacts, SDLC mapping, ML exemplar, intent expansion,
- Track D (lifecycle control) has compiler support (`managed` keyword), lifecycle types, `ensure_absent` pattern,
- the full ambient product is not yet end-to-end proven.

## 3. Design Decisions Resolved Now

These are the most important decisions to make implementation straightforward.

### R1. Correctness is store-driven, not signal-driven

Signals are durable accelerators. They are not the sole source of truth.

Operational rule:

1. webhooks, schedulers, and reviewers emit durable signals,
2. workers and reconcilers consume signals for low latency,
3. anti-entropy scans over authoritative state remain mandatory.

This preserves the SDLC reliability contract already described in the mega design.

### R2. SDLC remains a concrete specialization

`IssueLifecycleStage` remains valid for SDLC. We are **not** replacing current
SDLC with a generic global stage enum.

Generalization happens through:

1. artifact kinds,
2. evidence,
3. critique,
4. obligations,
5. conclusion criteria.

### R3. GitHub is an adapter surface, not the ontology

Issues, PRs, labels, comments, and reviews are one projection of internal state.
They must not become the canonical meaning layer.

### R4. Human feedback becomes a first-class obligation

A stray issue comment, PR comment, or review thread is not just text. It is a
potential critique event that may create a durable obligation.

The system is not ambient in a trustworthy way until it can:

1. ingest feedback,
2. persist it,
3. act on it,
4. close it explicitly.

### R5. Managed lifecycle is distinct from execution lifecycle

Current run-scope lifecycle:

```text
acquire -> use -> release
```

Needed managed lifecycle:

```text
ensure_present -> disable -> drain -> destroy -> verify_absent
```

This is a separate language/compiler concern, not a new global workflow stage
machine.

### R6. Two destruction modes are required

1. **Graceful**: `disable -> drain -> destroy -> verify_absent`
2. **Brutal**: `destroy -> verify_absent`

`Brutal` is always explicit, never default.

### R7. One non-SDLC exemplar is required before deep runtime generalization

The first required second exemplar will be:

**ML pipeline design / investigation**

Reason:

1. it is clearly non-SDLC,
2. it is evidence-heavy,
3. it strongly exercises research/design/evaluation concepts,
4. it avoids the trap of re-describing software delivery as “general work.”

## 4. Design Questions Resolved Now

The main design blockers are no longer open. They are resolved in the linked
canonical docs below.

| ID | Resolved question | Resolution |
|---|---|---|
| DG1 | Final DSL surface for managed lifecycle | Use a `managed {}` block on resource-like units with explicit `destroy_support`, `ensure_present`, `verify_present`, `disable`, `drain`, `destroy`, and `verify_absent`. Source: `docs/design/horizon/h12-managed-lifecycle-control.md`. |
| DG2 | Generic intent object shape | Introduce `InquiryIntent` as the domain-neutral intake object. Keep `IntentSheet` as the SDLC adapter surface. Source: `docs/design/modeling/intellectual-pipeline-kernel.md`. |
| DG3 | Feedback ingestion normalization | Capture provider-shaped feedback first, then classify into critique/approval. Durable obligation identity is source-object-shaped, not LLM-finding-shaped. Source: `docs/design/sdlc/ambient-feedback-model.md`. |
| DG4 | Section/lane disable model | Runtime on/off control targets managed units with stable ids. Workflow sections become `StageGate` / `LaneGate` managed units, not raw config flags. Source: `docs/design/horizon/h12-managed-lifecycle-control.md`. |
| DG5 | ML exemplar scope | The required second exemplar is a bounded offline ML investigation, not full MLOps. Source: `docs/design/modeling/intellectual-pipeline-kernel.md`. |

Blocking ambiguities at the design level: none.

## 5. End State Milestones

### M1. Pilot Production SDLC

One repo, cloud-backed, durable ledgers/signals, full stage chain, human-supervised.

### M2. Ambient Trusted SDLC

Comments/reviews become obligations, workers respond durably, ambient operation is
credible.

### M3. Intent-Driven Intellectual Workflow

A user can state high-level intent and the system can route through research,
design, evidence, critique, revision, and conclusion.

### M4. Language-Level Lifecycle Control

Any managed unit the system can turn on can also be disabled, drained, destroyed,
and verified absent through compiler-aware workflows and tests.

## 6. Work Tracks

The program is broken into four tracks. Track A is the shortest path to real
value. Tracks B-D deepen trust and generality.

### Track A — Finish Production SDLC

This reuses the existing SDLC lane tasks and is the immediate prerequisite for
everything else.

| ID | Task | Size | Deps | Acceptance |
|---|---|---|---|---|
| A1 | Complete `S-11`: local e2e idea -> design -> design-review with real GitHub issue and outcome ledger. | L | current branch | Existing `S-11` acceptance holds. |
| A2 | Complete `S-12`: real agent provider path in local mode. | L | A1 | Agent branch and PR are created from live run. |
| A3 | Complete `S-13`: real code-review path with review artifact posted. | M | A2 | PR review output posted and stage transition is correct. |
| A4 | Complete `S-14`: testing path with structured pass/fail outcomes. | M | A3 | Testing either advances to done or records failure with retry-safe semantics. |
| A5 | Complete `S-15`: full local progression idea -> done. | XL | A4 | One issue traverses full lifecycle without manual label editing. |
| A6 | Complete `S-16`: GCS/GCS-backed stores verified under cloud profile. | L | A5 | CAS/idempotency paths proven against real cloud backends. |
| A7 | Complete `S-17` and `S-18`: Cloud Run deployment + Pub/Sub signal delivery. | L | A6 | Worker deploys, receives durable signals, processes them. |
| A8 | Complete `S-19`: multi-worker contention and exactly-once proof. | L | A7 | 3+ workers process workload with no duplicate stage execution. |

### Track B — Ambient Trusted SDLC

This track turns SDLC from “workflow that runs” into “system you can trust to keep
up with real human interaction.”

| ID | Task | Size | Deps | Acceptance |
|---|---|---|---|---|
| AS1 | Make worker consumption signal-aware. Add signal consume/ack path so `WorkReady`, `ApprovalGranted`, and retry signals actively accelerate worker pickup instead of existing only at webhook/reconciler edges. | L | A7 | Worker can process via SignalStore and still remain correct when signals are missing. |
| AS2 | Implement feedback ingestion model. Map issue comments, PR comments, and review threads into typed critique events with stable idempotency keys and source references per `ambient-feedback-model.md`. | M | A5 | GitHub feedback is persisted and normalized according to the canonical feedback model. |
| AS3 | Add feedback obligation persistence. Extend ledger/model so critique events can create durable outstanding obligations. | L | AS2 | Feedback obligations survive restarts and are queryable by source comment/review id. |
| AS4 | Add feedback rediscovery / anti-entropy scan. Periodically re-scan unresolved comments/reviews so webhook loss causes delay, not loss. | M | AS3 | Lost webhook simulation still leads to eventual rediscovery of unresolved critique. |
| AS5 | Implement response loop. Worker/agent can post linked responses or follow-up actions that close feedback obligations explicitly. | L | AS3 | A reviewer comment can be marked addressed only after a response artifact or code change result is linked. |
| AS6 | Add approval and critique coexistence rules. Approval must not implicitly close critique obligations; critique must not be mistaken for approval. | M | AS2 | Conflicting human signals are modeled explicitly and tested. |
| AS7 | Add ambient execution report views. Reports include open feedback obligations, aged unresolved critiques, and recent closures. | M | AS3 | Machine-readable reports expose feedback backlog and closures. |
| AS8 | Run ambient soak test. Live repo test where comments/reviews arrive during execution and are eventually addressed. | L | AS1-AS7 | Soak run demonstrates end-to-end durable handling of mid-flight human input. |

### Track C — Intent-Driven Intellectual Kernel

This track proves the system can generalize beyond SDLC without collapsing into
vague abstraction.

| ID | Task | Size | Deps | Acceptance |
|---|---|---|---|---|
| IK1 | Introduce generic intent object. Add `InquiryIntent` with constraints, desired evidence, autonomy policy, and conclusion criteria, while leaving `IntentSheet` as the SDLC adapter surface. | M | current branch | Shared kernel model and intake surfaces use one authoritative generic intent definition. |
| IK2 | Define domain-neutral artifact/evidence/critique/obligation model. Promote the kernel concepts from the design note into a concrete modeling surface. | L | IK1 | Canonical model exists and is independent of GitHub-specific semantics. |
| IK3 | Define SDLC-as-specialization mapping. Show exact mapping from SDLC artifacts to kernel artifacts/obligations without changing the runtime yet. | M | IK2 | Mapping document is explicit and lossless enough to drive future migration. |
| IK4 | Model bounded ML exemplar. Implement the offline ML investigation exemplar defined in the kernel doc: problem statement, data plan, experiment evidence, critique, conclusion. | L | IK2 | ML exemplar fits the kernel without SDLC-only vocabulary and without expanding into full MLOps. |
| IK5 | Build one minimal non-SDLC prototype path. This can be dry-run only; the goal is proof of decomposition, not production launch. | L | IK4 | User can submit a high-level ML-style intent and receive research/design/evidence artifacts in a bounded loop. |
| IK6 | Add intent-expansion workflow. System can take high-level intent and derive explicit questions, plans, and required evidence before implementation work begins. | L | IK1-IK3 | High-level requests no longer need to be pre-broken into issue-stage terms by the user. |
| IK7 | Add conclusion semantics richer than success/failure. Distinguish adopted, rejected, deferred, needs-more-evidence, and blocked conclusions. | M | IK2 | Conclusion artifacts and reports preserve these outcomes explicitly. |
| IK8 | Revisit runtime generalization boundary. Only after IK5 succeeds, decide what moves from SDLC-specialized runtime to shared kernel runtime. | M | IK3-IK7 | Explicit migration decision made with non-SDLC pressure in hand. |

### Track D — Managed Lifecycle Control

This track makes “turn it on” imply “can turn it off” at the language/compiler
level.

| ID | Task | Size | Deps | Acceptance |
|---|---|---|---|---|
| LC1 | Implement DSL surface for managed lifecycle. Add the `managed {}` lifecycle surface defined in H12, including explicit destroy support and verification verbs. | L | current branch | Managed lifecycle surface exists in the language/compiler as designed. |
| LC2 | Implement managed unit granularity for disable/drain. Add compiler-aware targeting for stage/lane gates, ingress units, and whole-service units. | M | LC1 | Arbitrary sections of the pipeline can be disabled/drained through managed unit ids rather than ad-hoc config. |
| LC3 | Add `ensure_absent` pattern. Introduce the opposite of upsert at the DSL pattern layer. | M | LC1 | `dsl/std/patterns.dag` contains a first-class ensure-absent pattern with tests. |
| LC4 | Lower/IR support for managed lifecycle. Preserve lifecycle verbs as explicit graph constructs through lower/IR. | L | LC1, LC3 | Lowered graphs represent graceful and brutal destroy paths directly without handwritten side channels. |
| LC5 | Codegen lifecycle commands. Generated CLIs/workflows expose `ensure`, `disable`, `drain`, `destroy`, `status`. | L | LC4 | Managed units get generated lifecycle commands, not just create/apply. |
| LC6 | Testgen lifecycle obligations. Generate tests for graceful destroy, brutal destroy, and verify-absent for managed units. | L | LC4 | Lifecycle tests are compiler-derived, not handwritten one-offs. |
| LC7 | Apply managed lifecycle to SDLC runtime units. Model webhook ingress, worker, reconciler, and signal ingress using the new surface. | L | LC2, LC5, LC6 | SDLC sections can be disabled/drained/destroyed explicitly and safely. |
| LC8 | Replace handwritten rollback/cleanup paths. Delete ad-hoc removal logic where managed lifecycle now covers the same semantics. | M | LC7 | Cleanup logic is centralized in lifecycle workflows and tests. |

## 7. Suggested Execution Order

Recommended order:

1. **Track A first** — make SDLC real in production.
2. Start **AS2** and **LC1** early in parallel as implementation lanes with
   resolved design.
3. Once A7/A8 are close, execute **Track B** for ambient trust.
4. Start **IK1-IK4** once Track B semantics are stable in the runtime and ledger.
5. Delay **LC4-LC8** until Track A has exposed enough real managed-unit pressure.

## 8. Parallelization Guidance

Good parallel splits for multiple people:

1. **Person 1**: Track A runtime/prod closure (`A1-A8`)
2. **Person 2**: Track B feedback/ambient loop (`AS2-AS8`)
3. **Person 3**: Track C kernel modeling and ML exemplar (`IK1-IK5`)
4. **Person 4**: Track D lifecycle language/compiler design (`LC1-LC4`)

Merge point dependencies:

1. `AS3-AS8` depend on `AS2`
2. `IK5-IK8` depend on `IK1-IK4`
3. `LC5-LC8` depend on `LC1-LC4`
4. `AS1` depends on production signal delivery in Track A

## 9. What “Straightforward Implementation” Means Here

Implementation is straightforward only if we preserve these constraints:

1. one canonical meaning layer per concept,
2. no GitHub-specific semantics hidden in generic kernel types,
3. no lifecycle control hidden in handwritten CLI cleanup code,
4. no signal-only correctness assumptions,
5. no “ambient” claims until feedback obligations are durable and closed explicitly.

## 10. Deliverables by Milestone

### M1 Deliverables

1. SDLC on Cloud Run with durable ledgers/signals and multi-worker proof
2. one repo can traverse idea -> done

### M2 Deliverables

1. comments/reviews become durable obligations
2. worker responds to mid-flight human feedback reliably
3. ambient operation is credible

### M3 Deliverables

1. high-level intent intake
2. at least one non-SDLC exemplar (ML)
3. reusable kernel artifacts/obligations/evidence model

### M4 Deliverables

1. managed lifecycle in the language/compiler
2. generated disable/drain/destroy/status paths
3. compiler-derived lifecycle tests

## 11. Summary

There is a clear end-to-end path from the current SDLC branch to the broader
system you described. The path is not “one more sprint.” It is a staged program:

1. finish production SDLC,
2. make it ambient and trustworthy,
3. generalize the meaning layer,
4. push lifecycle control into the language/compiler.

That is the sequence that minimizes rework while still preserving the broader
vision.
