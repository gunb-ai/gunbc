# Design: Intellectual Pipeline Kernel

**Status**: Draft
**Date**: 2026-03-06
**Related tasks**: SDLC `B2`, SDLC `B1`
**Related docs**: `docs/design/sdlc/mega-modeling-design.md`, `docs/design/sdlc/domain-modeling-comprehensive.md`, `TODO/sdlc.md`

## Motivation

The current SDLC work is intentionally software-shaped: GitHub issues, PRs, code
review, CI, and stage labels. That is correct for Day 1. It is not yet the right
internal ontology for the longer-term goal.

The longer-term interaction model is higher level:

> "Can we make this sort of change?"
> "Can we solve this sort of problem?"

The user should be able to express intent and constraints without pre-deciding
whether the system needs research, design, implementation, experimentation, or a
combination of those. The pipeline should own the expansion from intent to a
defensible conclusion.

If the internal model remains "software ticket with stages", the system will
overfit to GitHub and code changes. That blocks reuse for other professional work
such as ML pipeline design, architecture studies, research memos, or policy
changes.

This doc asks a narrower question:

> Can SDLC be treated as one specialization of a broader intellectual/professional
> process without weakening the repo's modeling discipline?

## Desired Outcome

We want a reusable kernel for bounded intellectual work where:

1. The user submits high-level intent plus constraints.
2. The system turns that intent into explicit questions, plans, evidence, critique,
   revisions, and conclusions.
3. The existing control-plane machinery (signals, claims, ledgers, retries,
   approval gates, reconciliation) remains reusable across domains.
4. SDLC becomes one policy pack and adapter surface over the kernel, not the kernel
   itself.

## Satisfaction Criteria

This design is good enough only if all of the following hold:

1. At least three materially different exemplars decompose cleanly into the same
   core model.
2. The kernel does not use GitHub-specific or software-specific stage names.
3. Evidence and critique are first-class objects, not markdown conventions.
4. Human feedback can become durable obligations rather than best-effort text.
5. The design makes it clear what stays domain-specific and what is truly shared.

## Failure Mode If Wrong

There are several ways to fool ourselves:

1. We rename SDLC stages to generic words but keep software-only semantics hidden
   underneath.
2. We build an abstract stage enum too early and lose concrete pressure from the
   real SDLC pipeline.
3. We treat comments, reviews, and findings as unstructured text rather than typed
   critique/evidence.
4. We claim universality for work that is actually open-ended, tacit, or not
   evidence-bearing.
5. We produce a nice ontology that cannot explain what counts as success, failure,
   or closure in real workflows.

If the abstraction cannot explain how an ML experiment, a design review, and a code
review all satisfy the same core contracts, it is not a kernel. It is just a
renamed SDLC chart.

## Scope / Non-Goals

This doc does not:

1. Replace `IssueLifecycleStage` in the current SDLC implementation.
2. Require immediate runtime or DSL changes.
3. Claim that every human activity is representable by one pipeline.
4. Eliminate domain-specific policy packs such as SDLC, ML, or architecture.

This doc does:

1. Define a candidate shared ontology.
2. Separate the intellectual process from the operational control plane.
3. Stress-test the ontology against multiple kinds of professional work.

## Core Claim

The reusable thing is not a universal stage list. The reusable thing is a set of
typed objects and obligations that support bounded inquiry:

1. a problem statement,
2. candidate explanations or approaches,
3. planned interventions,
4. observed evidence,
5. critique,
6. revision,
7. conclusion.

The current SDLC controller already has much of the right operational structure.
What needs generalization is the meaning layer.

## Three Layers

### 1. Intellectual Kernel

This is the domain-neutral process model.

Suggested core concepts:

```text
ProblemStatement
Question
Hypothesis
Plan
Intervention
Evidence
Critique
Response
Conclusion
Obligation
```

Suggested process statuses:

```text
Active
AwaitingEvidence
AwaitingCritique
Revising
Concluded
TerminalFailed
```

### 2. Operational Control Plane

This is already mostly present in SDLC and should remain reusable:

```text
Signal
ClaimLease
RunKey
Artifact
ArtifactMarker
Outcome
ApprovalGate
RetryBudget
ReconcileRule
ExecutionReport
```

These concepts are orthogonal to the domain. They coordinate work; they do not
define what the work means.

### 3. Surface Adapters

These are environment-specific projections of the kernel:

1. GitHub issues / PRs / comments / labels
2. Pub/Sub or file-backed signals
3. CI systems
4. object stores
5. dashboards or CLI entrypoints

The adapter is not the ontology. GitHub comments are not "critique" by definition;
they are one place where critique may be expressed and then parsed into typed form.

## Scientific Method Framing

Internally, the best discipline is to treat the kernel as a practical form of the
scientific method:

1. define the question,
2. propose a hypothesis or candidate approach,
3. run an intervention or experiment,
4. collect evidence,
5. critique the result,
6. revise,
7. conclude.

This is not a rigid linear flow. Real work loops:

```text
question -> hypothesis -> intervention -> evidence -> critique -> revision -> conclusion
                                 ^___________________________|
```

The point of using this framing is not academic style. It is to force explicit
evidence, explicit critique, and explicit closure conditions so the system does not
mistake motion for progress.

## Minimal Kernel Ontology

The following is intentionally small. It is enough to test the abstraction without
committing the runtime to a premature generic stage machine.

### Canonical artifact kinds

```text
ArtifactKind =
  ProblemStatement
  Question
  ResearchNote
  Hypothesis
  Plan
  InterventionRecord
  Evidence
  Critique
  Response
  Conclusion
```

### Canonical event kinds

```text
EventKind =
  Submitted
  ArtifactRecorded
  EvidenceObserved
  CritiqueReceived
  ApprovalGranted
  RevisionRequested
  Resolved
```

### Canonical obligation kinds

```text
ObligationKind =
  MissingEvidence
  CritiqueResponseNeeded
  ApprovalNeeded
  RetryNeeded
  ReconcileNeeded
  ConclusionNeeded
```

### Canonical closure states

```text
ObligationStatus =
  Seen
  InProgress
  Addressed
  Closed
```

### Canonical conclusion dispositions

```text
ConclusionDisposition =
  Adopted
  Rejected
  Deferred
  NeedsMoreEvidence
  Blocked
```

## Resolved Design Decisions

### K1. The generic intake object is `InquiryIntent`, not `IntentSheet`

The generic kernel should not continue to overload `IntentSheet`. `IntentSheet`
already carries SDLC- and GitHub-shaped meaning (`owner`, `repo`, initial labels).
That is correct for SDLC intake and wrong for the reusable kernel.

The canonical general object is:

```text
InquiryIntent {
  intent_id: NonEmptyStr
  intent_version: Int
  summary: NonEmptyStr
  request: String
  motivation: String?
  desired_outcomes: List<SuccessCriterion>
  constraints: List<IntentConstraint>
  evidence_requirements: List<EvidenceRequirement>
  autonomy_policy: AutonomyPolicy
  conclusion_criteria: List<ConclusionCriterion>
  context_refs: List<IntentContextRef>
  created_at: Timestamp
  metadata: Json?
}
```

Supporting closed vocabularies:

```text
ConstraintStrength = Hard | Soft

AutonomyPolicy =
  ResearchOnly
  DesignOnly
  ApprovalBeforeExecution
  ExecuteWithinConstraints
```

Implications:

1. `InquiryIntent` is the generic kernel intake.
2. `IntentSheet` remains the SDLC adapter envelope and should eventually wrap or
   derive from `InquiryIntent` plus repo-specific target metadata.
3. Generic kernel types must not carry GitHub-specific fields such as owner, repo,
   labels, PR numbers, or provider URLs.

### K2. `Question` remains first-class, but optional per workflow

The kernel keeps `Question` distinct from `Hypothesis`.

Rationale:

1. some workflows need to separate "what are we asking?" from "what do we think
   will work?",
2. architecture and ML exemplars both exert real pressure for that distinction,
3. not every workflow must emit a standalone question artifact.

Rule:

1. `Question` exists in the shared ontology,
2. workflows may collapse it into the problem statement when a separate artifact
   adds no value,
3. the kernel must not require a separate question document in every path.

### K3. Feedback normalization is capture-first, then typed

The kernel and ambient SDLC path should not ingest human feedback directly into
workflow transitions. The canonical rule is:

1. capture source feedback as a durable record,
2. classify it into typed meaning,
3. create obligations or approvals from the typed meaning.

The detailed SDLC/GitHub mapping lives in
`docs/design/sdlc/ambient-feedback-model.md`, but the kernel-level decision is:

1. comments and reviews are adapter surfaces,
2. critique is the shared meaning,
3. no feedback is silently dropped because classification failed.

### K4. The required second exemplar is a bounded offline ML investigation

The non-SDLC exemplar is now explicitly scoped. It is **not** full MLOps.

Included:

1. problem statement,
2. data inventory and acquisition plan,
3. labeling strategy,
4. baseline evaluation plan,
5. one bounded offline experiment or experiment-ready plan,
6. evidence report,
7. critique,
8. conclusion.

Explicitly excluded:

1. online serving,
2. feature stores,
3. continuous training,
4. production monitoring,
5. deployment orchestration.

This keeps the exemplar pressure real without turning the kernel lane into a
second platform program.

## Stress Test: Exemplar 1 — SDLC Change Request

### Intent

"Can we make this change to the repo?"

### Kernel decomposition

| Kernel concept | SDLC interpretation |
|---|---|
| `ProblemStatement` | issue or intake sheet describing desired change |
| `Question` | what exact behavior or outcome should change under what constraints? |
| `Hypothesis` | design proposal for the change |
| `Plan` | implementation plan / task breakdown |
| `Intervention` | code changes on a branch |
| `Evidence` | tests, clippy, runtime traces, benchmark output |
| `Critique` | PR review comments, design review findings |
| `Response` | follow-up commits or written reviewer responses |
| `Conclusion` | merge, close, or reject with rationale |

### What is domain-specific

1. GitHub issue and PR surfaces
2. software-specific evidence like CI, clippy, tests
3. branch creation, merge, and code review policy

### Result

This fits cleanly. SDLC is a specialization, not a counterexample.

## Stress Test: Exemplar 2 — ML Pipeline Design

### Intent

"I want to solve this class of prediction or classification problem."

### Boundaries for this exemplar

This exemplar is intentionally:

1. offline,
2. bounded to one problem statement and one evaluation loop,
3. allowed to stop at "proceed / defer / reject" rather than deployment.

### Kernel decomposition

| Kernel concept | ML interpretation |
|---|---|
| `ProblemStatement` | target business or scientific problem |
| `Question` | is the problem learnable with available signals? |
| `Hypothesis` | candidate features, model families, labeling strategies |
| `Plan` | data acquisition, labeling, training, evaluation, deployment plan |
| `Intervention` | dataset build, training run, offline eval, prototype pipeline |
| `Evidence` | metrics, error slices, ablations, cost, bias, leakage checks |
| `Critique` | review findings about data quality, leakage, fairness, operational fit |
| `Response` | revised data plan, model change, different evaluation method |
| `Conclusion` | deploy, defer, reject, or continue research |

### What is domain-specific

1. data sources and acquisition contracts
2. experiment execution and evaluation metrics
3. deployment and drift-monitoring policy

### Result

This fits especially well. ML work is already evidence-heavy and critique-heavy.
The kernel holds as long as evidence remains typed and reproducible, not hand-waved.

## Stress Test: Exemplar 3 — Architecture / Research Decision

### Intent

"Should we adopt this architecture, vendor, or technical direction?"

### Kernel decomposition

| Kernel concept | Architecture / research interpretation |
|---|---|
| `ProblemStatement` | current system limitation or decision pressure |
| `Question` | what options are viable under constraints? |
| `Hypothesis` | candidate architectural thesis or recommendation |
| `Plan` | prototype, comparison matrix, benchmark, migration sketch |
| `Intervention` | prototype or focused investigation |
| `Evidence` | benchmarks, cost model, complexity analysis, risk ledger |
| `Critique` | reviewer challenges, contradictory findings, missing assumptions |
| `Response` | updated recommendation or narrowed scope |
| `Conclusion` | adopt, reject, defer, or run another investigation |

### What is domain-specific

1. evaluation dimensions
2. benchmark/prototype mechanics
3. organization-specific approval and rollout policy

### Result

This also fits. There may be no code change at all, but the work still follows the
same bounded inquiry loop.

## Boundary of Applicability

This kernel is appropriate for bounded professional work that has:

1. a durable problem statement,
2. explicit artifacts,
3. observable evidence,
4. critique or review loops,
5. closure conditions.

It is a bad fit for:

1. continuous ambient work with no closure condition,
2. work that is mostly tacit and cannot produce explicit evidence,
3. purely interpersonal or political negotiation with no stable artifact boundary.

This is not a universal human-process theory. It is a good model for professional
inquiry and execution under explicit constraints.

## Consequences for gunbc

### What should generalize first

Generalize these before stage names:

1. artifact kinds,
2. evidence,
3. critique,
4. obligations,
5. conclusion criteria.

### What should not generalize yet

Do not immediately replace SDLC runtime stages with a universal enum. Keep:

1. `IssueLifecycleStage` as the concrete SDLC state machine,
2. GitHub-specific workflows as the first adapter surface,
3. SDLC-specific policy in SDLC docs and DSL modules.

The kernel should first prove itself at the modeling layer with multiple exemplars.
Only then should it pressure the execution model.

### Near-term migration path

1. Keep SDLC Day 1 activation software-specific.
2. Model feedback, evidence, critique, and obligations in a domain-neutral way.
3. Add one second exemplar beyond SDLC before changing the runtime ontology.
4. Introduce `InquiryIntent` as the shared intake type; keep `IntentSheet` as the
   SDLC adapter until runtime migration is justified.

## Summary

The current SDLC design is already close to a reusable professional-work engine at
the control-plane level. The missing step is not a new scheduler or a new queue. It
is a cleaner meaning layer.

The right abstraction is:

1. domain-neutral inquiry artifacts and obligations,
2. reusable operational control-plane primitives,
3. domain-specific policy packs and adapter surfaces.

Under that structure, SDLC, ML pipeline design, and architecture/research decisions
all fit the same kernel without pretending they are identical.
