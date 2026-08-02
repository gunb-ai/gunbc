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

DESIGN refs: §2 (DFS existing machinery before minting vocabulary; no parallel ledger),
§3 (`gunbc.bootstrap.CompilerStage` is the stage authority; `FrontierProbeStage` is harness
vocabulary, not a second pipeline), §4 (emission is the downstream half of the two-stage
contract in [compiler-guarantee-recovery-gap-analysis.md](compiler-guarantee-recovery-gap-analysis.md)
§1), §5 (undeclared emission delta refuses — never widens to rerun-everything or
accept-anyway), §4b (discriminating RED: regression vs declared-change controls stay enrolled
after walls land), §6 (model-before-implement; scaffolds name dissolution triggers), §7
(behavioral equivalence over byte identity for self-host; bricking fear is a git revert,
not rollback machinery).

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
> The pipeline is **stage-aware**: each phase stamps which `CompilerStage` produced which
> artifact, and admission consumes those stamps plus a **generation lineage graph** — it
> does not re-derive the whole compiler from bytes alone, and it does not equate
> generation-to-generation output.

---

## 1. Displaced cost (§6)

| Cost today | Mechanism |
|---|---|
| **Self-host is one boolean** | Frontier disposition, regen verify, and realized comparison each answer a single match/mismatch bit. A stage-local refusal (infer ok, emit wrong) has no durable receipt tying *which stage* failed — the `#7485` containment class. |
| **Warm receipts cannot be trusted for merge** | `warm-merge-admission` can stamp resolve/materialization receipts, but has no typed rule for whether an emission delta was *expected* from the PR's source change. Stale-base refusals exist; *undeclared emitter drift* does not. |
| **Phases pay duplicate prelude work** | `phased-single-process-ci` wants regen, floor, and admission on one substrate. Without stage-stamped verdicts, each phase assumes the prior phase might have silently changed upstream facts — discovery/resolve/index rerun anyway. |
| **Byte gates cannot distinguish intent** | `generated_artifact_drift_gate`, `RegenVerifyGate`, and `self_host_realized_comparison` answer only *match / mismatch*. An intentional emitter migration with regen is indistinguishable from an emitter bug — review becomes the classifier (§5 specification-without-execution). |
| **Producer provenance is partial** | `EmitterProducer` × `EmissionQualification` and `mint_producer_emission_receipt` land per-module receipts, but digest fields remain scaffold-keyed (`producer_emission_receipt_digest_scaffold_note`) and `emitter_produced_baseline` is zero — no join from PR facts to admitted vs regression disposition. |
| **Generation is implicit** | `V2EmitterNative { generation: Int }` names a generation index on the producer axis only. There is no graph of *which generation's emitted artifact* a consumer read, so native-at-small-scale cutover cannot refuse stale-generation replay without widening to cold rerun. |

---

## 2. DFS — existing machinery (reuse, do not fork)

Apply DESIGN §2 before proposing vocabulary: the concept DAG already carries most of the
stage/provenance spine. **This design adds two genuinely new pieces** (§3); everything
below is live on main and should be cited, extended, or joined — not re-invented.

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

**What it does not yet buy:** PR-level declared-change vs regression, stage-stamped phase
pipeline, or generation lineage as a first-class graph.

### 2.2 Emitted-byte digest + comparison gates (live)

| Symbol | Role |
|---|---|
| `v2.compiler.self_host.canonical_emitted_bytes_digest` | Host-grounded digest over actual emitted bytes (`Medium<String>`) |
| `tools.floor_effect_gate_witness.regen_verify_gate_passes` | CI floor gate: `RegenVerifyGate` compares regen output to committed Rust |
| `v2.workflow.self_host_realized_comparison_gate` | Behavioral + staleness gates over realized comparison transport |
| `gunbc.ci_spec.RegenVerifyGate` | Spec row; regen skip policy keyed on merge-base diff vs regen input closure |

**What this already buys:** fail-closed byte mismatch on the regen path and behavioral
witnesses on a curated roster.

**What it does not yet buy:** classification of *why* bytes moved (declared migration vs
regression).

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

## 3. Two genuinely new pieces

Everything in §2 is reuse. **Only these are new design obligations:**

### 3.1 Declared-change vs regression (not generation equivalence)

**Problem:** Today's gates ask "does output match committed?" or "does generation N match
generation N−1?" Neither answers the operational question: **did this PR declare that these
surfaces may move?**

**New carrier (proposed name: `EmissionAdmissionVerdict`):**

| Verdict | Meaning |
|---|---|
| `AdmittedDeclared { stages, surfaces, evidence }` | Observed delta ⊆ declared change set; stage stamps consistent |
| `RefusedRegression { stage, surface, cause }` | Bytes/behavior moved without declaration or with failed witness |
| `RefusedUndeclaredDelta` | Delta on a registered surface with no `allowed_surface_moves` entry |
| `RefusedStageMismatch { declared, observed }` | Declared `Infer`-only but emit digest moved (or converse) |
| `RefusedStaleSubstrate { expected_revision, observed_revision }` | Warm receipt replay against wrong materialization revision |

**Declared change set (proposed):** derived from PR facts — affected-set projection,
touched emitter modules, explicit frontier row flips, regen-input-closure membership —
joined to `CompilerStage` and `ArtifactSurface` (generated-artifact registry paths, seed
files, behavioral probe coordinates). Human `AuthorExplicit` override is an open question
(§9).

**Law:** admission is **not** generation-to-generation equivalence. A declared emitter
migration may admit digest movement with behavioral green; an undeclared move refuses even
if bytes happen to match a prior generation.

### 3.2 Generation lineage graph

**Problem:** `V2EmitterNative { generation: Int }` tags the producer but does not record
which emitted artifact generation a consumer actually used. Native cutover, warm-merge
replay, and phased CI all need to refuse **stale-generation artifacts** without absorbing
into "rerun everything."

**New graph (proposed):**

```
GenerationNode { index, producer, receipt_digest, substrate_revision }
GenerationEdge { parent, child, delta: EmissionDelta }
ArtifactBinding { path, generation, emitted_source_digest }
ConsumerRead { consumer_id, artifact_binding, admission_verdict }
```

**Laws:**

- Every `ArtifactBinding` points at exactly one `GenerationNode` that holds a
  `ProducerEmissionReceipt` or an honest `SeedRetained` / scaffold row.
- Consumers (`RegenVerifyGate`, warm-merge admission, floor witness) record
  `ConsumerRead` — not just Bool pass/fail.
- Replay of a warm receipt refuses when `substrate_revision` or `generation` edge does
  not match the stamping run (fail-closed; no widen to cold rerun without counted cause).

**Dissolve-on:** when `frontier_probe_survey` binds execution-measured digests, lineage
edges upgrade from scaffold digests to measured digests without changing graph shape.

---

## 4. Bricking de-risk (operator ruling)

Emitted Rust is **plain text in git**. A bad emitter migration does not require rollback
machinery, feature flags, or absorbing "rerun regen until green":

1. `git checkout` the last good commit (or revert the PR).
2. Seed-retained modules continue to compile from committed `src/v1/**` until frontier rows
   flip with receipts.

This design **does not** introduce a rollback subsystem. It introduces **earlier, typed
refusal** so bricking is caught at regen/admission with a located cause (`RefusedRegression`,
`RefusedUndeclaredDelta`) rather than after merge or fleet deploy. The psychological "fear
of bricking" is already handled by version control; the gap is *classification at the
gate*.

---

## 5. Stage-aware pipeline model

### 5.1 Phases on one substrate (`phased-single-process-ci`)

```
┌─ Phase: Regen ─────────────────────────────────────────────────┐
│  regen_stage0 / generated_artifact_gate                        │
│  emit StageStamp + EmissionDelta list + GenerationNode       │
│  refuse: RefusedUndeclaredDelta (fail fast — floor not run)  │
└───────────────────────────┬──────────────────────────────────┘
                            │ stamps + lineage edges
┌─ Phase: Floor ────────────▼──────────────────────────────────┐
│  compile-clean + witness corpus on shared materialization    │
│  inherit stamps; refuse RefusedStaleSubstrate if replay      │
└───────────────────────────┬──────────────────────────────────┘
                            │
┌─ Phase: Admission ────────▼──────────────────────────────────┐
│  warm-merge-admission / merge gate                           │
│  validate receipts against DeclaredChangeSet + ConsumerRead    │
│  AdmissionVerdict → merge allow/refuse                         │
└────────────────────────────────────────────────────────────────┘
```

**Ordering law:** regen still gates floor (existing `ci_regen_floor_skip_policy_note`).
Admission may consume warm receipts only when `substrate_revision`, `DeclaredChangeSet`, and
generation lineage match the stamping run.

### 5.2 Prelude duplication metric

First slice for `phased-single-process-ci`: attribute resolve/index/discovery time per
phase with and without stamps. **RED control:** if phase *N+1* repeats resolve whose inputs
are unchanged per stamp, count > 0 ⇒ `RefusedStaleSubstrate` or a dedicated prelude
duplication refusal (roadmap red_control, made executable).

---

## 6. Consumer mapping

| Consumer | Declared-change source | Regression signal |
|---|---|---|
| `warm-merge-admission` | PR affected set + stamped receipts at merge base | Receipt replay produces undeclared `EmissionDelta` |
| `generated_artifact_drift_gate` | diff touches artifact authority modules | generate ≠ committed without `allowed_surface_moves` |
| `RegenVerifyGate` / `regen_verify_gate_passes` | diff touches `src/v2/compiler/*emit*` or frontier rows | seed digest mismatch undeclared |
| `self_host_realized_comparison` | frontier flip / emitter edit in PR | behavioral witness fails with digest-only move |
| `mint_producer_emission_receipt` | (future) declared emitter migration | mint with `BuildRefused` or wrong producer in closure |
| `gunbc.guarantee_measurement` | probe declares subject revision | `ExecutionDiverged` on emission path with clean compile |

---

## 7. Witness / RED discipline (§4b, §5)

Each verdict arm gets a discriminating pair:

| Arm | GREEN control | RED control |
|---|---|---|
| `AdmittedDeclared` | emitter edit + regen + declaration contains surface | same edit without declaration ⇒ `RefusedUndeclaredDelta` |
| `RefusedRegression` | planted emitter bug without source change ⇒ refusal | declaration cannot green behavioral failure |
| `RefusedStageMismatch` | declared `Infer`-only change | emit digest moves |
| `RefusedStaleSubstrate` | warm receipt from prior `substrate_revision` | replay after index rebuild without restamp |
| Generation stale read | consumer reads binding at generation G | replay claims G but artifact is G−1 |
| Prelude duplication | phase 2 skips resolve when stamp says preserved | forced re-resolve ⇒ counted refusal |

Probes that go green when a wall lands **flip to permanent regression controls** — they do
not retire (§4b dissolution-on rule).

---

## 8. Staged implementation plan (future PRs — not this document)

| Stage | Deliverable | Dissolution trigger |
|---|---|---|
| **0 — this document** | Sign-ready design + doc graph bind | Operator sign-off |
| **1 — vocabulary carrier** | Types + pure `admit_emission_delta` + synthetic unit witnesses | Interim bytes-only merge branches gain verdict arm |
| **2 — declared-change derivation** | `declared_change_set_from_pr_facts` wired to affected-set; refuse when selection fails | Hand-classified PR intent in review prose |
| **3 — regen phase stamp** | `regen_stage0` emits `StageStamp` + deltas; regen refuses undeclared | `regen_divergence_count` prose-oracle |
| **4 — generation lineage** | `GenerationNode` / `ArtifactBinding` / `ConsumerRead` on regen + warm paths | Implicit generation in producer tag only |
| **5 — warm-merge join** | #7522 admits warm receipts only with matching stamps + declaration | Cold rerun as default merge path |
| **6 — phased single process** | Shared substrate; prelude duplication witness | Per-phase `process_shared_index` duplication |
| **7 — guarantee path rows** | emission-path measurement carries admission projection | Emission path unmeasured for intent |

---

## 9. Open questions (operator sign-off)

### Q1 — `AuthorExplicit` override

Should a human declaration ever admit a delta without affected-set evidence?

| Option | Tradeoff |
|---|---|
| **A. Refuse always** | No escape hatch; missing affected-set evidence is `RefusedUndeclaredDelta` |
| **B. Typed override row** | `AuthorExplicit { author, rationale, surfaces }` admissible with counted audit + RED control |
| **C. FrontierAccepted only** | Override allowed only for enrolled scaffold rows, not production surfaces |

**Recommendation:** **B** with a construction wall — override row required, never a silent
flag; aligns with §5 "no escape hatches" while allowing genuine operator migrations.

### Q2 — Behavioral vs digest on self-host

For `SelfEmitted` / `Stage0SeedFile` surfaces, is digest match alone ever sufficient?

| Option | Tradeoff |
|---|---|
| **A. Digest only** | Faster CI; risks §5 validation-standing-where-construction-was-available |
| **B. Behavioral always** | Matches `BehavioralEquivalent` qualification; costs wet witness |
| **C. Tiered by surface** | Digest for formatting-only paths; behavioral when probe enrolled |

**Recommendation:** **C** — join to existing `EmissionQualification` and behavioral transport
roster; refuse `AdmittedDeclared` without `BehavioralEquivalent` when a probe is enrolled.

### Q3 — Frontier scaffold rows

Do `SeedRetained` / `UnpinnedFrontier` rows auto-widen `allowed_surface_moves`?

| Option | Tradeoff |
|---|---|
| **A. Auto-widen by row class** | Less author burden; risks over-admission |
| **B. Per-row `DeclaredChangeTemplate`** | Explicit; scales with roster size |
| **C. Census-derived default** | `witness_entry_eligibility_census` projects templates once |

**Recommendation:** **B** for production surfaces, **A** only for counted scaffold classes
with dissolution triggers named on the row.

### Q4 — Registry completeness

Is `SurfaceUnknown` hard refusal or `FrontierAccepted` on the merge path?

| Option | Tradeoff |
|---|---|
| **A. Hard refusal** | Fail-closed; blocks merge until registry complete |
| **B. FrontierAccepted** | Merge proceeds with counted debt; risks silent widen |
| **C. Refuse on warm path only** | Cold merge ok with debt; warm replay refuses |

**Recommendation:** **C** — matches §4b `FrontierAccepted` semantics and warm-merge's
higher trust bar.

### Q5 — Relationship to BothWays quotient delta

Do lawful BothWays edits always become `AdmittedDeclared` when the six lens laws pass?

| Option | Tradeoff |
|---|---|
| **A. Always admitted** | Semantic lawful ⇒ admission; digest move expected |
| **B. Requires explicit surface entry** | BothWays green ≠ emission admission without declaration |
| **C. Join only for `SelfEmitted` surfaces** | Seed-emitted paths use quotient delta; generated-artifact paths use registry |

**Recommendation:** **C** — [module-identity-storage-binding-design.md](module-identity-storage-binding-design.md)
comparison is semantic (R1 vs R2); emission admission still needs surface declaration for
registry-governed artifacts.

---

## 10. Sign-off bar

This design is ready for operator review when:

1. All symbols in §2 resolve on main (grep-verified at PR open).
2. The two new pieces (§3) are explicit and not conflated with existing gates.
3. Open questions (§9) have options + recommendations recorded in the PR body.
4. Doc graph bind lands (`gunbc.doc_graph_roots` → this slug).
5. No implementation code ships in the design PR.
