# Witness evidence lifecycle — route is a projection, evidence is the lifecycle

**Status: DESIGN NOTE, operator sign required before any code lands.** No implementation
follows from this note until step 1 below is signed.

## The incident that priced this

Four witnesses introduced by gunbc#7683 (Lane B, direct v2 Rust door) are red on the
nightly falsifier. The falsifier is not reporting a regression. It is reporting the first
completed execution any of them has ever had.

At the intermediate PR revision `c80605a`, all four terminated at the 5-second fast-lane
wall:

```
witness_direct_rust_door_provenanced_artifact_holds ... runtime error: eval budget
exceeded: 5002ms elapsed > 5000ms fast-lane budget (operator 5s rule 2026-07-12: a
witness this slow lives in a long/ test dir and runs via its dedicated lane, not per-PR
discovery)
```

The runner recorded `EvalBudgetExceeded`, prescribed relocation into `test/claim/long/`,
and never observed a returned Boolean. The witnesses were moved. The final revision
`79e5106` passed, because `test/claim/long/` is excluded from discovery. The pending
semantic obligation did not travel with the files; it ceased to exist.

```
semantic obligation is pending
  -> fast attempt produces cost evidence but no semantic evidence
  -> diagnostic prescribes a route change
  -> route change removes the witness from the only enumerated population
  -> pending semantic obligation disappears
```

The author did what the compiler instructed. At no point did the system represent
"this is slow AND its verdict is unknown"; it represented only "this is slow, move it",
and moving it discharged the only obligation the model carried.

## Three coupled defects

1. **State-space** — timeout and semantic failure share one red surface.
2. **Lifecycle** — a route transition does not carry the pending-verdict obligation forward.
3. **Population** — routing exclusions are applied before the witness universe is established.

The third made the first two survivable through merge.

### The population defect, verified

`v2.workflow.floor_discovery_producer` `floor_discovery_process_dag_file` returns its
state unchanged when the path is excluded — before `Filesystem.Read`, before
`floor_discovery_apply_test_sidecar_rules`. The excluded file is never read, so its test
declarations are never enumerated. It is **absent from the universe**, not
present-with-a-long-route.

Live consequence: adding a sibling test function to an existing long-lane file skips the
whole file before function enumeration, the existing path-pattern schedule row still
reconciles, and the new function has no executing consumer anywhere. Every admission
check stays green.

`floor_discovery_path_excluded` matches by raw substring. The registered pattern carries
a trailing slash, so `test/claim/longish/` happens not to match — a property of how the
string was typed, not a guarantee. This is the DESIGN section 3 class: a positional path
policy standing where the containment tree already names the structure.

### The state-space defect, located in the seed

The typed fact already exists and is destroyed in transit. `v1_interpreter.rs` declares
`InterpError::EvalBudgetExceeded { elapsed_ms, budget_ms }` — a proper variant carrying
both numbers. It is rendered to a string, surfaces at the claim boundary as an ordinary
runtime error, and is then reconstituted downstream by substring match:

```
falsifier_failure_mode  (claim_executor.rs)
    d.contains("eval budget exceeded")  ->  "BudgetExceeded"
```

The same function's own comment reads: *"THIS LIST IS A FORK, and that is the defect —
not the individual substrings."* A second consumer repeats the match in `cli_run.rs`.

Meanwhile a *typed* `ClaimOutcome::TimedOut { elapsed_ms, budget_ms, kind }` exists and is
used by the wall-clock kill path, and `claim_batch.rs` already carries the exact insight
this note formalizes:

> "Named as a kill, not a duration: the row stopped AT the budget, so elapsed is a
> ceiling. The clock is named because a cpu-budget kill and a wall-budget kill have
> different remedies."

So there are two representations of one concept — one typed, one stringly — and the
fast-lane eval deadline uses the stringly one. The interruption/verdict distinction is
recovered by grepping prose. That is simultaneously the DESIGN section 3 fork and the
section 5 conflation, and it is the mechanism that made the incident possible: a
`RuntimeError { message }` carrier cannot represent "no verdict was obtained", so nothing
downstream could preserve the pending obligation even in principle.

This is load-bearing for the program below. Step 4 is not introducing a distinction the
system lacks; it is promoting one the seed already makes, and deleting the substring
reconstruction that stands in for it.

## The modeling decision (step 1 — what this note asks to be signed)

| Fact | Kind |
| --- | --- |
| Witness identity and expectation | Declaration |
| Complexity / resource requirement | Declaration or structural derivation |
| Route | Derived projection |
| Attempt termination | Observation |
| Runtime cost | Runner-scoped observation |
| Semantic evidence current/pending | Derived lifecycle state |
| Lane budget | Scheduler policy |

**Route is neither a declaration nor a lifecycle. Route is a projection. Semantic
evidence is the lifecycle.**

The conservation law:

```
identity(w)           invariant under route changes
expectation(w)        invariant under route changes
semantic obligation(w) invariant under route changes

execution_plan(w)     may change
```

Merge admission requires, for every witness in the candidate:

```
current completed evidence AND observation matches expectation
```

An interruption never satisfies that predicate.

## Why the 5-second wall loses authority but need not disappear

The wall currently does four jobs; only the first is legitimate:

- limits resource consumption
- classifies the witness
- prescribes a source-tree relocation
- causes the witness to disappear from the enumerated population

A wall-clock threshold selecting a failure arm is what DESIGN section 5 names a smuggled
heuristic: its existence locates the anemic modeling it papers over. Cost is a fact this
substrate claims to carry structurally (section 4b lists "a computation exceeding its
declared complexity bound" among the differentiating classes). A witness whose cost must
be discovered by racing a stopwatch is one whose cost was never modeled.

The repository already declares the dissolution. `gunbc.ci_layer_roots`
`long_lane_exclusion_note` carries:

> "Dissolve-on: per-witness declared cost envelopes (witness-cost-locality admission law)
> subsume the dir grain."

So the directory-grain route is a declared scaffold with a named trigger, and this
incident is the displaced cost that fires it.

The correct result of a budget interruption:

```
BudgetExceeded
  => attempted route was insufficient
  AND cost is at least the granted budget
  AND semantic evidence remains pending
```

It must not imply `witness belongs under test/claim/long/`, and must never imply
`file moved under test/claim/long/ => obligation discharged`.

## Outcome algebra

`red` ceases to be a domain type. It is derived job colour.

```
WitnessVerdict = Holds | Violates { finding }

WitnessAttemptOutcome
  = Returned      { verdict, cost }
  | DiagnosedStop { stage, diagnostic, cost }
  | Interrupted   { cause, cost }

WitnessInterruption
  = BudgetExceeded { budget }
  | ProcessKilled | EvaluatorCrashed | RunnerUnavailable
  | TerminalReceiptMissing | Cancelled
```

Only `Returned` and `DiagnosedStop` project into a semantic observation; `Interrupted`
projects to absent. `verdict_matches_expectation` therefore cannot receive a timeout —
a construction wall, not a check.

Expectation is function-grain, not file-grain or batch-grain:

```
WitnessExpectation
  = MustReturnHolds
  | AdmittedViolation     { expected_finding }
  | AdmittedDiagnosedStop { expected_stage, expected_diagnostic }
```

`Interrupted`, including `BudgetExceeded`, is accepted by none of these. This retires the
current `QuarantineProbeExpectRed` conflation of returned-false, stable-diagnosed-stop,
and interrupted execution, and it retires batch-wide `expect_red` inversion: a batch is
an execution grouping and has no uniform verdict semantics. Matching is pointwise.

An unexpected green becomes useful — it refuses until the stale admission dissolves.

## Cost observation is a lower bound under a runner contract

Even a completed elapsed duration is not intrinsic to the witness. It is an observation
under a particular runner and environment.

```
CostObservation
  = CompletedElapsed { elapsed, runner_contract }
  | LowerBound       { floor,   runner_contract }
```

A budget interruption can construct only `LowerBound { floor: budget, .. }`. It cannot
construct a completed duration.

The four figures recorded at `c80605a` — 5002 / 5002 / 5002 / 5001 ms against a 5000 ms
budget — are pinned to the deadline by the kill mechanism. True cost could be 5.1s or
500s. They establish `cost >= 5000ms under that runner contract` and nothing else.

**No canonical runner exists to bind measurements to.** The CI hosts in this fleet are
broken differently from one another and the warm-cache host is the slowest. Three
consequences:

1. A lifted-budget run on a development worktree yields `CompletedElapsed` under a local
   contract no CI lane shares. Sufficient to produce semantic observations; sizes nothing.
2. `RunnerContractDigest` must be genuinely discriminating — host class, cache state,
   concurrency, memory cap — not a nominal lane label. Collapsed to `"fast"`/`"long"`,
   receipt currency silently accepts measurements from incomparable hosts and the reuse
   rule fails open exactly where it looks strongest.
3. `WitnessExecutionRequirement` must lean on structural terms (evaluation work, subject
   scale, memory, effects, isolation) and treat wall-clock as a derived expectation under
   a named runner, never as the declared fact. A hand-authored `expected_duration: 30s`
   unbacked by a completed measurement is the directory heuristic at function grain.

## Semantic evidence lifecycle (route-independent)

```
SemanticEvidenceStatus
  = PendingSemanticEvidence { reason }
  | CurrentSemanticEvidence { receipt }

PendingEvidenceReason
  = NeverAttempted | SubjectChanged | EvaluatorChanged
  | RunnerContractChanged | ExpectationChanged
  | PriorAttemptInterrupted { cause }
```

A fast timeout produces route-suitability evidence (fast route insufficient) and
`PendingSemanticEvidence(PriorAttemptInterrupted(BudgetExceeded))`. Re-deriving a route
does not discharge that state; only a completed candidate-bound attempt does.

A receipt binds witness identity, witness AST digest, dependency closure digest,
evaluator digest, runner contract digest (including budget), expectation digest,
execution environment digest, and the observation. "The file did not change" is not
sufficient for reuse.

## Ordering

Census must precede routing. Never the reverse.

```
parse all witness declarations
  -> construct canonical witness universe W(T)
  -> resolve expectation and execution requirement
  -> derive route
```

Route projections partition `W(T)` exactly — no overlap, no omitted member, no
independently authored roster.

## Program

| Step | Deliverable |
| --- | --- |
| 1 | This note signed — the modeling decision above |
| 2 | `DirectRustDoorProvenanceObservation`; run all four functions to completion under a lifted budget |
| 3 | Census before routing in `v2.workflow.floor_discovery_producer` |
| 4 | Attempt and evidence algebra; expectation split; pointwise matching |
| 5 | Route derived from execution requirement; directory semantics deleted |

Step 2 is independent of steps 3-5 and of this note's review. Its output is semantic
observations plus a local-runner cost datum — **not** envelope inputs, per the runner
caveat above.

`witness_direct_rust_door_provenanced_artifact_holds` collapses five independent
assertions into one Boolean (candidate accepted, emitted bytes equal expected, realized
closure excludes the seed emitter, producer is `V2EmitterInterpreted`, `SourceProduced`
qualification present). The typed observation makes the completed false attributable
rather than merely visible. Three of the four failing functions carry no known-red
admission and are therefore `MustReturnHolds`; only
`witness_v2_emitter_direct_rust_door_closing_contract_holds` is admitted. All four block
at `c80605a` regardless, because an interrupted execution satisfies neither.

## Mutation walls

| Mutation | Required result |
| --- | --- |
| Ordinary witness exceeds fast budget | `Interrupted(BudgetExceeded)`; expectation unsatisfied |
| Admitted-failure witness exceeds fast budget | Same; admission cannot absorb it |
| Fast timeout, relocation, no completed rerun | `PendingSemanticEvidence` |
| Long attempt also exceeds its budget | Still pending; no second escape hatch |
| New sibling function in a long file | Canonical census grows by exactly one |
| File moved between ordinary and long directories | Route unchanged unless its execution requirement changes |
| Green and admitted-failure functions share a file or batch | Each matched against its own expectation |
| Admitted failure changes shape | `ExpectedOutcomeDrifted` |
| Admitted failure becomes green | `StaleAdmission` |
| Runner contract or budget changes | Previous runtime receipt is not current |
| Route projection omits a census member | `ExecutionProjectionMissingWitness` |
| Execution plan contains an unknown member | `ExecutionProjectionContainsUnknownWitness` |
| Path exclusion applied before census in a mutated producer | Census-equivalence falsifier fails |
| `LowerBound(5s)` read as a measured duration | Refused |

The tests must call the production census, policy resolver, plan constructor, and receipt
matcher. A parallel validation checker reimplementing those rules recreates the
dual-authority risk this note exists to remove.

## Completion criterion

> A route change may alter where, when, and under what budget a witness executes. It may
> not alter whether the witness exists, what outcome it owes, or whether that outcome
> remains unobserved.

## Out of scope

Two unrelated falsifier reds discovered in the same run are dispatched separately and do
not belong in this closure: the Clone bound propagating onto a phantom type parameter
(`fn_wf_no_trigger_negative_control`, the #7691/#7708 fixpoint), and inert-carrier roster
drift on main.
