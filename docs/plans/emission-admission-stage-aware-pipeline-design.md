# Emission admission + stage-aware pipeline (declared-change vs regression)

**Status:** design-only (model-before-implement). This document is the deliverable for
work item `node://adhoc-48a1f19c-1f8` (session calm-eagle-92). No load-bearing `.dag`
carrier lands in the design PR; each implementation stage below is a separately dispatched,
separately-signed PR.

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
> Promotion from **Candidate** to **Promoted** generation requires stage, behavioral, and
> **successor-capability** evidence — proving the new generation can understand and validate
> the changed requirements needed to build the repair, not merely reproduce the old output.
> Generation *N* stays retained as a **recovery anchor** until generation *N+1* produces a
> valid changed successor.

---

## 1. Displaced cost (§6)

| Cost today | Mechanism |
|---|---|
| **Self-host is one boolean** | Frontier disposition, regen verify, and realized comparison each answer a single match/mismatch bit. A stage-local refusal (infer ok, emit wrong) has no durable receipt tying *which stage* failed — the `#7485` containment class. |
| **Detects movement, not fix-forward** | Byte gates detect unexpected output movement but cannot answer: *can the last accepted compiler understand and validate the changed requirements needed to build the repair?* A defective generation may be the only one that can rebuild its own old source while being unable to compile the new requirement revision. |
| **Warm receipts cannot be trusted for merge** | `warm-merge-admission` can stamp resolve/materialization receipts, but has no typed rule for whether an emission delta was *expected* from the PR's source change. Stale-base refusals exist; *undeclared emitter drift* does not. |
| **Phases pay duplicate prelude work** | `phased-single-process-ci` wants regen, floor, and admission on one substrate. Without stage-stamped verdicts, each phase assumes the prior phase might have silently changed upstream facts — discovery/resolve/index rerun anyway. |
| **Byte gates cannot distinguish intent** | `generated_artifact_drift_gate`, `RegenVerifyGate`, and `self_host_realized_comparison` answer only *match / mismatch*. An intentional emitter migration with regen is indistinguishable from an emitter bug — review becomes the classifier (§5 specification-without-execution). |
| **Producer provenance is partial** | `EmitterProducer` × `EmissionQualification` and `mint_producer_emission_receipt` land per-module receipts, but digest fields remain scaffold-keyed (`producer_emission_receipt_digest_scaffold_note`) and `emitter_produced_baseline` is zero — no join from PR facts to admitted vs regression disposition. |
| **Generation is implicit** | `V2EmitterNative { generation: Int }` names a generation index on the producer axis only. There is no graph of *which generation's emitted artifact* a consumer read, no candidate/promoted distinction, and no recovery anchor — so native cutover cannot refuse stale-generation replay without widening to cold rerun. |

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

### 3.1 Requirement revision identity

A **requirement revision** is a content-addressed identity for the compiler's obligation
set at a point in time — source language contract, IR shape, emitter contract, bootstrap
requirements, and enrolled witness roster. It is not a git commit and not a generation index;
two generations may share a revision, and one revision may span multiple candidate attempts.

```
RequirementRevision { revision_id: Hash, authority_modules: List<String>, witness_roster_digest: Hash }
```

**Law:** every `DeclaredTransition`, `CandidateGeneration`, and `PromotedGeneration` names
the `RequirementRevision` it was evaluated against. Comparing artifacts across different
revision IDs without an explicit `BridgeGeneration` protocol is `RefusedRevisionMismatch`.

### 3.2 Declared transition (revision N → N+1)

A **declared transition** is the typed statement of what may change between requirement
revisions — not merely which surfaces may move in a PR, but which parts of the requirement
contract are being revised and why.

```
DeclaredTransition {
  from_revision: RequirementRevision,
  to_revision: RequirementRevision,
  allowed_surface_moves: List<ArtifactSurface>,
  affected_stages: List<CompilerStage>,
  evidence: TransitionEvidence   // affected-set projection, operator declaration, bridge protocol ref
}
```

**Law:** a `CandidateGeneration` at revision *N+1* is inadmissible without a
`DeclaredTransition` from the currently `PromotedGeneration`'s revision. Human-authored
declarations are **inputs** to admission (see §9 Q1), never overrides of missing evidence.

### 3.3 Declared-change vs regression (within a revision)

Within a single requirement revision, admission still classifies emission deltas:

| Verdict | Meaning |
|---|---|
| `AdmittedDeclared { stages, surfaces, evidence }` | Observed delta ⊆ declared change set; stage stamps consistent |
| `RefusedRegression { stage, surface, cause }` | Bytes/behavior moved without declaration or with failed witness |
| `RefusedUndeclaredDelta` | Delta on a registered surface with no `allowed_surface_moves` entry |
| `RefusedStageMismatch { declared, observed }` | Declared `Infer`-only but emit digest moved (or converse) |
| `RefusedStaleSubstrate { expected_revision, observed_revision }` | Warm receipt replay against wrong materialization revision |

**Law:** admission is **not** generation-to-generation equivalence. A declared emitter
migration may admit digest movement with behavioral green; an undeclared move refuses even
if bytes happen to match a prior generation.

### 3.4 Candidate vs promoted generation state

Generation is not a single axis on `EmitterProducer` — it is a **lifecycle state**:

| State | Meaning |
|---|---|
| `Candidate { generation, revision, producer, receipts }` | Emitted and validated under candidate rules; not authoritative for promotion, warm reuse, or merge admission |
| `Promoted { generation, revision, producer, receipts, successor_capability }` | The sole authoritative generation for production routing, warm replay, and merge admission |

**Construction wall:** a `Candidate` **cannot be the sole authority promoting itself**.
Promotion requires evidence from outside the candidate's own emission path — at minimum
the retained `RecoveryAnchor` generation and an independent validation chain (§4).

**Non-blocking path:** candidate generation through the direct Rust door (`V1SeedEmitter` /
committed seed emit) **must not be blocked** by this design. The operator supports merging
this design note **before** the first behavioral-module promotion; only promotion and
authoritative admission are gated.

### 3.5 Generation lineage graph

```
GenerationNode { index, state: Candidate | Promoted, revision_id, producer, receipt_digest }
GenerationEdge { parent, child, delta: EmissionDelta, transition: DeclaredTransition }
ArtifactBinding { path, generation, state, emitted_source_digest }
ConsumerRead { consumer_id, artifact_binding, admission_verdict }
RecoveryAnchor { generation, revision_id, retained_until: PromotedSuccessor }
```

**Laws:**

- Every `ArtifactBinding` points at exactly one `GenerationNode` with an honest
  `ProducerEmissionReceipt` or `SeedRetained` row.
- Consumers record `ConsumerRead` — not just Bool pass/fail.
- Warm replay refuses when `substrate_revision`, `generation`, or `state` does not match
  the stamping run (`RefusedStaleSubstrate`).
- Only `Promoted` bindings are authoritative for merge admission and warm reuse.

### 3.6 Retained recovery anchor

**Recovery anchor** is the last `PromotedGeneration` *N* retained until generation *N+1*
produces a valid **changed** successor (not a byte-identical reproduction).

```
RecoveryAnchor {
  generation: N,
  revision_id,
  retained_until: PromotedSuccessor { generation: N+1, successor_capability: Present }
}
```

**Law:** the anchor is not deleted on candidate mint. It remains the authority that can
validate whether the candidate understands the *new* requirements — and the fallback that
can still build generation-*N* artifacts if promotion refuses.

Rollback (`git checkout` an earlier commit) remains possible independently; it is **not**
the fix-forward story and does not discharge the anchor obligation.

### 3.7 Bridge-generation protocol (incompatible requirement changes)

When `DeclaredTransition` crosses incompatible requirement revisions (new IR shape, new
emitter contract, new witness roster), a **bridge generation** mediates:

```
BridgeGeneration {
  anchor: RecoveryAnchor,
  candidate: CandidateGeneration,
  expand_phase: RequirementExpand,    // anchor understands expanded obligation surface
  migrate_phase: ArtifactMigrate,    // candidate artifacts produced under expanded contract
  contract_phase: RequirementContract // promoted revision is strictly smaller than expand
}
```

**Law:** bridge is mandatory when `from_revision` and `to_revision` are not provably
compatible by digest-inclusion alone. Skipping bridge and promoting across incompatible
revisions is `RefusedBridgeRequired`.

### 3.8 Successor-capability receipt (fix-forward, not reproduction)

A **successor-capability receipt** proves generation *N+1* can build and validate the
*changed* successor — not merely reproduce generation *N*'s output.

```
SuccessorCapabilityReceipt {
  anchor_generation: N,
  candidate_generation: N+1,
  transition: DeclaredTransition,
  stage_validation: StageValidationVerdict,
  behavior_validation: BehaviorValidationVerdict,
  fix_forward_proof: FixForwardProof   // candidate compiles/validates a changed-requirement artifact anchor cannot
}
```

**Law:** `PromotedGeneration N+1` is unwritable without `SuccessorCapabilityReceipt`.
Digest equality between *N* and *N+1* on unchanged surfaces is necessary but not
sufficient — it proves reproduction, not fix-forward.

**Dissolve-on:** when `frontier_probe_survey` binds execution-measured digests, lineage
edges and capability proofs upgrade from scaffold digests to measured digests without
changing graph shape.

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

1. **Retain** `PromotedGeneration N` as `RecoveryAnchor` until *N+1* passes successor-capability.
2. **Bridge** incompatible requirement changes through expand-migrate-contract rather than
   big-bang promotion.
3. **Refuse promotion** when only reproduction is proven (digest match without
   `FixForwardProof`).
4. **Fail fast** at candidate validation (stage → behavior → fix-forward) before downstream
   stages consume a refused artifact.

Rollback remains a valid operator escape hatch via version control. This design does not
introduce rollback machinery — it introduces **typed fix-forward validation** so promotion
cannot proceed on reproduction alone.

---

## 5. Temporal graph (explicit lifecycle)

```
AcceptedGeneration N  (Promoted)
        │
        ├─ RequirementTransition (N → N+1)     [DeclaredTransition + revision IDs]
        ├─ RecoveryAnchor N                    [retained until valid successor]
        │
        ▼
Candidate N+1
        │
        ├─ stage validation                    [per CompilerStage stamps; fail fast]
        ├─ behavior validation                 [BehavioralEquivalent where required; §9 Q2]
        ├─ bootstrap + fix-forward validation  [SuccessorCapabilityReceipt]
        │
        ▼
PromotedGeneration N+1                         [only if all gates pass]
        │
        └─ RecoveryAnchor N released           [only on valid *changed* successor]
```

**Retention law:** generation *N* stays `Promoted` and authoritative until generation *N+1*
produces a valid changed successor. A refused candidate does not demote *N*.

**Candidate law:** generation *N+1* in `Candidate` state may emit, compile, and run
witnesses — including through the direct Rust seed door — but cannot promote itself and
cannot authorize warm reuse or merge admission.

---

## 6. Stage-aware pipeline model

### 6.1 Phases on one substrate (`phased-single-process-ci`)

```
┌─ Phase: Regen ─────────────────────────────────────────────────┐
│  regen_stage0 / generated_artifact_gate                        │
│  emit StageStamp + EmissionDelta + Candidate/Promoted state    │
│  refuse: RefusedUndeclaredDelta (fail fast — floor not run)  │
└───────────────────────────┬──────────────────────────────────┘
                            │ stamps + lineage edges
┌─ Phase: Floor ────────────▼──────────────────────────────────┐
│  compile-clean + witness corpus on shared materialization    │
│  inherit stamps; refuse RefusedStaleSubstrate if replay      │
│  candidate artifacts: stage + behavior validation            │
└───────────────────────────┬──────────────────────────────────┘
                            │
┌─ Phase: Admission ────────▼──────────────────────────────────┐
│  warm-merge-admission / merge gate                           │
│  Promoted bindings only; validate DeclaredTransition + reads   │
│  AdmissionVerdict → merge allow/refuse                         │
└────────────────────────────────────────────────────────────────┘
```

**Ordering law:** regen still gates floor (existing `ci_regen_floor_skip_policy_note`).
Admission may consume warm receipts only for `Promoted` bindings when `substrate_revision`,
`DeclaredTransition`, and generation lineage match the stamping run.

### 6.2 Prelude duplication metric

First slice for `phased-single-process-ci`: attribute resolve/index/discovery time per
phase with and without stamps. **RED control:** if phase *N+1* repeats resolve whose inputs
are unchanged per stamp, count > 0 ⇒ `RefusedStaleSubstrate` or a dedicated prelude
duplication refusal (roadmap red_control, made executable).

---

## 7. Consumer mapping

| Consumer | Declared-change source | Regression / refusal signal |
|---|---|---|
| `warm-merge-admission` | `Promoted` receipts + `DeclaredTransition` at merge base | Undeclared `EmissionDelta`; `Candidate` binding on warm path |
| `generated_artifact_drift_gate` | diff touches artifact authority modules | generate ≠ committed without `allowed_surface_moves` |
| `RegenVerifyGate` / `regen_verify_gate_passes` | diff touches emit modules or frontier rows | seed digest mismatch undeclared |
| `self_host_realized_comparison` | frontier flip / emitter edit in PR | behavioral witness fails; digest relabelled as equivalence |
| `mint_producer_emission_receipt` | declared emitter migration | mint with `BuildRefused` or wrong producer in closure |
| promotion gate (new) | `SuccessorCapabilityReceipt` | reproduction without fix-forward proof |
| `gunbc.guarantee_measurement` | probe declares subject revision | `ExecutionDiverged` on emission path with clean compile |

---

## 8. Witness / RED discipline (§4b, §5)

Each verdict arm gets a discriminating pair:

| Arm | GREEN control | RED control |
|---|---|---|
| `AdmittedDeclared` | emitter edit + regen + declaration contains surface | same edit without declaration ⇒ `RefusedUndeclaredDelta` |
| `RefusedRegression` | planted emitter bug without source change ⇒ refusal | declaration cannot green behavioral failure |
| `RefusedStageMismatch` | declared `Infer`-only change | emit digest moves |
| `RefusedStaleSubstrate` | warm receipt from prior `substrate_revision` | replay after index rebuild without restamp |
| `Candidate` self-promotion | promotion refused without anchor evidence | candidate-only path claims `Promoted` |
| `SuccessorCapability` | candidate builds changed-requirement artifact | digest-only match between *N* and *N+1* |
| `SurfaceUnknown` | enrolled in candidate/audit only | `SurfaceUnknown` on promotion or warm path |
| Prelude duplication | phase 2 skips resolve when stamp says preserved | forced re-resolve ⇒ counted refusal |

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

**Decision:** Stricter **C**.

- Digest equality proves **byte agreement only**.
- Formatting-only byte-contract artifacts may close on digest alone.
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

| Stage | Deliverable | Dissolution trigger |
|---|---|---|
| **0 — this document** | Sign-ready design + doc graph bind | Operator sign-off (this revision) |
| **1 — vocabulary carrier** | Revision, transition, candidate/promoted, anchor types + pure folds + synthetic witnesses | Interim bytes-only merge branches |
| **2 — declared-change derivation** | `declared_transition_from_pr_facts` wired to affected-set; refuse when selection fails | Hand-classified PR intent in review prose |
| **3 — regen phase stamp** | `regen_stage0` emits `StageStamp` + deltas + candidate state; regen refuses undeclared | `regen_divergence_count` prose-oracle |
| **4 — lineage + anchor** | `GenerationNode` / `RecoveryAnchor` / `ConsumerRead` on regen + warm paths | Implicit generation in producer tag only |
| **5 — successor capability** | `SuccessorCapabilityReceipt` + fix-forward witness; promotion gate | Reproduction-only promotion |
| **6 — bridge protocol** | expand-migrate-contract for incompatible revision jumps | Big-bang incompatible promotion |
| **7 — warm-merge join** | #7522 admits `Promoted` receipts only with matching transition + lineage | Cold rerun as default merge path |
| **8 — phased single process** | Shared substrate; prelude duplication witness | Per-phase `process_shared_index` duplication |
| **9 — guarantee path rows** | emission-path measurement carries admission projection | Emission path unmeasured for intent |

---

## 11. Sign-off bar

This design is ready for operator merge when:

1. All symbols in §2 resolve on main (grep-verified).
2. §3 obligations are explicit — not reduced to declared-change + lineage alone.
3. §4 states fix-forward (not rollback-only) de-risk.
4. §5 temporal graph is explicit.
5. §9 operator decisions are recorded (not open questions).
6. Doc graph bind lands (`gunbc.doc_graph_roots` → this slug).
7. No implementation code ships in the design PR.
8. Candidate generation through the direct Rust door remains unblocked; only promotion
   and authoritative admission are gated.
