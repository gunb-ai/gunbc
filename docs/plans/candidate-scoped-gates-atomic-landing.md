# Candidate-scoped gates + atomic landing admission (operator-ruled 2026-08-02)

Status: RULED design, landed as the program charter for the merge-freshness lane —
the gap `generated-file-conflict-policy.md` explicitly named out of scope ("a branch
authored before an authority changed merging with nothing re-checking it against the
new state"). Composition with the landed conflict-policy program: commit-writer
admission proves the *write* is clean, the landing boundary here proves the *tree*
becoming main was tested, the drift gate proves *artifacts* match authority.

## The incident, and the three independent defects it proves

An extdeps scope-placement gate landed (#7571) eleven seconds before a green PR
(#7573) squash-merged onto it; the landing tree was never tested, main went red, and
because the gate's placement arm reads the historical freeze anchor, main's defect
became **every branch's defect**. The emergency state repair (enrolling
`extdeps.session_dashboard` as a scope carrier) is on main — note: not via #7644,
which was CLOSED unmerged in favor of the differently-shaped enrollment that landed.
Step 0 of the sequence below is therefore already discharged.

| Defect | Current shape (verified on main) | Consequence |
| --- | --- | --- |
| Wrong gate subject | The placement arm diffs `legacy_manifest_freeze_sha..HEAD` — stated in `gunbc.extdeps_scope_frontier` `extdeps_scope_frontier_law` itself, whose text simultaneously claims per-PR diff grain (false law text) | An old defect becomes every branch's defect |
| Missing landing-candidate validation | #7573's green run never contained the gate that landed eleven seconds before it; `gunbc.merge_admission` carries `require_up_to_date: false`, `merge_queue_required: false`, and `gunbc.ci_failure_class` `merge_freshness_gating_status` = `GatingComputedDeferred` | A green PR can still construct an untested red main |
| Failure evidence erased | Every spec gate runs through `tools.floor_effect_gate_witness`'s `*_gate_passes() -> Bool` adapters — `exit_ok(run_spec_gate(...))` collapses `ExitFailure { reason }` to `Bool(false)` before FullLedger sees it | FullLedger reports "returned Bool(false)" and cannot say why |

The historical-freeze question ("does the frozen manifest name anything created after
the freeze?") and the candidate-admission question ("which extdeps paths does this
candidate introduce?") currently share one `PostFreezeObservation` despite different
subjects and purposes. FullLedger did its job — the evidence was lost before it
arrived. The Bool collapse is shared by all floor spec gates, not unique to this one.

## 1. Three Git subjects, three types

No shared generic "base" carrier. The three facts:

```dag
type IntegrationCandidate {
  base_commit: CommitSha
  candidate_commit: CommitSha
  candidate_tree: GitTreeObjectId
}

type CandidateAddedPathsObservation
  = CandidateAddedPathsObserved { candidate: IntegrationCandidate, paths: List<String> }
  | CandidateAddedPathsDiffRefused { candidate: IntegrationCandidate, cause: String }
  | CandidateAddedPathsParseTruncated { candidate: IntegrationCandidate }

type LegacyFreezeObservation
  = LegacyPostFreezePathsObserved { freeze_commit: CommitSha, candidate_commit: CommitSha, paths: List<String> }
  | LegacyFreezeDiffRefused { freeze_commit: CommitSha, candidate_commit: CommitSha, cause: String }
  | LegacyFreezeParseTruncated { freeze_commit: CommitSha, candidate_commit: CommitSha }

type LegacyManifestDeltaObservation
  = LegacyManifestUntouched
  | LegacyManifestIntroduced
  | LegacyManifestRowsAdded { candidate: IntegrationCandidate, rows: List<String> }
  | LegacyManifestDeltaRefused { candidate: IntegrationCandidate, cause: String }
```

The commit identities carry the *ancestry* subjects — `merge-base` computes over
commit ancestry, which tree objects do not have, and a bare `GitObjectId` could denote
a blob or tag (review 46971 caught the earlier sketch feeding trees to `merge-base`,
an unrealizable law). For the same reason `candidate_tree` is typed `GitTreeObjectId`,
a tree-kind refinement that does not exist in `extdeps.git.object_store` today — the
implementation lane introduces it there (a kind-refined carrier over the same
constructed-hex `GitObjectId` invariants), so a blob or tag object is unrepresentable
as a landing subject rather than merely unexpected (review 47056; DESIGN §4). The
same lane should re-ground `extdeps.git.object_store` `GitCommitObject`'s `tree`
field (and its `SubtreeEntry`/gitlink kin where the kind is fixed by the format),
which today carry the same generic-`GitObjectId` looseness — one carrier, every
kind-fixed position, not a charter-local special. `candidate_tree` is retained
separately as the *exact-tree admission* subject — the thing the landing boundary
tests and lands — and is the tree of `candidate_commit`, carried so tree-identity
claims never re-derive it through a revision walk at the admission boundary.

Observation laws: candidate paths = `merge-base(base_commit, candidate_commit)...candidate_commit`;
freeze = `legacy_manifest_freeze_sha..candidate_commit`; manifest delta =
`base_commit...candidate_commit`. The candidate observation **reuses** the existing
`gunbc.diff_baseline` / `v2.workflow.floor_diff_observe` authority (real PR target,
merge-group handling, refuses unknown event shapes) — never a second
`origin/main...HEAD` reader inside the scope gate.

Two evaluation surfaces, explicitly distinct:

```dag
type ExtdepsScopeEvaluation
  = CandidateAdmission { candidate: IntegrationCandidate }
  | DefaultBranchAudit { tree: GitTreeObjectId }
```

`CandidateAdmission` judges only the candidate delta; `DefaultBranchAudit` walks the
whole live population under the existing cover law and remains the backstop. Its
subject is the same tree-kind carrier as `candidate_tree` (review 47071 caught this
row still generic after the `IntegrationCandidate` fix): every audit or admission
subject in this charter is a tree by construction, never a bare object id.

## 2. One typed, located, accumulated verdict

One scheduled gate (cost), three independently typed laws, refusals accumulated:

```dag
type ExtdepsScopeRefusal
  = AddedExtdepsPathUnenrolled { path: String }
  | LegacyManifestNamesPostFreezePath { path: String, freeze_commit: CommitSha }
  | LegacyManifestRowAdded { row: String }
  | CandidateDiffUnreadable { cause: String }
  | CandidateDiffParseTruncated
  | FreezeDiffUnreadable { freeze_commit: CommitSha, cause: String }
  | FreezeDiffParseTruncated { freeze_commit: CommitSha }
  | ManifestDeltaUnreadable { cause: String }

type ExtdepsScopeVerdict
  = ExtdepsScopeAdmitted { candidate: IntegrationCandidate, introduced_extdeps_paths: List<String> }
  | DefaultBranchAuditCovered { tree: GitTreeObjectId }
  | ExtdepsScopeRefused { subject: ExtdepsScopeEvaluation, first: ExtdepsScopeRefusal, rest: List<ExtdepsScopeRefusal> }
```

The verdict is total over both evaluation surfaces (review 47097 caught the earlier
sketch admitting only candidates, leaving `DefaultBranchAudit` with no faithful
success result): a clean audit lands `DefaultBranchAuditCovered`, and `ExtdepsScopeRefused`
carries the run's `ExtdepsScopeEvaluation` subject once, at the wrapper — so every
refusal row inherits its subject identities (candidate base/tree under
`CandidateAdmission`, audit tree under `DefaultBranchAudit`) by construction, and the
per-arm `candidate` fields the earlier sketch duplicated across six arms are gone
(§2: the arms keep only their law-local facts, e.g. `freeze_commit`; the subject
flows forward on the wrapper, never re-derived and never fabricated).

Load-bearing: **no early exit between the independent laws** (an unrostered path and
an illegal manifest row both report in one run), and **no empty refusal population**
(`first + rest`). The process boundary renders every refusal with refusal code, path
or row, the identities its carrier and the wrapper subject flow forward, and
remediation class.

## 3. Stop running effectful gates through Boolean claims

Add `RunnableProcessExitClaim { entry, function, profile }` whose execution law maps
`ExitSuccess → RunnablePassed`, `ExitFailure { code, reason } → RunnableFailed {
detail: reason, exit_code: code }`, anything else → `RunnableMalformed`. Replace
`*_gate_passes() -> Bool` with `*_gate_outcome() -> ProcessExit` for **all** spec
gates — no println side channel, no extdeps-only workaround. After migration,
`tools.floor_effect_gate_witness` `exit_ok` adapters delete. FullLedger keeps its
stop policy; its rows gain the cause.

## 4. Atomic landing: the base split does not close the eleven-second race

A perfect candidate-relative gate never executes on `B1 + P` unless something
constructs that tree and tests it before it becomes main. Smallest available
realization: the **GitHub merge queue** (`ci.yml` already listens on `merge_group`
with non-cancelling queue concurrency, from #6725; the bounded interleaving proof
distinguishes per-PR green from testing the queue-constructed landing tree).

```dag
type MergeQueueGrouping = AllGreen | HeadGreen

type DefaultBranchLandingPolicy
  = MergeQueueOnly { method: SquashMerge, grouping: AllGreen, required_check: NonEmptyStr }

data gunbc_default_branch_landing_policy: DefaultBranchLandingPolicy
  = MergeQueueOnly { method: SquashMerge, grouping: AllGreen, required_check: "ci" }
```

The independent Booleans `require_up_to_date` / `merge_queue_required` in
`gunbc.merge_admission` do **not** survive as the business-policy shape (they admit
nonsensical combinations). The coproduct change absorbs the staged flip already
declared by `gunbc.merge_admission` `merge_admission_required_check_binding_dissolution_trigger`
(`GatingComputedDeferred` → `GatingEnforced`), never mints a second trigger.

Operational rule: the dashboard's action becomes **enqueue, never direct merge**.
Routine bypass actors empty; an emergency bypass, if retained, is a separate
time-bounded receipt-bearing capability, not an ambient admin exemption.

Relationship to #7522: it is the topology/cost migration (ordered warm CI stages) and
explicitly preserves `GatingComputedDeferred`; queue activation does not wait for it.
The V2 keyed-receipt / current-context gate is defense in depth and the future native
SCM mechanism; the queue is the GitHub compatibility realization of atomic candidate
admission.

## 5. Acceptance set

**A. Permanent incident reproduction** — encode `[RunPrCi { pr: 7573, base: B0 },
MergePr { 7571 }, MergePr { 7573 }]`. `PerPrGateOnly` advances main to an unverified
tree (RED control). `MergeQueueOnly` / keyed receipt: the stale receipt is
inadmissible, candidate `B1 + #7573` runs the new gate, refuses the exact extdeps
path, main stays `B1`. (Fixture uses a synthetic unrostered path or the #7480-shaped
`dag/extdeps/container/oci/digest.dag` case, since the live repair is on main.)

**B. Candidate locality** — a historical unrostered path on main does not appear in
an unrelated candidate's observation; a candidate adding an unrostered path refuses
that exact path; adding declaration + carrier row admits; a stacked PR uses its
actual target branch, never hard-coded `origin/main`.

**C. Freeze independence** — post-freeze file added to the manifest refuses as
`LegacyManifestNamesPostFreezePath`; re-adding a removed pre-freeze row refuses as
`LegacyManifestRowAdded`; removing a legacy row admits; candidate-delta behavior
unchanged by unrelated post-freeze history.

**D. Located executor evidence** — a forced scope failure produces a ledger row with
refusal code, path, candidate base/tree identities, remediation class; the RED
control explicitly rejects output containing only `returned Bool(false)`.

**E. Queue interleaving** — queue #7571 before #7573: candidate 1 = `B0 + #7571` may
pass; candidate 2 = `B0 + #7571 + #7573` must execute the newly introduced gate and
refuse. `ALLGREEN` is the required grouping.

**F. Gate-local FullLedger** — one fixture introduces an unrostered path AND an
illegal manifest row; the verdict carries both refusals (the typed coproduct did not
merely re-house first-failure behavior).

## 6. Landing sequence

| Step | Change | Safety role |
| --- | --- | --- |
| 0 | State repair on main | **Done** (session_dashboard enrolled; #7644 closed unmerged in favor of the landed variant) |
| 1 | Require merge queue: Squash + `ALLGREEN`; dashboard switches to enqueue | Close the stale-green landing race |
| 2 | Split candidate/freeze/manifest observations; correct the false law text; schedule the `DefaultBranchAudit` backstop | Correct gate subject and blame |
| 3 | `RunnableProcessExitClaim`; migrate spec gates; delete Boolean adapters | Preserve located failures through FullLedger |
| 4 | Model the repository ruleset as desired state with read-back/drift verdict | Keep the queue boundary from becoming ambient config |
| 5 | Complete #7522/V2 production wires; later flip native `GatingEnforced` | Defense in depth, native-SCM route |
| 6 | Replace variant-tag gate hashing with resolved admission-plan/source-closure identity (fires `gate_content_hash_variant_tag_scaffold`'s declared dissolution) | Criteria identity semantic, not nominal |

**Sequencing bar:** do not land only the candidate-base split while direct
stale-green merges remain possible — activate the queue first, or retain a required
whole-tree default-branch audit until the queue boundary is proven live.

## Coordinator verification receipts and sharpenings (2026-08-02)

All three defects verified against main before this doc landed: the freeze..HEAD
placement subject and its contradictory grain claim (`extdeps_scope_frontier_law`),
the `exit_ok` Bool adapters (`tools.floor_effect_gate_witness`), the `merge_group`
listener + non-cancelling concurrency in generated `ci.yml`, the two false Booleans
and `GatingComputedDeferred`. Sharpenings binding on the lanes:

1. **The backstop must be scheduled, not a recipe.** Today the whole-population
   check is the wet live-cover witness, discovery-excluded with a documented local
   recipe — nobody runs it automatically. Step 2 enrolls it on the falsifier cadence
   regardless of rollout order; the sequencing bar's "required whole-tree audit"
   means an executing run.
2. **`ALLGREEN` serializes landings behind `ci` latency** (30–90 min under current
   fleet load, with runner-queue backlog receipts from 2026-08-01). Queue batching
   parameters and the srv1 build-cache repair are throughput dependencies of step 1;
   activate with eyes open.
3. **One infra work order, not two:** the dashboard daemon needs both the
   `admit_commit_writer` call at its write boundary
   (`CommitWriterDashboardInfrastructureOperatorOwner`, the typed countable row
   landed by #7608) and the switch from direct merge to enqueue. Route together.
4. **The App `workflows` permission gap bites step 3:** migrating spec gates
   regenerates `ci.yml`, and the heal writer cannot push workflow files (receipt:
   #7609, 2026-08-01). Grant the permission first or every step-3/5 change pays a
   manual regen push.
5. **Step 4 is the desired-state + read-back shape already built** for
   `gunbc.repo_local_git_config` (converge, `--local`-scoped read-back, typed
   refusal, named rollout carrier) — reuse the pattern and preferably the owner.
6. **Fixture overlap with the conflict-policy lanes:** the acceptance set's
   enrollment rows live in the same `commit_gate_roster` region #7606 edits —
   sequence after lane 4 lands or coordinate regions.

Resulting responsibilities: historical freeze proves legacy membership cannot grow;
candidate-scoped gate proves this candidate's new extdeps paths have valid placement;
direct `ProcessExit` runnables preserve every located refusal through CI; merge queue
(later the native merge actuator) proves the exact tree becoming main passed the
current criteria.

## HAND-RUST GATE receipt — rest replay bridge mints the modeled ContentHash

**Explicit deferral. Lane: v1 exit (ROADMAP §"eleven lanes" row — finish lines
"interpreter deleted" and "zero hand-maintained Rust"); near trigger: the
witness-realization plan's native rest-transport bridge, which deletes this seed
mint wholesale.** (Receipt form per review 47097; the class argument below says why
no separable schedule exists beyond those triggers.)

This receipt was authored when this PR carried the `v1_interpreter.rs`
`rest_bound_invocation_value` repair (review 47056 asked for it); the hunk itself
landed on main via the piecemeal heal chain — #7656 (import + scope carrier), #7663
(fixture rehydration), #7665 (the interpreter mint, factored as
`rest_structural_content_hash_value`) — and the receipt remains here as the program
record of that repair's class and triggers. The class: a **model-conformance repair
to an existing seed bridge**, not a new decision surface chosen in Rust. #7480 (landed) made
`std.content_hash` `ContentHash` a family coproduct; the seed's replay-fixture
bridge kept minting `input_digest` as a bare host string, so fixture-identity
structural `==` silently compared a string against the modeled coproduct — the
model↔realization fork DESIGN's open thread names, live. The hunk changes what the
existing mint constructs (`Fnv1a64` wrapping `Fnv1a64Structural`) to the shape the
single authority declares; no new capability, no new surface, arm count and fn
census unmoved. Like the `CompilerDiagnostic` seed-projection receipt
(`docs/plans/compile-clean-forcecheck.md`), this owes no separable dissolution
schedule: the mint cannot live anywhere but the seed's projection of the modeled
carrier, and it disappears exactly when the seed's rest transport bridge does
(witness-realization / v2-adoption trigger, the same trigger the #7480 open-thread
row already carries for the remaining v1 fingerprint calls). Checkable receipt, by
execution: `dag/test/claim/rest_exchange_replay_test.dag` — 6 of 9 witnesses RED
under the string mint (every fixture-matching test; the discriminating split), 9 of
9 PASS after, reproducible via `claim_batch --entry` on that file. Found and fixed
beside it, flagged for an owner rather than fixed here: the stage0 emitter resolves
a fully-qualified module-local data reference
(`extdeps.container.oci.digest.extdeps_external_authority_anchor`) into the wrong
crate module (`extdeps_cargo`, E0308) — the first enrolled scope carrier inside the
emitted set exposed it; dotted-path emitter bucket.
