# Emission admission + stage-aware pipeline (declared-change vs regression)

> **AMENDED — awaiting operator sign-off.** The nine corrections outstanding against the merged
> first revision are applied (§12); this revision has not been signed. **This document does not
> authorize promotion, warm reuse, recovery-anchor release, or deletion of a prior compiler
> generation. Candidate/shadow generation may continue.** That clause stands until the operator
> signs — not until the corrections land.
>
> No roadmap acceptance and no promotion authority derives from this document.

*(This replaces the rough-draft banner that recorded the nine corrections as outstanding. They
are no longer outstanding; the document is no longer a rough draft; it is also not signed.)*

**Status:** design-only (model-before-implement). This document is the deliverable for
work item `node://adhoc-48a1f19c-1f8` (session calm-eagle-92). No load-bearing `.dag`
carrier lands in the design PR; each implementation stage below is a separately dispatched,
separately-signed PR.

**Revision 2026-08-02 (session proud-fox-809, operator correction relayed via still-bat-561).**
[#7681](https://github.com/gunb-ai/gunbc/pull/7681) merged before its review completed, so the
first revision is a **merged rough draft**, not an accepted design. It is not reverted — it is
document-only and no promoted generation depends on it. This revision applies nine operator
corrections; §12 records what each one supersedes. The single largest one, stated as the
operator stated it:

> **Not:** a candidate carries a `FixForwardProof`.
> **But:** the actual `DeclaredTransition` derives exact **transition exercises**, whose stage
> and behavior receipts establish successor capability, under an explicit **BootstrapGenesis**
> and a retained anchor with a named **`NoTrustedFixForwardPath`** failure state.

Parent context: v1-deletion / self-host lane (`still-bat-561`). The operator framing:
self-host must not collapse to one boolean fixed-point check. Each compiler stage needs
durable provenance — implementation, inputs, outputs, refusal, seed fallback, and named
consumers — and emission must **fail fast per stage** with a typed, located reason rather
than deferring classification to review or a corpus-wide rerun.

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

Related: [five-minute-ci-gate-design.md](five-minute-ci-gate-design.md)
(`warm-merge-admission`, `phased-single-process-ci`) ·
[seed-honesty-discharge-design.md](seed-honesty-discharge-design.md) ·
[module-identity-storage-binding-design.md](module-identity-storage-binding-design.md)
(BothWays delta lens) · [post-zero-regen-gate-placement.md](post-zero-regen-gate-placement.md) ·
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

| Cost today | Mechanism |
|---|---|
| **Self-host is one boolean** | Frontier disposition, regen verify, and realized comparison each answer a single match/mismatch bit. A stage-local refusal (infer ok, emit wrong) has no durable receipt tying *which stage* failed — the `#7485` containment class. |
| **Detects movement, not fix-forward** | Byte gates detect unexpected output movement but cannot answer: *can the last accepted compiler understand and validate the changed requirements needed to build the repair?* A defective generation may be the only one that can rebuild its own old source while being unable to compile the new requirement revision. |
| **Warm receipts cannot be trusted for merge** | `warm-merge-admission` can stamp resolve/materialization receipts, but has no typed rule for whether an emission delta was *expected* from the PR's source change. Stale-base refusals exist; *undeclared emitter drift* does not. |
| **Phases pay duplicate prelude work** | `phased-single-process-ci` wants regen, floor, and admission on one substrate. Without per-stage execution receipts, each phase assumes the prior phase might have silently changed upstream facts — discovery/resolve/index rerun anyway. |
| **Byte gates cannot distinguish intent** | `generated_artifact_drift_gate`, `RegenVerifyGate`, and `self_host_realized_comparison` answer only *match / mismatch*. An intentional emitter migration with regen is indistinguishable from an emitter bug — review becomes the classifier (§5 specification-without-execution). |
| **Producer provenance is partial** | `EmitterProducer` × `EmissionQualification` and `mint_producer_emission_receipt` land per-module receipts, but digest fields remain scaffold-keyed (`producer_emission_receipt_digest_scaffold_note`) and `emitter_produced_baseline` is zero — no join from PR facts to admitted vs regression disposition. |
| **Generation is implicit** | `V2EmitterNative { generation: Int }` names a generation index on the producer axis only. There is no graph of *which generation's emitted artifact* a consumer read, no candidate/accepted distinction, and no recovery anchor — so native cutover cannot refuse stale-generation replay without widening to cold rerun. |
| **The trust root is unstated** | The v1 committed-Rust compiler is what everything is ultimately built by, but nothing names it as the trust root, states its boundary, or enumerates its residuals. An unstated root cannot be reasoned about, and its absence is what makes *declare the current tree promoted* look like a bookkeeping step rather than a self-promotion (§3.4). |

---

## 2. DFS — existing machinery (reuse, do not fork)

Apply DESIGN §2 before proposing vocabulary: the concept DAG already carries most of the
stage/provenance spine. **§3 names the genuinely new obligations**; everything below is
live on main and should be cited, extended, or joined — not re-invented.

### 2.1 Producer × qualification provenance (live)

Authority: `v2.compiler.self_host.emitter_producer_provenance`.

| Symbol | Role |
|---|---|
| `EmitterProducer` | Orthogonal producer axis: `V1SeedEmitter` \| `V2EmitterInterpreted` \| `V2EmitterNative { generation }` |
| `EmissionQualification` | Qualification axis: `SourceProduced` \| `CargoGreen` \| `BehavioralEquivalent` \| `ProductionRouted` |
| `ProducerEmissionReceipt` | Per-module receipt: module/emitter/emitted digests, producer, qualifications, build result, behavioral probe coordinates, `RealizedEmitterClosure` |
| `emitter_producer_mint_admission` | **Construction wall:** `V2EmitterInterpreted` / `V2EmitterNative` refuse when `v1.compiler.emit_rust` is reachable in `realized_closure` (`^v2_emitter_in_seed_emit_rust_closure`) |
| `mint_producer_emission_receipt` | Sole minter for `ProducerEmissionReceipt`; refuses when mint admission refuses |
| `emitter_produced_baseline` | Counted baseline `0` — frontier `EmitterProduced` rows stay empty until execution-measured verdict digests land (`frontier_probe_survey` dissolve-on) |
| `producer_emission_receipt_digest_scaffold_note` | Honest scaffold: digests still keyed on `symbol_identity_digest` until measured binding |

**What this already buys:** producer/qualification decomposition (replacing `SelfEmitted`
state-space conflation), a mint wall against hand-authored receipts, and structural refusal
of mislabeled V2 producers in seed closures.

**What it does not yet buy:** requirement-revision identity, candidate/promoted lifecycle,
fix-forward validation, recovery anchors, or PR-level declared-change vs regression.

### 2.2 Emitted-byte digest + comparison gates (live)

| Symbol | Role |
|---|---|
| `v2.compiler.self_host.canonical_emitted_bytes_digest` | Host-grounded digest over actual emitted bytes (`Medium<String>`) |
| `tools.floor_effect_gate_witness.regen_verify_gate_passes` | CI floor gate: `RegenVerifyGate` compares regen output to committed Rust |
| `v2.workflow.self_host_realized_comparison_gate` | Behavioral + staleness gates over realized comparison transport |
| `gunbc.ci_spec.RegenVerifyGate` | Spec row; regen skip policy keyed on merge-base diff vs regen input closure |

**What this already buys:** fail-closed byte mismatch on the regen path and behavioral
witnesses on a curated roster.

**What it does not yet buy:** classification of *why* bytes moved, or proof that a new
generation can build the *changed* successor (not merely reproduce the old one).

### 2.3 Stage vocabulary (live, lightly joined)

| Symbol | Role |
|---|---|
| `gunbc.bootstrap.CompilerStage` | Canonical pipeline stages (`Tokenize`…`Emit`) |
| `gunbc.bootstrap.ChangeClassification.affects_stages` | Bootstrap-authoring: which stages a bootstrap edit may move |
| `v2.compiler.self_host.frontier_probe_types.FrontierProbeStage` | Harness probe positions (`ProbeStageAssemble` / `ProbeStageEmit` / …) — distinct from `CompilerStage` by design |
| `v2.std.change.ChangeKind` + `ChangeSet` | Source-graph change kinds; consumed by `v2.lens.affected_set` |

### 2.4 Neighbors (consume, do not redefine)

| Authority | Relationship |
|---|---|
| `gunbc.generated_artifact` | Registry + drift witnesses — byte fixed-point; gains a declared-change arm |
| `gunbc.guarantee_measurement` | Per-path receipts — gains admission verdict projection on emission paths |
| [module-identity-storage-binding-design.md](module-identity-storage-binding-design.md) | BothWays quotient-delta lawful edits — orthogonal semantic comparison, not authority-vs-seed |

---

## 3. New design obligations

Everything in §2 is reuse. **These are the genuinely new concepts** — detecting unexpected
output movement alone is insufficient; the design must answer whether the last accepted
compiler can understand and validate the changed requirements needed to build the repair.

### 3.1 Two revision identities — requirement and validation

**Correction 5.** The first revision folded the witness roster into `RequirementRevision`,
which made enrolling a single witness a requirement bump demanding a declared transition.
That is not the intent and would price enrollment out of existence. The two facts separate:

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
- Neither identity is a stored transcribable field where it can be derived from content;
  a derived identity makes a forged one unrepresentable rather than merely refused
  (DESIGN §4b — construction over validation).

### 3.2 Requirement delta and declared transition

**Correction 1 (first half).** A transition does not merely list surfaces that may move — it
names, in typed form, **what about the requirement contract changed**. That typed list is what
§3.8's exercises are derived from, and it is the reason a synthesized proof subject is
unrepresentable rather than merely discouraged.

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
**optional** operator declaration. It is a record, not a coproduct, and the distinction is the
whole of §9 Q1: a coproduct would let an operator declaration stand *where* the machine
evidence is missing, which is precisely the override Q1 forbids. A refused selection refuses
the transition no matter what the operator wrote.

**Law:** a candidate at the target revision is inadmissible without a `DeclaredTransition`
from the currently accepted generation's revision. Human-authored declarations are **inputs**
to admission, never overrides of missing evidence.

### 3.3 Declared-change vs regression (within one revision)

**Scope, narrowed (correction 8).** This section is stated **within one requirement revision,
under one accepted genesis**. It classifies candidate and audit dispositions; it carries **no
promotion authority** and does not reach across revisions. That narrowing is what lets stage 2
land before the lineage stages — see §10.

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
`PromotedGeneration` and never said how the first one exists. That is not a footnote: the
available wrong answer — authoring a row declaring the current tree `Promoted` — is exactly
the self-promotion this section forbids, performed by hand instead of by a candidate. So
generation zero gets a **structurally distinct constructor**:

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
clean, but because it is the **explicit trust root**. The residuals it carries are declared and
enumerated rather than argued away, and the trust boundary is stated in the receipt as an
operator decision rather than implied by the tree's existence.

**Construction wall, and the reason it is the load-bearing one:** the generic
candidate-promotion constructor **must be incapable of producing a `BootstrapGenesis`**. A
candidate cannot become the trust root by any path through promotion; establishing a new
genesis is always a fresh operator trust decision. This is what stops
*declare-the-current-tree-promoted-because-we-need-a-parent*.

A **candidate** is emitted and validated under candidate rules and is not authoritative for
promotion, warm reuse, or merge admission. It **cannot be the sole authority promoting itself**
(§3.8's sole-validator condition).

**Non-blocking path:** candidate generation through the direct Rust door (`V1SeedEmitter` /
committed seed emit) **must not be blocked** by this design. The operator supports merging
this design note **before** the first behavioral-module promotion; only promotion and
authoritative admission are gated.

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
field, naming a state that **does not exist at the moment the anchor is minted**. Such a field
is either fabricated at construction or dishonest until something backfills it. The anchor is
therefore immutable and carries no forward reference; its lifecycle is a separate state:

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
  `AnchorReleased` with its own receipt, so nothing is discharged as a side effect of a
  candidate existing.
- The anchor is not deleted on candidate mint. It remains the fallback that can still build the
  prior generation's artifacts if promotion refuses.

Rollback (`git checkout` an earlier commit) remains possible independently; it is **not**
the fix-forward story and does not discharge the anchor obligation.

### 3.7 Bridge-generation protocol (incompatible requirement changes)

When `DeclaredTransition` crosses incompatible **requirement** revisions (new IR shape, new
emitter contract, a retired or weakened requirement), a **bridge generation** mediates. Note
what is *not* on that list after correction 5: a witness roster change moves the
`ValidationRevision` only and needs no bridge (§3.1) — the first revision listed it here, which
would have demanded expand-migrate-contract for enrolling a witness.

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
mode that shape invites is cheap and predictable — synthesize a fixture the anchor happens to
fail on, watch the candidate accept it, call that fix-forward. **A fixture the anchor cannot
build proves nothing about the requirement change that motivated the transition.**

The replacement is not a better-specified proof field. It is that **the complete receipt IS the
proof**, and its subject is not free:

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
corresponding `DeclaredTransition`.** It is not authored beside the transition and then checked
against it — deriving it is the only way to obtain one, so an exercise about a subject the
transition does not name has no constructor.

**Admission conditions** (all required):

1. the receipt's transition **equals** the exercise's transition;
2. each exercise's delta is a **member** of that transition's deltas;
3. each exercise's changed source is **bound to that delta**;
4. every receipt in the exercise names **the same exact artifact identities** — not merely
   compatible ones;
5. the candidate is **not the sole validator** of its own capability.

**The nuance the first revision got wrong, and which the operator supplied:** the anchor
**need not fail** to build the exercise. For a compatible transition both generations may
understand the changed source, and that is fine. The required fact is that the **candidate can
produce and validate a changed successor under the new requirement revision**. *The anchor
failing is never, by itself, positive evidence* — an anchor failure is a fact about the anchor,
not a capability of the candidate, and the first revision's `anchor_outcome: Refused` condition
mistook one for the other.

**Law:** `PromotedSuccessor` is unwritable without a `SuccessorCapabilityReceipt` satisfying
all five conditions. Digest equality between the two generations on unchanged surfaces is
necessary but not sufficient — it proves reproduction, not fix-forward.

**Dissolve-on:** when `frontier_probe_survey` binds execution-measured digests, lineage
edges and exercise receipts upgrade from scaffold digests to measured digests without
changing graph shape.

### 3.9 Stage execution receipt — a total result, not a stamp

**Correction 4.** §6.1 and the staged plan both leaned on a `StageStamp` that §3 never typed —
and it is the carrier for this document's headline cost, which is that a stage-local failure has
no receipt naming **which** stage failed. `RefusedStageMismatch` does not answer that: it
catches a *mis-declaration*, not a stage's own execution outcome, and the two must not be
allowed to substitute for one another.

**Before minting anything: check whether `gunbc.guarantee_measurement` extends to carry this.**
A parallel receipt beside it would be the §3 fork this document spends §2 avoiding.

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

`StageFallbackUsed` is a first-class third arm rather than a shade of accepted: a stage that
completed *by falling back* has not established what a stage that completed did, and collapsing
them is the state-space conflation DESIGN §5 forbids.

**Law:** `StageStamp` is **only the content identity of a `StageExecutionReceipt`** — never a
free-standing label, and never the receipt itself.

### 3.10 Generation continuity — including the state that means stop

**Correction 3.** §5's retention law covers the candidate being defective: *N* stays
authoritative, a refused candidate does not demote it, and the tree is fine. The **bricking case
is the inverse** — the anchor is the defective generation and no candidate can establish
successor capability — and the first revision had no state for it at all. A situation the model
can represent while having no name for it does not read as absent; it reads as healthy.

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

Its permitted next actions are **explicit and enumerated** — and the enumeration is the point,
because an unlisted escape is how this state would otherwise resolve itself into
*use-the-candidate-anyway*:

1. revert to an earlier valid anchor;
2. retreat the transition;
3. build an expand-migrate-contract bridge (§3.7);
4. repair through an external trusted toolchain or a reviewed Rust artifact;
5. establish a new `BootstrapGenesis` under a fresh operator trust decision (§3.4).

**There is no `use the candidate anyway` arm.** Note that rollback does not dissolve this
state on its own: rolling back reaches the same defective anchor.

### 3.11 Byte-only admission must be proved by reachability

**Correction 7.** §9 Q2 permits formatting-only byte-contract artifacts to close on digest
alone. That permission is only safe if the *formatting-only* fact is **proved**, never authored
— an authored `formatting_only: true` is Q1's human override wearing different clothes, since
the author who wants the digest-only path is exactly the author who would set the flag.

```
ByteOnlyAdmissionEvidence {
  serializer_purity_receipt,
  no_compiler_stage_consumes_bytes_as_semantic_input: ReachabilityNone,
  no_executable_path_consumes:                        ReachabilityNone,
  no_artifact_identity_or_requirement_derives_from_presentation_spelling: ReachabilityNone
}
```

**Law: unknown reachability means semantic validation is required.** Not "assume byte-only",
not "counted debt" — a reachability question that has not been answered has not been answered,
and the fail-closed arm is the expensive one. **There must be no authored `formatting_only`
anywhere in the realization.**

**Two prerequisites, measured against the live tree 2026-08-02** (while probing a G0 carrier
since withdrawn, and independently re-verified against the carrier). They are prerequisites for
this section's realization, not incidental notes.

**(a) The existing axis is wrong, not merely absent.** `gunbc.generated_artifact`'s
`artifact_commit_policy` assigns `CommitRequired { consumer: GitProtocol }` to
`Stage0CrateLayoutGeneratedRsArtifact` *and* to `GitignoreArtifact` — a compiled Rust source and
a git config file carry the identical classification, so consumer identity cannot separate
executable from non-executable. But the diagnosis is not *a missing axis*: **the consumer of a
generated `.rs` file is `rustc`. Git is how the file is stored.** Labelling its consumer
`GitProtocol` conflates storage with consumption, and it is the same conflation for all four
`Stage0*` rows — three `.dag` artifacts whose consumer is the gunbc compiler, and one `.rs`
whose consumer is `rustc`, all filed under the protocol that merely carries them. That reframes
the prerequisite from *invent a new fact* to **correct a fact already carried**, which is the
cheaper and more likely repair.

**(b) There is no emitted-artifact-to-module join.** `GeneratedArtifact` is a closed enum of
specific artifacts and emitted stage0 Rust as a surface is not among them
(the generated stage0 surface is now a derived population, `gunbc.stage0_rust_source_lifecycle_scaffold`
`derived_generated_stage0_repo_paths`, not a roster), and nothing
today derives **which compiler module emitted a given `.rs` path** -- which is exactly why that
derivation is the complement of the crate-layout claim rather than a module-to-path join. So an authorization minted
for a `.dag` module cannot be checked against a `.rs` surface at all. This also constrains
§10 stage 5: a transition exercise over an emitted `.rs` artifact has no identity join today,
and condition 4 of §3.8 — every receipt naming the same exact artifact identities — cannot be
satisfied for such a surface until the join exists. Stated as a prerequisite rather than
modelled around.

---

## 4. Fix-forward de-risk (replaces rollback-only framing)

A prior briefing claimed bricking fear was handled because emitted Rust is committed and
`rustc` builds it, so checking out an earlier commit suffices. That proves **rollback**,
not **fix-forward**.

**The actual risk:** generation *N* can rebuild its own old source while being unable to
understand the new source, IR, or requirement revision. The repair to a defective
generation *N+1* may only be buildable by the defective generation itself — leaving the
tree stuck even though `git checkout` still works.

**This design's answer:**

1. **Retain** the accepted generation as an immutable `RecoveryAnchor`; release is a separate
   transition consuming a valid successor-capability receipt (§3.6).
2. **Bridge** incompatible requirement changes through expand-migrate-contract rather than
   big-bang promotion.
3. **Refuse promotion** when only reproduction is proven — digest match without transition
   exercises derived from the transition's own deltas (§3.8).
4. **Fail fast** at candidate validation, on total stage-execution receipts that name the
   failing stage (§3.9), before downstream stages consume a refused artifact.
5. **Name the deadlock.** When no candidate can establish successor capability against a
   defective anchor, the state is `NoTrustedFixForwardPath` with enumerated permitted actions
   (§3.10) — not silence, and not an implicit fallback to the candidate.

**What the evidence is and is not** (operator correction, 2026-08-02): the required fact is
that the **candidate can produce and validate a changed successor under the new requirement
revision**. The anchor may well also understand the changed source — for a compatible
transition that is expected and fine. **An anchor failure is never by itself positive evidence
of candidate capability.**

Rollback remains a valid operator escape hatch via version control. This design does not
introduce rollback machinery — it introduces **typed successor-capability validation** so
promotion cannot proceed on reproduction alone.

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
trust boundary and its declared residuals — never by promotion, and never by a row asserting
the state it is supposed to establish.

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

First slice for `phased-single-process-ci`: attribute resolve/index/discovery time per
phase with and without stage receipts. **RED control:** if phase *N+1* repeats resolve whose
inputs are unchanged per its receipt, count > 0 ⇒ `RefusedStaleSubstrate` or a dedicated prelude
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
labelled *observing a refusal* a GREEN control, which inverts what a control is: a control is a
fixture whose verdict is known, and the discriminating fact is the **mutation that must change
that verdict**. Its `RefusedRegression` row ("declaration cannot green behavioral failure") was
not a control at all — it restated the law rather than naming an input. Four columns, and the
fourth is the one that makes a row evidence:

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

The fix-forward row carries **two** mutations on purpose. The first proves the subject is bound
to the transition; the second proves the receipt does not secretly depend on the anchor failing
— the exact confusion corrected in §3.8.

Probes that go green when a wall lands **flip to permanent regression controls** — they do
not retire (§4b dissolution-on rule).

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
order was wrong. It is resolved by **narrowing stage 2 explicitly** (§3.3) and by landing the
genesis first, so a lineage root exists before anything is stated relative to one.

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

Guarantee-path measurement rows (the first revision's stage 9) are not a separate stage: the
stage-execution receipt of stage 1 is that measurement, and minting a second one beside
`gunbc.guarantee_measurement` is the fork §2 exists to prevent.

---

## 11. The superseded sign-off bar, and why it did not catch this

The first revision ended in a sign-off bar whose completion read as merge readiness. It is
**retained here as a record, annotated**, rather than deleted — the durable lesson is not that
the bar was incomplete but *how* it failed, and that lesson would be lost with the text:

> **Every item it named is satisfiable while the highest-stakes concepts stay undefined.**

Read the list against the merged draft it passed. §2's symbols did resolve. §3's obligations
were explicit. §4 did state fix-forward. §5's graph was there. §9's decisions were recorded.
The doc-graph bind landed. No code shipped. All eight items met — while `FixForwardProof` was a
field name with no structure, `StageStamp` was named and never typed, and there was no
generation zero at all. **A checklist that counts sections cannot detect an undefined concept
inside one.** That is why §8's witness table now asks, for every row, which *mutation must
change the verdict*: a bar phrased as presence is satisfiable by presence, and a bar phrased as
discrimination is not.

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

The bar is superseded as a *readiness* device: it is what let #7681 merge before its review
finished.

- This document confers **no merge readiness**. Being written does not make it accepted.
- It confers **no promotion authority**. Nothing here authorizes a generation to be promoted,
  an anchor to be released, or a warm receipt to be reused; those come from the receipts §3
  describes, once they exist and execute.
- It confers **no roadmap acceptance**. A roadmap row is accepted by its own explicit
  acceptance, never by a design note describing it.
- It ships **no implementation code**, and each stage in §10 is a separately dispatched,
  separately reviewed PR.

Item 8 above is the one entry that survives as a live constraint rather than as record, because
it is a scope statement and not a readiness claim: candidate generation through the direct Rust
door remains unblocked, and only promotion and authoritative admission are gated. It is carried
into the staged plan as §10 stage 7.

---

## 12. Correction record (2026-08-02)

Nine operator corrections, relayed through still-bat-561. Each names what it supersedes so a
reader of the merged first revision can see what moved.

| # | Correction | Supersedes |
|---|---|---|
| 1 | `FixForwardProof` deleted; `RequirementDelta` + `TransitionExercise` derived from the transition; the complete receipt **is** the proof; five admission conditions; **anchor failure is not positive evidence** | §3.8's `fix_forward_proof` field and its anchor-must-fail reading |
| 2 | `AcceptedCompilerGeneration = BootstrapGenesis \| PromotedSuccessor`; `GenesisAdmissionReceipt`; the candidate-promotion constructor **cannot** produce a genesis | §3.4's `Candidate \| Promoted` pair, which had no generation zero |
| 3 | `GenerationContinuityVerdict` with `NoTrustedFixForwardPath` and enumerated permitted actions | §5's silence on anchor-defective-and-no-viable-successor |
| 4 | `StageExecutionReceipt` as a total result (`Accepted \| Refused \| FallbackUsed`); `StageStamp` is only its content identity; check `gunbc.guarantee_measurement` first | §6.1's untyped `StageStamp`, and any reading where `RefusedStageMismatch` substitutes for a stage's own outcome |
| 5 | `RequirementRevision` split from `ValidationRevision`; enrollment is validation-only; weakening needs an explicit validation transition | §3.1's single revision carrying `witness_roster_digest` |
| 6 | `RecoveryAnchor` immutable; `RecoveryAnchorState = AnchorRetained \| AnchorReleaseEligible \| AnchorReleased` | §3.6's `retained_until: PromotedSuccessor` forward reference |
| 7 | `ByteOnlyAdmissionEvidence` with serializer purity + three reachability-none proofs; unknown reachability ⇒ semantic validation; no authored `formatting_only` | §9 Q2's unqualified formatting-only permission |
| 8 | Stages reordered 0–10; stage 2 narrowed to within-revision with no promotion authority | §10's ordering, which stated §3.2 admissibility before the generation state existed |
| 9 | Witness table rewritten with Subject / Fixture / Expected verdict / **mutation that must change the verdict** | §8's table, which called observing a refusal a GREEN control and whose `RefusedRegression` row named no input |

**Three consequences found while applying them**, changed here though not among the nine:

- **Correction 5 silently invalidated a bridge trigger.** §3.7 listed *new witness roster* as an
  incompatible-requirement trigger; once the roster moves to `ValidationRevision`, that reading
  would demand expand-migrate-contract for enrolling a witness. Removed, with the reason stated
  inline so it cannot return.
- **`RefusedRegression` was carrying two causes.** §3.3's table let it mean both *moved without a
  declaration* and *moved with one but the witness failed*, while `RefusedUndeclaredDelta` sat
  beside it meaning the first. Two causes with different remedies sharing an arm is the
  state-space conflation DESIGN §5 names; the table now says which is which.
- **The first revision's stage 9 was a fork.** Guarantee-path measurement rows are not a
  separate stage — the stage-execution receipt of the new stage 1 *is* that measurement, and a
  second receipt beside `gunbc.guarantee_measurement` is what §2 exists to prevent.

**Kept from the first revision, unchanged:** the §2 DFS-existing-machinery inventory (reuse, do
not fork); candidate-cannot-promote-itself; the generation lineage graph; rollback-is-not-fix-forward
(§4); declared change as distinct from behavioral equivalence (§9 Q2's first clause); and probes
flipping to permanent regression controls rather than retiring (§8).

**Status of the first revision:** merged rough draft, not reverted — it is document-only and no
promoted generation depends on it.
