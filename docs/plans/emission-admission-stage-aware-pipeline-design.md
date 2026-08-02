# Emission admission + stage-aware pipeline (declared-change vs regression)

**Status:** Stage 1 LANDED (vocabulary carrier + `admit_emission_delta` + synthetic witnesses).
Stages 2–6 remain design-only until separately dispatched.

Work item: `node://adhoc-48a1f19c-1f8` (session calm-eagle-92).

Parent context: v1-deletion lane (`still-bat-561`). This design unblocks honest
warm-merge admission, phased single-process CI, and self-host/regen gates that
today can only compare bytes — not classify *why* bytes changed.

DESIGN refs: §2 (reuse existing stage/change authorities; no parallel ledger),
§3 (`gunbc.bootstrap.CompilerStage` is the stage authority;
`FrontierProbeStage` is harness vocabulary, not a second pipeline),
§4 (emission is the downstream half of the two-stage contract in
[compiler-guarantee-recovery-gap-analysis.md](compiler-guarantee-recovery-gap-analysis.md)
§1), §5 (undeclared emission delta refuses — never widens to rerun-everything or
accept-anyway), §4b (discriminating RED: regression vs declared-change controls
stay enrolled after walls land), §6 (model-before-implement; scaffolds name
dissolution triggers).

Related: [five-minute-ci-gate-design.md](five-minute-ci-gate-design.md)
(`warm-merge-admission`, `phased-single-process-ci`) ·
[seed-honesty-discharge-design.md](seed-honesty-discharge-design.md) ·
[module-identity-storage-binding-design.md](module-identity-storage-binding-design.md)
(BothWays delta lens) · [post-zero-regen-gate-placement.md](post-zero-regen-gate-placement.md) ·
`gunbc.bootstrap` · `gunbc.guarantee_measurement` · `v2.compiler.self_host` ·
`gunbc.generated_artifact`

---

## 0. One-sentence claim

> Every emission delta is admitted only when a **declared change set** names the
> compiler stages and artifact surfaces that may move; anything else is a
> **regression** and refuses. The pipeline is **stage-aware**: each phase
> (regen, floor, admission) stamps which `CompilerStage` produced which artifact,
> and admission consumes those stamps — it does not re-derive the whole compiler
> from bytes alone.

---

## 1. Displaced cost (§6)

| Cost today | Mechanism |
|---|---|
| **Warm receipts cannot be trusted for merge** | `warm-merge-admission` can stamp resolve/materialization receipts, but has no typed rule for whether an emission delta was *expected* from the PR's source change. Stale-base and stale-roster refusals exist; *undeclared emitter drift* does not — so admission either reruns cold or risks admitting a regression. |
| **Phases pay duplicate prelude work** | `phased-single-process-ci` wants regen, floor, and admission on one substrate. Without stage-stamped verdicts, each phase must assume the prior phase might have silently changed upstream facts — so discovery/resolve/index rerun anyway (the roadmap's red_control: "silently reruns full discovery when prior phase already fixed those facts"). |
| **Byte gates cannot distinguish intent** | `generated_artifact_drift_gate`, `regen_verify`, and `self_host_realized_comparison` answer only *match / mismatch*. A PR that intentionally changes `05_emit` output and includes regen is indistinguishable from an emitter bug — both are "bytes differ." Review becomes the classifier (§5 specification-without-execution). |
| **Self-host flips lack stage attribution** | `frontier_probe_types` locates blockers at `ProbeStageAssemble` / `ProbeStageEmit` / `ProbeStageSemanticDerivation`, but that vocabulary is harness-facing. There is no durable receipt tying an emission delta to `CompilerStage::Emit` vs an upstream `Infer` refusal planted at emit — the `#7485` containment class. |
| **Guarantee paths stay unmeasured at emission** | `gunbc.guarantee_measurement` names `source→each-emission-target` as an independent path grain, but receipts today do not carry *declared vs regression* disposition — only observed verdict/refusal. Rung honesty (§4b) cannot rank emission-path regressions separately from declared emitter migrations. |

---

## 2. The gap — what exists vs what is missing

### 2.1 Authorities already correct (reuse, do not fork)

| Fact | Authority | Live today? |
|---|---|---|
| Compiler pipeline stages | `gunbc.bootstrap.CompilerStage` (`Tokenize`…`Emit`) | ✓ `dag/gunbc/bootstrap.dag` |
| Stage input/output kinds | `gunbc.bootstrap.StageInput` / `StageOutput` / `ArtifactKind` | ✓ modeled, lightly exercised |
| Change → affected stages (bootstrap edits) | `gunbc.bootstrap.ChangeClassification.affects_stages` | ✓ modeled; not wired to CI/self-host |
| Field propagation per stage | `gunbc.bootstrap.FieldPropagation` / `TransformContract` | ✓ modeled; not executed |
| Source-graph change kinds | `v2.std.change.ChangeKind` + `ChangeSet` | ✓ consumed by `v2.lens.affected_set` |
| Emitted-byte digest | `v2.compiler.self_host.canonical_emitted_bytes_digest` | ✓ executed on self-host roster |
| Per-path guarantee receipts | `gunbc.guarantee_measurement.GuaranteeMeasurementReceipt` | ✓ Stage-0 vocabulary; no emission-admission join |
| Generated artifact registry | `gunbc.generated_artifact` + drift witnesses | ✓ byte fixed-point; no declared-change arm |
| Harness probe positions | `v2.compiler.self_host.frontier_probe_types.FrontierProbeStage` | ✓ distinct from `CompilerStage` by design |

### 2.2 The violation (three coupled gaps)

1. **No declared-change carrier.** PRs carry git diffs and affected-set projections, but nothing in-tree states *which compiler stages and emission surfaces are allowed to move* for this change. `ChangeClassification` in `gunbc.bootstrap` is bootstrap-authoring vocabulary only — not joined to pull-request admission.

2. **No emission-admission verdict.** Consumers collapse to `bytes_equal` / `bytes_differ`. Missing arms: `AdmittedDeclared { stages, surfaces }`, `RefusedRegression { stage, surface, cause }`, `RefusedUndeclaredDelta`, `RefusedStageMismatch { declared, observed }`. The absorbing fallback class (§5): treating any mismatch as "rerun regen" or "rerun whole floor" without a counted, typed reason.

3. **No stage-stamped phase pipeline.** Regen, floor, and admission are separate processes (separate `gunbc run` invocations, separate indices). `phased-single-process-ci` needs phase boundaries with *separate verdict stamps* on one shared materialization provider — but the stamp shape is unspecified.

---

## 3. Scope

### 3.1 In scope (this design)

- Vocabulary and join contracts for **declared change**, **emission delta**, and **admission verdict**.
- **Stage-aware pipeline run** model: phases, stage stamps, artifact surfaces.
- Classification rules: declared-change vs regression (fail-closed).
- First consumers named with discriminating witnesses (design-level; implementation is staged).
- Explicit integration with `warm-merge-admission`, `phased-single-process-ci`, regen/self-host, and generated-artifact drift.

### 3.2 Out of scope (named neighbors)

| Neighbor | Why out |
|---|---|
| Implementing warm-merge-admission (#7522) | Downstream consumer; this design is its admission *ontology*. |
| Materialization substrate (#7534) | Shared cache keys are orthogonal; this design consumes materialization receipts, does not define them. |
| Full `ChangeClassification` → live bootstrap planner | Bootstrap authoring tool; only the *join pattern* is reused for PR declared-change. |
| Byte-identical self-host fixed point | Retired non-goal (DESIGN §7); admission uses behavioral + digest receipts, not byte identity as the sole oracle. |
| DDC / `ddc_reference_compiler` | [seed-honesty-discharge-design.md](seed-honesty-discharge-design.md) — consumes admission receipts, does not define them. |
| Guarantee ladder disposition derivation | `gunbc.guarantee_measurement` Stage 0 only; dispositions stay on the claims carrier. |

### 3.3 Load-bearing files (escalate before edit under a stale brief)

- `dag/gunbc/bootstrap.dag` — extend only via new exported types or new fns; do not repurpose `CompilerStage` variants.
- `src/v2/std/change.dag` — declared-change projection from `ChangeSet`.
- New module home (proposed): `gunbc.emission_admission` under `dag/gunbc/` (product/workflow layer, not `std/`).
- `src/v2/workflow/ci_floor_plan.dag` / `dag/gunbc/commit_workflow.dag` — phase enrollment (later PRs).
- `dag/tools/generated_artifact_gate.dag` — regen phase consumer (later PR).

---

## 4. Target model

### 4.1 Artifact surface (what emission admission governs)

An **emission surface** is a typed target for bytes the compiler pipeline (or a
generated-artifact emitter) produces. Surfaces are enumerated — not free paths —
so admission cannot be bypassed via a novel string path.

```dag
type EmissionSurface
  = Stage0SeedFile { rel_path: FilePath }           // GENERATED_STAGE0_FILES roster
  | GeneratedArtifactSurface { artifact: GeneratedArtifact }
  | TargetLanguageBundle { target: GuaranteeTargetName, module: QualifiedModule }
  | OrchestrationMedium { medium: Symbol }            // e.g. bash slice enrolled in ci_spec
```

**Authority:** surfaces derive from existing registries (`gunbc.generated_artifact`,
`gunbc.stage0_emit_model.generated_stage0_files`, self-host frontier module list,
`gunbc.guarantee_measurement.GuaranteeTargetName`). A path literal not reachable
from a registry row is `SurfaceUnknown` and refuses — never admitted by glob.

### 4.2 Stage stamp (pipeline awareness)

Each pipeline phase produces a **stage stamp** recording which `CompilerStage`
values contributed to an artifact (not just the last stage — emit can plant
upstream `CompilerError` nodes).

```dag
type StageContribution
  = StageProduced { stage: CompilerStage }
  | StageForwarded { stage: CompilerStage, from_prior_phase: PipelinePhase }
  | UpstreamRefusalEmbedded { origin_stage: CompilerStage, diagnostic: NonEmptyStr }

type StageStamp {
  phase: PipelinePhase
  contributions: List<StageContribution>
  substrate_revision: ContentFingerprint   // shared materialization identity
}

type PipelinePhase
  = RegenPhase
  | FloorPhase
  | AdmissionPhase
```

**Law:** `FrontierProbeStage` projects to `CompilerStage` via an explicit total
function `compiler_stage_from_probe` when probe receipts are consumed — never the
identity map. Harness positions (`Assemble`) are not compiler stages.

### 4.3 Declared change set (author intent)

A **declared change set** is the PR's claim about which stages and surfaces may
move. It is derived — not hand-authored per PR — from facts already in the tree:

| Source | Projection |
|---|---|
| Git merge-base diff + `v2.lens.affected_set` | `ChangeSet` over module graph |
| `v2.std.change.RecomputePlan` / readiness layers | stages that must rerun vs preserve |
| Touch of emitter / translate / generated-artifact emit modules | `CompilerStage::Emit` (+ upstream if not lossless) |
| `gunbc.bootstrap.ChangeClassification` rows (when editing bootstrap types) | explicit `affects_stages` |
| Generated-artifact registry + diff | `EmissionSurface` rows for touched artifacts |

```dag
type DeclaredChangeSet {
  subject_revision: MeasurementRevision      // guarantee_measurement vocabulary
  allowed_stage_moves: List<CompilerStage>
  allowed_surface_moves: List<EmissionSurface>
  derivation_evidence: DeclaredChangeEvidence  // typed, located — not prose
}

type DeclaredChangeEvidence
  = DerivedFromAffectedSet { changes: ChangeSet }
  | DerivedFromBootstrapPlan { change: ChangeClassification }
  | AuthorExplicit { reason: NonEmptyStr }    // rare; counted; never default
```

**Fail-closed:** `DeclaredChangeUnknown` when affected-set refuses, selection
fails closed, or a touched path is outside modeled surfaces. Unknown ⇒ no
admission — not "allow and rerun."

### 4.4 Emission delta (observed fact)

```dag
type EmissionDelta {
  surface: EmissionSurface
  before: ContentFingerprint
  after: ContentFingerprint
  stage_stamp: StageStamp
  observed_at_phase: PipelinePhase
}
```

Deltas are observed by execution — regen compare, drift gate, self-host
`canonical_emitted_bytes_digest`, warm receipt replay. Numeric literals copied
from the current tree are not oracles (DESIGN §5 merge-blocking test rule).

### 4.5 Admission verdict (the seam)

```dag
type EmissionAdmissionVerdict
  = AdmittedDeclared {
      delta: EmissionDelta
      matched_declaration: DeclaredChangeSet
    }
  | RefusedRegression {
      delta: EmissionDelta
      cause: EmissionRegressionCause
    }
  | RefusedUndeclaredDelta {
      delta: EmissionDelta
      missing_from_declaration: EmissionSurface
    }
  | RefusedStageMismatch {
      delta: EmissionDelta
      declared_stages: List<CompilerStage>
      observed_stage: CompilerStage
    }
  | RefusedStaleSubstrate {
      stamp: StageStamp
      cause: NonEmptyStr
    }

type EmissionRegressionCause
  = SurfaceMovedWithoutDeclaration
  | StageAttributionMismatch
  | UpstreamRefusalMaskedAtEmit
  | BehavioralWitnessFailed
  | DigestMismatchUndeclared
```

**Core decision** (pure, total on well-formed inputs):

```
admit_emission_delta(delta, declared) ->
  if delta.before == delta.after -> AdmittedDeclared (no-op move)
  else if !surface_allowed(delta.surface, declared) -> RefusedUndeclaredDelta
  else if !stage_allowed(delta.stage_stamp, declared) -> RefusedStageMismatch
  else if upstream_refusal_masked(delta) -> RefusedRegression
  else -> AdmittedDeclared
```

No arm widens: missing declaration never triggers regen-all; mismatch never
falls through to green.

---

## 5. Stage-aware pipeline (phased single process)

### 5.1 End-to-end shape

```
┌─────────────────────────────────────────────────────────────────┐
│  One initialized process (shared materialization provider)       │
│                                                                  │
│  RegenPhase                                                      │
│    ├─ compute DeclaredChangeSet from PR facts                    │
│    ├─ run regen emitters → EmissionDelta[]                       │
│    ├─ StageStamp per delta (Emit + forwarded upstream)           │
│    └─ RegenVerdict = fold admit_emission_delta                   │
│           │                                                      │
│           ▼ (RegenVerdict must be Admitted* or Refused — blocks) │
│  FloorPhase                                                      │
│    ├─ reuse substrate_revision + declared set                    │
│    ├─ witness / compile / native runs                            │
│    └─ FloorVerdict (separate stamp; may not rerun resolve)       │
│           │                                                      │
│           ▼                                                      │
│  AdmissionPhase (merge / warm-merge-admission)                   │
│    ├─ validate warm receipts against declared + stamps           │
│    └─ AdmissionVerdict → merge gate                              │
└─────────────────────────────────────────────────────────────────┘
```

**Ordering law (from roadmap):** regen still gates floor. Admission may consume
warm receipts only when `substrate_revision` and `DeclaredChangeSet` match the
stamping run.

### 5.2 Prelude duplication metric

First slice for `phased-single-process-ci` (roadmap handback): attribute time
spent in resolve, index construction, and discovery per phase with and without
stamps. **RED control:** if phase *N+1* repeats resolve whose inputs are
unchanged per stamp, count > 0 ⇒ regression refusal (the roadmap red_control,
made executable).

---

## 6. Consumer mapping

| Consumer | Declared-change source | Regression signal |
|---|---|---|
| `warm-merge-admission` | PR affected set + stamped receipts at merge base | Receipt replay produces undeclared `EmissionDelta` |
| `generated_artifact_drift_gate` | diff touches artifact authority modules | `artifact_generate` ≠ committed without `allowed_surface_moves` |
| `regen_stage0 --verify` | diff touches `src/v2/compiler/*emit*` or frontier SelfEmitted rows | seed file digest mismatch undeclared |
| `self_host_realized_comparison` | frontier row flip / emitter edit in PR | behavioral witness fails with declared-only digest move |
| `ci_merge_admission_emit` | gate roster + floor stamp (existing) joined with admission verdict | stale-base / stale-roster (existing) + new undeclared-delta arm |
| `gunbc.guarantee_measurement` | probe harness declares subject revision | `ExecutionDiverged` on emission path with clean compile admission |

---

## 7. Witness / RED discipline (§4b, §5)

Each verdict arm gets a discriminating pair:

| Arm | GREEN control | RED control |
|---|---|---|
| `AdmittedDeclared` | emitter edit in PR + regen committed + `allowed_surface_moves` contains surface | same edit without declaration ⇒ `RefusedUndeclaredDelta` |
| `RefusedRegression` | planted emitter bug without source change ⇒ refusal | declaration cannot green a behavioral failure |
| `RefusedStageMismatch` | declared `Infer`-only change but emit digest moves | mismatch stage attribution |
| `RefusedStaleSubstrate` | warm receipt from prior `substrate_revision` | replay after index rebuild without restamp |
| Prelude duplication | phase 2 skips resolve when stamp says preserved | forced re-resolve ⇒ counted refusal |

Probes that go green when a wall lands **flip to permanent regression controls** —
they do not retire (§4b dissolution-on rule).

---

## 8. Staged plan (design only — implementation is separate PRs)

| Stage | Deliverable | Depends on |
|---|---|---|
| **0 — this document** | Sign-ready design + operator review | — |
| **1 — vocabulary carrier** | `gunbc.emission_admission` types + pure `admit_emission_delta` + unit witnesses on synthetic deltas | Stage 0 sign-off |
| **2 — declared-change derivation** | `declared_change_set_from_pr_facts` wired to affected-set; refusal when selection fails | Stage 1 |
| **3 — regen phase stamp** | `regen_stage0` / `generated_artifact_gate` emit `StageStamp` + deltas; regen refuses undeclared | Stage 2 |
| **4 — warm-merge join** | #7522 admits from warm receipts only with matching stamps + declaration | Stage 3 |
| **5 — phased single process** | `phased-single-process-ci` shares substrate; prelude duplication witness | Stage 4 |
| **6 — guarantee path rows** | emission-path measurement receipts carry admission verdict projection | Stage 1 (can parallel 3–5) |

**Dissolution triggers (scaffolds):**

- Stage 1: delete any interim `bytes_differ` only branch in merge admission once
  verdict arm executes in CI.
- Stage 3: retire `regen_divergence_count` prose-oracle when structured
  `EmissionDelta` list is the transport (extends regen ratchet retirement).
- Stage 5: delete duplicate `process_shared_index` construction in per-phase
  entrypoints when stamp hit rate is receipted ≥ target on representative PRs.

---

## 9. Open questions (operator sign-off)

1. **AuthorExplicit declaration:** should human override ever admit a delta without
   affected-set evidence, or is that always a §5 escape hatch to refuse?

2. **Behavioral vs digest on self-host:** for `SelfEmitted` modules, is digest
   match alone ever sufficient, or must `AdmittedDeclared` always include a
   behavioral witness when the surface is `Stage0SeedFile`?

3. **Frontier scaffold rows:** do counted `SeedRetained` / `UnpinnedFrontier` rows
   automatically widen `allowed_surface_moves`, or does each row carry an explicit
   `DeclaredChangeTemplate`?

4. **Registry completeness:** is `SurfaceUnknown` a hard refusal on the merge path,
   or a counted `FrontierAccepted` disposition until the generated-artifact
   registry covers all emit surfaces?

5. **Relationship to `BothWays`:** [module-identity-storage-binding-design.md](module-identity-storage-binding-design.md)
   intentional semantic edits compare `R1` vs `R2`, not authority vs seed — does
   emission admission treat quotient-delta lawful edits as always `AdmittedDeclared`
   when the six lens laws pass, even if digest moves?

---

## 10. Sign-off bar

This design is ready for operator review when:

- [ ] §4 types are agreed as the single admission authority (no parallel byte gate semantics).
- [ ] Stage 1–3 sequencing is accepted relative to #7522 / `phased-single-process-ci`.
- [ ] Open questions §9 have recorded verdicts (even if "defer with scaffold row").
- [ ] Parent lane (`still-bat-561`) confirms this unblocks v1-deletion emission gates
      without contradicting seed-honesty / real-fixpoint milestones.
