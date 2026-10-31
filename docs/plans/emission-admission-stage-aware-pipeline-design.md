# Emission admission + stage-aware pipeline (declared-change vs regression)

> **AMENDED — awaiting operator sign-off.** The nine corrections outstanding against the merged
> first revision are applied (§12); this revision has not been signed. **This document does not
> authorize promotion, warm reuse, recovery-anchor release, or deletion of a prior compiler
> generation. Candidate/shadow generation may continue.** That clause stands until the operator
> signs — not until the corrections land.
>
> No roadmap acceptance and no promotion authority derives from this document.

*(Replaces the rough-draft banner that recorded the nine corrections as outstanding. They are
applied; the document is no longer a rough draft; it is also not signed.)*

**Status:** design-only (model-before-implement). This document is the deliverable for
work item `node://adhoc-48a1f19c-1f8` (session calm-eagle-92). No load-bearing `.dag`
carrier lands in the design PR; each implementation stage below is a separately dispatched,
separately-signed PR.

**Revision 2026-08-02 (session proud-fox-809, operator correction relayed via still-bat-561).**
[#7681](https://github.com/gunb-ai/gunbc/pull/7681) merged before its review completed, so the
first revision is a **merged rough draft**, not an accepted design. Not reverted — it is
document-only and no promoted generation depends on it. This revision applies nine operator
corrections; §12 records what each supersedes. The largest, as the operator stated it:

> **Not:** a candidate carries a `FixForwardProof`.
> **But:** the actual `DeclaredTransition` derives exact **transition exercises**, whose stage
> and behavior receipts establish successor capability, under an explicit **BootstrapGenesis**
> and a retained anchor with a named **`NoTrustedFixForwardPath`** failure state.

Parent context: v1-deletion / self-host lane (`still-bat-561`). Operator framing: self-host
must not collapse to one boolean fixed-point check. Each compiler stage needs durable
provenance — implementation, inputs, outputs, refusal, seed fallback, named consumers — and
emission must **fail fast per stage** with a typed, located reason, never deferring
classification to review or a corpus-wide rerun.

**Downstream:** [#7683](https://github.com/gunb-ai/gunbc/pull/7683) builds on this design;
correcting this note is on the critical path for that lane.

DESIGN refs: §2 (DFS existing machinery before minting vocabulary; no parallel ledger),
§3 (`gunbc.bootstrap.CompilerStage` is the stage authority; `FrontierProbeStage` is harness
vocabulary, not a second pipeline), §4 (emission is the downstream half of the two-stage
contract in [compiler-guarantee-recovery-gap-analysis.md](compiler-guarantee-recovery-gap-analysis.md)
§1), §5 (undeclared emission delta refuses — never widens to rerun-everything or
accept-anyway), §4b (discriminating RED: regression vs declared-change controls stay enrolled
after walls land), §6 (model-before-implement; scaffolds name dissolution triggers), §7
(behavioral equivalence over byte identity for self-host).

Related:
[module-identity-storage-binding-design.md](https://github.com/gunb-ai/gunbc/blob/d1f6922580962d14870577f1352e62966073c4bb/docs/plans/module-identity-storage-binding-design.md)
(BothWays delta lens) · [post-zero-regen-gate-placement.md](https://github.com/gunb-ai/gunbc/blob/d1f6922580962d14870577f1352e62966073c4bb/docs/plans/post-zero-regen-gate-placement.md) ·
`gunbc.bootstrap` · `gunbc.guarantee_measurement` · `v2.compiler.self_host` ·
`v2.compiler.self_host.emitter_producer_provenance` · `gunbc.generated_artifact`

---

## 0. One-sentence claim

> Every emission delta is admitted only when a **declared change set** names the compiler
> stages and artifact surfaces that may move; anything else is a **regression** and refuses.
> An accepted generation is either an explicit **`BootstrapGenesis`** — the declared trust
> root, unreachable from the candidate-promotion constructor — or a **`PromotedSuccessor`**,
> and promotion requires a **`SuccessorCapabilityReceipt`** whose **transition exercises** are
> derived from the `RequirementDelta` rows of the actual `DeclaredTransition`, each backed by
> total stage-execution receipts naming the same exact artifact identities, with the candidate
> never its own sole validator.
> The prior accepted generation is retained as an immutable **recovery anchor**; release is a
> separate transition consuming a valid successor-capability receipt, and when no candidate can
> establish one the lineage state is the named, fail-closed **`NoTrustedFixForwardPath`** —
> which authorizes no promotion, no warm reuse, no anchor release, no old-emitter deletion, and
> no requirement contraction.

---

## 1. Displaced cost (§6)

Boolean self-host/byte gates detect movement but do not establish stage-local provenance, intended change, successor capability, accepted lineage or a recovery path. Warm reuse and shared CI phases require those receipts rather than repeated discovery or review-classified intent.

## 2. DFS — existing machinery (reuse, do not fork)

Reuse these owners; the new obligations in §3 extend their joins:

- `v2.compiler.self_host.emitter_producer_provenance`: orthogonal `EmitterProducer` and `EmissionQualification`, per-module `ProducerEmissionReceipt`, and sole minter `mint_producer_emission_receipt`. `emitter_producer_mint_admission` refuses V2 labels when realized closure reaches `v1.compiler.emit_rust` (`^v2_emitter_in_seed_emit_rust_closure`). `emitter_produced_baseline` remains 0 until execution-measured verdict digests; `producer_emission_receipt_digest_scaffold_note` names symbol-identity digest scaffolding.
- `v2.compiler.self_host.canonical_emitted_bytes_digest`, `gunbc.instruments.floor_effect_gate_witness.regen_verify_gate_passes`, `v2.workflow.self_host_realized_comparison_gate`, and `gunbc.ci.ci_spec.RegenVerifyGate`: actual bytes, fail-closed regeneration and curated behavioral/staleness checks. They do not classify intent or establish changed-successor capability.
- `gunbc.bootstrap.CompilerStage` is the pipeline-stage authority; `ChangeClassification.affects_stages` names affected stages. `FrontierProbeStage` is distinct harness vocabulary. `v2.std.change.ChangeKind` / `ChangeSet` feed `v2.lens.affected_set`.
- `gunbc.generated_artifact` supplies registry/drift checks; `gunbc.guarantee_measurement` supplies per-path receipts. Extend these rather than minting parallel ledgers. BothWays quotient-delta in [module identity/storage binding](https://github.com/gunb-ai/gunbc/blob/d1f6922580962d14870577f1352e62966073c4bb/docs/plans/module-identity-storage-binding-design.md) is orthogonal to emission admission.

The [original design and correction record](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/emission-admission-stage-aware-pipeline-design.md) retains dated inventories and review provenance. Current contracts and controls follow.

---

## 3. New design obligations

§2 is reuse. **These are the genuinely new concepts** — detecting unexpected output movement
is insufficient; the design must answer whether the last accepted compiler can understand and
validate the changed requirements needed to build the repair.

### 3.1 Two revision identities — requirement and validation

**Correction 5.** The first revision folded the witness roster into `RequirementRevision`,
making a single witness enrollment a requirement bump demanding a declared transition — not
the intent, and it would price enrollment out of existence. The two facts separate:

```
RequirementRevision { semantic_contract_digest, stage_contract_digest, artifact_contract_digest }
ValidationRevision  { witness_roster_digest, validator_contract_digest }
```

- A **requirement revision** is the compiler's *obligation* set: source language contract,
  IR shape, stage contracts, emitter/artifact contracts, bootstrap requirements. It is not a
  git commit and not a generation index; two generations may share one.
- A **validation revision** is the *evidence* apparatus: the enrolled witness roster and the
  validator contracts that judge it.

**Laws.**

- Adding a witness changes the `ValidationRevision` only. It needs no bridge, and a
  validation-only change may apply to the **same** compiler generation.
- **Removing or weakening** a witness is not symmetric with adding one: it needs an explicit
  **validation transition**, because it shrinks the evidence that every downstream receipt was
  measured against.
- Every `DeclaredTransition`, candidate, and accepted generation names **both** revisions it
  was evaluated against. Comparing artifacts across different requirement revision IDs without
  an explicit `BridgeGeneration` protocol is `RefusedRevisionMismatch`.
- Neither identity is a stored transcribable field where it can be derived from content; a
  derived identity makes a forged one unrepresentable, not merely refused (DESIGN §4b —
  construction over validation).

### 3.2 Requirement delta and declared transition

**Correction 1 (first half).** A transition does not merely list surfaces that may move — it
names, in typed form, **what about the requirement contract changed**. §3.8's exercises derive
from that typed list, which is why a synthesized proof subject is unrepresentable rather than
merely discouraged.

```
RequirementDelta
  = RequirementAdded        { … }
  | RequirementStrengthened { … }
  | StageContractChanged    { stage, … }
  | RepresentationChanged   { … }
  | RequirementRetired      { … }

DeclaredTransition {
  from_requirement_revision: RequirementRevision,
  to_requirement_revision:   RequirementRevision,
  from_validation_revision:  ValidationRevision,
  to_validation_revision:    ValidationRevision,
  deltas:                    List<RequirementDelta>,
  allowed_surface_moves:     List<ArtifactSurface>,
  affected_stages:           List<CompilerStage>,
  evidence:                  TransitionEvidence
}
```

`TransitionEvidence` carries a **required machine leg** (the affected-set projection) and an
**optional** operator declaration. It is a record, not a coproduct — the whole of §9 Q1: a
coproduct would let an operator declaration stand *where* machine evidence is missing, precisely
the override Q1 forbids. A refused selection refuses the transition whatever the operator wrote.

**Law:** a candidate at the target revision is inadmissible without a `DeclaredTransition`
from the currently accepted generation's revision. Human-authored declarations are **inputs**
to admission, never overrides of missing evidence.

### 3.3 Declared-change vs regression (within one revision)

**Scope, narrowed (correction 8).** Stated **within one requirement revision, under one
accepted genesis**: it classifies candidate and audit dispositions, carries **no promotion
authority**, and does not reach across revisions. That narrowing lets stage 2 land before the
lineage stages — see §10.

Within a single requirement revision, admission classifies emission deltas:

| Verdict | Meaning |
|---|---|
| `AdmittedDeclared { stages, surfaces, evidence }` | Observed delta ⊆ declared change set; stage-execution receipts (§3.9) consistent with it |
| `RefusedRegression { stage, surface, cause }` | Moved **with** a declaration, but the required witness failed. Moving *without* a declaration is `RefusedUndeclaredDelta` below — the two causes have different remedies and must not share an arm |
| `RefusedUndeclaredDelta` | Delta on a registered surface with no `allowed_surface_moves` entry |
| `RefusedStageMismatch { declared, observed }` | Declared `Infer`-only but emit digest moved (or converse) |
| `RefusedStaleSubstrate { expected_revision, observed_revision }` | Warm receipt replay against wrong materialization revision |

**Law:** admission is **not** generation-to-generation equivalence. A declared emitter
migration may admit digest movement with behavioral green; an undeclared move refuses even
if bytes happen to match a prior generation.

### 3.4 Accepted generation — genesis and successor

**Correction 2.** The first revision stated every law relative to an existing
`PromotedGeneration` and never said how the first one exists. Not a footnote: the available
wrong answer — authoring a row declaring the current tree `Promoted` — is exactly the
self-promotion this section forbids, done by hand instead of by a candidate. So generation zero
gets a **structurally distinct constructor**:

```
AcceptedCompilerGeneration
  = BootstrapGenesis   { receipt: GenesisAdmissionReceipt }
  | PromotedSuccessor  { receipt: SuccessorCapabilityReceipt }

GenesisAdmissionReceipt {
  source_revision:              ExactSourceRevision,
  generated_artifact_identities: List<ExactArtifactIdentity>,
  toolchain_identity:           ExactToolchainIdentity,
  build_receipt:                StageExecutionReceipt,      // executing, not declared
  regen_receipt:                StageExecutionReceipt,
  required_behavior_receipts:   List<StageExecutionReceipt>,
  declared_bootstrap_residuals: List<BootstrapResidual>,    // explicit, enumerated, not a summary
  operator_trust_boundary:      NonEmptyStr
}
```

The current **v1 committed-Rust compiler is the genesis** — *not* because it proves itself
clean, but because it is the **explicit trust root**. Its residuals are declared and enumerated,
not argued away, and the trust boundary is stated in the receipt as an operator decision, not
implied by the tree's existence.

**Construction wall, the load-bearing one:** the generic candidate-promotion constructor
**must be incapable of producing a `BootstrapGenesis`**. No path through promotion makes a
candidate the trust root; a new genesis is always a fresh operator trust decision. This stops
*declare-the-current-tree-promoted-because-we-need-a-parent*.

A **candidate** is emitted and validated under candidate rules; it is not authoritative for
promotion, warm reuse, or merge admission, and **cannot be the sole authority promoting itself**
(§3.8's sole-validator condition).

**Non-blocking path:** candidate generation through the direct Rust door (`V1SeedEmitter` /
committed seed emit) **must not be blocked** by this design. The operator supports merging this
note **before** the first behavioral-module promotion; only promotion and authoritative
admission are gated.

### 3.5 Generation lineage graph

```
GenerationNode { index, accepted: AcceptedCompilerGeneration | Candidate, requirement_revision, validation_revision, producer }
GenerationEdge { parent, child, delta: EmissionDelta, transition: DeclaredTransition }
ArtifactBinding { path, generation, emitted_source_digest }
ConsumerRead { consumer_id, artifact_binding, admission_verdict }
```

**Laws:**

- Every `ArtifactBinding` points at exactly one `GenerationNode` with an honest
  `ProducerEmissionReceipt` or `SeedRetained` row.
- Consumers record `ConsumerRead` — not just Bool pass/fail.
- Warm replay refuses when `substrate_revision`, `generation`, or acceptance state does not
  match the stamping run (`RefusedStaleSubstrate`).
- Only `AcceptedCompilerGeneration` bindings are authoritative for merge admission and warm
  reuse — and that includes the genesis, which is accepted by declared trust rather than by
  promotion.

### 3.6 Retained recovery anchor — immutable, never referencing the future

**Correction 6.** The first revision gave the anchor a `retained_until: PromotedSuccessor`
field, naming a state that **does not exist when the anchor is minted** — fabricated at
construction or dishonest until backfilled. The anchor is therefore immutable with no forward
reference; its lifecycle is a separate state:

```
RecoveryAnchor { generation: AcceptedCompilerGeneration }   // immutable, minted in AnchorRetained

RecoveryAnchorState
  = AnchorRetained
  | AnchorReleaseEligible { successor, receipt: SuccessorCapabilityReceipt }
  | AnchorReleased        { successor, release_receipt }
```

**Laws.**

- An anchor is minted in `AnchorRetained`. Only a function **consuming a valid
  successor-capability receipt** may produce `AnchorReleaseEligible`.
- **Eligibility is not release.** Release is another explicit transition producing
  `AnchorReleased` with its own receipt; nothing is discharged as a side effect of a candidate
  existing.
- The anchor is not deleted on candidate mint; it remains the fallback that can still build the
  prior generation's artifacts if promotion refuses.

Rollback (`git checkout` an earlier commit) remains possible independently; it is **not**
the fix-forward story and does not discharge the anchor obligation.

### 3.7 Bridge-generation protocol (incompatible requirement changes)

When `DeclaredTransition` crosses incompatible **requirement** revisions (new IR shape, new
emitter contract, a retired or weakened requirement), a **bridge generation** mediates. *Not*
on that list after correction 5: a witness roster change moves the `ValidationRevision` only
and needs no bridge (§3.1) — the first revision listed it here, which would have demanded
expand-migrate-contract for enrolling a witness.

```
BridgeGeneration {
  anchor: RecoveryAnchor,
  candidate: Candidate,
  expand_phase: RequirementExpand,    // anchor understands expanded obligation surface
  migrate_phase: ArtifactMigrate,    // candidate artifacts produced under expanded contract
  contract_phase: RequirementContract // promoted revision is strictly smaller than expand
}
```

**Law:** bridge is mandatory when `from_requirement_revision` and `to_requirement_revision`
are not provably compatible by digest-inclusion alone. Skipping bridge and promoting across incompatible
revisions is `RefusedBridgeRequired`.

### 3.8 Transition exercises and the successor-capability receipt

**Correction 1 (second half) — `FixForwardProof` is DELETED as a free-standing concept.** In
the first revision it was a field name and a comment: no structure, no constructor. The failure
mode that invites is cheap — synthesize a fixture the anchor happens to fail on, watch the
candidate accept it, call that fix-forward. **A fixture the anchor cannot build proves nothing
about the requirement change that motivated the transition.**

The replacement is not a better-specified proof field: **the complete receipt IS the proof**,
and its subject is not free:

```
TransitionExercise {
  delta:          RequirementDelta,       // MUST be a member of the transition's deltas
  changed_source: ExactSourceIdentity,    // MUST be bound to that delta
  candidate_receipts: List<StageExecutionReceipt>,
  anchor_receipts:    List<StageExecutionReceipt>
}

SuccessorCapabilityReceipt {
  anchor_generation:    AcceptedCompilerGeneration,
  candidate_generation: Candidate,
  transition:           DeclaredTransition,
  exercises:            List<TransitionExercise>,
  validators:           List<ValidatorIdentity>
}
```

**A `TransitionExercise` may only be derived from an actual `RequirementDelta` in the
corresponding `DeclaredTransition`.** It is never authored beside the transition and checked
against it — derivation is the only way to obtain one, so an exercise about a subject the
transition does not name has no constructor.

**Admission conditions** (all required):

1. the receipt's transition **equals** the exercise's transition;
2. each exercise's delta is a **member** of that transition's deltas;
3. each exercise's changed source is **bound to that delta**;
4. every receipt in the exercise names **the same exact artifact identities** — not merely
   compatible ones;
5. the candidate is **not the sole validator** of its own capability.

**The nuance the first revision got wrong (operator-supplied):** the anchor **need not fail**
to build the exercise. For a compatible transition both generations may understand the changed
source. The required fact is that the **candidate can produce and validate a changed successor
under the new requirement revision**. *The anchor failing is never, by itself, positive
evidence* — it is a fact about the anchor, not a capability of the candidate; the first
revision's `anchor_outcome: Refused` condition mistook one for the other.

**Law:** `PromotedSuccessor` is unwritable without a `SuccessorCapabilityReceipt` satisfying
all five conditions. Digest equality between the two generations on unchanged surfaces is
necessary but not sufficient — it proves reproduction, not fix-forward.

**Dissolve-on:** when `frontier_probe_survey` binds execution-measured digests, lineage edges
and exercise receipts upgrade from scaffold to measured digests without changing graph shape.

### 3.9 Stage execution receipt — a total result, not a stamp

**Correction 4.** §6.1 and the staged plan both leaned on a `StageStamp` that §3 never typed —
the carrier for this document's headline cost: a stage-local failure has no receipt naming
**which** stage failed. `RefusedStageMismatch` does not answer that: it catches a
*mis-declaration*, not a stage's own execution outcome, and the two must not substitute for one
another.

**Before minting anything: check whether `gunbc.guarantee_measurement` extends to carry this.**
A parallel receipt beside it would be the §3 fork §2 exists to avoid.

```
StageExecutionSubject {
  generation, requirement_revision, validation_revision,
  stage:          CompilerStage,
  implementation: DeclarationRef,          // which code actually ran
  input_artifacts: List<ExactArtifactIdentity>,
  substrate_revision
}

StageExecutionVerdict = StageAccepted | StageRefused { … } | StageFallbackUsed { … }

StageExecutionReceipt { subject: StageExecutionSubject, verdict: StageExecutionVerdict, execution_identity }
```

`StageFallbackUsed` is a first-class third arm, not a shade of accepted: a stage that completed
*by falling back* has not established what a completed stage did; collapsing them is the
state-space conflation DESIGN §5 forbids.

**Law:** `StageStamp` is **only the content identity of a `StageExecutionReceipt`** — never a
free-standing label, and never the receipt itself.

### 3.10 Generation continuity — including the state that means stop

**Correction 3.** §5's retention law covers a defective candidate: *N* stays authoritative, a
refused candidate does not demote it, the tree is fine. The **bricking case is the inverse** —
the anchor is defective and no candidate can establish successor capability — and the first
revision had no state for it. A situation the model can represent but not name does not read as
absent; it reads as healthy.

```
GenerationContinuityVerdict
  = ProgressAvailable
  | CandidateRefusedAnchorRetained
  | BridgeRequired
  | AnchorDegradedButRepairable
  | NoTrustedFixForwardPath        // fail-closed emergency state
```

`NoTrustedFixForwardPath` **authorizes none of**: candidate promotion, warm reuse, anchor
release, old-emitter deletion, requirement contraction.

Its permitted next actions are **explicit and enumerated** — the enumeration is the point: an
unlisted escape is how this state would otherwise resolve into *use-the-candidate-anyway*:

1. revert to an earlier valid anchor;
2. retreat the transition;
3. build an expand-migrate-contract bridge (§3.7);
4. repair through an external trusted toolchain or a reviewed Rust artifact;
5. establish a new `BootstrapGenesis` under a fresh operator trust decision (§3.4).

**There is no `use the candidate anyway` arm.** Rollback does not dissolve this state on its
own: rolling back reaches the same defective anchor.

### 3.11 Byte-only admission must be proved by reachability

**Correction 7.** §9 Q2 permits formatting-only byte-contract artifacts to close on digest
alone. That is safe only if the *formatting-only* fact is **proved**, never authored — an
authored `formatting_only: true` is Q1's human override in different clothes: the author who
wants the digest-only path is exactly the one who would set the flag.

```
ByteOnlyAdmissionEvidence {
  serializer_purity_receipt,
  no_compiler_stage_consumes_bytes_as_semantic_input: ReachabilityNone,
  no_executable_path_consumes:                        ReachabilityNone,
  no_artifact_identity_or_requirement_derives_from_presentation_spelling: ReachabilityNone
}
```

**Law: unknown reachability means semantic validation is required.** Not "assume byte-only",
not "counted debt" — an unanswered reachability question is unanswered, and the fail-closed arm
is the expensive one. **There must be no authored `formatting_only` anywhere in the
realization.**

**Two prerequisites, measured against the live tree 2026-08-02** (while probing a G0 carrier
since withdrawn, and independently re-verified against the carrier). Prerequisites for this
section's realization, not incidental notes.

**(a) The existing axis is wrong, not merely absent.** `gunbc.generated_artifact`'s
`artifact_commit_policy` assigns `CommitRequired { consumer: GitProtocol }` to
`Stage0CrateLayoutGeneratedRsArtifact` *and* to `GitignoreArtifact` — a compiled Rust source and
a git config file carry identical classification, so consumer identity cannot separate
executable from non-executable. But the diagnosis is not *a missing axis*: **the consumer of a
generated `.rs` file is `rustc`. Git is how the file is stored.** Labelling its consumer
`GitProtocol` conflates storage with consumption — the same conflation for all four `Stage0*`
rows: three `.dag` artifacts whose consumer is the gunbc compiler and one `.rs` whose consumer
is `rustc`, all filed under the protocol that merely carries them. That reframes the
prerequisite from *invent a new fact* to **correct a fact already carried** — the cheaper and
more likely repair.

**(b) There is no emitted-artifact-to-module join.** `GeneratedArtifact` is a closed enum of
specific artifacts and emitted stage0 Rust as a surface is not among them (the generated stage0
surface is now a derived population, `gunbc.stage0.stage0_rust_source_lifecycle_scaffold`
`derived_generated_stage0_repo_paths`, not a roster), and nothing today derives **which
compiler module emitted a given `.rs` path** -- exactly why that derivation is the complement of
the crate-layout claim rather than a module-to-path join. So an authorization minted for a
`.dag` module cannot be checked against a `.rs` surface at all. This also constrains §10 stage
5: a transition exercise over an emitted `.rs` artifact has no identity join today, and
condition 4 of §3.8 — every receipt naming the same exact artifact identities — cannot be
satisfied for such a surface until the join exists. Stated as a prerequisite, not modelled
around.

---

## 4. Fix-forward de-risk (replaces rollback-only framing)

Rollback can recover old artifacts without enabling the repair's changed requirements. §§3.6–3.10 therefore require an immutable anchor, bridge for incompatible requirements, transition-derived exercises, total stage receipts and a named deadlock. Reproduction alone cannot promote; anchor failure alone cannot prove candidate capability. Version-control rollback remains independent and introduces no new rollback machinery.

---

## 5. Temporal graph (explicit lifecycle)

```
BootstrapGenesis (generation 0)                [GenesisAdmissionReceipt; explicit trust root]
        │                                       NOT reachable from candidate promotion
        ▼
AcceptedCompilerGeneration N
        │
        ├─ DeclaredTransition (N → N+1)        [deltas + requirement/validation revision IDs]
        ├─ RecoveryAnchor N : AnchorRetained   [immutable; no forward reference]
        │
        ▼
Candidate N+1
        │
        ├─ stage execution receipts            [total; Accepted | Refused | FallbackUsed]
        ├─ transition exercises                [derived from the transition's own deltas]
        ├─ successor-capability admission      [5 conditions, §3.8]
        │
        ├──── refused ──▶ GenerationContinuityVerdict
        │                   CandidateRefusedAnchorRetained | BridgeRequired
        │                   | AnchorDegradedButRepairable | NoTrustedFixForwardPath
        ▼
PromotedSuccessor N+1                          [only if all conditions hold]
        │
        ├─ RecoveryAnchor N : AnchorReleaseEligible   [receipt consumed, not yet released]
        └─ RecoveryAnchor N : AnchorReleased          [separate explicit transition]
```

**Genesis law:** generation 0 exists by a `GenesisAdmissionReceipt` naming an explicit operator
trust boundary and its declared residuals — never by promotion, never by a row asserting the
state it is supposed to establish.

**Retention law:** generation *N* stays accepted and authoritative until *N+1* establishes
successor capability. A refused candidate does not demote *N*, and eligibility for release is
not release.

**Candidate law:** generation *N+1* in candidate state may emit, compile, and run witnesses —
including through the direct Rust seed door — but cannot promote itself and cannot authorize
warm reuse or merge admission.

---

## 6. Stage-aware pipeline model

### 6.1 Phases on one substrate (`phased-single-process-ci`)

```
┌─ Phase: Regen ─────────────────────────────────────────────────┐
│  regen_stage0 / generated_artifact_gate                        │
│  emit StageExecutionReceipt + EmissionDelta + candidate state  │
│  refuse: RefusedUndeclaredDelta (fail fast — floor not run)  │
└───────────────────────────┬──────────────────────────────────┘
                            │ stage receipts + lineage edges
┌─ Phase: Floor ────────────▼──────────────────────────────────┐
│  compile-clean + witness corpus on shared materialization    │
│  inherit receipts; refuse RefusedStaleSubstrate if replay    │
│  candidate artifacts: stage + behavior validation            │
└───────────────────────────┬──────────────────────────────────┘
                            │
┌─ Phase: Admission ────────▼──────────────────────────────────┐
│  warm-merge-admission / merge gate                           │
│  accepted-generation bindings only; validate transition + reads│
│  AdmissionVerdict → merge allow/refuse                         │
└────────────────────────────────────────────────────────────────┘
```

**Ordering law:** regen still gates floor (existing `ci_regen_floor_skip_policy_note`).
Admission may consume warm receipts only for `AcceptedCompilerGeneration` bindings when
`substrate_revision`, `DeclaredTransition`, and generation lineage match the stamping run.

### 6.2 Prelude duplication metric

First slice for `phased-single-process-ci`: attribute resolve/index/discovery time per phase
with and without stage receipts. **RED control:** if phase *N+1* repeats a resolve whose inputs
are unchanged per its receipt, count > 0 ⇒ `RefusedStaleSubstrate` or a dedicated prelude
duplication refusal (roadmap red_control, made executable).

---

## 7. Consumer mapping

| Consumer | Declared-change source | Regression / refusal signal |
|---|---|---|
| `warm-merge-admission` | accepted-generation receipts + `DeclaredTransition` at merge base | Undeclared `EmissionDelta`; candidate binding on warm path |
| `generated_artifact_drift_gate` | diff touches artifact authority modules | generate ≠ committed without `allowed_surface_moves` |
| `RegenVerifyGate` / `regen_verify_gate_passes` | diff touches emit modules or frontier rows | seed digest mismatch undeclared |
| `self_host_realized_comparison` | frontier flip / emitter edit in PR | behavioral witness fails; digest relabelled as equivalence |
| `mint_producer_emission_receipt` | declared emitter migration | mint with `BuildRefused` or wrong producer in closure |
| promotion gate (new) | `SuccessorCapabilityReceipt` exercises derived from transition deltas | reproduction only; exercise subject the transition does not name; candidate as sole validator |
| `gunbc.guarantee_measurement` | probe declares subject revision | `ExecutionDiverged` on emission path with clean compile |

---

## 8. Witness / RED discipline (§4b, §5)

**Correction 9 — the first revision's table was wrong in kind, not merely incomplete.** It
labelled *observing a refusal* a GREEN control, inverting what a control is: a fixture whose
verdict is known, where the discriminating fact is the **mutation that must change that
verdict**. Its `RefusedRegression` row ("declaration cannot green behavioral failure") was not a
control at all — it restated the law rather than naming an input. Four columns; the fourth
makes a row evidence:

| Subject | Fixture | Expected verdict | Mutation that MUST change the verdict |
|---|---|---|---|
| Declared delta | emitter edit + regen; transition declares the surface, the stage, and a delta binding the changed source | admitted | remove the surface from `allowed_surface_moves` ⇒ `RefusedUndeclaredDelta` |
| Undeclared delta | same emitter edit; transition declares no surface | `RefusedUndeclaredDelta` | add the surface to the transition ⇒ admitted |
| Behavioral regression | planted emitter defect, no source change, behavior receipt red | refused | repair the defect so the behavior receipt executes green ⇒ admitted |
| Stage mismatch | transition declares `Infer` only; emit artifact identity moves | `RefusedStageMismatch` | declare `Emit` in `affected_stages` ⇒ admitted |
| Stage-local failure | emit stage refuses; upstream stages accept | `StageRefused` located at `Emit`, refusal names the stage | make the emit stage accept ⇒ located refusal disappears; make `Infer` refuse instead ⇒ located refusal moves to `Infer` |
| Candidate self-promotion | candidate supplies its own capability receipt with itself as sole validator | refused (sole-validator condition) | supply an independent validator ⇒ condition 5 satisfied |
| Fix-forward | exercise derived from a delta the transition names; candidate receipts green under the new requirement revision | successor capability established | point the exercise at a subject the transition does **not** name ⇒ no exercise can be derived; **and** separately: make the anchor also succeed ⇒ verdict must **not** change (an anchor failure is not the evidence) |
| Anchor release | valid successor-capability receipt consumed | `AnchorReleaseEligible` | withhold the receipt ⇒ stays `AnchorRetained`; reaching `AnchorReleased` requires the separate release transition |
| Continuity deadlock | anchor defective, every candidate refused | `NoTrustedFixForwardPath` | repair the anchor ⇒ `AnchorDegradedButRepairable`; land a capable candidate ⇒ `ProgressAvailable` |

The fix-forward row carries **two** mutations on purpose: the first proves the subject is bound
to the transition; the second proves the receipt does not secretly depend on the anchor failing
— the confusion corrected in §3.8.

Probes that go green when a wall lands **flip to permanent regression controls** — they do not
retire (§4b dissolution-on rule).

---

## 9. Operator decisions (resolved)

### Q1 — Human override

**Decision:** A human may author a **typed declaration of intended change** as an input to
admission. A human may **not** override missing affected-set or requirement evidence.

- Author-declared change = **input** to `DeclaredTransition`, not an admitting escape hatch.
- A frontier row may authorize **candidate** or **shadow** work; it must **not** authorize
  promotion.

### Q2 — Behavioral vs digest

**Decision:** Stricter **C**. *(Amended 2026-08-02 — the formatting-only clause now requires
the reachability proofs of §3.11; it is a proved property, never an authored label.)*

- Digest equality proves **byte agreement only**.
- Formatting-only byte-contract artifacts may close on digest alone **only with
  `ByteOnlyAdmissionEvidence` (§3.11)**; unknown reachability requires semantic validation.
- `SelfEmitted`, executable stage0, and semantic compiler surfaces require **build and
  behavioral evidence** appropriate to the declared requirement revision.
- A digest must **never** be relabelled `BehavioralEquivalent`.

### Q3 — Scaffold auto-widening

**Decision:** **B only**.

- No scaffold class automatically widens production emission authority.
- Each admitted move needs an **exact row** or a derived exact surface identity.
- A class-level template may generate **Candidate** rows; it cannot confer promotion authority.

### Q4 — Unknown surfaces

**Decision:** **A** at authoritative admission.

- `SurfaceUnknown` **refuses** promotion and warm reuse.
- It may remain `FrontierAccepted` only in a clearly non-authoritative candidate or audit
  state; it cannot merge into the promoted generation as merely counted debt.

### Q5 — BothWays quotient delta

**Decision:** **B**.

- A lawful semantic representation change is **not** automatically an admitted emission
  change; the target artifact surface still needs an **explicit declared entry**.
- BothWays answers whether the representation change is lawful; emission admission answers
  whether this artifact was allowed to move in this generation. Neither entails the other.

---

## 10. Staged implementation plan (future PRs — not this document)

**Correction 8 — reordered.** The first revision put declared-change derivation (its stage 2)
before generation state (its stage 4), while stating §3.2 admissibility against a promoted
generation that would not exist until stage 4. Either stage 2 was narrower than §3.2 or the
order was wrong. Resolved by **narrowing stage 2 explicitly** (§3.3) and landing the genesis
first, so a lineage root exists before anything is stated relative to one.

| Stage | Deliverable | Dissolution trigger |
|---|---|---|
| **0 — this document** | Corrected design; superseded claims marked (§12) | Operator review of *this* revision |
| **1 — genesis + identities + stage results** | `BootstrapGenesis` / `GenesisAdmissionReceipt`; `RequirementRevision` and `ValidationRevision`; **total** `StageExecutionReceipt` (extend `gunbc.guarantee_measurement` if it reaches) | Implicit trust root; fused revision identity; untyped stage stamp |
| **2 — within-revision classification** | Declared-delta vs regression **only**: same requirement revision, same accepted genesis, candidate/audit disposition, **no promotion authority** (§3.3) | Hand-classified PR intent in review prose |
| **3 — cross-revision transition** | `RequirementDelta` + `DeclaredTransition` derivation wired to affected-set; refuse when selection fails | Within-revision classification standing in for a transition |
| **4 — lineage + anchor lifecycle** | `GenerationNode` / `ArtifactBinding` / `ConsumerRead`; `RecoveryAnchor` + `RecoveryAnchorState` | Implicit generation in producer tag only |
| **5 — exercises + successor capability** | `TransitionExercise` derived from transition deltas; `SuccessorCapabilityReceipt` with its five admission conditions | Reproduction-only promotion |
| **6 — continuity + bridge** | `GenerationContinuityVerdict` incl. `NoTrustedFixForwardPath`; expand-migrate-contract bridge | Unnamed deadlock; big-bang incompatible promotion |
| **7 — direct Rust door** | Bind the direct Rust door as a **candidate-producing** consumer | Candidate emission outside the lineage graph |
| **8 — promotion gate + anchor release** | Promotion gate; `AnchorReleaseEligible` → `AnchorReleased` as an explicit transition | Release as a side effect of a candidate existing |
| **9 — warm-merge join** | #7522 admits accepted-generation receipts only, with matching transition + lineage | Cold rerun as default merge path |
| **10 — phased single process** | Shared substrate; prelude duplication witness | Per-phase `process_shared_index` duplication |

Guarantee-path measurement rows (the first revision's stage 9) are not a separate stage: stage
1's stage-execution receipt is that measurement, and a second one beside
`gunbc.guarantee_measurement` is the fork §2 exists to prevent.

---

## 11. The superseded sign-off bar, and why it did not catch this

The first revision ended in a sign-off bar whose completion read as merge readiness. It is
**retained as a record, annotated**, not deleted — the durable lesson is *how* it failed, not
that it was incomplete, and that lesson would be lost with the text:

> **Every item it named is satisfiable while the highest-stakes concepts stay undefined.**

Read against the merged draft it passed: §2's symbols resolved, §3's obligations were explicit,
§4 stated fix-forward, §5's graph was there, §9's decisions were recorded, the doc-graph bind
landed, no code shipped. All eight items met — while `FixForwardProof` was a field name with no
structure, `StageStamp` was named and never typed, and there was no generation zero. **A
checklist that counts sections cannot detect an undefined concept inside one.** That is why §8's
witness table now asks, per row, which *mutation must change the verdict*: a bar phrased as
presence is satisfiable by presence; one phrased as discrimination is not.

The original items, retained verbatim as the record:

1. All symbols in §2 resolve on main (grep-verified).
2. §3 obligations are explicit — not reduced to declared-change + lineage alone.
3. §4 states fix-forward (not rollback-only) de-risk.
4. §5 temporal graph is explicit.
5. §9 operator decisions are recorded (not open questions).
6. Doc graph bind lands (`gunbc.doc_graph_roots` → this slug).
7. No implementation code ships in the design PR.
8. Candidate generation through the direct Rust door remains unblocked; only promotion
   and authoritative admission are gated.

### What this document is not

The bar is superseded as a *readiness* device: it let #7681 merge before its review finished.

- This document confers **no merge readiness**. Being written does not make it accepted.
- It confers **no promotion authority**. Nothing here authorizes promoting a generation,
  releasing an anchor, or reusing a warm receipt; those come from the receipts §3 describes,
  once they exist and execute.
- It confers **no roadmap acceptance**. A roadmap row is accepted by its own explicit
  acceptance, never by a design note describing it.
- It ships **no implementation code**; each §10 stage is a separately dispatched, separately
  reviewed PR.

Item 8 alone survives as a live constraint rather than record, being a scope statement and not
a readiness claim: candidate generation through the direct Rust door remains unblocked; only
promotion and authoritative admission are gated. Carried into the staged plan as §10 stage 7.

---

## 12. Correction record (2026-08-02)

The [immutable nine-correction record](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/emission-admission-stage-aware-pipeline-design.md) retains each superseded statement and the first revision's merged-but-unaccepted standing. The resulting obligations are in §§3.1–3.11, the discriminating controls in §8, decisions in §9 and ordered implementation in §10. In particular, validation-only enrollment needs no bridge; undeclared movement differs from declared behavioral regression; stage execution extends the existing guarantee measurement rather than creating a parallel receipt.
